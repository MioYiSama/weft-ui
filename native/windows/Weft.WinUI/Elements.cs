using System;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.UI.Text;

namespace Weft.WinUI;

// Built-in elements. Each wraps its control as `Element` and exposes the
// element's properties under their PascalCase names; properties never set
// keep the platform's defaults. Lengths are DIPs.

/// <summary><c>Column</c> and <c>Row</c>: a linear panel with one main axis.</summary>
public sealed class WeftStack : WeftPaddedElement
{
    private readonly WeftLinearPanel panel;

    public WeftStack(bool vertical)
    {
        panel = new WeftLinearPanel(vertical ? WeftAxis.Vertical : WeftAxis.Horizontal);
        Box.Border.Child = panel;
        Children = new WeftRegion(panel);
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

    public WeftOverlay()
    {
        Box.Border.Child = panel;
        Children = new WeftRegion(panel);
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

    private WeftBrush foreground;

    public WeftText() => Box.Border.Child = block;

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

    public bool Bold { set => Weight = value ? WeftWeight.Bold : WeftWeight.Regular; }
    public WeftWeight Weight { set => block.FontWeight = WeftTextStyle.Weight(value); }
    public bool Italic { set => block.FontStyle = WeftTextStyle.Style(value); }
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
    private WeftSession? session;
    private WeftBrush foreground;

    public WeftSpan(WeftText parent) => parent.AppendSpan(run);

    internal void AttachTheme(WeftSession owner) => session = owner;
    internal void ReapplyColors() => foreground.Reapply(session?.Theme, session?.ActualTheme ?? ElementTheme.Default);

    public string Value { set => run.Text = value; }
    public WeftColor Color
    {
        set => run.Foreground = foreground.Set(value, session?.Theme, session?.ActualTheme ?? ElementTheme.Default);
    }
    public double Size { set => run.FontSize = value; }
    public WeftWeight Weight { set => run.FontWeight = WeftTextStyle.Weight(value); }
    public bool Italic { set => run.FontStyle = WeftTextStyle.Style(value); }
    public WeftDecoration Decoration { set => run.TextDecorations = WeftTextStyle.Decorations(value); }
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

/// <summary>
/// <c>Image</c>: a remote image. With one fixed side the other follows the
/// aspect ratio, with both the image fills them, and with neither it keeps
/// its own size.
/// </summary>
public sealed class WeftImage : WeftDecoratedElement
{
    private readonly Image image = new() { Stretch = Stretch.None };

    public WeftImage() => Box.Border.Child = image;

    public string Src
    {
        set => image.Source = Uri.TryCreate(value, UriKind.Absolute, out var uri) ? new BitmapImage(uri) : null;
    }

    protected override void Resize()
    {
        base.Resize();
        image.Stretch = (double.IsNaN(Box.Width), double.IsNaN(Box.Height)) switch
        {
            (true, true) => Stretch.None,
            (false, false) => Stretch.Fill,
            _ => Stretch.Uniform,
        };
    }
}
