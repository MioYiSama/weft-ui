#include "assets.hpp"

#include <fontconfig/fontconfig.h>
#include <pango/pangocairo.h>
#include <pango/pangofc-fontmap.h>

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <utility>

namespace weft {

const std::string &weft_directory() {
  static const auto directory = [] {
    g_autofree char *executable = g_file_read_link("/proc/self/exe", nullptr);
    g_autofree char *parent = g_path_get_dirname(executable ? executable : ".");
    return std::string(parent) + "/weft";
  }();
  return directory;
}

const std::string &asset_directory() {
  static const auto directory = weft_directory() + "/assets";
  return directory;
}

void register_asset_fonts() {
  [[maybe_unused]] static const bool registered = [] {
    const auto fonts = asset_directory() + "/fonts";
    if (!g_file_test(fonts.c_str(), G_FILE_TEST_IS_DIR)) return false;
    if (!FcConfigAppFontAddDir(nullptr, reinterpret_cast<const FcChar8 *>(fonts.c_str()))) {
#ifndef NDEBUG
      std::fprintf(stderr, "[weft warn] weft: cannot register fonts in %s\n", fonts.c_str());
#endif
      return false;
    }
    // GTK may have cached Fontconfig matches before the app session started.
    auto *font_map = pango_cairo_font_map_get_default();
    if (PANGO_IS_FC_FONT_MAP(font_map)) pango_fc_font_map_config_changed(PANGO_FC_FONT_MAP(font_map));
    return true;
  }();
}

std::optional<Asset> find_asset(const std::string &directory, std::string_view name, int scale) {
  if (name.empty() || name.find_first_not_of("abcdefghijklmnopqrstuvwxyz0123456789_") != name.npos) {
    return std::nullopt;
  }
  std::string path;
  path.reserve(directory.size() + 1 + name.size() + 8);
  path.append(directory).append("/").append(name);
  const auto stem = path.size();
  path.append(".svg");
  if (g_file_test(path.c_str(), G_FILE_TEST_IS_REGULAR)) return Asset{std::move(path), 1};

  const auto raster = [&](int density) -> bool {
    constexpr std::string_view suffixes[] = {"", "@2x", "@3x"};
    path.resize(stem);
    path.append(suffixes[density - 1]);
    const auto basename = path.size();
    for (auto extension : {".png", ".jpg", ".jpeg", ".webp"}) {
      path.resize(basename);
      path.append(extension);
      if (g_file_test(path.c_str(), G_FILE_TEST_IS_REGULAR)) return true;
    }
    return false;
  };
  const int target = std::clamp(scale, 1, 3);
  for (int density = target; density <= 3; ++density) {
    if (raster(density)) return Asset{std::move(path), density};
  }
  for (int density = target - 1; density >= 1; --density) {
    if (raster(density)) return Asset{std::move(path), density};
  }
  return std::nullopt;
}

namespace {

// The paintable keeps the SVG's natural size stable when the picture is
// resized. GtkPicture's snapshot supplies the fitted (and possibly cropped)
// logical size, which we convert to physical pixels for the SVG loader.
struct WeftSvg {
  GObject parent_instance;
  char *path;
  GdkTexture *texture;
  int width, height, scale;
  int raster_width, raster_height;
};

struct WeftSvgClass {
  GObjectClass parent_class;
};

bool rasterize(WeftSvg *svg, int width, int height, GError **error) {
  svg->raster_width = width;
  svg->raster_height = height;
  g_autoptr(GdkPixbuf) pixels = gdk_pixbuf_new_from_file_at_scale(svg->path, width, height, true, error);
  if (!pixels) return false;
  g_clear_object(&svg->texture);
  svg->texture = gdk_texture_new_for_pixbuf(pixels);
  return true;
}

void svg_interface(GdkPaintableInterface *paintable) {
  paintable->get_flags = [](GdkPaintable *) {
    return GdkPaintableFlags(GDK_PAINTABLE_STATIC_SIZE | GDK_PAINTABLE_STATIC_CONTENTS);
  };
  paintable->get_intrinsic_width = [](GdkPaintable *paintable) { return reinterpret_cast<WeftSvg *>(paintable)->width; };
  paintable->get_intrinsic_height = [](GdkPaintable *paintable) { return reinterpret_cast<WeftSvg *>(paintable)->height; };
  paintable->snapshot = [](GdkPaintable *paintable, GdkSnapshot *snapshot, double width, double height) {
    auto *svg = reinterpret_cast<WeftSvg *>(paintable);
    if (width <= 0 || height <= 0) return;
    const auto pixels = [svg](double logical) { return int(std::clamp(std::ceil(logical * svg->scale), 1.0, double(G_MAXINT))); };
    const int w = pixels(width), h = pixels(height);
    if (w != svg->raster_width || h != svg->raster_height) {
      // A failed resize can still display the last successful rasterization.
      rasterize(svg, w, h, nullptr);
    }
    gdk_paintable_snapshot(GDK_PAINTABLE(svg->texture), snapshot, width, height);
  };
}

G_DEFINE_TYPE_WITH_CODE(WeftSvg, weft_svg, G_TYPE_OBJECT,
                        G_IMPLEMENT_INTERFACE(GDK_TYPE_PAINTABLE, svg_interface))

void weft_svg_init(WeftSvg *) {}

void weft_svg_class_init(WeftSvgClass *klass) {
  G_OBJECT_CLASS(klass)->finalize = [](GObject *object) {
    auto *svg = reinterpret_cast<WeftSvg *>(object);
    g_clear_object(&svg->texture);
    g_free(svg->path);
    G_OBJECT_CLASS(weft_svg_parent_class)->finalize(object);
  };
}

}  // namespace

GdkPaintable *svg_paintable(const char *path, int scale, GError **error) {
  int width = 0, height = 0;
  if (!gdk_pixbuf_get_file_info(path, &width, &height) || width <= 0 || height <= 0) {
    g_set_error_literal(error, GDK_PIXBUF_ERROR, GDK_PIXBUF_ERROR_CORRUPT_IMAGE, "cannot read SVG dimensions");
    return nullptr;
  }
  auto *svg = static_cast<WeftSvg *>(g_object_new(weft_svg_get_type(), nullptr));
  svg->path = g_strdup(path);
  svg->width = width;
  svg->height = height;
  svg->scale = scale;
  // Validate the file before exposing a paintable; defer full-size rendering
  // until GTK knows the viewport, rather than rasterizing at intrinsic size.
  if (rasterize(svg, std::min(width, 64), std::min(height, 64), error)) return GDK_PAINTABLE(svg);
  g_object_unref(svg);
  return nullptr;
}

}  // namespace weft
