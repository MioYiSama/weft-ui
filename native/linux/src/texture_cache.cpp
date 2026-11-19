#include "texture_cache.hpp"

#include <cstdint>
#include <utility>

namespace weft {

TextureCache::TextureCache(std::size_t capacity) : capacity_(capacity) {}

TextureCache::Entry::Entry(GdkTexture *texture, std::size_t bytes) : texture(g_object_ref(texture)), bytes(bytes) {}

TextureCache::Entry::~Entry() { g_object_unref(texture); }

void TextureCache::unlink(Entry &entry) {
  if (entry.older) entry.older->newer = entry.newer;
  else oldest_ = entry.newer;
  if (entry.newer) entry.newer->older = entry.older;
  else newest_ = entry.older;
}

void TextureCache::append(Entry &entry) {
  entry.older = newest_;
  entry.newer = nullptr;
  if (newest_) newest_->newer = &entry;
  else oldest_ = &entry;
  newest_ = &entry;
}

void TextureCache::touch(Entry &entry) {
  if (&entry == newest_) return;
  unlink(entry);
  append(entry);
}

GdkTexture *TextureCache::find(const std::string &source) {
  auto it = entries_.find(source);
  if (it == entries_.end()) return nullptr;
  touch(it->second);
  return it->second.texture;
}

void TextureCache::insert(std::string &&source, GdkTexture *texture) {
  auto bytes = std::uint64_t{4} * gdk_texture_get_width(texture) * gdk_texture_get_height(texture);
  if (bytes > capacity_) return;
  auto [it, inserted] = entries_.try_emplace(std::move(source), texture, static_cast<std::size_t>(bytes));
  auto &entry = it->second;
  if (!inserted) {
    touch(entry);
    return;
  }
  while (used_ > capacity_ - entry.bytes) {
    auto &victim = *oldest_;
    unlink(victim);
    used_ -= victim.bytes;
    entries_.erase(*victim.source);
  }
  entry.source = &it->first;
  append(entry);
  used_ += entry.bytes;
}

}  // namespace weft
