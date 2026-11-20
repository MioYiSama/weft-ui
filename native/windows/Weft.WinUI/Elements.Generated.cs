// Generated from protocol/weft.elements.json by protocol/elements.ts; do not edit.

namespace Weft.WinUI;

public enum WeftCrossAlign
{
    Start,
    Center,
    End,
    Stretch,
}

public enum WeftJustify
{
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

public enum WeftAlignment
{
    TopStart,
    Top,
    TopEnd,
    Start,
    Center,
    End,
    BottomStart,
    Bottom,
    BottomEnd,
}

public enum WeftAxis
{
    Vertical,
    Horizontal,
}

public static class WeftUnions
{
    public static WeftCrossAlign ParseCrossAlign(string value) => value switch
    {
        "center" => WeftCrossAlign.Center,
        "end" => WeftCrossAlign.End,
        "stretch" => WeftCrossAlign.Stretch,
        _ => WeftCrossAlign.Start,
    };

    public static WeftJustify ParseJustify(string value) => value switch
    {
        "center" => WeftJustify.Center,
        "end" => WeftJustify.End,
        "spaceBetween" => WeftJustify.SpaceBetween,
        "spaceAround" => WeftJustify.SpaceAround,
        "spaceEvenly" => WeftJustify.SpaceEvenly,
        _ => WeftJustify.Start,
    };

    public static WeftAlignment ParseAlignment(string value) => value switch
    {
        "top" => WeftAlignment.Top,
        "topEnd" => WeftAlignment.TopEnd,
        "start" => WeftAlignment.Start,
        "center" => WeftAlignment.Center,
        "end" => WeftAlignment.End,
        "bottomStart" => WeftAlignment.BottomStart,
        "bottom" => WeftAlignment.Bottom,
        "bottomEnd" => WeftAlignment.BottomEnd,
        _ => WeftAlignment.TopStart,
    };

    public static WeftAxis ParseAxis(string value) => value switch
    {
        "horizontal" => WeftAxis.Horizontal,
        _ => WeftAxis.Vertical,
    };
}
