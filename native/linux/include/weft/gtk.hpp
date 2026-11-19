// Linux shell of Weft: GTK 4 widgets driven by the native kernel's ops
// (protocol/SPEC.md). The generated views (weft/Generated.cpp) subclass
// `Scope` per template and build their widgets from the element wrappers; the
// app calls `start_app()` for its root widget.
//
// Everything runs on the GTK main thread: the kernel's callbacks, timers
// and completed platform calls all arrive on the default main context.

#pragma once

#include <gtk/gtk.h>
#include <weft/ops.hpp>

#include <cstddef>
#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <string_view>
#include <unordered_map>
#include <vector>

#include <weft/runtime.h>

namespace weft {

class Scope;
class Session;
struct Point;

/// Implemented by the generated views: starts the app compiled to
/// `weft/bundle.hbc` next to the executable and returns its root widget, a
/// scrolled page whose content starts at the top left corner, or the reason
/// the app failed to start.
GtkWidget *start_app();

// Values of value slots.

/// An `Int` slot: a length in logical pixels.
double number(const ops::Value &value);
bool boolean(const ops::Value &value);
std::string_view string(const ops::Value &value);
/// A `Color` slot; invalid colors are transparent (with a warning in debug builds).
GdkRGBA color(const ops::Value &value);

/// A constant color of the generated views, written as `0xRRGGBBAA`.
constexpr GdkRGBA rgba(uint32_t rgba) {
  return {float(rgba >> 24) / 255.f, float(rgba >> 16 & 0xff) / 255.f, float(rgba >> 8 & 0xff) / 255.f,
          float(rgba & 0xff) / 255.f};
}

/// Parses `#RGB`, `#RRGGBB` or `#RRGGBBAA` in either hex case.
bool parse_color(std::string_view text, GdkRGBA &color);

/// The main axis of a `Column` / `Row`.
enum class Axis { Vertical, Horizontal };

/// Where a stack places its children on the cross axis.
enum class Align { Start, Center };

/// A built-in element: owns a reference to its widget. Properties never set
/// keep the platform's defaults; lengths are logical pixels.
class Element {
 public:
  Element(const Element &) = delete;
  Element &operator=(const Element &) = delete;
  virtual ~Element();

  GtkWidget *widget() const { return widget_; }

 protected:
  /// Takes the widget's floating reference.
  explicit Element(GtkWidget *widget);

 private:
  GtkWidget *widget_;
};

/// An ordered run of sibling nodes: the children of a stack, or the
/// top-level nodes of a scope. Mount points are placeholders replaced by the
/// widgets of their child scopes, so a scope with top-level points has no
/// fixed number of widgets; it renders into the stack of the region it is
/// mounted in, at the position its point has there.
class Region {
 public:
  /// The children of `container`, a `WeftStack`.
  explicit Region(GtkWidget *container) : container_(container) {}
  /// The top-level nodes of `scope`.
  explicit Region(Scope &scope) : scope_(&scope) {}
  Region(const Region &) = delete;
  Region &operator=(const Region &) = delete;

  /// Appends a static element.
  void add(const Element &element);
  /// Appends mount point `index` of `owner`.
  void point(Scope &owner, size_t index);

 private:
  friend struct Point;
  friend class Scope;

  /// A widget, or a mount point when `point` is set.
  struct Part {
    GtkWidget *widget;
    Point *point;
  };

  size_t count() const;
  void collect(std::vector<GtkWidget *> &widgets) const;
  size_t offset(const Point &point) const;
  void insert(size_t index, GtkWidget *widget);
  void remove(size_t index, size_t count);
  /// The stack region holding `point`'s widgets and their index there, or
  /// false while the scope holding it is not mounted.
  static bool locate(const Point &point, Region *&container, size_t &index);

  std::vector<Part> parts_;
  GtkWidget *container_ = nullptr;
  Scope *scope_ = nullptr;
};

/// `Column` / `Row`: a `WeftStack`. Children keep their natural size and
/// start at the leading edge of the cross axis; a fixed width or height
/// sizes the content box, with the padding around it, as CSS boxes do.
class Stack final : public Element {
 public:
  explicit Stack(Axis axis, Align align = Align::Start);

  Region &children() { return children_; }

  void set_gap(double gap);
  void set_padding(double padding);
  void set_width(double width);
  void set_height(double height);
  void set_background(GdkRGBA color);

 private:
  Region children_;
};

/// `Text`: a `GtkLabel` styled with Pango attributes.
class Text final : public Element {
 public:
  Text();

  void set_value(std::string_view value);
  void set_color(GdkRGBA color);
  void set_size(double size);
  void set_bold(bool bold);

 private:
  void restyle();

  GdkRGBA color_{};
  bool has_color_ = false;
  double size_ = 0;
  bool bold_ = false;
};

/// `Button`: a `GtkButton` whose content is a horizontal stack of its children.
class Button final : public Element {
 public:
  explicit Button(std::function<void()> click);

  Region &children() { return content_.children(); }

  void set_disabled(bool disabled);

 private:
  static void clicked(GtkButton *button, Button *self);

  std::function<void()> click_;
  Stack content_;
};

// Controlled `Input` and `Checkbox`. The widget reports every user edit
// through `edit`, which dispatches it synchronously; when the app refuses it
// (`edit` returns false) the widget shows its value slot again. Values written
// by `set` ops are recognized by comparing with the slot, so they are never
// reported back. Text being composed by an input method is not reported, so
// it is never rolled back mid-composition; the committed text is.

/// `Input`: a `GtkEntry`, 200 px wide (CSS inputs default to about 20 characters).
class Input final : public Element {
 public:
  explicit Input(std::function<bool(std::string_view)> edit);
  ~Input() override;

  /// The controlled value slot.
  void set_value(std::string_view value);
  void set_placeholder(std::string_view placeholder);
  void set_disabled(bool disabled);

 private:
  static void changed(GtkEditable *editable, Input *self);
  static void preedit_changed(GtkText *text, const char *preedit, Input *self);
  void edited();
  /// Replaces the text, keeping the caret where it was as far as the new text allows.
  void show(const std::string &text);

  std::function<bool(std::string_view)> edit_;
  std::string value_;
  bool composing_ = false;
  /// Whether `show` is replacing the text.
  bool showing_ = false;
};

/// `Checkbox`: a `GtkCheckButton` without a label.
class Checkbox final : public Element {
 public:
  explicit Checkbox(std::function<bool(bool)> edit);

  /// The controlled value slot.
  void set_checked(bool checked);
  void set_disabled(bool disabled);

 private:
  static void toggled(GtkCheckButton *button, Checkbox *self);

  std::function<bool(bool)> edit_;
  bool value_ = false;
};

/// `Image`: a `GtkPicture` of a remote (`http:` / `https:`) or local image,
/// downloaded once per process. With one fixed side the other follows the
/// aspect ratio, with both the image fills them, and with neither it keeps
/// its own size (one image pixel per logical pixel).
class Image final : public Element {
 public:
  Image();

  void set_src(std::string_view src);
  void set_width(double width);
  void set_height(double height);

 private:
  void resize();

  int width_ = -1;
  int height_ = -1;
};

/// One template instance of the view protocol. The generated views subclass
/// it per template: the constructor creates the template's elements and
/// places them into `roots()` and the regions of stacks, and `apply` writes
/// value slots to them.
class Scope {
 public:
  Scope(uint32_t id, Session &session, size_t points = 0);
  Scope(const Scope &) = delete;
  Scope &operator=(const Scope &) = delete;
  virtual ~Scope();

  /// Scope id assigned by the JS logic layer.
  uint32_t id() const { return id_; }

  /// Writes value slot `slot` (a `set` op).
  virtual void apply(uint32_t slot, const ops::Value &value);

 protected:
  /// The template's top-level nodes; they render into the region the scope is mounted in.
  Region &roots() { return roots_; }

  /// Creates an element that lives as long as the scope.
  template <class T, class... Args>
  T &make(Args &&...args) {
    auto element = std::make_unique<T>(std::forward<Args>(args)...);
    T &made = *element;
    elements_.push_back(std::move(element));
    return made;
  }

  /// Reports that event slot `slot` fired. The handler runs and its ops are
  /// applied before this returns; false means a controlled edit was refused
  /// and the widget must show its value slot again.
  bool dispatch(uint32_t slot);
  bool dispatch(uint32_t slot, bool value);
  bool dispatch(uint32_t slot, std::string_view value);

 private:
  friend class Region;
  friend struct Point;
  friend class Session;

  uint32_t id_;
  Session &session_;
  Region roots_;
  /// Mount points by index, registered by `Region::point`.
  std::vector<std::unique_ptr<Point>> points_;
  /// The point the scope is mounted at, if any.
  Point *parent_ = nullptr;
  std::vector<std::unique_ptr<Element>> elements_;
};

/// One Weft app running in the native kernel: owns the kernel runtime and
/// the scope tree its ops build. It lives as long as the root widget of
/// `start`.
class Session {
 public:
  /// Creates the scope of template `tpl` for a `create` op, or null for an unknown template.
  using Factory = std::unique_ptr<Scope> (*)(uint32_t tpl, uint32_t id, Session &session);

  /// Starts the app compiled to `weft/bundle.hbc` next to the executable,
  /// whose views implement the layout with hash `layout_hash`; see `start_app`.
  static GtkWidget *start(uint64_t layout_hash, Factory make);

  Session(const Session &) = delete;
  Session &operator=(const Session &) = delete;
  ~Session();

  /// Reports event slot `slot` of `scope` with an encoded payload; see `Scope::dispatch`.
  bool dispatch(uint32_t scope, uint32_t slot, const std::string &payload);

 private:
  Session(GtkWidget *page, Factory make);
  void open(const std::string &bundle, uint64_t layout_hash);
  void commit(const uint8_t *ops, size_t len);
  void apply(const ops::Op &op);
  void mount(uint32_t parent, uint32_t index, std::vector<uint32_t> ids);
  void destroy(Scope &scope);
  void schedule(uint64_t delay_ms);
  void call(uint32_t id, std::string module, std::string method, std::vector<uint8_t> args);
  void resolve(uint32_t id, bool ok, const std::vector<uint8_t> &result);
  /// Shows `message` instead of the app's views.
  void fail(const std::string &message);

  static void on_commit(void *ctx, const uint8_t *ops, size_t len);
  static void on_schedule(void *ctx, uint64_t delay_ms);
  static bool on_supports(void *ctx, const char *module);
  static void on_call(void *ctx, uint32_t id, const char *module, const char *method, const uint8_t *args,
                      size_t len);
  static void on_log(void *ctx, int level, const char *msg, size_t len);

  Factory make_;
  GtkWidget *page_;
  std::unordered_map<uint32_t, std::unique_ptr<Scope>> scopes_;
  /// Destroyed scopes, freed once the main loop is idle: a widget's signal
  /// handler may still run code of the scope that its event destroyed.
  std::vector<std::unique_ptr<Scope>> dead_;
  guint sweep_ = 0;
  /// The pending tick and when it fires, in monotonic milliseconds.
  guint timer_ = 0;
  int64_t due_ = 0;
  /// Whether a batch of ops is being applied; the kernel is not reentrant.
  bool committing_ = false;
  bool failed_ = false;
  /// The last error the kernel logged: why `weft_runtime_new` failed.
  std::string last_error_;
  /// Cancels the platform calls in flight when the session ends.
  GCancellable *cancellable_;
  /// The bytecode, page aligned, alive as long as the runtime.
  GMappedFile *bytecode_ = nullptr;
  weft_runtime *runtime_ = nullptr;
};

}  // namespace weft
