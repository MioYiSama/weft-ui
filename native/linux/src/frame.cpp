#include "frame.hpp"

#include <stdexcept>

namespace weft {

JsonNode *parse_json(std::string_view json, const char *what) {
  JsonParser *parser = json_parser_new();
  GError *error = nullptr;
  bool ok = json_parser_load_from_data(parser, json.data(), gssize(json.size()), &error);
  JsonNode *root = ok ? json_node_copy(json_parser_get_root(parser)) : nullptr;
  g_object_unref(parser);
  if (!ok) {
    std::string message = std::string("weft: invalid ") + what + " (" + error->message + ")";
    g_error_free(error);
    throw std::runtime_error(message);
  }
  return root;
}

namespace frame {

std::pair<JsonNode *, std::vector<uint8_t>> unframe(const std::vector<uint8_t> &data) {
  if (data.size() < 4) throw std::runtime_error("weft: truncated frame");
  size_t length = size_t(data[0]) | size_t(data[1]) << 8 | size_t(data[2]) << 16 | size_t(data[3]) << 24;
  if (length > data.size() - 4) throw std::runtime_error("weft: invalid frame head");
  JsonNode *head = parse_json({reinterpret_cast<const char *>(data.data() + 4), length}, "frame head");
  if (!JSON_NODE_HOLDS_OBJECT(head)) {
    json_node_unref(head);
    throw std::runtime_error("weft: invalid frame head");
  }
  return {head, {data.begin() + 4 + std::ptrdiff_t(length), data.end()}};
}

std::vector<uint8_t> frame(const std::string &head, const uint8_t *body, size_t len) {
  std::vector<uint8_t> data;
  data.reserve(4 + head.size() + len);
  for (int shift = 0; shift < 32; shift += 8) data.push_back(uint8_t(head.size() >> shift & 0xff));
  data.insert(data.end(), head.begin(), head.end());
  data.insert(data.end(), body, body + len);
  return data;
}

}  // namespace frame
}  // namespace weft
