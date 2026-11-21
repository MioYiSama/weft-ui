// UI tests of the app, driving its widgets in process; run with
// `ctest --test-dir build` on a desktop session after building the app.

#include <gtk/gtk.h>
#include <weft/gtk.hpp>

#include <cstdio>
#include <cstdlib>
#include <string>
#include <string_view>
#include <vector>

namespace {

void check(bool passed, const char *message) {
  if (passed) return;
  std::fprintf(stderr, "%s\n", message);
  std::exit(1);
}

void pump() {
  while (g_main_context_iteration(nullptr, false)) {}
}

void collect(GtkWidget *widget, GType type, std::vector<GtkWidget *> &out) {
  if (g_type_is_a(G_OBJECT_TYPE(widget), type)) out.push_back(widget);
  for (auto *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
    collect(child, type, out);
  }
}

std::vector<GtkWidget *> all(GtkWidget *root, GType type, size_t count) {
  std::vector<GtkWidget *> found;
  collect(root, type, found);
  if (found.size() != count) {
    std::fprintf(stderr, "Expected %zu %s widgets, found %zu\n", count, g_type_name(type), found.size());
    std::exit(1);
  }
  return found;
}

GtkWidget *label(GtkWidget *widget, std::string_view text) {
  if (GTK_IS_LABEL(widget) && std::string_view(gtk_label_get_text(GTK_LABEL(widget))) == text) return widget;
  for (auto *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
    if (auto *found = label(child, text)) return found;
  }
  return nullptr;
}

void click(GtkWidget *root, std::string_view text) {
  auto *caption = label(root, text);
  auto *button = caption ? gtk_widget_get_ancestor(caption, GTK_TYPE_BUTTON) : nullptr;
  if (!button) {
    std::fprintf(stderr, "Button '%.*s' missing\n", int(text.size()), text.data());
    std::exit(1);
  }
  g_signal_emit_by_name(button, "clicked");
  pump();
}

bool active(GtkWidget *widget, bool value) {
  auto *control = GTK_SWITCH(widget);
  return gtk_switch_get_active(control) == value && gtk_switch_get_state(control) == value;
}

std::string_view choice(GtkWidget *picker) {
  return gtk_label_get_text(GTK_LABEL(gtk_menu_button_get_child(GTK_MENU_BUTTON(picker))));
}

void select(GtkWidget *picker, unsigned index) {
  auto *menu = GTK_MENU_BUTTON(picker);
  gtk_menu_button_popup(menu);
  auto *popover = gtk_menu_button_get_popover(menu);
  auto *list = gtk_popover_get_child(popover);
  auto *option = gtk_widget_get_first_child(list);
  while (index-- && option) option = gtk_widget_get_next_sibling(option);
  check(option && GTK_IS_BUTTON(option), "Picker option missing");
  g_signal_emit_by_name(option, "clicked");
  check(!gtk_widget_get_visible(GTK_WIDGET(popover)), "Picker popover did not close");
}

void type(GtkWidget *area, const char *text) {
  g_signal_emit_by_name(area, "insert-at-cursor", text);
}

bool contains(GtkWidget *area, std::string_view expected) {
  auto *buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(area));
  GtkTextIter begin, end;
  gtk_text_buffer_get_bounds(buffer, &begin, &end);
  g_autofree char *text = gtk_text_buffer_get_text(buffer, &begin, &end, true);
  return expected == text;
}

void controls(GtkWidget *root) {
  click(root, "page:controls");

  const auto switches = all(root, GTK_TYPE_SWITCH, 4);
  const auto checkboxes = all(root, GTK_TYPE_CHECK_BUTTON, 2);
  const auto scales = all(root, GTK_TYPE_SCALE, 2);
  const auto pickers = all(root, GTK_TYPE_MENU_BUTTON, 3);
  const auto textareas = all(root, GTK_TYPE_TEXT_VIEW, 2);
  all(root, g_type_from_name("WeftSpinner"), 1);
  all(root, g_type_from_name("WeftProgress"), 2);
  const auto dividers = all(root, GTK_TYPE_SEPARATOR, 4);
  check(gtk_orientable_get_orientation(GTK_ORIENTABLE(dividers[1])) == GTK_ORIENTATION_HORIZONTAL &&
        gtk_orientable_get_orientation(GTK_ORIENTABLE(dividers[2])) == GTK_ORIENTATION_VERTICAL,
        "Expected horizontal and vertical dividers");
  check(!gtk_widget_is_sensitive(switches.back()) && !gtk_widget_is_sensitive(checkboxes.back()) &&
        !gtk_widget_is_sensitive(scales.back()) && !gtk_widget_is_sensitive(pickers.back()) &&
        !gtk_widget_is_sensitive(textareas.back()), "Disabled controls are interactive");

  check(active(switches[0], false) && active(switches[1], true) && active(switches[2], false),
        "Wrong initial switch values");
  auto *checkbox = GTK_CHECK_BUTTON(checkboxes[0]);
  auto *scale = GTK_RANGE(scales[0]);
  check(gtk_check_button_get_active(checkbox), "Wrong initial checkbox value");
  check(gtk_range_get_value(scale) == 40, "Wrong initial slider value");
  check(choice(pickers[0]) == "本地" && choice(pickers[1]) == "选择频道", "Wrong initial Picker labels");
  check(contains(textareas[0], ""), "Wrong initial TextArea value");

  gtk_widget_activate(switches[1]);
  gtk_widget_activate(switches[2]);
  gtk_widget_activate(checkboxes[0]);
  gtk_range_set_value(scale, 62.5);
  select(pickers[0], 1);
  type(textareas[0], "First\nSecond");
  check(active(switches[1], false) && active(switches[2], true), "Unlocked switches rejected edits");
  check(!gtk_check_button_get_active(checkbox), "Unlocked checkbox rejected edit");
  check(gtk_range_get_value(scale) == 65, "Unlocked slider did not snap to 65");
  check(choice(pickers[0]) == "远程", "Unlocked Picker rejected an option");
  check(contains(textareas[0], "First\nSecond"), "Unlocked TextArea rejected multiline input");

  gtk_widget_activate(switches[0]);
  check(active(switches[0], true), "Could not enable lock");
  gtk_widget_activate(switches[1]);
  gtk_widget_activate(switches[2]);
  gtk_widget_activate(checkboxes[0]);
  gtk_range_set_value(scale, 95);
  select(pickers[0], 0);
  select(pickers[1], 1);
  type(textareas[0], " rejected");
  pump();
  check(active(switches[1], false) && active(switches[2], true), "Locked switches did not restore");
  check(!gtk_check_button_get_active(checkbox), "Locked checkbox did not restore");
  check(gtk_range_get_value(scale) == 65, "Locked slider did not restore");
  check(choice(pickers[0]) == "远程" && choice(pickers[1]) == "选择频道", "Locked pickers did not restore");
  check(contains(textareas[0], "First\nSecond"), "Locked TextArea did not restore");

  gtk_widget_activate(switches[0]);
  select(pickers[1], 1);
  check(active(switches[0], false) && choice(pickers[1]) == "测试版", "Picker could not select after unlocking");
}

void inputs(GtkWidget *root) {
  click(root, "page:inputs");
  const auto entries = all(root, GTK_TYPE_ENTRY, 5);
  const auto textareas = all(root, GTK_TYPE_TEXT_VIEW, 1);
  auto *draft = gtk_editable_get_delegate(GTK_EDITABLE(entries[0]));
  auto *window = gtk_widget_get_root(root);
  auto expect_focus = [&](GtkWidget *widget, std::string_view message_label) {
    check(gtk_root_get_focus(window) == widget, "Unexpected keyboard focus");
    check(label(root, message_label), "onFocusChange did not update the label");
  };
  check(!gtk_entry_get_visibility(GTK_ENTRY(entries[1])), "Password input is not masked");
  check(gtk_entry_get_input_purpose(GTK_ENTRY(entries[2])) == GTK_INPUT_PURPOSE_NUMBER &&
        gtk_entry_get_input_purpose(GTK_ENTRY(entries[3])) == GTK_INPUT_PURPOSE_EMAIL &&
        gtk_entry_get_input_purpose(GTK_ENTRY(entries[4])) == GTK_INPUT_PURPOSE_PHONE,
        "Wrong keyboard purposes");

  click(root, "聚焦文本框");
  expect_focus(GTK_WIDGET(draft), "已聚焦");
  gtk_widget_grab_focus(entries[1]);
  pump();
  auto *password = gtk_editable_get_delegate(GTK_EDITABLE(entries[1]));
  expect_focus(GTK_WIDGET(password), "未聚焦");
  click(root, "放弃焦点");  // User focus changes must not write back to requestFocus.
  expect_focus(GTK_WIDGET(password), "未聚焦");
  click(root, "聚焦文本框");
  expect_focus(GTK_WIDGET(draft), "已聚焦");
  click(root, "放弃焦点");
  expect_focus(nullptr, "未聚焦");
  click(root, "聚焦文本框");
  expect_focus(GTK_WIDGET(draft), "已聚焦");

  g_signal_emit_by_name(draft, "insert-at-cursor", "1234567890123456789😀z");
  check(std::string_view(gtk_editable_get_text(draft)) == "1234567890123456789" && label(root, "19/20"),
        "Input split the emoji at the UTF-16 boundary");
  g_signal_emit_by_name(draft, "activate");  // GtkText's Return key binding.
  check(*gtk_editable_get_text(draft) == '\0' && label(root, "0/20"), "onSubmit did not clear the draft");

  g_signal_emit_by_name(draft, "insert-at-cursor", "123456789012345678😀z");
  check(std::string_view(gtk_editable_get_text(draft)) == "123456789012345678😀" && label(root, "20/20"),
        "Input did not count the preserved emoji as two UTF-16 units");
  gtk_widget_grab_focus(textareas.front());
  pump();
  expect_focus(textareas.front(), "未聚焦");
  const std::string expected = std::string(78, 'a') + '\n';
  type(textareas.front(), (expected + "😀z").c_str());
  check(contains(textareas.front(), expected), "TextArea split the emoji at the UTF-16 boundary");
}

void run(void (*verify)(GtkWidget *)) {
  GtkWidget *window = gtk_window_new();
  GtkWidget *root = weft::start_app();
  gtk_window_set_child(GTK_WINDOW(window), root);
  gtk_window_set_default_size(GTK_WINDOW(window), 390, 844);
  gtk_window_present(GTK_WINDOW(window));
  pump();

  // The first batch is applied when start_app returns; a failed launch
  // shows its reason in a label styled `error` instead of the catalog.
  GtkWidget *first = gtk_widget_get_first_child(root);
  check(first && !gtk_widget_has_css_class(first, "error"), "The app did not start");
  verify(root);
  gtk_window_destroy(GTK_WINDOW(window));
  pump();
}

}  // namespace

int main() {
  if (!gtk_init_check()) {
    std::fprintf(stderr, "UI tests need a display (DISPLAY or WAYLAND_DISPLAY)\n");
    return 1;
  }
  g_object_set(gtk_settings_get_default(), "gtk-enable-animations", false, nullptr);
  run(controls);
  run(inputs);
  return 0;
}
