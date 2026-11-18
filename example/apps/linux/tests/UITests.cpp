// The example's click flow over every built-in element, driving the app's
// widgets in process; run with `pnpm --filter example test:linux` on a
// desktop session (e.g. the Docker desktop in native/linux/docker) after
// `pnpm --filter example build:linux`.

#include <gtk/gtk.h>
#include <weft/gtk.hpp>

#include <cstdio>
#include <cstdlib>
#include <functional>
#include <string>
#include <string_view>
#include <vector>

namespace {

GtkWidget *root;

[[noreturn]] void fail(const std::string &message) {
  std::fprintf(stderr, "FAILED: %s\n", message.c_str());
  std::exit(1);
}

void check(bool condition, const std::string &message) {
  if (!condition) fail(message);
}

/// Runs the main loop until `condition` holds, for at most five seconds.
void wait_for(const std::function<bool()> &condition, const std::function<std::string()> &failure) {
  int64_t deadline = g_get_monotonic_time() + 5 * G_USEC_PER_SEC;
  while (!condition()) {
    if (g_get_monotonic_time() > deadline) fail(failure());
    if (!g_main_context_iteration(nullptr, false)) g_usleep(5000);
  }
}

/// Descendants of `widget` of type `type`, in tree order; removed widgets are not in the tree.
void collect(GtkWidget *widget, GType type, std::vector<GtkWidget *> &found) {
  for (GtkWidget *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
    if (G_TYPE_CHECK_INSTANCE_TYPE(child, type)) found.push_back(child);
    collect(child, type, found);
  }
}

std::vector<GtkWidget *> all(GtkWidget *within, GType type) {
  std::vector<GtkWidget *> found;
  collect(within, type, found);
  return found;
}

GtkWidget *text(GtkWidget *within, std::string_view value) {
  for (GtkWidget *label : all(within, GTK_TYPE_LABEL)) {
    if (gtk_label_get_text(GTK_LABEL(label)) == value) return label;
  }
  return nullptr;
}

void wait_for_text(std::string_view value) {
  wait_for([&] { return text(root, value) != nullptr; }, [&] { return "no text \"" + std::string(value) + "\""; });
}

/// Buttons whose content holds the text `value`, in order.
std::vector<GtkWidget *> buttons_with_text(std::string_view value) {
  std::vector<GtkWidget *> buttons;
  for (GtkWidget *button : all(root, GTK_TYPE_BUTTON)) {
    if (!GTK_IS_CHECK_BUTTON(button) && text(button, value)) buttons.push_back(button);
  }
  return buttons;
}

GtkWidget *button_with_text(std::string_view value) {
  auto buttons = buttons_with_text(value);
  if (buttons.empty()) fail("no button \"" + std::string(value) + "\"");
  return buttons.front();
}

/// Activates `widget` as a click or the space key would.
void press(GtkWidget *widget) { gtk_widget_activate(widget); }

GtkWidget *checkbox(size_t index) {
  auto boxes = all(root, GTK_TYPE_CHECK_BUTTON);
  if (index >= boxes.size()) fail("no checkbox " + std::to_string(index));
  return boxes[index];
}

bool checked(GtkWidget *box) { return gtk_check_button_get_active(GTK_CHECK_BUTTON(box)); }

std::string value(GtkWidget *field) { return gtk_editable_get_text(GTK_EDITABLE(field)); }

/// Types `text` one character at a time at the end of the field, each an
/// insertion into its buffer as a key press makes it.
void type(GtkWidget *field, std::string_view text) {
  for (char c : text) {
    int position = int(g_utf8_strlen(gtk_editable_get_text(GTK_EDITABLE(field)), -1));
    gtk_editable_insert_text(GTK_EDITABLE(field), &c, 1, &position);
  }
}

}  // namespace

int main() {
  if (!gtk_init_check()) fail("UI tests need a display (DISPLAY or WAYLAND_DISPLAY)");
  GtkWidget *window = gtk_window_new();
  root = weft::start_app();
  gtk_window_set_child(GTK_WINDOW(window), root);
  gtk_window_set_default_size(GTK_WINDOW(window), 480, 640);
  gtk_window_present(GTK_WINDOW(window));
  wait_for([&] { return gtk_widget_get_mapped(window); }, [] { return "the window is not shown"; });

  // Button and `if`.
  press(button_with_text("Click me"));
  wait_for_text("You clicked!");

  // A controlled input: drafts longer than 12 characters are refused and rolled back.
  auto fields = all(root, GTK_TYPE_ENTRY);
  check(!fields.empty(), "no input");
  GtkWidget *field = fields.front();
  GtkWidget *add = button_with_text("Add");
  check(!gtk_widget_get_sensitive(add), "Add is enabled without a draft");
  type(field, "Buy milk and bread");
  check(value(field) == "Buy milk and", "the input shows \"" + value(field) + "\"");
  check(gtk_widget_get_sensitive(add), "Add stays disabled");

  // `for` rows.
  press(add);
  wait_for_text("Buy milk and");
  wait_for([&] { return value(field).empty(); }, [&] { return "the input shows \"" + value(field) + "\""; });
  type(field, "Eggs");
  wait_for([&] { return gtk_widget_get_sensitive(add); }, [] { return "Add stays disabled"; });
  press(add);
  wait_for_text("2 todos");

  // Checkboxes in the rows.
  GtkWidget *first = checkbox(0);
  check(!checked(first), "the first checkbox starts checked");
  press(first);
  wait_for([&] { return checked(first); }, [] { return "the first checkbox stays unchecked"; });
  check(!checked(checkbox(1)), "the second checkbox is checked");

  // Removing the first row keeps the second.
  press(buttons_with_text("Remove").at(0));
  wait_for_text("1 todos");
  check(!text(root, "Buy milk and"), "the removed row is shown");
  check(text(root, "Eggs"), "the kept row is gone");
  check(!checked(checkbox(0)), "the kept row's checkbox is checked");

  gtk_window_destroy(GTK_WINDOW(window));
  std::puts("click flow passed");
  return 0;
}
