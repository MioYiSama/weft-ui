#include "weft/gtk.hpp"

#include <algorithm>
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

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_setenv("GTK_A11Y", "none", false);
  if (!gtk_init_check()) return 77;
  g_test_add_func("/controls/picker-strings", picker_strings);
  g_test_add_func("/controls/slider-rounding", slider_rounding);
  return g_test_run();
}
