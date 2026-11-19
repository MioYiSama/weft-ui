#include "frame.hpp"

#include <cstring>
#include <stdexcept>
#include <string_view>

namespace {

std::vector<uint8_t> from_hex(std::string_view text) {
  std::vector<uint8_t> bytes;
  int high = -1;
  for (char ch : text) {
    if (g_ascii_isspace(ch)) continue;
    int digit = g_ascii_xdigit_value(ch);
    g_assert_cmpint(digit, >=, 0);
    if (high < 0) {
      high = digit;
    } else {
      bytes.push_back(uint8_t((high << 4) | digit));
      high = -1;
    }
  }
  g_assert_cmpint(high, ==, -1);
  return bytes;
}

void request(JsonObject *item) {
  auto [head, body] = weft::frame::unframe(from_hex(json_object_get_string_member(item, "hex")));
  g_assert_true(json_node_equal(head, json_object_get_member(item, "head")));
  std::string_view expected = json_object_get_string_member(item, "body");
  g_assert_cmpmem(body.data(), body.size(), expected.data(), expected.size());
  json_node_unref(head);
}

void response(JsonObject *item) {
  JsonNode *expected_head = json_object_get_member(item, "head");
  std::string_view expected_body = json_object_get_string_member(item, "body");
  char *head = json_to_string(expected_head, false);
  auto data = weft::frame::frame(head, reinterpret_cast<const uint8_t *>(expected_body.data()), expected_body.size());
  g_free(head);

  uint32_t length;
  g_assert_cmpuint(data.size(), >=, sizeof(length));
  std::memcpy(&length, data.data(), sizeof(length));
  length = GUINT32_FROM_LE(length);
  g_assert_cmpuint(length, <=, data.size() - sizeof(length));

  JsonParser *parser = json_parser_new();
  GError *error = nullptr;
  json_parser_load_from_data(parser, reinterpret_cast<const char *>(data.data() + sizeof(length)), length, &error);
  g_assert_no_error(error);
  g_assert_true(json_node_equal(json_parser_get_root(parser), expected_head));
  g_object_unref(parser);

  size_t offset = sizeof(length) + length;
  g_assert_cmpmem(data.data() + offset, data.size() - offset, expected_body.data(), expected_body.size());
}

void malformed(JsonObject *item) {
  auto data = from_hex(json_object_get_string_member(item, "hex"));
  try {
    auto result = weft::frame::unframe(data);
    json_node_unref(result.first);
  } catch (const std::runtime_error &) {
    return;
  }
  g_error("unframe accepted malformed input");
}

void run_cases(JsonObject *corpus, const char *group, void (*test)(JsonObject *)) {
  JsonArray *cases = json_object_get_array_member(corpus, group);
  g_assert_nonnull(cases);
  for (guint i = 0; i < json_array_get_length(cases); ++i) {
    JsonObject *item = json_array_get_object_element(cases, i);
    g_print("%s/%s\n", group, json_object_get_string_member(item, "name"));
    test(item);
  }
}

}  // namespace

int main() {
  JsonParser *parser = json_parser_new();
  GError *error = nullptr;
  json_parser_load_from_file(parser, WEFT_CORPUS_DIR "/frames.json", &error);
  g_assert_no_error(error);
  JsonObject *corpus = json_node_get_object(json_parser_get_root(parser));
  run_cases(corpus, "requests", request);
  run_cases(corpus, "responses", response);
  run_cases(corpus, "malformed", malformed);
  g_object_unref(parser);
}
