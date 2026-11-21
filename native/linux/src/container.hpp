// GTK adapters for Weft's platform-independent layout algorithm.
#pragma once

#include <gtk/gtk.h>
#include <weft/layout.hpp>

namespace weft {

enum class Edge { All, X, Y, Top, Bottom, Start, End, Count };

struct Edges {
  std::optional<double> values[static_cast<int>(Edge::Count)];

  layout::Insets resolve() const;
  void set(Edge edge, double value) { values[static_cast<int>(edge)] = value; }
};

// Owned by the GTK widget so a layout child can borrow it across moves.
struct WidgetLayout {
  layout::Rules size;
  Edges margin;
  GtkWidget *padding_widget = nullptr;
  double grow = 0;
  bool fill = false;
  bool spacer = false;
  std::optional<double> spacer_size;
};

struct WidgetStyle {
  GdkRGBA background{};
  std::optional<GdkRGBA> placeholder;
  GdkRGBA border_color{};
  double radius = 0;
  double border = 0;
  Shadow shadow = Shadow::None;
  bool clip = false;
};

WidgetLayout &attach_layout(GtkWidget *widget);
WidgetStyle &widget_style(GtkWidget *widget);
GtkWidget *widget_new(GtkWidget *child = nullptr);
void picture_set_density(GtkPicture *picture, int density);
GtkWidget *spinner_new();
void spinner_set_color(GtkWidget *widget, GdkRGBA color);
GtkWidget *progress_new();
void progress_set_color(GtkWidget *widget, GdkRGBA color, bool dark);
void progress_set_value(GtkWidget *widget, double value);
GtkWidget *linear_new(Axis axis, CrossAlign align = CrossAlign::Start, bool scrolling = false);
GtkWidget *overlay_new();
GtkWidget *text_new();
void container_set_gap(GtkWidget *widget, double gap);
void container_set_align(GtkWidget *widget, CrossAlign align);
void container_set_justify(GtkWidget *widget, Justify justify);
void container_set_alignment(GtkWidget *widget, Alignment alignment);
void container_set_padding(GtkWidget *widget, Edge edge, double value);
void container_on_allocate(GtkWidget *widget, void (*callback)(GtkWidget *));

// GtkLabel's line limit applies to each paragraph. Labels that need an
// overall limit supply their width-dependent measurement and text here.
struct TextLayout {
  static std::optional<layout::Size> measure(GtkWidget *label, layout::Proposal proposal);
  static void before_allocate(GtkWidget *label, int width);
};

}  // namespace weft
