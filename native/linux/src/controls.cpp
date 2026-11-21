#include "weft/gtk.hpp"

#include <algorithm>
#include <cmath>
#include <utility>

#include "appearance.hpp"
#include "container.hpp"

namespace weft {

Switch::Switch(std::function<bool(bool)> edit) : Element(gtk_switch_new()), edit_(std::move(edit)) {
  g_signal_connect(widget(), "state-set", G_CALLBACK(state_set), this);
}

void Switch::set_checked(bool checked) {
  value_ = checked;
  gtk_switch_set_active(GTK_SWITCH(widget()), checked);
  gtk_switch_set_state(GTK_SWITCH(widget()), checked);
}

void Switch::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

gboolean Switch::state_set(GtkSwitch *button, gboolean active, Switch *self) {
  if (bool(active) == self->value_ || self->edit_(active)) return FALSE;
  // GTK publishes the proposed accessible state after this signal. Restore
  // both the slider and accessibility state before the next redraw.
  g_idle_add_full(G_PRIORITY_HIGH_IDLE, [](gpointer data) -> gboolean {
    auto *control = GTK_SWITCH(data);
    gtk_switch_set_active(control, gtk_switch_get_state(control));
    return G_SOURCE_REMOVE;
  }, g_object_ref(button), g_object_unref);
  return TRUE;
}

namespace {

constexpr int kTextAreaRows = 3;
constexpr int kTextAreaWidth = 200;
constexpr int kTextInsetX = 8;
constexpr int kTextInsetY = 4;

char *buffer_text(GtkTextBuffer *buffer) {
  GtkTextIter begin, end;
  gtk_text_buffer_get_bounds(buffer, &begin, &end);
  return gtk_text_buffer_get_text(buffer, &begin, &end, true);
}

}  // namespace

TextControl::TextControl(GtkWidget *widget, std::function<void(bool)> focus)
    : Element(widget), focus_(std::move(focus)) {}

TextControl::~TextControl() {
  cancel_pending_focus();
  if (focus_controller_) {
    g_signal_handlers_disconnect_by_data(focus_controller_, this);
    gtk_widget_remove_controller(focus_target_, focus_controller_);
  }
}

void TextControl::set_focus_target(GtkWidget *target) {
  focus_target_ = target;
  if (!focus_) return;
  focus_controller_ = gtk_event_controller_focus_new();
  g_signal_connect(focus_controller_, "enter", G_CALLBACK(entered), this);
  g_signal_connect(focus_controller_, "leave", G_CALLBACK(left), this);
  gtk_widget_add_controller(focus_target_, focus_controller_);
}

void TextControl::set_focused(bool focused) {
  if (std::exchange(focused_, focused) == focused) return;
  if (focused) {
    if (gtk_widget_get_mapped(focus_target_)) gtk_widget_grab_focus(focus_target_);
    // Initial values can arrive before the scope is attached to the window.
    else pending_focus_ = g_signal_connect_after(focus_target_, "map", G_CALLBACK(mapped), this);
    return;
  }
  cancel_pending_focus();
  auto *root = gtk_widget_get_root(focus_target_);
  auto *focus = root ? gtk_root_get_focus(root) : nullptr;
  // GtkEntry delegates editing and keyboard focus to a child GtkText.
  if (focus && (focus == focus_target_ || gtk_widget_is_ancestor(focus, focus_target_))) {
    gtk_root_set_focus(root, nullptr);
  }
}

void TextControl::cancel_pending_focus() {
  if (pending_focus_) g_signal_handler_disconnect(focus_target_, std::exchange(pending_focus_, 0));
}

void TextControl::mapped(GtkWidget *target, TextControl *self) {
  self->cancel_pending_focus();
  gtk_widget_grab_focus(target);
}

void TextControl::entered(GtkEventControllerFocus *, TextControl *self) { self->focus_(true); }

void TextControl::left(GtkEventControllerFocus *, TextControl *self) { self->focus_(false); }

std::string_view TextControl::limit_edit(std::string_view text) const {
  if (!(max_length_ >= 0 && max_length_ < text.size())) return text;
  auto remaining = size_t(max_length_);
  const char *at = text.data();
  const char *end = at + text.size();
  // GTK yields valid UTF-8. A four-byte code point uses two UTF-16 units.
  while (at != end && remaining) {
    const char *next = g_utf8_next_char(at);
    const size_t units = next - at == 4 ? 2 : 1;
    if (units > remaining) break;
    remaining -= units;
    at = next;
  }
  return text.substr(0, at - text.data());
}

TextArea::TextArea(std::function<bool(std::string_view)> edit, std::function<void(bool)> focus)
    : TextControl(gtk_overlay_new(), std::move(focus)), edit_(std::move(edit)), view_(GTK_TEXT_VIEW(gtk_text_view_new())),
      buffer_(gtk_text_view_get_buffer(view_)), placeholder_(gtk_label_new(nullptr)) {
  set_focus_target(GTK_WIDGET(view_));
  auto *scroll = GTK_SCROLLED_WINDOW(gtk_scrolled_window_new());
  gtk_scrolled_window_set_policy(scroll, GTK_POLICY_NEVER, GTK_POLICY_AUTOMATIC);
  gtk_scrolled_window_set_has_frame(scroll, true);
  gtk_scrolled_window_set_min_content_width(scroll, kTextAreaWidth);
  gtk_scrolled_window_set_child(scroll, GTK_WIDGET(view_));
  gtk_overlay_set_child(GTK_OVERLAY(widget()), GTK_WIDGET(scroll));

  gtk_text_view_set_wrap_mode(view_, GTK_WRAP_WORD_CHAR);
  gtk_text_view_set_left_margin(view_, kTextInsetX);
  gtk_text_view_set_right_margin(view_, kTextInsetX);
  gtk_text_view_set_top_margin(view_, kTextInsetY);
  gtk_text_view_set_bottom_margin(view_, kTextInsetY);
  auto *metrics = pango_context_get_metrics(gtk_widget_get_pango_context(GTK_WIDGET(view_)), nullptr, nullptr);
  const int line_height = PANGO_PIXELS_CEIL(pango_font_metrics_get_height(metrics));
  pango_font_metrics_unref(metrics);
  gtk_scrolled_window_set_min_content_height(scroll, kTextAreaRows * line_height + 2 * kTextInsetY);

  // GtkTextView has no placeholder. Keep the hint outside its editable buffer.
  gtk_label_set_wrap(GTK_LABEL(placeholder_), true);
  gtk_label_set_wrap_mode(GTK_LABEL(placeholder_), PANGO_WRAP_WORD_CHAR);
  gtk_label_set_xalign(GTK_LABEL(placeholder_), 0);
  gtk_widget_set_valign(placeholder_, GTK_ALIGN_START);
  gtk_widget_set_margin_start(placeholder_, kTextInsetX + 1);
  gtk_widget_set_margin_end(placeholder_, kTextInsetX + 1);
  gtk_widget_set_margin_top(placeholder_, kTextInsetY + 1);
  gtk_widget_add_css_class(placeholder_, "dim-label");
  gtk_widget_set_can_target(placeholder_, false);
  gtk_accessible_update_state(GTK_ACCESSIBLE(placeholder_), GTK_ACCESSIBLE_STATE_HIDDEN, true, -1);
  gtk_overlay_add_overlay(GTK_OVERLAY(widget()), placeholder_);
  gtk_overlay_set_clip_overlay(GTK_OVERLAY(widget()), placeholder_, true);
  gtk_widget_set_visible(placeholder_, false);

  g_signal_connect(buffer_, "changed", G_CALLBACK(changed), this);
  g_signal_connect(buffer_, "begin-user-action", G_CALLBACK(begin_edit), this);
  g_signal_connect_after(buffer_, "end-user-action", G_CALLBACK(end_edit), this);
  g_signal_connect(view_, "preedit-changed", G_CALLBACK(preedit_changed), this);
}

TextArea::~TextArea() {
  g_signal_handlers_disconnect_by_data(buffer_, this);
  g_signal_handlers_disconnect_by_data(view_, this);
}

void TextArea::set_value(std::string_view value) {
  value_ = value;
  if (!composing_) {
    g_autofree char *text = buffer_text(buffer_);
    if (value_ != text) show();
  }
}

void TextArea::set_placeholder(std::string_view placeholder) {
  std::string text(placeholder);
  gtk_label_set_text(GTK_LABEL(placeholder_), text.c_str());
  gtk_accessible_update_property(GTK_ACCESSIBLE(view_), GTK_ACCESSIBLE_PROPERTY_PLACEHOLDER, text.c_str(), -1);
  update_placeholder();
}

void TextArea::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

void TextArea::changed(GtkTextBuffer *, TextArea *self) {
  self->update_placeholder();
  if (!self->editing_) self->edited();
}

// Replacing a selection can delete and insert in the same user action. Only
// dispatch the final text, and restore rejected edits outside the insertion.
void TextArea::begin_edit(GtkTextBuffer *, TextArea *self) { self->editing_ = true; }

void TextArea::end_edit(GtkTextBuffer *, TextArea *self) {
  self->edited();
  self->editing_ = false;
}

void TextArea::preedit_changed(GtkTextView *, const char *preedit, TextArea *self) {
  bool composing = preedit && *preedit;
  if (composing == self->composing_) return;
  self->composing_ = composing;
  self->update_placeholder();
  if (!composing && !self->editing_) self->edited();
}

void TextArea::edited() {
  if (showing_ || composing_) return;
  g_autofree char *text = buffer_text(buffer_);
  const std::string_view raw(text);
  if (raw == value_) return;
  const auto limited = limit_edit(raw);
  if (limited.size() != raw.size()) {
    const auto suffix = g_utf8_strlen(raw.data() + limited.size(), -1);
    GtkTextIter begin, end;
    gtk_text_buffer_get_end_iter(buffer_, &end);
    begin = end;
    gtk_text_iter_backward_chars(&begin, suffix);
    showing_ = true;
    gtk_text_buffer_delete(buffer_, &begin, &end);
    showing_ = false;
  }
  if (limited != value_ && !edit_(limited)) show();
}

void TextArea::show() {
  GtkTextIter at;
  gtk_text_buffer_get_iter_at_mark(buffer_, &at, gtk_text_buffer_get_insert(buffer_));
  const int caret = gtk_text_iter_get_offset(&at);
  showing_ = true;
  if (editing_) {
    // The end-user-action signal runs before GTK closes the undo group.
    // set_text() would start an irreversible action inside that group.
    GtkTextIter begin, end;
    gtk_text_buffer_get_bounds(buffer_, &begin, &end);
    gtk_text_buffer_delete(buffer_, &begin, &end);
    if (!value_.empty()) gtk_text_buffer_insert(buffer_, &begin, value_.data(), value_.size());
  } else {
    gtk_text_buffer_set_text(buffer_, value_.data(), value_.size());
  }
  showing_ = false;
  gtk_text_buffer_get_iter_at_offset(buffer_, &at, std::min(caret, gtk_text_buffer_get_char_count(buffer_)));
  gtk_text_buffer_place_cursor(buffer_, &at);
}

void TextArea::update_placeholder() {
  gtk_widget_set_visible(placeholder_, !composing_ && gtk_text_buffer_get_char_count(buffer_) == 0 &&
                                           *gtk_label_get_text(GTK_LABEL(placeholder_)) != '\0');
}

Slider::Slider(std::function<bool(double)> edit)
    : Element(gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL, kMin, kMax, kStep)), edit_(std::move(edit)) {
  gtk_scale_set_draw_value(GTK_SCALE(widget()), false);
  g_signal_connect(widget(), "value-changed", G_CALLBACK(changed), this);
}

void Slider::set_value(double value) {
  value_ = value;
  show(snap(value));
}

void Slider::set_min(double min) {
  min_ = min;
  sync_range();
}

void Slider::set_max(double max) {
  max_ = max;
  sync_range();
}

void Slider::set_step(double step) {
  step_ = step > 0 ? step : 1;
  sync_range();
}

void Slider::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

void Slider::changed(GtkRange *range, Slider *self) {
  if (self->showing_) return;
  const double raw = gtk_range_get_value(range);
  const double proposed = self->snap(raw);
  if (proposed == self->snap(self->value_) || !self->edit_(proposed)) {
    self->show(self->snap(self->value_));
  } else if (gtk_range_get_value(range) == raw) {
    self->show(proposed);
  }
}

double Slider::snap(double value) const {
  const double upper = std::max(min_, max_);
  const double steps = std::floor((upper - min_) / step_);
  const double proposed = std::round((std::clamp(value, min_, upper) - min_) / step_);
  return min_ + std::min(steps, proposed) * step_;
}

void Slider::sync_range() {
  auto *range = GTK_RANGE(widget());
  showing_ = true;
  gtk_range_set_range(range, min_, std::max(min_, max_));
  gtk_range_set_increments(range, step_, step_);
  gtk_range_set_value(range, snap(value_));
  showing_ = false;
}

void Slider::show(double value) {
  showing_ = true;
  gtk_range_set_value(GTK_RANGE(widget()), value);
  showing_ = false;
}

Picker::Picker(std::function<bool(double)> edit)
    : Element(gtk_menu_button_new()), edit_(std::move(edit)), options_box_(gtk_box_new(GTK_ORIENTATION_VERTICAL, 0)),
      label_(GTK_LABEL(gtk_label_new(nullptr))) {
  auto *popover = GTK_POPOVER(gtk_popover_new());
  gtk_popover_set_child(popover, options_box_);
  auto *button = GTK_MENU_BUTTON(widget());
  gtk_menu_button_set_popover(button, GTK_WIDGET(popover));
  gtk_label_set_ellipsize(label_, PANGO_ELLIPSIZE_END);
  gtk_label_set_xalign(label_, 0);
  gtk_widget_set_hexpand(GTK_WIDGET(label_), true);
  gtk_menu_button_set_child(button, GTK_WIDGET(label_));
  gtk_menu_button_set_always_show_arrow(button, true);
}

Picker::~Picker() {
  for (auto *button : options_) g_signal_handlers_disconnect_by_data(button, this);
}

void Picker::set_selected(double selected) {
  selected_ = selected;
  show();
}

void Picker::set_options(ops::StringList options) {
  clear_options();
  options_.reserve(options.count);
  std::string label;
  for (auto option : options) {
    label = option;
    auto *button = GTK_BUTTON(gtk_button_new_with_label(label.c_str()));
    gtk_widget_add_css_class(GTK_WIDGET(button), "flat");
    g_object_set_data(G_OBJECT(button), "weft-index", GUINT_TO_POINTER(options_.size()));
    g_signal_connect(button, "clicked", G_CALLBACK(clicked), this);
    gtk_box_append(GTK_BOX(options_box_), GTK_WIDGET(button));
    options_.push_back(button);
  }
  show();
}

void Picker::set_placeholder(std::string_view placeholder) {
  placeholder_ = placeholder;
  show();
}

void Picker::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

void Picker::clicked(GtkButton *button, Picker *self) {
  const double index = GPOINTER_TO_UINT(g_object_get_data(G_OBJECT(button), "weft-index"));
  auto *menu = GTK_MENU_BUTTON(self->widget());
  gtk_menu_button_popdown(menu);
  if (index == self->selected_) return;
  gtk_label_set_text(self->label_, gtk_button_get_label(button));
  if (!self->edit_(index)) self->show();
}

void Picker::clear_options() {
  for (auto *button : options_) {
    g_signal_handlers_disconnect_by_data(button, this);
    gtk_box_remove(GTK_BOX(options_box_), GTK_WIDGET(button));
  }
  options_.clear();
}

void Picker::show() {
  const char *label = selected_ >= 0 && selected_ < options_.size()
                          ? gtk_button_get_label(options_[size_t(selected_)])
                          : placeholder_.c_str();
  gtk_label_set_text(label_, label);
}

Spinner::Spinner() : Element(spinner_new()) {
  set_width(20);
  set_height(20);
  reapply_colors();
}

void Spinner::set_color(Color color) {
  color_ = color;
  reapply_colors();
}

void Spinner::reapply_colors() {
  const bool dark_mode = dark();
  const auto color = color_.value_or(Color::rgba(dark_mode ? 0xf5f5f7ff : 0x000000ff));
  spinner_set_color(widget(), color.resolve(dark_mode));
}

Progress::Progress() : Element(progress_new()) {
  set_height(4);
  set_fill(true);
  reapply_colors();
}

void Progress::set_value(double value) { progress_set_value(widget(), value); }

void Progress::set_color(Color color) {
  color_ = color;
  reapply_colors();
}

void Progress::reapply_colors() {
  const bool dark_mode = dark();
  progress_set_color(widget(), color_.resolve(dark_mode), dark_mode);
}

Divider::Divider(bool vertical)
    : Element(widget_new(gtk_separator_new(vertical ? GTK_ORIENTATION_VERTICAL : GTK_ORIENTATION_HORIZONTAL))) {
  if (vertical) set_width(1);
  else set_height(1);
  set_fill(true);
  gtk_widget_set_can_target(widget(), false);
}

void Divider::set_color(Color color) {
  color_ = color;
  gtk_widget_set_visible(gtk_widget_get_first_child(widget()), false);
  reapply_colors();
}

void Divider::reapply_colors() {
  if (!color_) return;
  widget_style(widget()).background = color_->resolve();
  gtk_widget_queue_draw(widget());
}

}  // namespace weft
