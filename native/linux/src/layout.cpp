#include <weft/layout.hpp>

#include <algorithm>
#include <utility>

namespace weft::layout {
namespace {

double clamp_length(double value, std::optional<double> min, std::optional<double> max, double padding) {
  if (max) value = std::min(value, *max + padding);
  return std::max(value, std::max(0.0, min.value_or(0)) + padding);
}

// Discover an exact dimension before measuring dependent dimensions such as wrapped text.
Length propose_axis(Length proposed, std::optional<double> fixed, std::optional<double> min,
                    std::optional<double> max, double padding) {
  if (fixed) return Length::exact(clamp_length(*fixed + padding, min, max, padding));
  if (proposed.is_exact()) return Length::exact(clamp_length(proposed.value, min, max, padding));
  if (proposed.bounded()) return Length::at_most(clamp_length(proposed.value, min, max, padding));
  return max ? Length::at_most(clamp_length(*max + padding, min, max, padding)) : proposed;
}

struct Axes {
  bool vertical;

  template <class T>
  void orient(T &width, T &height) const {
    if (vertical) std::swap(width, height);
  }
};

struct Along {
  double start, end, before, after;
};

Along along(Axes axes, Insets box) {
  return axes.vertical ? Along{box.top, box.bottom, box.left, box.right}
                       : Along{box.left, box.right, box.top, box.bottom};
}

double growth(const Child &child) {
  return child.spacer ? (child.spacer_size ? 0 : 1) : std::max(0.0, child.grow);
}

bool stretch(const Child &child, Axes axes, CrossAlign align) {
  return !child.spacer && (child.fill || align == CrossAlign::Stretch) &&
         !(axes.vertical ? child.fixed_width : child.fixed_height);
}

double cross_offset(CrossAlign align, double free) {
  if (align == CrossAlign::Center) return free / 2;
  if (align == CrossAlign::End) return free;
  return 0;
}

// Leading offset and extra space between consecutive items.
std::pair<double, double> distribution(Justify justify, double rest, size_t count) {
  switch (justify) {
    case Justify::Start: return {0, 0};
    case Justify::Center: return {rest / 2, 0};
    case Justify::End: return {rest, 0};
    case Justify::SpaceBetween: return {0, count > 1 ? rest / (count - 1) : 0};
    case Justify::SpaceAround: return count ? std::pair{rest / (2 * count), rest / count} : std::pair{0.0, 0.0};
    case Justify::SpaceEvenly: return {rest / (count + 1), rest / (count + 1)};
  }
  return {0, 0};
}

std::pair<double, double> alignment_factors(Alignment alignment) {
  switch (alignment) {
    case Alignment::TopStart: return {0, 0};
    case Alignment::Top: return {0.5, 0};
    case Alignment::TopEnd: return {1, 0};
    case Alignment::Start: return {0, 0.5};
    case Alignment::Center: return {0.5, 0.5};
    case Alignment::End: return {1, 0.5};
    case Alignment::BottomStart: return {0, 1};
    case Alignment::Bottom: return {0.5, 1};
    case Alignment::BottomEnd: return {1, 1};
  }
  return {0, 0};
}

}  // namespace

Length Length::exact(double value) { return {Mode::Exact, std::max(0.0, value)}; }
Length Length::at_most(double value) { return {Mode::AtMost, std::max(0.0, value)}; }

double Length::fit(double natural) const {
  if (is_exact()) return value;
  return bounded() ? std::min(value, natural) : natural;
}

Length Length::inset(double amount) const {
  return bounded() ? Length{mode, std::max(0.0, value - amount)} : *this;
}

Proposal Rules::propose(Proposal proposal, Insets padding) const {
  const double x = padding.horizontal(), y = padding.vertical();
  auto w = propose_axis(proposal.width, width, min_width, max_width, x);
  auto h = propose_axis(proposal.height, height, min_height, max_height, y);
  if (aspect_ratio && *aspect_ratio > 0) {
    if (w.is_exact() && !height) {
      h = Length::exact(clamp_length((w.value - x) / *aspect_ratio + y, min_height, max_height, y));
    } else if (h.is_exact() && !width) {
      w = Length::exact(clamp_length((h.value - y) * *aspect_ratio + x, min_width, max_width, x));
    }
  }
  return {w, h};
}

Size Rules::clamp(Size size, Insets padding) const {
  return {clamp_length(size.width, min_width, max_width, padding.horizontal()),
          clamp_length(size.height, min_height, max_height, padding.vertical())};
}

Size linear(Linear options, Insets padding, Proposal proposal, std::span<Child> children) {
  const Axes axes{options.axis == Axis::Vertical};
  const auto [start, end, before, after] = along(axes, padding);
  auto [main, cross] = proposal;
  axes.orient(main, cross);
  main = main.inset(start + end);
  cross = cross.inset(before + after);

  double used = children.empty() ? 0 : options.gap * (children.size() - 1);
  double natural_cross = 0, total_grow = 0;
  const Child *last_growing = nullptr;
  const auto measure = [&](Child &child, Along margin, Length main_size) {
    const double available_cross = cross.value - margin.before - margin.after;
    const auto cross_size = !cross.bounded() ? Length{} : stretch(child, axes, options.align)
                                               ? Length::exact(available_cross) : Length::at_most(available_cross);
    double m, c;
    if (child.spacer) {
      m = child.spacer_size.value_or(main_size.value);
      c = 0;
    } else {
      Proposal child_proposal{main_size, cross_size};
      axes.orient(child_proposal.width, child_proposal.height);
      auto measured = child.measure(child.data, child_proposal);
      m = measured.width;
      c = measured.height;
      axes.orient(m, c);
    }
    child.frame.size = {m, c};
    used += m;
    natural_cross = std::max(natural_cross, c + margin.before + margin.after);
  };

  for (Child &child : children) {
    const auto margin = along(axes, child.margin);
    used += margin.start + margin.end;
    if (const double grow = growth(child); grow > 0) {
      total_grow += grow;
      last_growing = &child;
    } else {
      measure(child, margin, {});
    }
  }
  if (last_growing) {
    const double free = main.bounded() ? std::max(0.0, main.value - used) : 0;
    double remaining = free;
    for (Child &child : children) {
      const double grow = growth(child);
      if (grow <= 0) continue;
      const double portion = &child == last_growing ? remaining : free * (grow / total_grow);
      remaining -= portion;
      measure(child, along(axes, child.margin), main.bounded() ? Length::exact(portion) : Length{});
    }
  }

  const double main_size = main.fit(used), cross_size = cross.fit(natural_cross);
  auto [position, spacing] = distribution(options.justify, main_size - used, children.size());
  position += start;
  for (Child &child : children) {
    const auto margin = along(axes, child.margin);
    auto &[x, y, size] = child.frame;
    x = position + margin.start;
    const double free_cross = cross_size - size.height - margin.before - margin.after;
    y = before + margin.before + (child.fill ? 0 : cross_offset(options.align, free_cross));
    position = x + size.width + margin.end + options.gap + spacing;
    axes.orient(x, y);
    axes.orient(size.width, size.height);
  }
  Size size{main_size + start + end, cross_size + before + after};
  axes.orient(size.width, size.height);
  return size;
}

Size stack(Alignment alignment, Insets padding, Proposal proposal, std::span<Child> children) {
  auto [width, height] = proposal;
  width = width.inset(padding.horizontal());
  height = height.inset(padding.vertical());
  Size natural;
  for (Child &child : children) {
    if (child.fill) continue;
    const auto &margin = child.margin;
    const auto w = width.bounded() ? Length::at_most(width.value - margin.horizontal()) : Length{};
    const auto h = height.bounded() ? Length::at_most(height.value - margin.vertical()) : Length{};
    const auto size = child.frame.size = child.measure(child.data, {w, h});
    natural.width = std::max(natural.width, size.width + margin.horizontal());
    natural.height = std::max(natural.height, size.height + margin.vertical());
  }
  const double w = width.fit(natural.width), h = height.fit(natural.height);
  const auto [x, y] = alignment_factors(alignment);
  for (Child &child : children) {
    const auto &margin = child.margin;
    const double inner_w = w - margin.horizontal(), inner_h = h - margin.vertical();
    auto &frame = child.frame;
    if (child.fill) frame.size = child.measure(child.data, {Length::exact(inner_w), Length::exact(inner_h)});
    frame.x = padding.left + margin.left + (inner_w - frame.size.width) * x;
    frame.y = padding.top + margin.top + (inner_h - frame.size.height) * y;
  }
  return {w + padding.horizontal(), h + padding.vertical()};
}

}  // namespace weft::layout
