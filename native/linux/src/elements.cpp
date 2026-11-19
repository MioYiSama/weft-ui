// Built-in elements. Each wraps its widget and exposes the element's
// properties as `set_<name>`; properties never set keep the platform's
// defaults. Lengths are logical pixels.

#include <libsoup/soup.h>

#include <cmath>
#include <cstdio>
#include <string>
#include <utility>

#include "stack.hpp"
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

GdkRGBA color(const ops::Value &value) {
  GdkRGBA parsed{};
  if (!parse_color(string(value), parsed)) {
#ifndef NDEBUG
    std::fprintf(stderr, "[weft warn] weft: invalid color %.*s, using transparent\n", int(string(value).size()),
                 string(value).data());
#endif
    return {};
  }
  return parsed;
}

bool parse_color(std::string_view text, GdkRGBA &color) {
  if ((text.size() != 4 && text.size() != 7 && text.size() != 9) || text[0] != '#') return false;
  uint32_t value = 0;
  for (char c : text.substr(1)) {
    int digit = g_ascii_xdigit_value(c);
    if (digit < 0) return false;
    value = value << 4 | uint32_t(digit);
  }
  switch (text.size()) {
    case 4:
      color = rgba((value >> 8 & 0xf) * 17 << 24 | (value >> 4 & 0xf) * 17 << 16 | (value & 0xf) * 17 << 8 | 0xff);
      break;
    case 7:
      color = rgba(value << 8 | 0xff);
      break;
    default:
      color = rgba(value);
  }
  return true;
}

// Element.

Element::Element(GtkWidget *widget) : widget_(GTK_WIDGET(g_object_ref_sink(widget))) {}

Element::~Element() {
  // The widget may outlive the element, e.g. while GTK emits one of its
  // signals; its handlers must not reach the element any more.
  g_signal_handlers_disconnect_by_data(widget_, this);
  g_object_unref(widget_);
}

// Stack.

Stack::Stack(Axis axis, Align align) : Element(stack_new(axis, align)), children_(widget()) {}

void Stack::set_gap(double gap) { stack_set_gap(WEFT_STACK(widget()), pixels(gap)); }

void Stack::set_padding(double padding) { stack_set_padding(WEFT_STACK(widget()), pixels(padding)); }

void Stack::set_width(double width) { stack_set_width(WEFT_STACK(widget()), pixels(width)); }

void Stack::set_height(double height) { stack_set_height(WEFT_STACK(widget()), pixels(height)); }

void Stack::set_background(GdkRGBA color) { stack_set_background(WEFT_STACK(widget()), color); }

// Text.

Text::Text() : Element(gtk_label_new(nullptr)) { gtk_label_set_xalign(GTK_LABEL(widget()), 0); }

void Text::set_value(std::string_view value) { gtk_label_set_text(GTK_LABEL(widget()), owned(value).c_str()); }

void Text::set_color(GdkRGBA color) {
  color_ = color;
  has_color_ = true;
  restyle();
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
  if (has_color_) {
    auto channel = [](float value) { return guint16(std::lround(value * 65535)); };
    pango_attr_list_insert(attributes,
                           pango_attr_foreground_new(channel(color_.red), channel(color_.green), channel(color_.blue)));
    pango_attr_list_insert(attributes, pango_attr_foreground_alpha_new(std::max<guint16>(channel(color_.alpha), 1)));
  }
  // Absolute sizes are in logical pixels, like CSS `font-size`.
  if (size_ > 0) pango_attr_list_insert(attributes, pango_attr_size_new_absolute(int(size_ * PANGO_SCALE)));
  if (bold_) pango_attr_list_insert(attributes, pango_attr_weight_new(PANGO_WEIGHT_BOLD));
  gtk_label_set_attributes(GTK_LABEL(widget()), attributes);
  pango_attr_list_unref(attributes);
}

// Button.

Button::Button(std::function<void()> click)
    : Element(gtk_button_new()), click_(std::move(click)), content_(Axis::Horizontal, Align::Center) {
  gtk_button_set_child(GTK_BUTTON(widget()), content_.widget());
  if (click_) g_signal_connect(widget(), "clicked", G_CALLBACK(clicked), this);
}

void Button::clicked(GtkButton *, Button *self) { self->click_(); }

void Button::set_disabled(bool disabled) { gtk_widget_set_sensitive(widget(), !disabled); }

// Input.

namespace {

/// Width of an `Input`, which has no width property.
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

Image::Image() : Element(gtk_picture_new()) {
  gtk_picture_set_can_shrink(GTK_PICTURE(widget()), true);
  resize();
}

void Image::set_src(std::string_view src) {
  auto *picture = GTK_PICTURE(widget());
  std::string source(src);
  g_object_set_data_full(G_OBJECT(picture), "weft-src", g_strdup(source.c_str()), g_free);
  if (auto *cached = textures().find(source)) {
    gtk_picture_set_paintable(picture, GDK_PAINTABLE(cached));
    return;
  }
  gtk_picture_set_paintable(picture, nullptr);
  if (source.empty()) return;
  auto *load = new Load{{}, source};
  g_weak_ref_init(&load->picture, picture);
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
  width_ = pixels(width);
  resize();
}

void Image::set_height(double height) {
  height_ = pixels(height);
  resize();
}

void Image::resize() {
  gtk_widget_set_size_request(widget(), width_, height_);
  gtk_picture_set_content_fit(GTK_PICTURE(widget()), width_ >= 0 && height_ >= 0 ? GTK_CONTENT_FIT_FILL
                                                                                  : GTK_CONTENT_FIT_CONTAIN);
}

}  // namespace weft
