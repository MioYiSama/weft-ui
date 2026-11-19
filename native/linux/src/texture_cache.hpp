#pragma once

#include <gdk/gdk.h>

#include <cstddef>
#include <string>
#include <unordered_map>

namespace weft {

class TextureCache {
 public:
  explicit TextureCache(std::size_t capacity);

  TextureCache(const TextureCache &) = delete;
  TextureCache &operator=(const TextureCache &) = delete;

  // Borrows the cache's reference, or returns nullptr on a miss.
  GdkTexture *find(const std::string &source);
  // Keeps its own reference if inserted. An existing source keeps its texture.
  void insert(std::string &&source, GdkTexture *texture);

 private:
  // Link the map nodes directly: their keys and values stay put on rehash.
  struct Entry {
    Entry(GdkTexture *texture, std::size_t bytes);
    ~Entry();
    Entry(const Entry &) = delete;
    Entry &operator=(const Entry &) = delete;

    GdkTexture *texture;
    std::size_t bytes;
    const std::string *source = nullptr;
    Entry *older = nullptr;
    Entry *newer = nullptr;
  };

  void unlink(Entry &entry);
  void append(Entry &entry);
  void touch(Entry &entry);

  std::unordered_map<std::string, Entry> entries_;
  Entry *oldest_ = nullptr;
  Entry *newest_ = nullptr;
  std::size_t used_ = 0;
  std::size_t capacity_;
};

}  // namespace weft
