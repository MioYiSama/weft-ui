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

public enum WeftShadow
{
    None,
    Small,
    Medium,
    Large,
}

public enum WeftWeight
{
    Regular,
    Light,
    Medium,
    Semibold,
    Bold,
}

public enum WeftTextAlign
{
    Start,
    Center,
    End,
}

public enum WeftDecoration
{
    None,
    Underline,
    LineThrough,
}

public enum WeftInputType
{
    Text,
    Password,
    Number,
    Email,
    Phone,
}

public enum WeftReturnKey
{
    Done,
    Go,
    Next,
    Search,
    Send,
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

    public static WeftShadow ParseShadow(string value) => value switch
    {
        "small" => WeftShadow.Small,
        "medium" => WeftShadow.Medium,
        "large" => WeftShadow.Large,
        _ => WeftShadow.None,
    };

    public static WeftWeight ParseWeight(string value) => value switch
    {
        "light" => WeftWeight.Light,
        "medium" => WeftWeight.Medium,
        "semibold" => WeftWeight.Semibold,
        "bold" => WeftWeight.Bold,
        _ => WeftWeight.Regular,
    };

    public static WeftTextAlign ParseTextAlign(string value) => value switch
    {
        "center" => WeftTextAlign.Center,
        "end" => WeftTextAlign.End,
        _ => WeftTextAlign.Start,
    };

    public static WeftDecoration ParseDecoration(string value) => value switch
    {
        "underline" => WeftDecoration.Underline,
        "lineThrough" => WeftDecoration.LineThrough,
        _ => WeftDecoration.None,
    };

    public static WeftInputType ParseInputType(string value) => value switch
    {
        "password" => WeftInputType.Password,
        "number" => WeftInputType.Number,
        "email" => WeftInputType.Email,
        "phone" => WeftInputType.Phone,
        _ => WeftInputType.Text,
    };

    public static WeftReturnKey ParseReturnKey(string value) => value switch
    {
        "go" => WeftReturnKey.Go,
        "next" => WeftReturnKey.Next,
        "search" => WeftReturnKey.Search,
        "send" => WeftReturnKey.Send,
        _ => WeftReturnKey.Done,
    };
}
