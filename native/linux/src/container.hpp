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

WidgetLayout &attach_layout(GtkWidget *widget);
GtkWidget *linear_new(Axis axis, CrossAlign align = CrossAlign::Start, bool scrolling = false);
GtkWidget *overlay_new();
GtkWidget *spacer_new();
void container_set_gap(GtkWidget *widget, double gap);
void container_set_align(GtkWidget *widget, CrossAlign align);
void container_set_justify(GtkWidget *widget, Justify justify);
void container_set_alignment(GtkWidget *widget, Alignment alignment);
void container_set_padding(GtkWidget *widget, Edge edge, double value);
void container_set_background(GtkWidget *widget, const GdkRGBA &color);

}  // namespace weft
