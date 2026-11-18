#include <sys/utsname.h>

#include <algorithm>
#include <cstdio>
#include <exception>
#include <unordered_set>
#include <utility>

#include "platform.hpp"
#include "point.hpp"
#include "stack.hpp"
#include "weft/gtk.hpp"

namespace weft {

namespace {

/// Why a batch of ops does not fit the views.
struct OpError : std::runtime_error {
  using std::runtime_error::runtime_error;
};

/// Console messages and diagnostics go to stderr.
void write_log(int level, std::string_view message) {
  static constexpr const char *kLevels[] = {"debug", "info", "warn", "error"};
  std::fprintf(stderr, "[weft %s] %.*s\n", kLevels[std::clamp(level, 0, 3)], int(message.size()), message.data());
}

/// `weft/bundle.hbc` next to the executable.
std::string bundle_path() {
  char *executable = g_file_read_link("/proc/self/exe", nullptr);
  char *directory = g_path_get_dirname(executable ? executable : ".");
  char *path = g_build_filename(directory, "weft", "bundle.hbc", nullptr);
  std::string result(path);
  g_free(path);
  g_free(directory);
  g_free(executable);
  return result;
}

/// The OS version reported as `platform.version`: the distribution's
/// version, else the kernel release.
std::string os_version() {
  char *version = g_get_os_info(G_OS_INFO_KEY_VERSION_ID);
  if (version) {
    std::string result(version);
    g_free(version);
    return result;
  }
  utsname name{};
  return uname(&name) == 0 ? name.release : "";
}

/// Scope 0: holds the root component instance at mount point 0.
class RootScope final : public Scope {
 public:
  RootScope(Session &session, GtkWidget *page) : Scope(0, session, 1), page_(page) { page_.point(*this, 0); }

 private:
  Region page_;
};

}  // namespace

GtkWidget *Session::start(uint64_t layout_hash, Factory make) {
  GtkWidget *page = stack_new(Axis::Vertical, Align::Start);
  gtk_widget_set_halign(page, GTK_ALIGN_START);
  gtk_widget_set_valign(page, GTK_ALIGN_START);
  GtkWidget *view = gtk_scrolled_window_new();
  gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(view), page);
  // The session ends with the root widget.
  auto *session = new Session(page, make);
  g_object_set_data_full(G_OBJECT(view), "weft-session", session,
                         [](gpointer session) { delete static_cast<Session *>(session); });
  session->open(bundle_path(), layout_hash);
  return view;
}

Session::Session(GtkWidget *page, Factory make) : make_(make), page_(page), cancellable_(g_cancellable_new()) {
  scopes_[0] = std::make_unique<RootScope>(*this, page);
}

Session::~Session() {
  weft_runtime_free(runtime_);
  g_cancellable_cancel(cancellable_);
  g_object_unref(cancellable_);
  if (timer_) g_source_remove(timer_);
  if (sweep_) g_source_remove(sweep_);
  scopes_.clear();
  dead_.clear();
  if (bytecode_) g_mapped_file_unref(bytecode_);
}

void Session::open(const std::string &bundle, uint64_t layout_hash) {
  GError *error = nullptr;
  // Mapped files are page aligned and stay unchanged: the kernel keeps using the bytecode.
  bytecode_ = g_mapped_file_new(bundle.c_str(), false, &error);
  if (!bytecode_) {
    fail("weft: cannot read the bundle " + bundle + ": " + error->message);
    g_error_free(error);
    return;
  }
  std::string version = os_version();
  weft_host host{
      .ctx = this,
      .commit = on_commit,
      .schedule = on_schedule,
      .supports = on_supports,
      .call = on_call,
      .log = on_log,
      .version = version.c_str(),
  };
  runtime_ = weft_runtime_new(&host, reinterpret_cast<const uint8_t *>(g_mapped_file_get_contents(bytecode_)),
                              g_mapped_file_get_length(bytecode_), layout_hash);
  if (!runtime_) fail(last_error_.empty() ? "weft: the app failed to start" : last_error_);
}

bool Session::dispatch(uint32_t scope, uint32_t slot, const std::string &payload) {
  if (committing_) {
    write_log(WEFT_LOG_ERROR, "weft: an event was dispatched while ops were applied");
    return true;
  }
  if (!runtime_) return true;
  return weft_runtime_dispatch(runtime_, scope, slot, reinterpret_cast<const uint8_t *>(payload.data()),
                               payload.size());
}

void Session::commit(const uint8_t *ops, size_t len) {
  if (failed_) return;
  committing_ = true;
  try {
    ops::Reader reader(ops, len);
    for (ops::Op op; reader.next(op);) apply(op);
  } catch (const std::exception &error) {
    committing_ = false;
    fail(std::string("weft: cannot apply ops (") + error.what() + ")");
  }
  committing_ = false;
}

void Session::apply(const ops::Op &op) {
  switch (op.code) {
    case ops::Code::Create: {
      auto scope = make_(op.index, op.scope, *this);
      if (!scope) throw OpError("the views have no template " + std::to_string(op.index));
      scopes_[op.scope] = std::move(scope);
      break;
    }
    case ops::Code::Set: {
      auto found = scopes_.find(op.scope);
      if (found != scopes_.end()) found->second->apply(op.index, op.value);
      break;
    }
    case ops::Code::Branch:
      mount(op.scope, op.index, op.child == 0 ? std::vector<uint32_t>{} : std::vector<uint32_t>{op.child});
      break;
    case ops::Code::Attach:
      mount(op.scope, op.index, {op.child});
      break;
    case ops::Code::Rows: {
      std::vector<uint32_t> rows(op.count);
      for (uint32_t i = 0; i < op.count; i++) rows[i] = op.row(i);
      mount(op.scope, op.index, std::move(rows));
      break;
    }
  }
}

void Session::mount(uint32_t parent, uint32_t index, std::vector<uint32_t> ids) {
  auto found = scopes_.find(parent);
  if (found == scopes_.end()) return;
  Scope &scope = *found->second;
  if (index >= scope.points_.size() || !scope.points_[index]) {
    throw OpError("scope " + std::to_string(parent) + " has no mount point " + std::to_string(index));
  }
  Point &point = *scope.points_[index];
  std::unordered_set<uint32_t> kept(ids.begin(), ids.end());
  for (Scope *old : point.children) {
    if (!kept.contains(old->id())) destroy(*old);
  }
  std::vector<Scope *> children;
  children.reserve(ids.size());
  for (uint32_t id : ids) {
    auto child = scopes_.find(id);
    if (child != scopes_.end()) children.push_back(child->second.get());
  }
  point.mount(std::move(children));
}

void Session::destroy(Scope &scope) {
  for (auto &point : scope.points_) {
    if (!point) continue;
    for (Scope *child : point->children) destroy(*child);
  }
  auto found = scopes_.find(scope.id());
  if (found == scopes_.end()) return;
  dead_.push_back(std::move(found->second));
  scopes_.erase(found);
  if (!sweep_) {
    sweep_ = g_idle_add(
        [](gpointer data) {
          auto *session = static_cast<Session *>(data);
          session->sweep_ = 0;
          session->dead_.clear();
          return G_SOURCE_REMOVE;
        },
        this);
  }
}

void Session::schedule(uint64_t delay_ms) {
  auto delay = guint(std::min<uint64_t>(delay_ms, G_MAXINT));
  int64_t at = g_get_monotonic_time() / 1000 + delay;
  if (timer_ && at >= due_) return;
  if (timer_) g_source_remove(timer_);
  due_ = at;
  timer_ = g_timeout_add(delay, [](gpointer data) {
    auto *session = static_cast<Session *>(data);
    session->timer_ = 0;
    if (session->runtime_) weft_runtime_tick(session->runtime_);
    return G_SOURCE_REMOVE;
  }, this);
}

void Session::call(uint32_t id, std::string module, std::string method, std::vector<uint8_t> args) {
  platform::call(module, method, std::move(args), cancellable_,
                 [this, id](bool ok, std::vector<uint8_t> result) { resolve(id, ok, result); });
}

void Session::resolve(uint32_t id, bool ok, const std::vector<uint8_t> &result) {
  if (runtime_) weft_runtime_resolve(runtime_, id, ok, result.data(), result.size());
}

void Session::fail(const std::string &message) {
  write_log(WEFT_LOG_ERROR, message);
  failed_ = true;
  while (GtkWidget *child = gtk_widget_get_first_child(page_)) gtk_widget_unparent(child);
  GtkWidget *label = gtk_label_new(message.c_str());
  gtk_label_set_wrap(GTK_LABEL(label), true);
  gtk_label_set_xalign(GTK_LABEL(label), 0);
  gtk_widget_add_css_class(label, "error");
  gtk_widget_set_margin_start(label, 16);
  gtk_widget_set_margin_end(label, 16);
  gtk_widget_set_margin_top(label, 16);
  gtk_widget_set_margin_bottom(label, 16);
  gtk_widget_set_parent(label, page_);
}

// Kernel callbacks; every one arrives on the main thread. Exceptions must not
// unwind into the kernel.

void Session::on_commit(void *ctx, const uint8_t *ops, size_t len) { static_cast<Session *>(ctx)->commit(ops, len); }

void Session::on_schedule(void *ctx, uint64_t delay_ms) { static_cast<Session *>(ctx)->schedule(delay_ms); }

bool Session::on_supports(void *, const char *module) { return module && platform::supported(module); }

void Session::on_call(void *ctx, uint32_t id, const char *module, const char *method, const uint8_t *args,
                      size_t len) {
  try {
    static_cast<Session *>(ctx)->call(id, module, method, {args, args + len});
  } catch (const std::exception &error) {
    write_log(WEFT_LOG_ERROR, std::string("weft: cannot start a platform call (") + error.what() + ")");
  }
}

void Session::on_log(void *ctx, int level, const char *msg, size_t len) {
  std::string_view message(msg, len);
  write_log(level, message);
  if (level >= WEFT_LOG_ERROR) static_cast<Session *>(ctx)->last_error_ = message;
}

}  // namespace weft
