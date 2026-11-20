#include "appearance.hpp"
#include "weft/gtk.hpp"

namespace {

int natural(GtkWidget *widget, GtkOrientation axis) {
  int size = 0;
  gtk_widget_measure(widget, axis, -1, nullptr, &size, nullptr, nullptr);
  return size;
}

void hidden_linear() {
  weft::Linear column(weft::Axis::Vertical), first(weft::Axis::Vertical), hidden(weft::Axis::Vertical);
  weft::Linear last(weft::Axis::Vertical);
  column.set_gap(5);
  column.set_padding(3);
  column.set_border(2);
  for (auto *child : {&first, &hidden, &last}) {
    child->set_width(20);
    child->set_height(10);
    column.children().add(*child);
  }
  hidden.set_hidden(true);
  last.set_opacity(0.5);

  auto *widget = column.widget();
  g_assert_cmpint(natural(widget, GTK_ORIENTATION_HORIZONTAL), ==, 30);
  g_assert_cmpint(natural(widget, GTK_ORIENTATION_VERTICAL), ==, 35);
  gtk_widget_allocate(widget, 30, 35, -1, nullptr);
  g_assert_cmpint(gtk_widget_get_width(hidden.widget()), ==, 0);
  g_assert_cmpfloat_with_epsilon(gtk_widget_get_opacity(last.widget()), 0.5, 1.0 / 255);
  graphene_rect_t bounds;
  g_assert_true(gtk_widget_compute_bounds(last.widget(), widget, &bounds));
  g_assert_cmpfloat(bounds.origin.x, ==, 5);
  g_assert_cmpfloat(bounds.origin.y, ==, 20);

  hidden.set_hidden(false);
  g_assert_cmpint(natural(widget, GTK_ORIENTATION_VERTICAL), ==, 50);
  gtk_widget_allocate(widget, 30, 50, -1, nullptr);
  g_assert_true(gtk_widget_compute_bounds(last.widget(), widget, &bounds));
  g_assert_cmpfloat(bounds.origin.y, ==, 35);
}

void hidden_stack() {
  weft::Stack stack;
  weft::Linear shown(weft::Axis::Vertical), hidden(weft::Axis::Vertical);
  shown.set_width(20);
  hidden.set_width(80);
  hidden.set_hidden(true);
  stack.children().add(shown);
  stack.children().add(hidden);
  g_assert_cmpint(natural(stack.widget(), GTK_ORIENTATION_HORIZONTAL), ==, 20);
  gtk_widget_allocate(stack.widget(), 20, 20, -1, nullptr);
  g_assert_cmpint(gtk_widget_get_width(hidden.widget()), ==, 0);

  hidden.set_hidden(false);
  g_assert_cmpint(natural(stack.widget(), GTK_ORIENTATION_HORIZONTAL), ==, 80);
  gtk_widget_allocate(stack.widget(), 80, 20, -1, nullptr);
  g_assert_cmpint(gtk_widget_get_width(hidden.widget()), ==, 80);
}

void decorated_snapshot() {
  static constexpr weft::theme::Entry entries[] = {{"surface", 0xff0000ff, 0x0000ffff}};
  weft::theme::set(entries);
  weft::Stack box;
  weft::Appearance::set_test_dark(false);
  weft::Appearance appearance([](void *data) { static_cast<weft::Stack *>(data)->reapply_colors(); }, &box);
  box.set_width(40);
  box.set_height(30);
  box.set_padding(3);
  box.set_background(weft::Color::token("surface"));
  box.set_radius(8);
  box.set_border(2);
  box.set_clip(true);
  box.set_shadow(weft::Shadow::Medium);
  auto *widget = box.widget();
  g_assert_cmpint(natural(widget, GTK_ORIENTATION_HORIZONTAL), ==, 50);
  g_assert_cmpint(natural(widget, GTK_ORIENTATION_VERTICAL), ==, 40);
  gtk_widget_allocate(widget, 50, 40, -1, nullptr);

  for (bool dark : {false, true}) {
    weft::Appearance::set_test_dark(dark);
    auto *snapshot = gtk_snapshot_new();
    GTK_WIDGET_GET_CLASS(widget)->snapshot(widget, snapshot);
    auto *node = gtk_snapshot_free_to_node(snapshot);
    g_assert_nonnull(node);
    g_assert_cmpint(gsk_render_node_get_node_type(node), ==, GSK_CONTAINER_NODE);
    g_assert_cmpuint(gsk_container_node_get_n_children(node), ==, 3);
    auto *shadow = gsk_container_node_get_child(node, 0);
    g_assert_cmpint(gsk_render_node_get_node_type(shadow), ==, GSK_OUTSET_SHADOW_NODE);
    g_assert_cmpfloat(gsk_outset_shadow_node_get_dy(shadow), ==, 4);
    g_assert_cmpfloat(gsk_outset_shadow_node_get_blur_radius(shadow), ==, 12);
    auto *background = gsk_container_node_get_child(node, 1);
    g_assert_cmpint(gsk_render_node_get_node_type(background), ==, GSK_ROUNDED_CLIP_NODE);
    g_assert_cmpfloat(gsk_rounded_clip_node_get_clip(background)->corner[0].width, ==, 8);
    auto *fill = gsk_rounded_clip_node_get_child(background);
    const auto expected_fill = weft::Color::token("surface").resolve(dark);
    g_assert_true(gdk_rgba_equal(gsk_color_node_get_color(fill), &expected_fill));
    auto *border = gsk_container_node_get_child(node, 2);
    g_assert_cmpint(gsk_render_node_get_node_type(border), ==, GSK_BORDER_NODE);
    g_assert_cmpfloat(gsk_border_node_get_widths(border)[0], ==, 2);
    const auto expected_border = weft::Color::rgba(dark ? 0xffffff26u : 0x0000001fu).resolve();
    g_assert_true(gdk_rgba_equal(gsk_border_node_get_colors(border), &expected_border));
    gsk_render_node_unref(node);
  }
  weft::theme::set({});
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_setenv("GTK_A11Y", "none", false);
  if (!gtk_init_check()) return 77;
  g_test_add_func("/appearance/hidden-linear", hidden_linear);
  g_test_add_func("/appearance/hidden-stack", hidden_stack);
  g_test_add_func("/appearance/decorated-snapshot", decorated_snapshot);
  return g_test_run();
}
