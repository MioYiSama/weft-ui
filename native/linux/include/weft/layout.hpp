#pragma once

#include <optional>
#include <span>

#include <weft/elements.hpp>

namespace weft::layout {

struct Insets {
  double top = 0, right = 0, bottom = 0, left = 0;

  double horizontal() const { return left + right; }
  double vertical() const { return top + bottom; }
};

struct Length {
  enum class Mode { Unbounded, AtMost, Exact };
  Mode mode = Mode::Unbounded;
  double value = 0;

  static Length exact(double value);
  static Length at_most(double value);
  bool bounded() const { return mode != Mode::Unbounded; }
  bool is_exact() const { return mode == Mode::Exact; }
  double fit(double natural) const;
  Length inset(double amount) const;
};

struct Proposal {
  Length width, height;
};

struct Size {
  double width = 0, height = 0;
};

struct Frame {
  double x = 0, y = 0;
  Size size;
};

struct Rules {
  std::optional<double> width, height, min_width, max_width, min_height, max_height, aspect_ratio;

  // Apply content-box rules, expressing the result in border-box dimensions.
  Proposal propose(Proposal proposal, Insets padding) const;
  Size clamp(Size size, Insets padding) const;
};

struct Child {
  const void *data;
  Size (*measure)(const void *, Proposal);
  Insets margin;
  double grow = 0;
  bool fill = false;
  bool fixed_width = false;
  bool fixed_height = false;
  bool spacer = false;
  std::optional<double> spacer_size;
  Frame frame;  // Output, using the same order as the inputs.
};

struct Linear {
  Axis axis = Axis::Vertical;
  double gap = 0;
  CrossAlign align = CrossAlign::Start;
  Justify justify = Justify::Start;
};

// Children must already exclude hidden elements. Proposals and output size
// include padding (and any border added to it), but not the container's margin.
Size linear(Linear options, Insets padding, Proposal proposal, std::span<Child> children);
Size stack(Alignment alignment, Insets padding, Proposal proposal, std::span<Child> children);

}  // namespace weft::layout
