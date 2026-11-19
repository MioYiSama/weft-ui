#pragma once

#include <json-glib/json-glib.h>

#include <cstddef>
#include <cstdint>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace weft {

/// Parses `json`, returning an owned node or throwing with `what` on invalid JSON.
JsonNode *parse_json(std::string_view json, const char *what);

namespace frame {

/// Splits `u32 head length (LE) | head (UTF-8 JSON) | body`, returning an owned head node.
std::pair<JsonNode *, std::vector<uint8_t>> unframe(const std::vector<uint8_t> &data);

std::vector<uint8_t> frame(const std::string &head, const uint8_t *body, size_t len);

}  // namespace frame
}  // namespace weft
