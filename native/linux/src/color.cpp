#include "weft/gtk.hpp"

#include <algorithm>
#include <cstdio>

#include "appearance.hpp"

namespace weft {
namespace {

std::span<const theme::Entry> entries;

void warn_color(const char *reason, std::string_view text) {
#ifndef NDEBUG
  std::fprintf(stderr, "[weft warn] weft: %s %.*s, using transparent\n", reason, int(text.size()),
               text.empty() ? "" : text.data());
#else
  (void)reason;
  (void)text;
#endif
}

}  // namespace

void theme::set(std::span<const Entry> colors) { entries = colors; }

Color Color::token(std::string_view name) {
  auto entry = std::ranges::find(entries, name, &theme::Entry::name);
  if (entry != entries.end()) return Color(&*entry);
  warn_color("unknown theme token", name);
  return rgba(0);
}

Color Color::parse(std::string_view text) {
  if (text.starts_with('@')) return token(text.substr(1));
  if ((text.size() == 4 || text.size() == 7 || text.size() == 9) && text[0] == '#') {
    uint32_t value = 0;
    for (char c : text.substr(1)) {
      int digit = g_ascii_xdigit_value(c);
      if (digit < 0) {
        warn_color("invalid color", text);
        return rgba(0);
      }
      value = value << 4 | uint32_t(digit);
    }
    if (text.size() == 4) {
      value = (value >> 8 & 0xf) * 17 << 24 | (value >> 4 & 0xf) * 17 << 16 | (value & 0xf) * 17 << 8 | 0xff;
    } else if (text.size() == 7) {
      value = value << 8 | 0xff;
    }
    return rgba(value);
  }
  warn_color("invalid color", text);
  return rgba(0);
}

GdkRGBA Color::resolve() const { return resolve(dark()); }

GdkRGBA Color::resolve(bool dark) const {
  const auto *token = std::get_if<const theme::Entry *>(&value_);
  uint32_t rgba = token ? (dark ? (*token)->dark : (*token)->light) : std::get<uint32_t>(value_);
  return {float(rgba >> 24) / 255.f, float(rgba >> 16 & 0xff) / 255.f, float(rgba >> 8 & 0xff) / 255.f,
          float(rgba & 0xff) / 255.f};
}

Color color(const ops::Value &value) { return Color::parse(string(value)); }

}  // namespace weft
