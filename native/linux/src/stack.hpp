// `WeftStack`: the container widget of `Column` / `Row` and button content.

#pragma once

#include <gtk/gtk.h>

#include "weft/gtk.hpp"

G_BEGIN_DECLS

#define WEFT_TYPE_STACK (weft_stack_get_type())
G_DECLARE_FINAL_TYPE(WeftStack, weft_stack, WEFT, STACK, GtkWidget)

G_END_DECLS

namespace weft {

/// A stack of children along `axis`, each at its natural size (or its size
/// request, when it has one), placed at the start of the cross axis or
/// centered on it. A fixed width or height sizes the content box.
GtkWidget *stack_new(Axis axis, Align align);
void stack_set_gap(WeftStack *stack, int gap);
void stack_set_padding(WeftStack *stack, int padding);
void stack_set_width(WeftStack *stack, int width);
void stack_set_height(WeftStack *stack, int height);
void stack_set_background(WeftStack *stack, const GdkRGBA &color);
/// Child `index`, or null past the last child.
GtkWidget *stack_child(GtkWidget *stack, size_t index);

}  // namespace weft
