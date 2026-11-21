#include "weft/gtk.hpp"

#include <algorithm>
#include <string>
#include <utility>
#include <vector>

namespace {

void picker_strings() {
  weft::Picker picker([](double) { return false; });
  auto *button = GTK_MENU_BUTTON(picker.widget());
  auto *label = GTK_LABEL(gtk_menu_button_get_child(button));
  picker.set_placeholder("Choose an option");
  picker.set_selected(1);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "Choose an option");
  {
    uint8_t encoded[] = {uint8_t(weft::protocol::Tag::Strings), 2, 0, 0, 0,
                         3, 0, 0, 0, 'o', 'n', 'e',
                         2, 0, 0, 0, 0xc3, 0xa9};
    const uint8_t *at = encoded;
    picker.set_options(weft::strings(weft::ops::read_value(at, encoded + sizeof(encoded))));
    std::ranges::fill(encoded, 0);
  }
  g_assert_cmpstr(gtk_label_get_text(label), ==, "é");
  picker.set_selected(0);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "one");
  picker.set_selected(2);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "Choose an option");
  picker.set_selected(-1);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "Choose an option");
  picker.set_selected(1);
  picker.set_options({});
  g_assert_cmpstr(gtk_label_get_text(label), ==, "Choose an option");
}

void slider_rounding() {
  std::vector<double> edits;
  weft::Slider *controlled = nullptr;
  weft::Slider slider([&](double value) {
    edits.push_back(value);
    if (value == 13) return false;
    controlled->set_value(value);
    return true;
  });
  controlled = &slider;
  auto *range = GTK_RANGE(slider.widget());
  slider.set_min(-2);
  slider.set_max(17);
  slider.set_step(5);
  slider.set_value(3);
  g_assert_true(edits.empty());
  g_assert_cmpfloat(gtk_range_get_value(range), ==, 3);

  // Halfway between steps 3 and 8; round up, anchored at min, not 0.
  gtk_range_set_value(range, 5.5);
  g_assert_cmpuint(edits.size(), ==, 1);
  g_assert_cmpfloat(edits.back(), ==, 8);
  g_assert_cmpfloat(gtk_range_get_value(range), ==, 8);
  gtk_range_set_value(range, 8.25);
  g_assert_cmpuint(edits.size(), ==, 1);
  g_assert_cmpfloat(gtk_range_get_value(range), ==, 8);

  // The upper bound is not itself a valid step, and 13 is rejected.
  gtk_range_set_value(range, 17);
  g_assert_cmpuint(edits.size(), ==, 2);
  g_assert_cmpfloat(edits.back(), ==, 13);
  g_assert_cmpfloat(gtk_range_get_value(range), ==, 8);
  slider.set_max(4);
  g_assert_cmpuint(edits.size(), ==, 2);
  g_assert_cmpfloat(gtk_range_get_value(range), ==, 3);
}

void input_type_and_submit() {
  int submits = 0;
  weft::Input input([](std::string_view) { return false; }, [&] { ++submits; });
  auto *entry = GTK_ENTRY(input.widget());
  using weft::InputType;
  const std::pair<InputType, GtkInputPurpose> types[] = {
      {InputType::Number, GTK_INPUT_PURPOSE_NUMBER},
      {InputType::Email, GTK_INPUT_PURPOSE_EMAIL},
      {InputType::Phone, GTK_INPUT_PURPOSE_PHONE},
      {InputType::Password, GTK_INPUT_PURPOSE_PASSWORD},
      {InputType::Text, GTK_INPUT_PURPOSE_FREE_FORM},
  };
  input.set_value("unchanged");
  for (auto [type, purpose] : types) {
    input.set_type(type);
    g_assert_cmpint(gtk_entry_get_input_purpose(entry), ==, purpose);
    g_assert_cmpint(gtk_entry_get_visibility(entry), ==, type != InputType::Password);
    g_assert_cmpstr(gtk_editable_get_text(GTK_EDITABLE(entry)), ==, "unchanged");
  }
  auto *delegate = gtk_editable_get_delegate(GTK_EDITABLE(entry));
  g_signal_emit_by_name(delegate, "preedit-changed", "composing");
  g_signal_emit_by_name(entry, "activate");
  g_assert_cmpint(submits, ==, 0);
  g_signal_emit_by_name(delegate, "preedit-changed", "");
  g_signal_emit_by_name(entry, "activate");
  g_assert_cmpint(submits, ==, 1);
}

template <class Control, class Insert, class Read>
void edit_limits(Insert insert, Read read) {
  bool accept = true;
  std::vector<std::string> edits;
  Control *controlled = nullptr;
  Control field([&](std::string_view value) {
    edits.emplace_back(value);
    if (accept) controlled->set_value(value);
    return accept;
  });
  controlled = &field;
  auto *widget = field.widget();
  field.set_max_length(5);
  field.set_value("over the limit");
  g_assert_true(read(widget) == "over the limit");
  g_assert_true(edits.empty());

  field.set_value("");
  insert(widget, "é😀bcdef");
  g_assert_true(read(widget) == "é😀bc");
  g_assert_true(edits == std::vector<std::string>{"é😀bc"});
  insert(widget, "d");
  g_assert_true(read(widget) == "é😀bc");
  g_assert_cmpuint(edits.size(), ==, 1);

  field.set_value("");
  insert(widget, "abcd😀e");
  g_assert_true(read(widget) == "abcd");
  g_assert_cmpstr(edits.back().c_str(), ==, "abcd");

  field.set_value("");
  accept = false;
  insert(widget, "é😀bcdef");
  g_assert_true(read(widget).empty());
  g_assert_cmpstr(edits.back().c_str(), ==, "é😀bc");

  accept = true;
  field.set_max_length(0);
  insert(widget, "😀");
  g_assert_true(read(widget).empty());
  g_assert_cmpuint(edits.size(), ==, 3);
  field.set_max_length(-1);
  insert(widget, "😀abcdefg");
  g_assert_true(read(widget) == "😀abcdefg");
  g_assert_cmpstr(edits.back().c_str(), ==, "😀abcdefg");
}

void input_edit_limits() {
  edit_limits<weft::Input>([](GtkWidget *widget, const char *text) {
    auto *editable = gtk_editable_get_delegate(GTK_EDITABLE(widget));
    gtk_editable_set_position(editable, -1);
    g_signal_emit_by_name(editable, "insert-at-cursor", text);
  }, [](GtkWidget *widget) -> std::string_view {
    return gtk_editable_get_text(GTK_EDITABLE(widget));
  });
}

void textarea_edit_limits() {
  auto view = [](GtkWidget *widget) {
    return gtk_scrolled_window_get_child(GTK_SCROLLED_WINDOW(gtk_overlay_get_child(GTK_OVERLAY(widget))));
  };
  edit_limits<weft::TextArea>([view](GtkWidget *widget, const char *text) {
    auto *target = view(widget);
    auto *buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(target));
    GtkTextIter end;
    gtk_text_buffer_get_end_iter(buffer, &end);
    gtk_text_buffer_place_cursor(buffer, &end);
    g_signal_emit_by_name(target, "insert-at-cursor", text);
  }, [view](GtkWidget *widget) -> std::string {
    auto *buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(view(widget)));
    GtkTextIter begin, end;
    gtk_text_buffer_get_bounds(buffer, &begin, &end);
    g_autofree char *text = gtk_text_buffer_get_text(buffer, &begin, &end, true);
    return text;
  });
}

void focus_edges() {
  std::vector<bool> input_events, area_events;
  auto reject = [](std::string_view) { return false; };
  weft::Input input(reject, nullptr, [&](bool focused) { input_events.push_back(focused); });
  weft::TextArea area(reject, [&](bool focused) { area_events.push_back(focused); });
  input.set_focused(true);  // The value can be set before mounting.
  area.set_focused(true);
  area.set_focused(false);  // A false edge cancels the pending grab.
  auto *window = GTK_WINDOW(gtk_window_new());
  auto *box = GTK_BOX(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0));
  auto *button = gtk_button_new_with_label("Other control");
  gtk_box_append(box, button);
  gtk_box_append(box, input.widget());
  gtk_box_append(box, area.widget());
  gtk_window_set_child(window, GTK_WIDGET(box));
  gtk_window_present(window);
  while (g_main_context_iteration(nullptr, false)) {}
  auto *root = GTK_ROOT(window);
  auto focus_is_in = [root](GtkWidget *widget) {
    auto *focus = gtk_root_get_focus(root);
    return focus && gtk_widget_is_ancestor(focus, widget);
  };
  g_assert_true(focus_is_in(input.widget()));
  g_assert_true(input_events == std::vector<bool>{true});
  g_assert_true(area_events.empty());

  area.set_focused(true);
  g_assert_true(focus_is_in(area.widget()));
  input.set_focused(true);
  input.set_focused(false);
  g_assert_true(focus_is_in(area.widget()));
  area.set_focused(false);
  g_assert_null(gtk_root_get_focus(root));

  input.set_focused(true);
  gtk_widget_grab_focus(button);
  input.set_focused(true);
  g_assert_true(gtk_root_get_focus(root) == button);
  input.set_focused(false);
  input.set_focused(true);
  g_assert_true(focus_is_in(input.widget()));
  input.set_focused(false);
  g_assert_null(gtk_root_get_focus(root));
  const std::vector<bool> expected_input{true, false, true, false, true, false};
  const std::vector<bool> expected_area{true, false};
  g_assert_true(input_events == expected_input);
  g_assert_true(area_events == expected_area);
  gtk_window_destroy(window);
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_setenv("GTK_A11Y", "none", false);
  if (!gtk_init_check()) return 77;
  g_test_add_func("/controls/picker-strings", picker_strings);
  g_test_add_func("/controls/slider-rounding", slider_rounding);
  g_test_add_func("/controls/input-type-and-submit", input_type_and_submit);
  g_test_add_func("/controls/input-edit-limits", input_edit_limits);
  g_test_add_func("/controls/textarea-edit-limits", textarea_edit_limits);
  g_test_add_func("/controls/focus-edges", focus_edges);
  return g_test_run();
}
