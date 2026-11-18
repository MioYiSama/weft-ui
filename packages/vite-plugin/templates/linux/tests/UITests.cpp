// UI tests of the app, driving its widgets in process; run with
// `ctest --test-dir build` on a desktop session after building the app.

#include <gtk/gtk.h>
#include <weft/gtk.hpp>

#include <cstdio>

int main() {
  if (!gtk_init_check()) {
    std::fprintf(stderr, "UI tests need a display (DISPLAY or WAYLAND_DISPLAY)\n");
    return 1;
  }
  GtkWidget *window = gtk_window_new();
  GtkWidget *root = weft::start_app();
  gtk_window_set_child(GTK_WINDOW(window), root);
  gtk_window_present(GTK_WINDOW(window));
  while (g_main_context_iteration(nullptr, false)) {
  }

  // Launches: the first batch of ops is applied when start_app returns, and
  // a failed start shows its reason in a label styled `error`.
  GtkWidget *page = gtk_viewport_get_child(GTK_VIEWPORT(gtk_scrolled_window_get_child(GTK_SCROLLED_WINDOW(root))));
  GtkWidget *first = gtk_widget_get_first_child(page);
  bool launched = first && !gtk_widget_has_css_class(first, "error");
  gtk_window_destroy(GTK_WINDOW(window));
  if (!launched) {
    std::fprintf(stderr, "the app did not start\n");
    return 1;
  }
  return 0;
}
