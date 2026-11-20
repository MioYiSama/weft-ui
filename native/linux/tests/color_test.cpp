#include "weft/gtk.hpp"

namespace {

void check_rgba(const GdkRGBA &actual, const GdkRGBA &expected) {
  g_assert_true(gdk_rgba_equal(&actual, &expected));
}

void fixed_colors() {
  for (bool dark : {false, true}) {
    check_rgba(weft::Color::parse("#fAb").resolve(dark), {1, 170.f / 255, 187.f / 255, 1});
    check_rgba(weft::Color::parse("#00fF00").resolve(dark), {0, 1, 0, 1});
    check_rgba(weft::Color::parse("#FF000080").resolve(dark), {1, 0, 0, 128.f / 255});
    check_rgba(weft::Color::rgba(0x008000ffu).resolve(dark), {0, 128.f / 255, 0, 1});
  }
}

void themed_colors() {
  static constexpr weft::theme::Entry entries[] = {{"accent", 0xff0000ffu, 0x0000ff80u}};
  weft::theme::set(entries);
  const auto constant = weft::Color::token("accent");
  const auto slot = weft::color(weft::ops::Value{std::string_view("@accent")});
  for (auto color : {constant, slot}) {
    check_rgba(color.resolve(), {1, 0, 0, 1});
    check_rgba(color.resolve(false), {1, 0, 0, 1});
    check_rgba(color.resolve(true), {0, 0, 1, 128.f / 255});
  }
  weft::theme::set({});
}

void invalid_colors() {
  for (const char *invalid : {"", "#invalid!", "red", "@unknown"}) {
    const auto color = weft::Color::parse(invalid);
    check_rgba(color.resolve(false), {});
    check_rgba(color.resolve(true), {});
  }
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_test_add_func("/color/fixed", fixed_colors);
  g_test_add_func("/color/themed", themed_colors);
  g_test_add_func("/color/invalid", invalid_colors);
  return g_test_run();
}
