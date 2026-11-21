using System;
using System.Collections.Generic;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Media;
using Windows.UI.Text;

namespace Weft.WinUI;

// Built-in elements. Each wraps its control as `Element` and exposes the
// element's properties under their PascalCase names; properties never set
// keep the platform's defaults. Lengths are DIPs.

/// <summary><c>Column</c> and <c>Row</c>: a linear panel with one main axis.</summary>
public sealed class WeftStack : WeftPaddedElement
{
    private readonly WeftLinearPanel panel;

    public WeftStack(bool vertical, Action? tap = null)
    {
        panel = new WeftLinearPanel(vertical ? WeftAxis.Vertical : WeftAxis.Horizontal);
        Box.Border.Child = panel;
        Children = new WeftRegion(panel);
        if (tap is not null) Box.EnableTap(tap);
    }

    public WeftRegion Children { get; }

    public double Gap { set => panel.Gap = value; }
    public WeftCrossAlign Align { set => panel.Align = value; }
    public WeftJustify Justify { set => panel.Justify = value; }
}

/// <summary><c>Stack</c>: children occupy the same content box, ordered back to front.</summary>
public sealed class WeftOverlay : WeftPaddedElement
{
    private readonly WeftStackPanel panel = new();

    public WeftOverlay(Action? tap = null)
    {
        Box.Border.Child = panel;
        Children = new WeftRegion(panel);
        if (tap is not null) Box.EnableTap(tap);
    }

    public WeftRegion Children { get; }
    public WeftAlignment Alignment { set => panel.Alignment = value; }
}

/// <summary><c>Scroll</c>: a one-axis viewport around a linear panel.</summary>
public sealed class WeftScroll : WeftPaddedElement
{
    private readonly WeftAxis direction;
    private readonly ScrollViewer scroll;
    private readonly WeftLinearPanel panel;

    public WeftScroll(WeftAxis direction = WeftAxis.Vertical)
    {
        this.direction = direction;
        var vertical = direction == WeftAxis.Vertical;
        panel = new WeftLinearPanel(direction, scrolling: true);
        scroll = new ScrollViewer
        {
            Content = panel,
            HorizontalScrollMode = vertical ? ScrollMode.Disabled : ScrollMode.Enabled,
            VerticalScrollMode = vertical ? ScrollMode.Enabled : ScrollMode.Disabled,
            HorizontalContentAlignment = vertical ? HorizontalAlignment.Stretch : HorizontalAlignment.Left,
            VerticalContentAlignment = vertical ? VerticalAlignment.Top : VerticalAlignment.Stretch,
            ZoomMode = ZoomMode.Disabled,
        };
        Box.Border.Child = scroll;
        Children = new WeftRegion(panel);
        Grow = 1;
        Fill = true;
        Indicators = true;
    }

    public WeftRegion Children { get; }
    public double Gap { set => panel.Gap = value; }
    public WeftCrossAlign Align { set => panel.Align = value; }

    public bool Indicators
    {
        set
        {
            var bar = value ? ScrollBarVisibility.Auto : ScrollBarVisibility.Hidden;
            scroll.VerticalScrollBarVisibility = direction == WeftAxis.Vertical ? bar : ScrollBarVisibility.Disabled;
            scroll.HorizontalScrollBarVisibility = direction == WeftAxis.Horizontal ? bar : ScrollBarVisibility.Disabled;
        }
    }
}

/// <summary><c>Spacer</c>: an empty element, fixed or flexible along its parent's main axis.</summary>
public sealed class WeftSpacer
{
    private sealed class SpacerElement : FrameworkElement { }

    public WeftSpacer() => Element.SetValue(WeftLayout.IsSpacerProperty, true);

    public UIElement Element { get; } = new SpacerElement();
    public double Size { set => Element.SetValue(WeftLayout.SpacerSizeProperty, Math.Max(0, value)); }
}

/// <summary><c>Text</c>: a <see cref="TextBlock"/>.</summary>
public sealed class WeftText : WeftPaddedElement
{
    private readonly TextBlock block = new() { TextWrapping = TextWrapping.Wrap };
    private List<WeftSpan>? fontSpans;
    private string? font;
    private WeftAssets.FontVariant? face;
    private WeftBrush foreground;

    public WeftText(Action? tap = null)
    {
        Box.Border.Child = block;
        if (tap is not null) Box.EnableTap(tap);
    }

    public string Value
    {
        set => block.Text = value;
    }

    public WeftColor Color
    {
        set => block.Foreground = foreground.Set(value, Theme, Appearance);
    }

    internal override void ReapplyColors()
    {
        base.ReapplyColors();
        foreground.Reapply(Theme, Appearance);
    }

    public double Size
    {
        set => block.FontSize = value;
    }

    public WeftWeight Weight { set { block.FontWeight = WeftTextStyle.Weight(value); UpdateFont(); } }
    public bool Italic { set { block.FontStyle = WeftTextStyle.Style(value); UpdateFont(); } }
    public string Font { set { font = value; UpdateFont(); } }

    internal WeftAssets.FontVariant? FindFont(string? family, ushort? weight, FontStyle? style) =>
        (family ?? font) is { Length: > 0 } name
            ? WeftAssets.Current?.FindFont(name, weight ?? block.FontWeight.Weight, (style ?? block.FontStyle) == FontStyle.Italic)
            : null;

    internal void TrackFont(WeftSpan span) => (fontSpans ??= []).Add(span);

    private void UpdateFont()
    {
        var next = FindFont(null, null, null);
        if (next != face)
        {
            face = next;
            if (next is null) block.ClearValue(TextBlock.FontFamilyProperty);
            else block.FontFamily = new FontFamily(next.Source);
        }
        if (fontSpans is not null)
            foreach (var span in fontSpans) span.UpdateFont();
    }
    public WeftDecoration Decoration { set => block.TextDecorations = WeftTextStyle.Decorations(value); }

    public WeftTextAlign Align
    {
        set => block.TextAlignment = value switch
        {
            WeftTextAlign.Center => TextAlignment.Center,
            WeftTextAlign.End => TextAlignment.Right,
            _ => TextAlignment.Left,
        };
    }

    public double MaxLines
    {
        set
        {
            var lines = (int)Math.Clamp(value, 0, int.MaxValue);
            block.MaxLines = lines;
            block.TextTrimming = lines > 0 ? TextTrimming.CharacterEllipsis : TextTrimming.None;
        }
    }

    public double LineHeight
    {
        set
        {
            block.LineHeight = Math.Max(0, value);
            block.LineStackingStrategy = LineStackingStrategy.BlockLineHeight;
        }
    }

    internal void AppendSpan(Run run) => block.Inlines.Add(run);
}

/// <summary><c>Span</c>: a styled <see cref="Run"/> in its parent text block.</summary>
public sealed class WeftSpan
{
    private readonly Run run = new();
    private readonly WeftText parent;
    private WeftSession? session;
    private WeftBrush foreground;
    private string? font;
    private ushort? weight;
    private FontStyle? style;
    private WeftAssets.FontVariant? face;
    private bool tracksFont;

    public WeftSpan(WeftText parent)
    {
        this.parent = parent;
        parent.AppendSpan(run);
    }

    internal void AttachTheme(WeftSession owner) => session = owner;
    internal void ReapplyColors() => foreground.Reapply(session?.Theme, session?.ActualTheme ?? ElementTheme.Default);

    public string Value { set => run.Text = value; }
    public WeftColor Color
    {
        set => run.Foreground = foreground.Set(value, session?.Theme, session?.ActualTheme ?? ElementTheme.Default);
    }
    public double Size { set => run.FontSize = value; }
    public WeftWeight Weight
    {
        set
        {
            var next = WeftTextStyle.Weight(value);
            run.FontWeight = next;
            weight = next.Weight;
            TrackFont();
        }
    }
    public bool Italic
    {
        set
        {
            var next = WeftTextStyle.Style(value);
            run.FontStyle = next;
            style = next;
            TrackFont();
        }
    }
    public string Font { set { font = value; TrackFont(); } }
    public WeftDecoration Decoration { set => run.TextDecorations = WeftTextStyle.Decorations(value); }

    private void TrackFont()
    {
        if (!tracksFont)
        {
            tracksFont = true;
            parent.TrackFont(this);
        }
        UpdateFont();
    }

    internal void UpdateFont()
    {
        var next = parent.FindFont(font, weight, style);
        if (next == face) return;
        face = next;
        if (next is null) run.ClearValue(Run.FontFamilyProperty);
        else run.FontFamily = new FontFamily(next.Source);
    }
}

internal static class WeftTextStyle
{
    internal static FontWeight Weight(WeftWeight weight) => weight switch
    {
        WeftWeight.Light => FontWeights.Light,
        WeftWeight.Medium => FontWeights.Medium,
        WeftWeight.Semibold => FontWeights.SemiBold,
        WeftWeight.Bold => FontWeights.Bold,
        _ => FontWeights.Normal,
    };

    internal static FontStyle Style(bool italic) => italic ? FontStyle.Italic : FontStyle.Normal;

    internal static TextDecorations Decorations(WeftDecoration decoration) => decoration switch
    {
        WeftDecoration.Underline => TextDecorations.Underline,
        WeftDecoration.LineThrough => TextDecorations.Strikethrough,
        _ => TextDecorations.None,
    };
}

/// <summary><c>Button</c>: a <see cref="Button"/> whose content is a horizontal stack of its children.</summary>
public sealed class WeftButton : WeftElement
{
    private readonly Button button = new();

    public WeftButton(Action? click)
    {
        var content = new WeftLinearPanel(WeftAxis.Horizontal, flexibleChildren: false) { Align = WeftCrossAlign.Center };
        button.Content = content;
        Children = new WeftRegion(content);
        if (click is not null) button.Click += (_, _) => click();
    }

    protected override FrameworkElement Control => button;

    public WeftRegion Children { get; }

    public bool Disabled
    {
        set => button.IsEnabled = !value;
    }
}


