#include "platform.hpp"

#include "frame.hpp"
#include "weft/protocol.hpp"

#include <json-glib/json-glib.h>
#include <libsoup/soup.h>

#include <map>
#include <memory>
#include <optional>
#include <stdexcept>
#include <utility>

namespace weft::platform {

namespace {

std::vector<uint8_t> bytes_of(std::string_view text) { return {text.begin(), text.end()}; }

/// A call in flight: its cancellable (referenced) and completion.
struct Pending {
  Pending(GCancellable *cancellable, Done done)
      : cancellable(G_CANCELLABLE(g_object_ref(cancellable))), done(std::move(done)) {}
  ~Pending() { g_object_unref(cancellable); }

  void finish(bool ok, std::vector<uint8_t> result) {
    if (!g_cancellable_is_cancelled(cancellable)) done(ok, std::move(result));
  }

  GCancellable *cancellable;
  Done done;
};

/// Completes `pending` from an idle callback: results never arrive inside the kernel's `call`.
void finish_later(std::unique_ptr<Pending> pending, bool ok, std::vector<uint8_t> result) {
  struct Later {
    std::unique_ptr<Pending> pending;
    bool ok;
    std::vector<uint8_t> result;
  };
  g_idle_add_full(
      G_PRIORITY_DEFAULT,
      [](gpointer data) {
        auto *later = static_cast<Later *>(data);
        later->pending->finish(later->ok, std::move(later->result));
        return G_SOURCE_REMOVE;
      },
      new Later{std::move(pending), ok, std::move(result)}, [](gpointer data) { delete static_cast<Later *>(data); });
}

std::string to_json(JsonNode *node) {
  char *text = json_to_string(node, false);
  std::string json(text);
  g_free(text);
  return json;
}

// fetch.request through libsoup.

SoupSession *http() {
  static SoupSession *session = soup_session_new();
  return session;
}

/// Sends the request; `pending` is released once the request is sent.
void fetch(const std::vector<uint8_t> &args, std::unique_ptr<Pending> &pending) {
  auto [head, body] = frame::unframe(args);
  JsonObject *object = json_node_get_object(head);
  std::string url = json_object_get_string_member_with_default(object, "url", "");
  std::string method = json_object_get_string_member_with_default(object, "method", "GET");
  SoupMessage *message = soup_message_new(method.c_str(), url.c_str());
  if (!message) {
    json_node_unref(head);
    throw std::runtime_error("weft: invalid URL " + url);
  }
  if (json_object_has_member(object, "headers") && JSON_NODE_HOLDS_OBJECT(json_object_get_member(object, "headers"))) {
    JsonObject *headers = json_object_get_object_member(object, "headers");
    SoupMessageHeaders *request = soup_message_get_request_headers(message);
    json_object_foreach_member(
        headers,
        [](JsonObject *, const char *name, JsonNode *value, gpointer request) {
          const char *text = JSON_NODE_HOLDS_VALUE(value) ? json_node_get_string(value) : nullptr;
          soup_message_headers_append(static_cast<SoupMessageHeaders *>(request), name, text ? text : "");
        },
        request);
  }
  json_node_unref(head);
  if (!body.empty()) {
    GBytes *bytes = g_bytes_new(body.data(), body.size());
    // A null content type keeps the request's Content-Type header.
    soup_message_set_request_body_from_bytes(message, nullptr, bytes);
    g_bytes_unref(bytes);
  }
  GCancellable *cancellable = pending->cancellable;
  soup_session_send_and_read_async(
      http(), message, G_PRIORITY_DEFAULT, cancellable,
      [](GObject *session, GAsyncResult *result, gpointer data) {
        std::unique_ptr<Pending> pending(static_cast<Pending *>(data));
        GError *error = nullptr;
        GBytes *bytes = soup_session_send_and_read_finish(SOUP_SESSION(session), result, &error);
        if (!bytes) {
          pending->finish(false, bytes_of(error->message));
          g_error_free(error);
          return;
        }
        auto *message = soup_session_get_async_result_message(SOUP_SESSION(session), result);
        std::string head = "{\"status\":" + std::to_string(soup_message_get_status(message)) + "}";
        gsize len = 0;
        auto *body = static_cast<const uint8_t *>(g_bytes_get_data(bytes, &len));
        pending->finish(true, frame::frame(head, body, len));
        g_bytes_unref(bytes);
      },
      pending.release());
  g_object_unref(message);
}

// storage.get / set / remove: a JSON object of strings in
// $XDG_DATA_HOME/<app id>/storage.json (~/.local/share by default). The app
// id is the GApplication's, else the program name.

std::string storage_path() {
  const char *id = nullptr;
  if (GApplication *app = g_application_get_default()) id = g_application_get_application_id(app);
  if (!id || !*id) id = g_get_prgname();
  if (!id || !*id) id = "weft";
  char *path = g_build_filename(g_get_user_data_dir(), id, "storage.json", nullptr);
  std::string result(path);
  g_free(path);
  return result;
}

std::map<std::string, std::string> &stored() {
  static std::optional<std::map<std::string, std::string>> values;
  if (values) return *values;
  values.emplace();
  char *contents = nullptr;
  gsize length = 0;
  if (!g_file_get_contents(storage_path().c_str(), &contents, &length, nullptr)) return *values;
  try {
    JsonNode *root = parse_json({contents, length}, "storage file");
    if (JSON_NODE_HOLDS_OBJECT(root)) {
      json_object_foreach_member(
          json_node_get_object(root),
          [](JsonObject *, const char *name, JsonNode *value, gpointer values) {
            if (JSON_NODE_HOLDS_VALUE(value) && json_node_get_value_type(value) == G_TYPE_STRING) {
              static_cast<std::map<std::string, std::string> *>(values)->emplace(name, json_node_get_string(value));
            }
          },
          &*values);
    }
    json_node_unref(root);
  } catch (const std::runtime_error &) {
    // A damaged file starts over empty.
  }
  g_free(contents);
  return *values;
}

/// Writes the file through a temporary one, so a crash never leaves it half written.
void save(const std::map<std::string, std::string> &values) {
  JsonObject *object = json_object_new();
  for (auto &[key, value] : values) json_object_set_string_member(object, key.c_str(), value.c_str());
  JsonNode *root = json_node_init_object(json_node_alloc(), object);
  json_object_unref(object);
  std::string json = to_json(root);
  json_node_unref(root);
  std::string path = storage_path();
  char *directory = g_path_get_dirname(path.c_str());
  g_mkdir_with_parents(directory, 0700);
  g_free(directory);
  GError *error = nullptr;
  if (!g_file_set_contents(path.c_str(), json.data(), gssize(json.size()), &error)) {
    std::string message = std::string("weft: cannot write ") + path + " (" + error->message + ")";
    g_error_free(error);
    throw std::runtime_error(message);
  }
}

std::vector<uint8_t> storage(const std::string &method, const std::vector<uint8_t> &args) {
  JsonNode *root = parse_json({reinterpret_cast<const char *>(args.data()), args.size()}, "storage arguments");
  std::vector<std::string> list;
  if (JSON_NODE_HOLDS_ARRAY(root)) {
    JsonArray *array = json_node_get_array(root);
    for (guint i = 0; i < json_array_get_length(array); i++) {
      JsonNode *item = json_array_get_element(array, i);
      if (!JSON_NODE_HOLDS_VALUE(item) || json_node_get_value_type(item) != G_TYPE_STRING) break;
      list.emplace_back(json_node_get_string(item));
    }
  }
  json_node_unref(root);
  if (list.empty()) throw std::runtime_error("weft: storage." + method + " expects a JSON array of strings");
  auto &values = stored();
  if (method == protocol::kStorageGet) {
    auto found = values.find(list[0]);
    if (found == values.end()) return bytes_of("null");
    JsonNode *value = json_node_init_string(json_node_alloc(), found->second.c_str());
    std::string json = to_json(value);
    json_node_unref(value);
    return bytes_of(json);
  }
  if (method == protocol::kStorageSet) {
    if (list.size() != 2) throw std::runtime_error("weft: storage.set expects [key, value]");
    values[list[0]] = list[1];
  } else if (values.erase(list[0]) == 0) {
    return {};
  }
  save(values);
  return {};
}

}  // namespace

bool supported(std::string_view module) {
  return module == protocol::kFetchModule || module == protocol::kStorageModule;
}

void call(const std::string &module, const std::string &method, std::vector<uint8_t> args,
          GCancellable *cancellable, Done done) {
  auto pending = std::make_unique<Pending>(cancellable, std::move(done));
  try {
    if (module == protocol::kFetchModule && method == protocol::kFetchRequest) {
      fetch(args, pending);
    } else if (module == protocol::kStorageModule &&
               (method == protocol::kStorageGet || method == protocol::kStorageSet || method == protocol::kStorageRemove)) {
      auto result = storage(method, args);
      finish_later(std::move(pending), true, std::move(result));
    } else {
      throw std::runtime_error("weft: " + module + "." + method + " is not implemented");
    }
  } catch (const std::runtime_error &error) {
    // `pending` is still owned here: `fetch` only releases it once the request is sent.
    if (pending) finish_later(std::move(pending), false, bytes_of(error.what()));
  }
}

}  // namespace weft::platform
