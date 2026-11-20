#include "appearance.hpp"

#include <gtk/gtk.h>

#include <algorithm>
#include <cstring>
#include <vector>

namespace weft {
namespace {

constexpr const char *kPortal = "org.freedesktop.portal.Desktop";
constexpr const char *kPortalPath = "/org/freedesktop/portal/desktop";
constexpr const char *kPortalSettings = "org.freedesktop.portal.Settings";
constexpr const char *kAppearance = "org.freedesktop.appearance";
constexpr const char *kColorScheme = "color-scheme";

bool prefers_dark(GVariant *value) {
  // Some portal versions wrap the Read result in another variant.
  g_autoptr(GVariant) inner = nullptr;
  while (g_variant_is_of_type(value, G_VARIANT_TYPE_VARIANT)) {
    GVariant *next = g_variant_get_variant(value);
    g_clear_pointer(&inner, g_variant_unref);
    value = inner = next;
  }
  // 0 is no preference, 1 is dark, and 2 is light.
  return g_variant_is_of_type(value, G_VARIANT_TYPE_UINT32) && g_variant_get_uint32(value) == 1;
}

void apply_gtk_preference(bool dark) {
  if (GtkSettings *settings = gtk_settings_get_default()) {
    g_object_set(settings, "gtk-application-prefer-dark-theme", dark, nullptr);
  }
}

bool dark_preferred = false;

}  // namespace

struct Appearance::Portal {
  GDBusConnection *connection = nullptr;
  guint subscription = 0;
  std::vector<Appearance *> observers;

  void start() {
    connection = g_bus_get_sync(G_BUS_TYPE_SESSION, nullptr, nullptr);
    if (connection) {
      subscription = g_dbus_connection_signal_subscribe(
          connection, kPortal, kPortalSettings, "SettingChanged", kPortalPath, kAppearance,
          G_DBUS_SIGNAL_FLAGS_NONE,
          [](GDBusConnection *, const gchar *, const gchar *, const gchar *, const gchar *, GVariant *params,
             gpointer data) {
            if (!g_variant_is_of_type(params, G_VARIANT_TYPE("(ssv)"))) return;
            const char *setting;
            GVariant *value;
            g_variant_get(params, "(&s&sv)", nullptr, &setting, &value);
            g_autoptr(GVariant) owned = value;
            if (std::strcmp(setting, kColorScheme) == 0) {
              static_cast<Portal *>(data)->update(prefers_dark(owned));
            }
          }, this, nullptr);
      g_autoptr(GVariant) result = g_dbus_connection_call_sync(
          connection, kPortal, kPortalPath, kPortalSettings, "Read", g_variant_new("(ss)", kAppearance, kColorScheme),
          G_VARIANT_TYPE("(v)"), G_DBUS_CALL_FLAGS_NONE, 500, nullptr, nullptr);
      if (result) {
        g_autoptr(GVariant) value = g_variant_get_child_value(result, 0);
        dark_preferred = prefers_dark(value);
      }
    }
    apply_gtk_preference(dark_preferred);
  }

  void stop() {
    dark_preferred = false;
    if (subscription) g_dbus_connection_signal_unsubscribe(connection, subscription);
    subscription = 0;
    g_clear_object(&connection);
  }

  void update(bool dark) {
    if (dark == dark_preferred) return;
    dark_preferred = dark;
    apply_gtk_preference(dark);
    for (Appearance *observer : observers) observer->changed_(observer->data_);
  }
};

Appearance::Portal &Appearance::portal() {
  static Portal instance;
  return instance;
}

Appearance::Appearance(void (*changed)(void *), void *data) : changed_(changed), data_(data) {
  auto &state = portal();
  if (state.observers.empty()) state.start();
  state.observers.push_back(this);
}

Appearance::~Appearance() {
  auto &state = portal();
  std::erase(state.observers, this);
  if (state.observers.empty()) state.stop();
}

bool dark() { return dark_preferred; }

}  // namespace weft
