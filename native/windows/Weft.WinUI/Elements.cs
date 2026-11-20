using System;
using Microsoft.UI.Text;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Media.Imaging;

namespace Weft.WinUI;

// Built-in elements. Each wraps its control as `Element` and exposes the
// element's properties under their PascalCase names; properties never set
// keep the platform's defaults. Lengths are DIPs.

/// <summary>An element that can paint a fixed color or a theme token.</summary>
public abstract class WeftPaintedElement : WeftElement
{
    private WeftColor color;
    private SolidColorBrush? brush;
    private WeftSession? session;

    internal void AttachTheme(WeftSession owner) => session = owner;

    internal void ReapplyColors()
    {
        if (color.IsToken) brush!.Color = Resolve();
    }

    protected void Paint(WeftColor value)
    {
        color = value;
        if (brush is null)
        {
            brush = new SolidColorBrush(Resolve());
            SetColorBrush(brush);
        }
        else
        {
            brush.Color = Resolve();
        }
    }

    private Windows.UI.Color Resolve() =>
        color.Resolve(session?.Theme, session?.ActualTheme ?? ElementTheme.Light);

    protected abstract void SetColorBrush(SolidColorBrush brush);
}

/// <summary><c>Column</c> and <c>Row</c>: a linear panel with one main axis.</summary>
public sealed class WeftStack : WeftPaddedElement
{
    private readonly WeftLinearPanel panel;

    public WeftStack(bool vertical)
    {
        panel = new WeftLinearPanel(vertical ? WeftAxis.Vertical : WeftAxis.Horizontal);
        Children = new WeftRegion(panel);
    }

    protected override FrameworkElement Control => panel;
    public WeftRegion Children { get; }

    public double Gap { set => panel.Gap = value; }
    public WeftCrossAlign Align { set => panel.Align = value; }
    public WeftJustify Justify { set => panel.Justify = value; }
    public WeftColor Background { set => Paint(value); }

    protected override void SetPadding(Thickness value) => panel.Padding = value;
    protected override void SetColorBrush(SolidColorBrush brush) => panel.Background = brush;
}

/// <summary><c>Stack</c>: children occupy the same content box, ordered back to front.</summary>
public sealed class WeftOverlay : WeftPaddedElement
{
    private readonly WeftStackPanel panel = new();

    public WeftOverlay() => Children = new WeftRegion(panel);

    protected override FrameworkElement Control => panel;
    public WeftRegion Children { get; }
    public WeftAlignment Alignment { set => panel.Alignment = value; }

    protected override void SetPadding(Thickness value) => panel.Padding = value;
    protected override void SetColorBrush(SolidColorBrush brush) => panel.Background = brush;
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
        Children = new WeftRegion(panel);
        Grow = 1;
        Fill = true;
        Indicators = true;
    }

    protected override FrameworkElement Control => scroll;
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

    protected override void SetPadding(Thickness value) => scroll.Padding = value;
    protected override void SetColorBrush(SolidColorBrush brush) => scroll.Background = brush;
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

    protected override FrameworkElement Control => block;
    protected override void SetPadding(Thickness value) => block.Padding = value;

    public string Value
    {
        set => block.Text = value;
    }

    public WeftColor Color
    {
        set => Paint(value);
    }

    protected override void SetColorBrush(SolidColorBrush brush) => block.Foreground = brush;

    public double Size
    {
        set => block.FontSize = value;
    }

    public bool Bold
    {
        set => block.FontWeight = value ? FontWeights.Bold : FontWeights.Normal;
    }
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
public sealed class WeftImage : WeftElement
{
    private readonly Image image = new() { Stretch = Stretch.None };

    protected override FrameworkElement Control => image;

    public string Src
    {
        set => image.Source = Uri.TryCreate(value, UriKind.Absolute, out var uri) ? new BitmapImage(uri) : null;
    }

    protected override void Resize()
    {
        base.Resize();
        image.Stretch = (double.IsNaN(image.Width), double.IsNaN(image.Height)) switch
        {
            (true, true) => Stretch.None,
            (false, false) => Stretch.Fill,
            _ => Stretch.Uniform,
        };
    }
}
