#pragma once

#include <gtk/gtk.h>

#include <optional>
#include <string>
#include <string_view>

namespace weft {

struct Asset {
  std::string path;
  int density;
};

// The bundle and generated assets are relative to the executable, not cwd.
const std::string &weft_directory();
void register_asset_fonts();
std::optional<Asset> find_asset(const std::string &directory, std::string_view name, int scale);
const std::string &asset_directory();

// Owns an SVG's intrinsic size and rasterizes it for the snapshot's pixel size.
GdkPaintable *svg_paintable(const char *path, int scale, GError **error);

}  // namespace weft
