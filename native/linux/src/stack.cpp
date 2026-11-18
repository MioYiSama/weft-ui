#include "stack.hpp"

#include <algorithm>

struct _WeftStack {
  GtkWidget parent_instance;
  weft::Axis axis;
  weft::Align align;
  int gap;
  int padding;
  /// Fixed content size, or -1.
  int width;
  int height;
  GdkRGBA background;
};

G_DEFINE_TYPE(WeftStack, weft_stack, GTK_TYPE_WIDGET)

namespace {

/// The size a stack gives `child` along `orientation`: its size request if
/// it has one (the minimum size, which GTK raises to the request), else its
/// natural size. `for_size` is the size in the other orientation, or -1.
int extent(GtkWidget *child, GtkOrientation orientation, int request, int for_size) {
  int minimum = 0;
  int natural = 0;
  gtk_widget_measure(child, orientation, for_size, &minimum, &natural, nullptr, nullptr);
  return request >= 0 ? minimum : natural;
}

/// The width and height a stack gives `child`. Widths come first, so text
/// and images get their height for that width; an image with only a fixed
/// height gets its width for that height instead.
void child_size(GtkWidget *child, int &width, int &height) {
  int requested_width = -1;
  int requested_height = -1;
  gtk_widget_get_size_request(child, &requested_width, &requested_height);
  if (requested_width < 0 && requested_height >= 0) {
    height = extent(child, GTK_ORIENTATION_VERTICAL, requested_height, -1);
    width = extent(child, GTK_ORIENTATION_HORIZONTAL, requested_width, height);
  } else {
    width = extent(child, GTK_ORIENTATION_HORIZONTAL, requested_width, -1);
    height = extent(child, GTK_ORIENTATION_VERTICAL, requested_height, width);
  }
}

bool is_main(const WeftStack *stack, GtkOrientation orientation) {
  return (stack->axis == weft::Axis::Vertical) == (orientation == GTK_ORIENTATION_VERTICAL);
}

void measure(GtkWidget *widget, GtkOrientation orientation, int, int *minimum, int *natural, int *minimum_baseline,
             int *natural_baseline) {
  auto *stack = WEFT_STACK(widget);
  int fixed = orientation == GTK_ORIENTATION_HORIZONTAL ? stack->width : stack->height;
  int size = 0;
  if (fixed >= 0) {
    size = fixed;
  } else {
    bool main = is_main(stack, orientation);
    int count = 0;
    for (GtkWidget *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
      if (!gtk_widget_should_layout(child)) continue;
      int width = 0;
      int height = 0;
      child_size(child, width, height);
      int along = orientation == GTK_ORIENTATION_HORIZONTAL ? width : height;
      size = main ? size + along : std::max(size, along);
      count++;
    }
    if (main && count > 1) size += stack->gap * (count - 1);
  }
  // Children never shrink, so the minimum is the natural size.
  *minimum = *natural = size + 2 * stack->padding;
  *minimum_baseline = *natural_baseline = -1;
}

void size_allocate(GtkWidget *widget, int width, int height, int) {
  auto *stack = WEFT_STACK(widget);
  bool vertical = stack->axis == weft::Axis::Vertical;
  int cross = (vertical ? width : height) - 2 * stack->padding;
  int position = stack->padding;
  for (GtkWidget *child = gtk_widget_get_first_child(widget); child; child = gtk_widget_get_next_sibling(child)) {
    if (!gtk_widget_should_layout(child)) continue;
    int child_width = 0;
    int child_height = 0;
    child_size(child, child_width, child_height);
    int offset = stack->padding;
    if (stack->align == weft::Align::Center) offset += (cross - (vertical ? child_width : child_height)) / 2;
    GtkAllocation allocation = vertical ? GtkAllocation{offset, position, child_width, child_height}
                                        : GtkAllocation{position, offset, child_width, child_height};
    gtk_widget_size_allocate(child, &allocation, -1);
    position += (vertical ? child_height : child_width) + stack->gap;
  }
}

void snapshot(GtkWidget *widget, GtkSnapshot *snapshot) {
  auto *stack = WEFT_STACK(widget);
  if (stack->background.alpha > 0) {
    graphene_rect_t bounds =
        GRAPHENE_RECT_INIT(0, 0, float(gtk_widget_get_width(widget)), float(gtk_widget_get_height(widget)));
    gtk_snapshot_append_color(snapshot, &stack->background, &bounds);
  }
  GTK_WIDGET_CLASS(weft_stack_parent_class)->snapshot(widget, snapshot);
}

GtkSizeRequestMode request_mode(GtkWidget *) { return GTK_SIZE_REQUEST_CONSTANT_SIZE; }

void dispose(GObject *object) {
  while (GtkWidget *child = gtk_widget_get_first_child(GTK_WIDGET(object))) gtk_widget_unparent(child);
  G_OBJECT_CLASS(weft_stack_parent_class)->dispose(object);
}

}  // namespace

static void weft_stack_class_init(WeftStackClass *klass) {
  G_OBJECT_CLASS(klass)->dispose = dispose;
  auto *widget = GTK_WIDGET_CLASS(klass);
  widget->measure = measure;
  widget->size_allocate = size_allocate;
  widget->snapshot = snapshot;
  widget->get_request_mode = request_mode;
  gtk_widget_class_set_css_name(widget, "weft-stack");
  gtk_widget_class_set_accessible_role(widget, GTK_ACCESSIBLE_ROLE_GENERIC);
}

static void weft_stack_init(WeftStack *stack) {
  stack->axis = weft::Axis::Vertical;
  stack->align = weft::Align::Start;
  stack->gap = 0;
  stack->padding = 0;
  stack->width = -1;
  stack->height = -1;
  stack->background = {};
}

namespace weft {

GtkWidget *stack_new(Axis axis, Align align) {
  auto *stack = WEFT_STACK(g_object_new(WEFT_TYPE_STACK, nullptr));
  stack->axis = axis;
  stack->align = align;
  return GTK_WIDGET(stack);
}

void stack_set_gap(WeftStack *stack, int gap) {
  stack->gap = gap;
  gtk_widget_queue_resize(GTK_WIDGET(stack));
}

void stack_set_padding(WeftStack *stack, int padding) {
  stack->padding = padding;
  gtk_widget_queue_resize(GTK_WIDGET(stack));
}

void stack_set_width(WeftStack *stack, int width) {
  stack->width = width;
  gtk_widget_queue_resize(GTK_WIDGET(stack));
}

void stack_set_height(WeftStack *stack, int height) {
  stack->height = height;
  gtk_widget_queue_resize(GTK_WIDGET(stack));
}

void stack_set_background(WeftStack *stack, const GdkRGBA &color) {
  stack->background = color;
  gtk_widget_queue_draw(GTK_WIDGET(stack));
}

GtkWidget *stack_child(GtkWidget *stack, size_t index) {
  GtkWidget *child = gtk_widget_get_first_child(stack);
  for (; child && index > 0; index--) child = gtk_widget_get_next_sibling(child);
  return child;
}

}  // namespace weft
