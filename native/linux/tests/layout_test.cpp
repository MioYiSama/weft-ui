#include <json-glib/json-glib.h>
#include <weft/layout.hpp>

#include <string_view>
#include <vector>

namespace {

using namespace weft;
using namespace weft::layout;

struct Leaf {
  Size intrinsic;
  Rules rules;
};

Size measure(const void *data, Proposal proposal) {
  const auto &leaf = *static_cast<const Leaf *>(data);
  const auto [w, h] = leaf.rules.propose(proposal, {});
  return leaf.rules.clamp({w.fit(leaf.intrinsic.width), h.fit(leaf.intrinsic.height)}, {});
}

std::optional<double> optional_number(JsonObject *item, const char *key) {
  if (!json_object_has_member(item, key)) return std::nullopt;
  return json_object_get_double_member(item, key);
}

bool optional_bool(JsonObject *item, const char *key) {
  return json_object_has_member(item, key) && json_object_get_boolean_member(item, key);
}

Insets insets(JsonArray *array) {
  if (!array) return {};
  return {json_array_get_double_element(array, 0), json_array_get_double_element(array, 1),
          json_array_get_double_element(array, 2), json_array_get_double_element(array, 3)};
}

Length length(JsonArray *array, guint index) {
  JsonNode *node = json_array_get_element(array, index);
  if (JSON_NODE_HOLDS_NULL(node)) return {};
  auto *item = json_node_get_object(node);
  if (auto exact = optional_number(item, "exact")) return Length::exact(*exact);
  return Length::at_most(json_object_get_double_member(item, "atMost"));
}

void expect(double actual, JsonArray *array, guint offset) {
  // The reference records fractional pixel positions to three decimals.
  g_assert_cmpfloat_with_epsilon(actual, json_array_get_double_element(array, offset), 0.001);
}

void run_case(JsonObject *item) {
  g_print("%s\n", json_object_get_string_member(item, "name"));
  const Axis axis = parse_axis(json_object_get_string_member(item, "axis"));
  const auto padding = insets(json_object_get_array_member(item, "padding"));
  auto *bounds = json_object_get_array_member(item, "proposal");
  const Length main = length(bounds, 0), cross = length(bounds, 1);
  const Proposal proposal = axis == Axis::Vertical ? Proposal{cross, main} : Proposal{main, cross};
  auto *items = json_object_get_array_member(item, "children");
  const auto count = json_array_get_length(items);

  std::vector<Leaf> leaves;
  std::vector<Child> children;
  leaves.reserve(count);
  children.reserve(count);
  for (guint i = 0; i < count; ++i) {
    auto *child = json_array_get_object_element(items, i);
    auto *intrinsic = json_object_get_array_member(child, "intrinsic");
    const Rules rules{optional_number(child, "width"), optional_number(child, "height"),
                      optional_number(child, "minWidth"), optional_number(child, "maxWidth"),
                      optional_number(child, "minHeight"), optional_number(child, "maxHeight"),
                      optional_number(child, "aspectRatio")};
    leaves.push_back({{json_array_get_double_element(intrinsic, 0), json_array_get_double_element(intrinsic, 1)}, rules});
    const auto margin = json_object_has_member(child, "margin") ? json_object_get_array_member(child, "margin") : nullptr;
    children.push_back({&leaves.back(), measure, insets(margin), optional_number(child, "grow").value_or(0),
                        optional_bool(child, "fill"), rules.width.has_value(), rules.height.has_value(),
                        optional_bool(child, "spacer"), optional_number(child, "size"), {}});
  }

  Size measured;
  const std::string_view kind = json_object_get_string_member(item, "kind");
  if (kind == "linear") {
    const Linear options{axis, json_object_get_double_member(item, "gap"),
                         parse_cross_align(json_object_get_string_member(item, "align")),
                         parse_justify(json_object_get_string_member(item, "justify"))};
    measured = linear(options, padding, proposal, children);
  } else {
    g_assert_true(kind == "stack");
    measured = stack(parse_alignment(json_object_get_string_member(item, "alignment")), padding, proposal, children);
  }
  auto *size = json_object_get_array_member(item, "size");
  expect(measured.width, size, 0);
  expect(measured.height, size, 1);
  auto *frames = json_object_get_array_member(item, "frames");
  g_assert_cmpuint(json_array_get_length(frames), ==, children.size());
  for (guint i = 0; i < count; ++i) {
    auto *expected = json_array_get_array_element(frames, i);
    const auto &frame = children[i].frame;
    expect(frame.x, expected, 0);
    expect(frame.y, expected, 1);
    expect(frame.size.width, expected, 2);
    expect(frame.size.height, expected, 3);
  }
}

void content_box_constraints() {
  const Insets padding{3, 7, 5, 11};
  Rules rules;
  rules.width = 90;
  rules.max_width = 50;
  rules.min_height = 30;
  rules.aspect_ratio = 2;
  const auto [width, height] = rules.propose({Length::exact(120), Length::at_most(10)}, padding);
  g_assert_true(width.is_exact());
  g_assert_true(height.is_exact());
  g_assert_cmpfloat(width.value, ==, 68);   // max width 50 plus horizontal padding
  g_assert_cmpfloat(height.value, ==, 38);  // ratio gives 25; min height raises it to 30
}

}  // namespace

int main() {
  JsonParser *parser = json_parser_new();
  GError *error = nullptr;
  json_parser_load_from_file(parser, WEFT_CORPUS_DIR "/layout.json", &error);
  g_assert_no_error(error);
  auto *corpus = json_node_get_object(json_parser_get_root(parser));
  auto *cases = json_object_get_array_member(corpus, "cases");
  for (guint i = 0; i < json_array_get_length(cases); ++i) run_case(json_array_get_object_element(cases, i));
  g_object_unref(parser);
  content_box_constraints();
}
