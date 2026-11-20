// Built-in elements. Each wraps its widget and exposes the element's
// properties as `set_<name>`; properties never set keep the platform's
// defaults. Lengths are logical pixels.

#include <libsoup/soup.h>

#include <algorithm>
#include <cmath>
#include <string>
#include <utility>

#include "appearance.hpp"
#include "container.hpp"
#include "texture_cache.hpp"
#include "weft/gtk.hpp"

namespace weft {

namespace {

int pixels(double length) { return int(std::lround(length)); }

std::string owned(std::string_view text) { return std::string(text); }

}  // namespace

// Values.

double number(const ops::Value &value) {
  auto *number = std::get_if<double>(&value);
  return number ? *number : 0;
}

bool boolean(const ops::Value &value) {
  auto *flag = std::get_if<bool>(&value);
  return flag && *flag;
}

std::string_view string(const ops::Value &value) {
  auto *text = std::get_if<std::string_view>(&value);
  return text ? *text : std::string_view();
}

// Element.

Element::Element(GtkWidget *widget) : widget_(GTK_WIDGET(g_object_ref_sink(widget))), layout_(attach_layout(widget)) {}

Element::~Element() {
  // The widget may outlive the element, e.g. while GTK emits one of its
  // signals; its handlers must not reach the element any more.
  g_signal_handlers_disconnect_by_data(widget_, this);
  g_object_unref(widget_);
}

void Element::set_opacity(double opacity) { gtk_widget_set_opacity(widget_, std::clamp(opacity, 0.0, 1.0)); }
void Element::set_hidden(bool hidden) { gtk_widget_set_visible(widget_, !hidden); }

void Element::set_width(double width) {
  layout_.size.width = width;
  gtk_widget_queue_resize(widget_);
}
void Element::set_height(double height) {
  layout_.size.height = height;
  gtk_widget_queue_resize(widget_);
}
void Element::set_min_width(double min_width) {
  layout_.size.min_width = min_width;
  gtk_widget_queue_resize(widget_);
}
void Element::set_max_width(double max_width) {
  layout_.size.max_width = max_width;
  gtk_widget_queue_resize(widget_);
}
void Element::set_min_height(double min_height) {
  layout_.size.min_height = min_height;
  gtk_widget_queue_resize(widget_);
}
void Element::set_max_height(double max_height) {
  layout_.size.max_height = max_height;
  gtk_widget_queue_resize(widget_);
}
void Element::set_aspect_ratio(double aspect_ratio) {
  layout_.size.aspect_ratio = aspect_ratio;
  gtk_widget_queue_resize(widget_);
}
void Element::set_grow(double grow) {
  layout_.grow = grow;
  gtk_widget_queue_resize(widget_);
}
void Element::set_fill(bool fill) {
  layout_.fill = fill;
  gtk_widget_queue_resize(widget_);
}

void Element::set_margin_edge(Edge edge, double value) {
  layout_.margin.set(edge, value);
  auto [top, right, bottom, left] = layout_.margin.resolve();
  // GTK margins are unsigned; layout and allocation still use the signed values.
  const auto native = [](double margin) { return pixels(std::clamp(margin, 0.0, double(G_MAXINT16))); };
  gtk_widget_set_margin_top(widget_, native(top));
  gtk_widget_set_margin_end(widget_, native(right));
  gtk_widget_set_margin_bottom(widget_, native(bottom));
  gtk_widget_set_margin_start(widget_, native(left));
  gtk_widget_queue_resize(widget_);
}
void Element::set_margin(double margin) { set_margin_edge(Edge::All, margin); }
void Element::set_margin_x(double margin_x) { set_margin_edge(Edge::X, margin_x); }
void Element::set_margin_y(double margin_y) { set_margin_edge(Edge::Y, margin_y); }
void Element::set_margin_top(double margin_top) { set_margin_edge(Edge::Top, margin_top); }
void Element::set_margin_bottom(double margin_bottom) { set_margin_edge(Edge::Bottom, margin_bottom); }
void Element::set_margin_start(double margin_start) { set_margin_edge(Edge::Start, margin_start); }
void Element::set_margin_end(double margin_end) { set_margin_edge(Edge::End, margin_end); }

void Decorated::set_background(Color color) {
  background_ = color;
  widget_style(widget()).background = color.resolve();
  gtk_widget_queue_draw(widget());
}

void Decorated::set_radius(double radius) {
  widget_style(widget()).radius = std::max(0.0, radius);
  gtk_widget_queue_draw(widget());
}

void Decorated::set_border(double border) {
  widget_style(widget()).border = std::max(0.0, border);
  resolve_border_color();
  gtk_widget_queue_resize(widget());
  gtk_widget_queue_draw(widget());
}

void Decorated::set_border_color(Color color) {
  border_color_ = color;
  resolve_border_color();
  gtk_widget_queue_draw(widget());
}

void Decorated::set_shadow(Shadow shadow) {
  widget_style(widget()).shadow = shadow;
  gtk_widget_queue_draw(widget());
}

void Decorated::set_clip(bool clip) {
  widget_style(widget()).clip = clip;
  gtk_widget_queue_draw(widget());
}

void Decorated::resolve_border_color() {
  const auto color = border_color_.value_or(Color::rgba(dark() ? 0xffffff26u : 0x0000001fu));
  widget_style(widget()).border_color = color.resolve();
}

void Decorated::reapply_colors() {
  const bool background_changes = background_.is_token();
  const bool border_changes = widget_style(widget()).border > 0 && (!border_color_ || border_color_->is_token());
  if (background_changes) widget_style(widget()).background = background_.resolve();
  if (border_changes) resolve_border_color();
  if (background_changes || border_changes) gtk_widget_queue_draw(widget());
}

Padded::Padded(GtkWidget *widget, GtkWidget *content) : Decorated(widget) {
  layout().padding_widget = content ? content : widget;
}

void Padded::set_padding_edge(Edge edge, double value) { container_set_padding(layout().padding_widget, edge, value); }
void Padded::set_padding(double padding) { set_padding_edge(Edge::All, padding); }
void Padded::set_padding_x(double padding_x) { set_padding_edge(Edge::X, padding_x); }
void Padded::set_padding_y(double padding_y) { set_padding_edge(Edge::Y, padding_y); }
void Padded::set_padding_top(double padding_top) { set_padding_edge(Edge::Top, padding_top); }
void Padded::set_padding_bottom(double padding_bottom) { set_padding_edge(Edge::Bottom, padding_bottom); }
void Padded::set_padding_start(double padding_start) { set_padding_edge(Edge::Start, padding_start); }
void Padded::set_padding_end(double padding_end) { set_padding_edge(Edge::End, padding_end); }

// Containers.

Linear::Linear(Axis axis, CrossAlign align) : Padded(linear_new(axis, align)), children_(widget()) {}

void Linear::set_gap(double gap) { container_set_gap(widget(), gap); }
void Linear::set_align(CrossAlign align) { container_set_align(widget(), align); }
void Linear::set_justify(Justify justify) { container_set_justify(widget(), justify); }

Stack::Stack() : Padded(overlay_new()), children_(widget()) {}
void Stack::set_alignment(Alignment alignment) { container_set_alignment(widget(), alignment); }

Scroll::Scroll(Axis direction) : Scroll(direction, linear_new(direction, CrossAlign::Start, true)) {}

Scroll::Scroll(Axis direction, GtkWidget *content)
    : Padded(widget_new(gtk_scrolled_window_new()), content), direction_(direction),
      scroll_(GTK_SCROLLED_WINDOW(gtk_widget_get_first_child(widget()))), content_(content), children_(content) {
  gtk_scrolled_window_set_propagate_natural_width(scroll_, true);
  gtk_scrolled_window_set_propagate_natural_height(scroll_, true);
  gtk_scrolled_window_set_child(scroll_, content_);
  set_indicators(true);
  layout().grow = 1;
  layout().fill = true;
}

void Scroll::set_gap(double gap) { container_set_gap(content_, gap); }
void Scroll::set_align(CrossAlign align) { container_set_align(content_, align); }
void Scroll::set_indicators(bool indicators) {
  const auto active = indicators ? GTK_POLICY_AUTOMATIC : GTK_POLICY_EXTERNAL;
  const bool horizontal = direction_ == Axis::Horizontal;
  gtk_scrolled_window_set_policy(scroll_, horizontal ? active : GTK_POLICY_NEVER,
                                 horizontal ? GTK_POLICY_NEVER : active);
}

Spacer::Spacer() : Element(widget_new()) { layout().spacer = true; }
void Spacer::set_size(double size) {
  layout().spacer_size = std::max(0.0, size);
  gtk_widget_queue_resize(widget());
}

// Text.

Text::Text() : Padded(linear_new(Axis::Vertical, CrossAlign::Stretch)), label_(GTK_LABEL(gtk_label_new(nullptr))) {
  gtk_label_set_xalign(label_, 0);
  gtk_label_set_wrap(label_, true);
  gtk_label_set_wrap_mode(label_, PANGO_WRAP_WORD_CHAR);
  gtk_widget_set_parent(GTK_WIDGET(label_), widget());
}

void Text::set_value(std::string_view value) { gtk_label_set_text(label_, owned(value).c_str()); }

void Text::set_color(Color color) {
  color_ = color;
  restyle();
}

void Text::reapply_colors() {
  Decorated::reapply_colors();
  if (color_ && color_->is_token()) restyle();
}

void Text::set_size(double size) {
  size_ = size;
  restyle();
}

void Text::set_bold(bool bold) {
  bold_ = bold;
  restyle();
}

void Text::restyle() {
  PangoAttrList *attributes = pango_attr_list_new();
  if (color_) {
    const GdkRGBA color = color_->resolve();
    auto channel = [](float value) { return guint16(std::lround(value * 65535)); };
    pango_attr_list_insert(attributes,
                           pango_attr_foreground_new(channel(color.red), channel(color.green), channel(color.blue)));
    pango_attr_list_insert(attributes, pango_attr_foreground_alpha_new(std::max<guint16>(channel(color.alpha), 1)));
  }
  // Absolute sizes are in logical pixels, like CSS `font-size`.
  if (size_ > 0) pango_attr_list_insert(attributes, pango_attr_size_new_absolute(int(size_ * PANGO_SCALE)));
  if (bold_) pango_attr_list_insert(attributes, pango_attr_weight_new(PANGO_WEIGHT_BOLD));
  gtk_label_set_attributes(label_, attributes);
  pango_attr_list_unref(attributes);
}

// Button.

Button::Button(std::function<void()> click)
    : Element(gtk_button_new()), click_(std::move(click)), content_(Axis::Horizontal, CrossAlign::Center) {
  gtk_button_set_child(GTK_BUTTON(widget()), content_.widget());
  if (click_) g_signal_connect(widget(), "clicked", G_CALLBACK(clicked), this);
}

void Button::clicked(GtkButton *, Button *self) { self->click_(); }

void Button::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

// Input.

namespace {

/// Natural width of an `Input` when no width is specified.
constexpr int kInputWidth = 200;

}  // namespace

Input::Input(std::function<bool(std::string_view)> edit) : Element(gtk_entry_new()), edit_(std::move(edit)) {
  gtk_widget_set_size_request(widget(), kInputWidth, -1);
  g_signal_connect(widget(), "changed", G_CALLBACK(changed), this);
  GtkEditable *text = gtk_editable_get_delegate(GTK_EDITABLE(widget()));
  g_signal_connect(text, "preedit-changed", G_CALLBACK(preedit_changed), this);
}

Input::~Input() {
  g_signal_handlers_disconnect_by_data(gtk_editable_get_delegate(GTK_EDITABLE(widget())), this);
}

void Input::set_value(std::string_view value) {
  value_ = value;
  if (!composing_ && gtk_editable_get_text(GTK_EDITABLE(widget())) != value_) show(value_);
}

void Input::set_placeholder(std::string_view placeholder) {
  gtk_entry_set_placeholder_text(GTK_ENTRY(widget()), owned(placeholder).c_str());
}

void Input::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

void Input::changed(GtkEditable *, Input *self) { self->edited(); }

void Input::preedit_changed(GtkText *, const char *preedit, Input *self) {
  bool composing = preedit && *preedit;
  if (composing == self->composing_) return;
  self->composing_ = composing;
  if (!composing) self->edited();
}

void Input::edited() {
  if (showing_ || composing_) return;
  std::string text = gtk_editable_get_text(GTK_EDITABLE(widget()));
  if (text == value_) return;
  // The handler applies the app's ops, which may write `value_`, before this returns.
  if (!edit_(text)) show(value_);
}

void Input::show(const std::string &text) {
  auto *editable = GTK_EDITABLE(widget());
  int caret = gtk_editable_get_position(editable);
  // Replacing the text deletes it first; that intermediate edit is not the user's.
  showing_ = true;
  gtk_editable_set_text(editable, text.c_str());
  showing_ = false;
  gtk_editable_set_position(editable, std::min(caret, int(g_utf8_strlen(text.c_str(), -1))));
}

// Checkbox.

Checkbox::Checkbox(std::function<bool(bool)> edit) : Element(gtk_check_button_new()), edit_(std::move(edit)) {
  g_signal_connect(widget(), "toggled", G_CALLBACK(toggled), this);
}

void Checkbox::set_checked(bool checked) {
  value_ = checked;
  gtk_check_button_set_active(GTK_CHECK_BUTTON(widget()), checked);
}

void Checkbox::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

void Checkbox::toggled(GtkCheckButton *button, Checkbox *self) {
  bool now = gtk_check_button_get_active(button);
  if (now == self->value_) return;
  // Accepted edits write `value_` through a `set` op before `edit_` returns.
  if (!self->edit_(now)) gtk_check_button_set_active(button, self->value_);
}

// Image.

namespace {

/// Downloaded images by source, shared by every `Image` of the process.
TextureCache &textures() {
  static TextureCache textures(64 * 1024 * 1024);
  return textures;
}

SoupSession *images_session() {
  static SoupSession *session = soup_session_new();
  return session;
}

/// A load in flight: the picture it is for, unless it is gone, and the source.
struct Load {
  GWeakRef picture;
  std::string src;
};

/// Shows `bytes` in the picture of `load` if it still shows that source.
void loaded(Load *load, GBytes *bytes, GError *error) {
  GdkTexture *texture = nullptr;
  if (bytes) texture = gdk_texture_new_from_bytes(bytes, &error);
  if (!texture) {
    std::fprintf(stderr, "[weft warn] weft: cannot load the image %s (%s)\n", load->src.c_str(),
                 error ? error->message : "unknown error");
  }
  if (auto *picture = static_cast<GtkPicture *>(g_weak_ref_get(&load->picture))) {
    auto *shown = static_cast<const char *>(g_object_get_data(G_OBJECT(picture), "weft-src"));
    if (texture && shown && load->src == shown) gtk_picture_set_paintable(picture, GDK_PAINTABLE(texture));
    g_object_unref(picture);
  }
  if (texture) {
    textures().insert(std::move(load->src), texture);
    g_object_unref(texture);
  }
  if (error) g_error_free(error);
  if (bytes) g_bytes_unref(bytes);
  g_weak_ref_clear(&load->picture);
  delete load;
}

}  // namespace

Image::Image() : Decorated(widget_new(gtk_picture_new())), picture_(GTK_PICTURE(gtk_widget_get_first_child(widget()))) {
  gtk_picture_set_can_shrink(picture_, true);
}

void Image::set_src(std::string_view src) {
  std::string source(src);
  g_object_set_data_full(G_OBJECT(picture_), "weft-src", g_strdup(source.c_str()), g_free);
  if (auto *cached = textures().find(source)) {
    gtk_picture_set_paintable(picture_, GDK_PAINTABLE(cached));
    return;
  }
  gtk_picture_set_paintable(picture_, nullptr);
  if (source.empty()) return;
  auto *load = new Load{{}, source};
  g_weak_ref_init(&load->picture, picture_);
  if (g_str_has_prefix(source.c_str(), "http:") || g_str_has_prefix(source.c_str(), "https:")) {
    SoupMessage *message = soup_message_new(SOUP_METHOD_GET, source.c_str());
    if (!message) {
      loaded(load, nullptr, g_error_new(G_IO_ERROR, G_IO_ERROR_INVALID_ARGUMENT, "invalid URL"));
      return;
    }
    soup_session_send_and_read_async(
        images_session(), message, G_PRIORITY_LOW, nullptr,
        [](GObject *session, GAsyncResult *result, gpointer data) {
          GError *error = nullptr;
          GBytes *bytes = soup_session_send_and_read_finish(SOUP_SESSION(session), result, &error);
          auto *message = soup_session_get_async_result_message(SOUP_SESSION(session), result);
          if (bytes && !SOUP_STATUS_IS_SUCCESSFUL(soup_message_get_status(message))) {
            error = g_error_new(G_IO_ERROR, G_IO_ERROR_FAILED, "HTTP %u", soup_message_get_status(message));
            g_bytes_unref(bytes);
            bytes = nullptr;
          }
          loaded(static_cast<Load *>(data), bytes, error);
        },
        load);
    g_object_unref(message);
    return;
  }
  GFile *file = g_file_new_for_commandline_arg(source.c_str());
  g_file_load_bytes_async(
      file, nullptr,
      [](GObject *file, GAsyncResult *result, gpointer data) {
        GError *error = nullptr;
        GBytes *bytes = g_file_load_bytes_finish(G_FILE(file), result, nullptr, &error);
        loaded(static_cast<Load *>(data), bytes, error);
      },
      load);
  g_object_unref(file);
}

void Image::set_width(double width) {
  Element::set_width(width);
  update_fit();
}

void Image::set_height(double height) {
  Element::set_height(height);
  update_fit();
}

void Image::update_fit() {
  const auto &size = layout().size;
  gtk_picture_set_content_fit(picture_, size.width && size.height ? GTK_CONTENT_FIT_FILL : GTK_CONTENT_FIT_CONTAIN);
}

}  // namespace weft
