#include "appearance.hpp"
#include "weft/gtk.hpp"

namespace {

GtkLabel *label_of(const weft::Text &text) {
  auto *label = gtk_widget_get_first_child(text.widget());
  g_assert_true(GTK_IS_LABEL(label));
  g_assert_null(gtk_widget_get_next_sibling(label));
  return GTK_LABEL(label);
}

int integer_attr(PangoAttrIterator *iter, PangoAttrType type) {
  auto *attr = pango_attr_iterator_get(iter, type);
  g_assert_nonnull(attr);
  return reinterpret_cast<PangoAttrInt *>(attr)->value;
}

void spans_merge() {
  static constexpr weft::theme::Entry colors[] = {{"accent", 0x0000ffff, 0x00ff00ff}};
  weft::theme::set(colors);
  weft::Appearance::set_test_dark(false);
  weft::Text text(3);
  text.set_size(18);
  text.set_weight(weft::Weight::Medium);
  text.set_italic(true);
  text.set_decoration(weft::Decoration::Underline);
  text.span(0).set_value("é");
  text.span(0).set_weight(weft::Weight::Semibold);
  text.span(1).set_value("<");
  text.span(1).set_color(weft::Color::token("accent"));
  text.span(1).set_italic(false);
  text.span(1).set_decoration(weft::Decoration::LineThrough);
  text.span(2).set_value("🌿");
  text.span(2).set_size(22);

  auto *label = label_of(text);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "é<🌿");
  {
    g_autoptr(PangoAttrIterator) attrs = pango_attr_list_get_iterator(gtk_label_get_attributes(label));
    int from, to;
    pango_attr_iterator_range(attrs, &from, &to);
    g_assert_cmpint(from, ==, 0);
    g_assert_cmpint(to, ==, 2);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_WEIGHT), ==, PANGO_WEIGHT_SEMIBOLD);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_STYLE), ==, PANGO_STYLE_ITALIC);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_UNDERLINE), ==, PANGO_UNDERLINE_SINGLE);
    g_assert_true(pango_attr_iterator_next(attrs));
    pango_attr_iterator_range(attrs, &from, &to);
    g_assert_cmpint(from, ==, 2);
    g_assert_cmpint(to, ==, 3);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_WEIGHT), ==, PANGO_WEIGHT_MEDIUM);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_STYLE), ==, PANGO_STYLE_NORMAL);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_STRIKETHROUGH), ==, 1);
    auto *foreground = reinterpret_cast<PangoAttrColor *>(pango_attr_iterator_get(attrs, PANGO_ATTR_FOREGROUND));
    g_assert_nonnull(foreground);
    g_assert_cmpuint(foreground->color.blue, ==, G_MAXUINT16);
    g_assert_true(pango_attr_iterator_next(attrs));
    pango_attr_iterator_range(attrs, &from, &to);
    g_assert_cmpint(from, ==, 3);
    g_assert_cmpint(to, ==, 7);
    g_assert_cmpint(integer_attr(attrs, PANGO_ATTR_ABSOLUTE_SIZE), ==, 22 * PANGO_SCALE);
  }

  text.span(0).set_value("");
  weft::Appearance::set_test_dark(true);
  text.reapply_colors();
  g_assert_cmpstr(gtk_label_get_text(label), ==, "<🌿");
  g_autoptr(PangoAttrIterator) attrs = pango_attr_list_get_iterator(gtk_label_get_attributes(label));
  auto *foreground = reinterpret_cast<PangoAttrColor *>(pango_attr_iterator_get(attrs, PANGO_ATTR_FOREGROUND));
  g_assert_nonnull(foreground);
  g_assert_cmpuint(foreground->color.green, ==, G_MAXUINT16);
  g_assert_cmpuint(foreground->color.blue, ==, 0);
  g_assert_true(pango_attr_iterator_next(attrs));
  int from, to;
  pango_attr_iterator_range(attrs, &from, &to);
  g_assert_cmpint(from, ==, 1);
  g_assert_cmpint(to, ==, 5);
  weft::theme::set({});
}

void font_families() {
  weft::Text text(2);
  text.set_font("Inter");
  text.span(0).set_value("base");
  text.span(1).set_value("override");
  text.span(1).set_font("Noto Sans");
  auto *label = label_of(text);
  const auto family = [](PangoAttrIterator *iter) {
    auto *attr = pango_attr_iterator_get(iter, PANGO_ATTR_FAMILY);
    g_assert_nonnull(attr);
    return reinterpret_cast<PangoAttrString *>(attr)->value;
  };
  {
    g_autoptr(PangoAttrIterator) attrs = pango_attr_list_get_iterator(gtk_label_get_attributes(label));
    g_assert_cmpstr(family(attrs), ==, "Inter");
    g_assert_true(pango_attr_iterator_next(attrs));
    g_assert_cmpstr(family(attrs), ==, "Noto Sans");
  }
  text.span(1).set_font("");
  g_autoptr(PangoAttrIterator) attrs = pango_attr_list_get_iterator(gtk_label_get_attributes(label));
  g_assert_cmpstr(family(attrs), ==, "Inter");
  g_assert_true(pango_attr_iterator_next(attrs));
  g_assert_cmpstr(family(attrs), ==, "Inter");
}

void intrinsic_width_in_centered_column() {
  constexpr int width = 300;
  weft::Linear column(weft::Axis::Vertical, weft::CrossAlign::Center);
  weft::Text text;
  text.set_value("A short title");
  column.children().add(text);

  auto *widget = column.widget();
  gtk_widget_allocate(widget, width, 100, -1, nullptr);
  graphene_rect_t bounds;
  g_assert_true(gtk_widget_compute_bounds(text.widget(), widget, &bounds));
  g_assert_cmpfloat(bounds.size.width, <, width);
  g_assert_cmpfloat_with_epsilon(bounds.origin.x + bounds.size.width / 2, width / 2.0, 0.51);

  text.set_fill(true);
  gtk_widget_allocate(widget, width, 100, -1, nullptr);
  g_assert_cmpint(gtk_widget_get_width(GTK_WIDGET(label_of(text))), ==, width);
}

void line_height_and_limit() {
  weft::Text text;
  text.set_size(15);
  text.set_line_height(28);
  text.set_align(weft::TextAlign::End);
  text.set_value("One paragraph with many words and an unbreakable abcdefghijklmnopqrstuvwxyz to wrap.");
  text.set_max_lines(2);
  auto *label = label_of(text);
  g_assert_cmpfloat(gtk_label_get_xalign(label), ==, 1);
  g_assert_cmpint(gtk_label_get_justify(label), ==, GTK_JUSTIFY_RIGHT);
  g_assert_cmpint(gtk_label_get_wrap_mode(label), ==, PANGO_WRAP_WORD_CHAR);
  gtk_widget_allocate(text.widget(), 115, 220, -1, nullptr);
  auto *layout = gtk_label_get_layout(label);
  g_assert_cmpint(pango_layout_get_line_count(layout), ==, 2);
  g_assert_true(pango_layout_is_ellipsized(layout));

  g_autoptr(PangoLayoutIter) iter = pango_layout_get_iter(layout);
  const int baseline = pango_layout_iter_get_baseline(iter);
  g_assert_true(pango_layout_iter_next_line(iter));
  g_assert_cmpint(pango_layout_iter_get_baseline(iter) - baseline, ==, 28 * PANGO_SCALE);

  text.set_max_lines(0);
  gtk_widget_allocate(text.widget(), 115, 220, -1, nullptr);
  layout = gtk_label_get_layout(label);
  g_assert_cmpint(pango_layout_get_line_count(layout), >, 2);
  g_assert_false(pango_layout_is_ellipsized(layout));
}

void line_limit_across_paragraphs() {
  weft::Text text;
  text.set_value("One\nTwo\nThree");
  text.set_max_lines(2);
  auto *label = label_of(text);
  gtk_widget_allocate(text.widget(), 200, 150, -1, nullptr);
  g_assert_cmpint(pango_layout_get_line_count(gtk_label_get_layout(label)), ==, 2);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "One\nTwo…");

  text.set_max_lines(-1);
  gtk_widget_allocate(text.widget(), 200, 150, -1, nullptr);
  g_assert_cmpint(pango_layout_get_line_count(gtk_label_get_layout(label)), ==, 3);
  g_assert_cmpstr(gtk_label_get_text(label), ==, "One\nTwo\nThree");
}

}  // namespace

int main(int argc, char **argv) {
  g_test_init(&argc, &argv, nullptr);
  g_setenv("GTK_A11Y", "none", false);
  if (!gtk_init_check()) return 77;
  g_test_add_func("/text/spans-merge", spans_merge);
  g_test_add_func("/text/font-families", font_families);
  g_test_add_func("/text/intrinsic-width-in-centered-column", intrinsic_width_in_centered_column);
  g_test_add_func("/text/line-height-and-limit", line_height_and_limit);
  g_test_add_func("/text/limit-across-paragraphs", line_limit_across_paragraphs);
  return g_test_run();
}
