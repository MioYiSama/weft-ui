#include "texture_cache.hpp"

namespace {

GdkTexture *texture(int width, int height) {
  static const guint8 pixels[16]{};
  g_autoptr(GBytes) bytes = g_bytes_new_static(pixels, sizeof(pixels));
  return gdk_memory_texture_new(width, height, GDK_MEMORY_R8G8B8A8, bytes, width * 4);
}

void evicts_oldest_by_bytes() {
  weft::TextureCache cache(16);
  g_autoptr(GdkTexture) small = texture(1, 1);
  g_autoptr(GdkTexture) tall = texture(1, 2);
  g_autoptr(GdkTexture) wide = texture(2, 1);
  cache.insert("first", small);
  cache.insert("second", tall);
  cache.insert("third", small);
  cache.insert("fourth", wide);
  g_assert_null(cache.find("first"));
  g_assert_null(cache.find("second"));
  g_assert_true(cache.find("third") == small);
  g_assert_true(cache.find("fourth") == wide);
}

void hit_refreshes_recency() {
  weft::TextureCache cache(8);
  g_autoptr(GdkTexture) image = texture(1, 1);
  cache.insert("first", image);
  cache.insert("second", image);
  g_assert_true(cache.find("first") == image);
  cache.insert("third", image);
  g_assert_null(cache.find("second"));
  g_assert_true(cache.find("first") == image);
  g_assert_true(cache.find("third") == image);
}

void rejects_oversize() {
  weft::TextureCache cache(4);
  g_autoptr(GdkTexture) small = texture(1, 1);
  GdkTexture *large = texture(2, 1);
  gpointer alive = large;
  g_object_add_weak_pointer(G_OBJECT(large), &alive);
  cache.insert("small", small);
  cache.insert("large", large);
  g_object_unref(large);
  g_assert_null(cache.find("large"));
  g_assert_null(alive);
  g_assert_true(cache.find("small") == small);
}

void releases_its_reference() {
  gpointer alive;
  gpointer remaining;
  {
    weft::TextureCache cache(4);
    GdkTexture *image = texture(1, 1);
    alive = image;
    g_object_add_weak_pointer(G_OBJECT(image), &alive);
    cache.insert("image", image);
    g_object_unref(image);
    g_assert_nonnull(alive);

    g_autoptr(GdkTexture) displayed = g_object_ref(cache.find("image"));
    GdkTexture *other = texture(1, 1);
    remaining = other;
    g_object_add_weak_pointer(G_OBJECT(other), &remaining);
    cache.insert("other", other);
    g_object_unref(other);
    g_assert_null(cache.find("image"));
    g_assert_nonnull(alive);
    g_clear_object(&displayed);
    g_assert_null(alive);
    g_assert_nonnull(remaining);
  }
  g_assert_null(remaining);
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_test_add_func("/textures/evicts-oldest-by-bytes", evicts_oldest_by_bytes);
  g_test_add_func("/textures/hit-refreshes-recency", hit_refreshes_recency);
  g_test_add_func("/textures/rejects-oversize", rejects_oversize);
  g_test_add_func("/textures/releases-its-reference", releases_its_reference);
  return g_test_run();
}
