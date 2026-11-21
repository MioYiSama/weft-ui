// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

#pragma once

#include <string_view>

namespace weft {

enum class CrossAlign { Start, Center, End, Stretch };

[[nodiscard]] constexpr CrossAlign parse_cross_align(std::string_view value) noexcept {
    if (value == "center") return CrossAlign::Center;
    if (value == "end") return CrossAlign::End;
    if (value == "stretch") return CrossAlign::Stretch;
    return CrossAlign::Start;
}

enum class Justify { Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly };

[[nodiscard]] constexpr Justify parse_justify(std::string_view value) noexcept {
    if (value == "center") return Justify::Center;
    if (value == "end") return Justify::End;
    if (value == "spaceBetween") return Justify::SpaceBetween;
    if (value == "spaceAround") return Justify::SpaceAround;
    if (value == "spaceEvenly") return Justify::SpaceEvenly;
    return Justify::Start;
}

enum class Alignment { TopStart, Top, TopEnd, Start, Center, End, BottomStart, Bottom, BottomEnd };

[[nodiscard]] constexpr Alignment parse_alignment(std::string_view value) noexcept {
    if (value == "top") return Alignment::Top;
    if (value == "topEnd") return Alignment::TopEnd;
    if (value == "start") return Alignment::Start;
    if (value == "center") return Alignment::Center;
    if (value == "end") return Alignment::End;
    if (value == "bottomStart") return Alignment::BottomStart;
    if (value == "bottom") return Alignment::Bottom;
    if (value == "bottomEnd") return Alignment::BottomEnd;
    return Alignment::TopStart;
}

enum class Axis { Vertical, Horizontal };

[[nodiscard]] constexpr Axis parse_axis(std::string_view value) noexcept {
    if (value == "horizontal") return Axis::Horizontal;
    return Axis::Vertical;
}

enum class Shadow { None, Small, Medium, Large };

[[nodiscard]] constexpr Shadow parse_shadow(std::string_view value) noexcept {
    if (value == "small") return Shadow::Small;
    if (value == "medium") return Shadow::Medium;
    if (value == "large") return Shadow::Large;
    return Shadow::None;
}

enum class Weight { Regular, Light, Medium, Semibold, Bold };

[[nodiscard]] constexpr Weight parse_weight(std::string_view value) noexcept {
    if (value == "light") return Weight::Light;
    if (value == "medium") return Weight::Medium;
    if (value == "semibold") return Weight::Semibold;
    if (value == "bold") return Weight::Bold;
    return Weight::Regular;
}

enum class TextAlign { Start, Center, End };

[[nodiscard]] constexpr TextAlign parse_text_align(std::string_view value) noexcept {
    if (value == "center") return TextAlign::Center;
    if (value == "end") return TextAlign::End;
    return TextAlign::Start;
}

enum class Decoration { None, Underline, LineThrough };

[[nodiscard]] constexpr Decoration parse_decoration(std::string_view value) noexcept {
    if (value == "underline") return Decoration::Underline;
    if (value == "lineThrough") return Decoration::LineThrough;
    return Decoration::None;
}

} // namespace weft
