#include "container.hpp"

#include <algorithm>
#include <cmath>
#include <memory>
#include <vector>

using namespace weft;
using namespace weft::layout;

struct WeftLayoutChild {
  GtkLayoutChild parent_instance;
  const WidgetLayout *data;
};
struct WeftLayoutChildClass { GtkLayoutChildClass parent_class; };
G_DEFINE_TYPE(WeftLayoutChild, weft_layout_child, GTK_TYPE_LAYOUT_CHILD)

struct LayoutState {
  Linear linear;
  Alignment alignment = Alignment::TopStart;
  Edges padding;
  std::optional<Axis> scroll_axis;
  bool overlay = false;
  std::vector<Child> children;
};

struct WeftLinearLayout {
  GtkLayoutManager parent_instance;
  LayoutState state;
};
struct WeftLinearLayoutClass { GtkLayoutManagerClass parent_class; };
G_DEFINE_TYPE(WeftLinearLayout, weft_linear_layout, GTK_TYPE_LAYOUT_MANAGER)

struct WeftStackLayout { WeftLinearLayout parent_instance; };
struct WeftStackLayoutClass { WeftLinearLayoutClass parent_class; };
G_DEFINE_TYPE(WeftStackLayout, weft_stack_layout, weft_linear_layout_get_type())

struct WeftLinear {
  GtkWidget parent_instance;
  GdkRGBA background;
  void (*on_allocate)(GtkWidget *);
};
struct WeftLinearClass { GtkWidgetClass parent_class; };
G_DEFINE_TYPE(WeftLinear, weft_linear, GTK_TYPE_WIDGET)

struct WeftWidget { GtkWidget parent_instance; };
struct WeftWidgetClass { GtkWidgetClass parent_class; };
G_DEFINE_TYPE(WeftWidget, weft_widget, GTK_TYPE_WIDGET)

namespace {

GQuark layout_key() { return g_quark_from_static_string("weft-layout"); }

const WidgetLayout *widget_layout(GtkWidget *widget) {
  return static_cast<const WidgetLayout *>(g_object_get_qdata(G_OBJECT(widget), layout_key()));
}

bool is_container(GtkWidget *widget) {
  return G_TYPE_CHECK_INSTANCE_TYPE(widget, weft_linear_get_type());
}

LayoutState &state(GtkLayoutManager *manager) { return reinterpret_cast<WeftLinearLayout *>(manager)->state; }
LayoutState &state(GtkWidget *widget) { return state(gtk_widget_get_layout_manager(widget)); }

int pixels(double value) { return int(std::lround(value)); }

Insets native_margin(GtkWidget *widget) {
  return {double(gtk_widget_get_margin_top(widget)), double(gtk_widget_get_margin_end(widget)),
          double(gtk_widget_get_margin_bottom(widget)), double(gtk_widget_get_margin_start(widget))};
}

Size measure_widget(const void *pointer, Proposal proposal);

Size measure_native(GtkWidget *widget, Proposal proposal) {
  const auto [w, h] = proposal;
  if (w.is_exact() && h.is_exact()) return {w.value, h.value};
  const Insets margin = native_margin(widget);
  const auto measure = [&](GtkOrientation orientation, Length length, int for_size) {
    if (length.is_exact()) return length.value;
    int natural = 0;
    gtk_widget_measure(widget, orientation, for_size, nullptr, &natural, nullptr, nullptr);
    return length.fit(std::max(0.0, natural - (orientation == GTK_ORIENTATION_HORIZONTAL ? margin.horizontal()
                                                                                        : margin.vertical())));
  };

  if (GTK_IS_PICTURE(widget)) {
    if (auto *image = gtk_picture_get_paintable(GTK_PICTURE(widget))) {
      const double ratio = gdk_paintable_get_intrinsic_aspect_ratio(image);
      if (ratio > 0) {
        if (w.is_exact()) return {w.value, h.fit(w.value / ratio)};
        if (h.is_exact()) return {w.fit(h.value * ratio), h.value};
      }
    }
  }

  if (gtk_widget_get_request_mode(widget) == GTK_SIZE_REQUEST_WIDTH_FOR_HEIGHT) {
    const double height = measure(GTK_ORIENTATION_VERTICAL, h, -1);
    return {measure(GTK_ORIENTATION_HORIZONTAL, w, pixels(height + margin.vertical())), height};
  }
  const double width = measure(GTK_ORIENTATION_HORIZONTAL, w, h.is_exact() ? pixels(h.value + margin.vertical()) : -1);
  const double height = measure(GTK_ORIENTATION_VERTICAL, h, pixels(width + margin.horizontal()));
  return {width, height};
}

Size run(GtkLayoutManager *manager, GtkWidget *widget, Proposal proposal) {
  auto &box = state(manager);
  auto &children = box.children;
  children.clear();
  for (GtkWidget *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
    if (!gtk_widget_should_layout(child)) continue;
    const auto *entry = reinterpret_cast<WeftLayoutChild *>(gtk_layout_manager_get_layout_child(manager, child));
    const auto *data = entry->data;
    children.push_back({child, measure_widget, data ? data->margin.resolve() : native_margin(child),
                        data ? data->grow : 0, data && data->fill, data && data->size.width.has_value(),
                        data && data->size.height.has_value(), data && data->spacer,
                        data ? data->spacer_size : std::nullopt, {}});
  }
  const auto padding = box.padding.resolve();
  if (box.overlay) return stack(box.alignment, padding, proposal, children);
  // A viewport gives its child a finite allocation even on the scrolling
  // axis. Keep that axis unbounded for the algorithm so grow stays intrinsic.
  if (box.scroll_axis == Axis::Vertical) proposal.height = {};
  if (box.scroll_axis == Axis::Horizontal) proposal.width = {};
  return linear(box.linear, padding, proposal, children);
}

Size measure_widget(const void *pointer, Proposal proposal) {
  auto *widget = const_cast<GtkWidget *>(static_cast<const GtkWidget *>(pointer));
  const auto *data = widget_layout(widget);
  if (!data && !is_container(widget)) return measure_native(widget, proposal);
  const auto padding = data && data->padding_widget ? state(data->padding_widget).padding.resolve() : Insets{};
  const auto measure = [&](Proposal bounds) {
    if (bounds.width.is_exact() && bounds.height.is_exact()) return Size{bounds.width.value, bounds.height.value};
    if (!is_container(widget)) return measure_native(widget, bounds);
    auto *manager = gtk_widget_get_layout_manager(widget);
    return run(manager, widget, bounds);
  };
  if (!data) return measure(proposal);
  const auto bounds = data->size.propose(proposal, padding);
  const auto measured = measure(bounds);
  const auto clamped = data->size.clamp(measured, padding);
  // When a minimum establishes the width, remeasure height at that width.
  if (clamped.width != measured.width && !bounds.height.is_exact()) {
    return data->size.clamp(measure({Length::exact(clamped.width), bounds.height}), padding);
  }
  return clamped;
}

GtkLayoutChild *create_child(GtkLayoutManager *manager, GtkWidget *, GtkWidget *child) {
  auto *entry = static_cast<WeftLayoutChild *>(g_object_new(weft_layout_child_get_type(), "layout-manager", manager,
                                                           "child-widget", child, nullptr));
  entry->data = widget_layout(child);
  return GTK_LAYOUT_CHILD(entry);
}

GtkSizeRequestMode request_mode(GtkLayoutManager *manager, GtkWidget *) {
  return state(manager).scroll_axis == Axis::Horizontal ? GTK_SIZE_REQUEST_WIDTH_FOR_HEIGHT
                                                       : GTK_SIZE_REQUEST_HEIGHT_FOR_WIDTH;
}

void measure(GtkLayoutManager *manager, GtkWidget *widget, GtkOrientation orientation, int for_size, int *minimum,
             int *natural, int *minimum_baseline, int *natural_baseline) {
  const bool x = orientation == GTK_ORIENTATION_HORIZONTAL;
  // GTK's height-for-width argument is an allocation in the other axis.
  const auto constraint = for_size < 0 ? Length{} : Length::exact(for_size);
  const auto measured = measure_widget(widget, x ? Proposal{{}, constraint} : Proposal{constraint, {}});
  *natural = pixels(x ? measured.width : measured.height);
  const auto &box = state(manager);
  const auto *data = widget_layout(widget);
  const bool scroll_extent = box.scroll_axis == (x ? Axis::Horizontal : Axis::Vertical);
  const auto &size = data ? data->size : Rules{};
  const auto padding = box.padding.resolve();
  const double min_padding = x ? padding.horizontal() : padding.vertical();
  const auto min_size = x ? size.min_width : size.min_height;
  const auto fixed = x ? size.width : size.height;
  // GtkViewport uses the minimum to decide its scrollable extent. Elsewhere
  // keep GTK's minimum flexible, since a Weft parent already chose the size.
  *minimum = scroll_extent || fixed ? *natural : std::min(*natural, pixels(min_padding + min_size.value_or(0)));
  *minimum_baseline = *natural_baseline = -1;
}

void allocate(GtkLayoutManager *manager, GtkWidget *widget, int width, int height, int) {
  run(manager, widget, {Length::exact(width), Length::exact(height)});
  for (const auto &child : state(manager).children) {
    auto *native = const_cast<GtkWidget *>(static_cast<const GtkWidget *>(child.data));
    const auto margin = native_margin(native);
    const auto &frame = child.frame;
    const int x = pixels(frame.x), y = pixels(frame.y);
    // GTK applies its own margins; give it a margin box so the resulting
    // border box is the one picked by the algorithm (also for negative margins).
    GtkAllocation allocation{x - pixels(margin.left), y - pixels(margin.top),
                             pixels(frame.x + frame.size.width) - x + pixels(margin.horizontal()),
                             pixels(frame.y + frame.size.height) - y + pixels(margin.vertical())};
    gtk_widget_size_allocate(native, &allocation, -1);
  }
  if (const auto callback = reinterpret_cast<WeftLinear *>(widget)->on_allocate) callback(widget);
}

void finalize_layout(GObject *object) {
  std::destroy_at(&reinterpret_cast<WeftLinearLayout *>(object)->state);
  G_OBJECT_CLASS(weft_linear_layout_parent_class)->finalize(object);
}

void snapshot(GtkWidget *widget, GtkSnapshot *snapshot) {
  const auto &color = reinterpret_cast<WeftLinear *>(widget)->background;
  if (color.alpha > 0) {
    const auto bounds = GRAPHENE_RECT_INIT(0, 0, float(gtk_widget_get_width(widget)), float(gtk_widget_get_height(widget)));
    gtk_snapshot_append_color(snapshot, &color, &bounds);
  }
  GTK_WIDGET_CLASS(weft_linear_parent_class)->snapshot(widget, snapshot);
}

void dispose(GObject *object) {
  while (auto *child = gtk_widget_get_first_child(GTK_WIDGET(object))) gtk_widget_unparent(child);
  G_OBJECT_CLASS(weft_linear_parent_class)->dispose(object);
}

GtkWidget *container_new(GType manager_type) {
  auto *widget = GTK_WIDGET(g_object_new(weft_linear_get_type(), nullptr));
  gtk_widget_set_layout_manager(widget, GTK_LAYOUT_MANAGER(g_object_new(manager_type, nullptr)));
  return widget;
}

}  // namespace

static void weft_layout_child_init(WeftLayoutChild *) {}
static void weft_layout_child_class_init(WeftLayoutChildClass *) {}

static void weft_linear_layout_init(WeftLinearLayout *self) { std::construct_at(&self->state); }
static void weft_linear_layout_class_init(WeftLinearLayoutClass *klass) {
  G_OBJECT_CLASS(klass)->finalize = finalize_layout;
  auto *manager = GTK_LAYOUT_MANAGER_CLASS(klass);
  manager->create_layout_child = create_child;
  manager->layout_child_type = weft_layout_child_get_type();
  manager->get_request_mode = request_mode;
  manager->measure = measure;
  manager->allocate = allocate;
}

static void weft_stack_layout_init(WeftStackLayout *self) { self->parent_instance.state.overlay = true; }
static void weft_stack_layout_class_init(WeftStackLayoutClass *) {}

static void weft_linear_init(WeftLinear *) {}
static void weft_linear_class_init(WeftLinearClass *klass) {
  G_OBJECT_CLASS(klass)->dispose = dispose;
  auto *widget = GTK_WIDGET_CLASS(klass);
  widget->snapshot = snapshot;
  gtk_widget_class_set_css_name(widget, "weft-linear");
}

static void weft_widget_init(WeftWidget *) {}
static void weft_widget_class_init(WeftWidgetClass *) {}

namespace weft {

Insets Edges::resolve() const {
  const auto value = [this](Edge edge) { return values[static_cast<int>(edge)]; };
  const double all = value(Edge::All).value_or(0);
  const double x = value(Edge::X).value_or(all), y = value(Edge::Y).value_or(all);
  return {value(Edge::Top).value_or(y), value(Edge::End).value_or(x),
          value(Edge::Bottom).value_or(y), value(Edge::Start).value_or(x)};
}

WidgetLayout &attach_layout(GtkWidget *widget) {
  auto *data = new WidgetLayout;
  g_object_set_qdata_full(G_OBJECT(widget), layout_key(), data, [](gpointer p) { delete static_cast<WidgetLayout *>(p); });
  // M1 uses physical left/right for start/end, independent of system locale.
  gtk_widget_set_direction(widget, GTK_TEXT_DIR_LTR);
  return *data;
}

GtkWidget *linear_new(Axis axis, CrossAlign align, bool scrolling) {
  auto *widget = container_new(weft_linear_layout_get_type());
  auto &box = state(widget);
  box.linear.axis = axis;
  box.linear.align = align;
  if (scrolling) box.scroll_axis = axis;
  return widget;
}

GtkWidget *overlay_new() { return container_new(weft_stack_layout_get_type()); }
GtkWidget *spacer_new() { return GTK_WIDGET(g_object_new(weft_widget_get_type(), nullptr)); }

void container_set_gap(GtkWidget *widget, double gap) {
  state(widget).linear.gap = std::max(0.0, gap);
  gtk_widget_queue_resize(widget);
}
void container_set_align(GtkWidget *widget, CrossAlign align) {
  state(widget).linear.align = align;
  gtk_widget_queue_resize(widget);
}
void container_set_justify(GtkWidget *widget, Justify justify) {
  state(widget).linear.justify = justify;
  gtk_widget_queue_allocate(widget);
}
void container_set_alignment(GtkWidget *widget, Alignment alignment) {
  state(widget).alignment = alignment;
  gtk_widget_queue_allocate(widget);
}
void container_set_padding(GtkWidget *widget, Edge edge, double value) {
  state(widget).padding.set(edge, std::max(0.0, value));
  gtk_widget_queue_resize(widget);
}
void container_set_background(GtkWidget *widget, const GdkRGBA &color) {
  reinterpret_cast<WeftLinear *>(widget)->background = color;
  gtk_widget_queue_draw(widget);
}

void container_on_allocate(GtkWidget *widget, void (*callback)(GtkWidget *)) {
  reinterpret_cast<WeftLinear *>(widget)->on_allocate = callback;
}

}  // namespace weft
