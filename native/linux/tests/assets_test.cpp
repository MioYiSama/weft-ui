#include <fontconfig/fontconfig.h>

#include "assets.hpp"
#include "weft/gtk.hpp"

namespace {

void variant(std::string_view name, int scale, int density, const char *file) {
  const auto &directory = weft::asset_directory();
  const auto asset = weft::find_asset(directory, name, scale);
  const auto expected = directory + "/" + file;
  g_assert_true(asset.has_value());
  g_assert_cmpint(asset->density, ==, density);
  g_assert_cmpstr(asset->path.c_str(), ==, expected.c_str());
}

void selects_density() {
  variant("logo", 1, 1, "logo.png");
  variant("logo", 2, 2, "logo@2x.png");
  variant("logo", 3, 2, "logo@2x.png");
  variant("logo", 4, 2, "logo@2x.png");
  variant("only_high", 1, 2, "only_high@2x.png");
  variant("star", 3, 1, "star.svg");
  g_assert_false(weft::find_asset(weft::asset_directory(), "missing", 1).has_value());
  g_assert_false(weft::find_asset(weft::asset_directory(), "../assets/logo", 1).has_value());
}

void image_and_placeholder() {
  weft::Image image;
  auto *widget = image.widget();
  auto *picture = GTK_PICTURE(gtk_widget_get_first_child(widget));
  g_assert_cmpint(gtk_picture_get_content_fit(picture), ==, GTK_CONTENT_FIT_COVER);
  image.set_fit(weft::ImageFit::Contain);
  g_assert_cmpint(gtk_picture_get_content_fit(picture), ==, GTK_CONTENT_FIT_CONTAIN);
  image.set_fit(weft::ImageFit::Fill);
  g_assert_cmpint(gtk_picture_get_content_fit(picture), ==, GTK_CONTENT_FIT_FILL);
  image.set_src("asset:only_high");
  auto *paintable = gtk_picture_get_paintable(picture);
  g_assert_nonnull(paintable);
  g_assert_cmpint(gdk_paintable_get_intrinsic_width(paintable), ==, 128);
  int natural = 0;
  gtk_widget_measure(widget, GTK_ORIENTATION_HORIZONTAL, -1, nullptr, &natural, nullptr, nullptr);
  g_assert_cmpint(natural, ==, 64);
  gtk_widget_measure(widget, GTK_ORIENTATION_VERTICAL, -1, nullptr, &natural, nullptr, nullptr);
  g_assert_cmpint(natural, ==, 40);
  image.set_height(20);
  gtk_widget_measure(widget, GTK_ORIENTATION_HORIZONTAL, -1, nullptr, &natural, nullptr, nullptr);
  g_assert_cmpint(natural, ==, 32);

  image.set_width(32);
  image.set_background(weft::Color::rgba(0x334455ff));
  image.set_placeholder(weft::Color::rgba(0xffbb66ff));
  image.set_src("asset:missing");
  g_assert_null(gtk_picture_get_paintable(picture));
  gtk_widget_allocate(widget, 32, 20, -1, nullptr);
  auto *snapshot = gtk_snapshot_new();
  GTK_WIDGET_GET_CLASS(widget)->snapshot(widget, snapshot);
  g_autoptr(GskRenderNode) rendered = gtk_snapshot_free_to_node(snapshot);
  g_assert_nonnull(rendered);
  g_assert_cmpint(gsk_render_node_get_node_type(rendered), ==, GSK_COLOR_NODE);
  const GdkRGBA *color = gsk_color_node_get_color(rendered);
  g_assert_cmpfloat_with_epsilon(color->red, 1, 0.001);
  g_assert_cmpfloat_with_epsilon(color->green, 187 / 255.0, 0.001);
  g_assert_cmpfloat_with_epsilon(color->blue, 102 / 255.0, 0.001);
}

void svg_uses_target_pixels() {
  auto path = weft::asset_directory() + "/star.svg";
  g_autoptr(GError) error = nullptr;
  g_autoptr(GdkPaintable) svg = weft::svg_paintable(path.c_str(), 2, &error);
  g_assert_no_error(error);
  g_assert_nonnull(svg);
  g_assert_cmpint(gdk_paintable_get_intrinsic_width(svg), ==, 32);
  g_assert_cmpfloat(gdk_paintable_get_intrinsic_aspect_ratio(svg), ==, 1);
  auto *snapshot = gtk_snapshot_new();
  gdk_paintable_snapshot(svg, snapshot, 27, 27);
  g_autoptr(GskRenderNode) rendered = gtk_snapshot_free_to_node(snapshot);
  g_assert_nonnull(rendered);
  g_assert_cmpint(gsk_render_node_get_node_type(rendered), ==, GSK_TEXTURE_NODE);
  auto *texture = gsk_texture_node_get_texture(rendered);
  g_assert_cmpint(gdk_texture_get_width(texture), ==, 54);
  g_assert_cmpint(gdk_texture_get_height(texture), ==, 54);
}

void registers_fonts() {
  weft::register_asset_fonts();
  const auto expected = weft::asset_directory() + "/fonts/Inter;Example-Regular.ttf";
  const auto *fonts = FcConfigGetFonts(FcConfigGetCurrent(), FcSetApplication);
  g_assert_nonnull(fonts);
  for (int i = 0; i < fonts->nfont; i++) {
    FcChar8 *file = nullptr;
    if (FcPatternGetString(fonts->fonts[i], FC_FILE, 0, &file) == FcResultMatch &&
        expected == reinterpret_cast<const char *>(file)) return;
  }
  g_error("Expected %s in Fontconfig's app fonts", expected.c_str());
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_setenv("GTK_A11Y", "none", false);
  if (!gtk_init_check()) return 77;
  g_test_add_func("/assets/density", selects_density);
  g_test_add_func("/assets/image-and-placeholder", image_and_placeholder);
  g_test_add_func("/assets/svg-target-pixels", svg_uses_target_pixels);
  g_test_add_func("/assets/register-fonts", registers_fonts);
  return g_test_run();
}
