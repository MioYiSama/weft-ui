using System;
using Microsoft.UI.Xaml;

namespace Weft.WinUI;

internal struct EdgeValues
{
    internal double All;
    internal double? X, Y, Top, Bottom, Start, End;

    internal readonly Thickness Resolve() => new(Start ?? X ?? All, Top ?? Y ?? All, End ?? X ?? All, Bottom ?? Y ?? All);
}

/// <summary>A control with Weft content-box constraints and parent layout properties.</summary>
public abstract class WeftElement
{
    protected abstract FrameworkElement Control { get; }
    public UIElement Element => Control;

    private EdgeValues margin;
    private double? width, height, minWidth, maxWidth, minHeight, maxHeight;

    public double Width { set { width = Nonnegative(value); Resize(); } }
    public double Height { set { height = Nonnegative(value); Resize(); } }
    public double MinWidth { set { minWidth = Nonnegative(value); Resize(); } }
    public double MaxWidth { set { maxWidth = Nonnegative(value); Resize(); } }
    public double MinHeight { set { minHeight = Nonnegative(value); Resize(); } }
    public double MaxHeight { set { maxHeight = Nonnegative(value); Resize(); } }

    public double Grow { set => Control.SetValue(WeftLayout.GrowProperty, value); }
    public bool Fill { set => Control.SetValue(WeftLayout.FillProperty, value); }
    public double AspectRatio { set => Control.SetValue(WeftLayout.AspectRatioProperty, value); }

    public double Margin { set { margin.All = value; UpdateMargin(); } }
    public double MarginX { set { margin.X = value; UpdateMargin(); } }
    public double MarginY { set { margin.Y = value; UpdateMargin(); } }
    public double MarginTop { set { margin.Top = value; UpdateMargin(); } }
    public double MarginBottom { set { margin.Bottom = value; UpdateMargin(); } }
    public double MarginStart { set { margin.Start = value; UpdateMargin(); } }
    public double MarginEnd { set { margin.End = value; UpdateMargin(); } }

    public double Opacity { set => Control.Opacity = Math.Clamp(value, 0, 1); }
    public bool Hidden { set => Control.Visibility = value ? Visibility.Collapsed : Visibility.Visible; }

    private protected virtual LayoutInsets ContentInsets => default;
    private static double Nonnegative(double value) => Math.Max(0, value);
    private void UpdateMargin() => Control.Margin = margin.Resolve();

    protected virtual void Resize()
    {
        var control = Control;
        var insets = ContentInsets;
        if (minWidth is double minW) control.MinWidth = minW + insets.Width;
        if (maxWidth is double maxW) control.MaxWidth = maxW + insets.Width;
        if (minHeight is double minH) control.MinHeight = minH + insets.Height;
        if (maxHeight is double maxH) control.MaxHeight = maxH + insets.Height;
        if (width is double w) control.Width = w + insets.Width;
        if (height is double h) control.Height = h + insets.Height;
    }
}

/// <summary>A painted element with separately configurable padding on all four sides.</summary>
public abstract class WeftPaddedElement : WeftDecoratedElement
{
    private EdgeValues padding;

    public double Padding { set { padding.All = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingX { set { padding.X = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingY { set { padding.Y = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingTop { set { padding.Top = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingBottom { set { padding.Bottom = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingStart { set { padding.Start = Math.Max(0, value); UpdatePadding(); } }
    public double PaddingEnd { set { padding.End = Math.Max(0, value); UpdatePadding(); } }

    private void UpdatePadding()
    {
        var value = padding.Resolve();
        if (value.Equals(Box.Border.Padding)) return;
        Box.Border.Padding = value;
        Box.UpdateClip();
        Resize();
    }
}
