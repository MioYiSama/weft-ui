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
public abstract class WeftPaintedElement
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

/// <summary>
/// <c>Column</c> / <c>Row</c>: a <see cref="StackPanel"/>. Children start at
/// the leading edge of the cross axis without stretching; a fixed width or
/// height sizes the content box, with the padding around it, as CSS boxes do.
/// </summary>
public sealed class WeftStack : WeftPaintedElement
{
    private readonly StackPanel panel;
    private double padding;
    private double width = double.NaN;
    private double height = double.NaN;

    public WeftStack(bool vertical)
    {
        panel = new StackPanel { Orientation = vertical ? Orientation.Vertical : Orientation.Horizontal };
        Children = new WeftRegion(panel, vertical ? StackAlignment.Leading : StackAlignment.Top);
    }

    public UIElement Element => panel;

    public WeftRegion Children { get; }

    public double Gap
    {
        set => panel.Spacing = value;
    }

    public double Padding
    {
        set
        {
            padding = value;
            panel.Padding = new Thickness(value);
            Resize();
        }
    }

    public double Width
    {
        set
        {
            width = value;
            Resize();
        }
    }

    public double Height
    {
        set
        {
            height = value;
            Resize();
        }
    }

    public WeftColor Background
    {
        set => Paint(value);
    }

    protected override void SetColorBrush(SolidColorBrush brush) => panel.Background = brush;

    private void Resize()
    {
        panel.Width = width + 2 * padding;
        panel.Height = height + 2 * padding;
    }
}

/// <summary><c>Text</c>: a <see cref="TextBlock"/>.</summary>
public sealed class WeftText : WeftPaintedElement
{
    private readonly TextBlock block = new();

    public UIElement Element => block;

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
public sealed class WeftButton
{
    private readonly Button button = new();

    public WeftButton(Action? click)
    {
        var content = new StackPanel { Orientation = Orientation.Horizontal };
        button.Content = content;
        Children = new WeftRegion(content, StackAlignment.Center);
        if (click is not null) button.Click += (_, _) => click();
    }

    public UIElement Element => button;

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
public sealed class WeftImage
{
    private readonly Image image = new() { Stretch = Stretch.None };
    private double width = double.NaN;
    private double height = double.NaN;

    public UIElement Element => image;

    public string Src
    {
        set => image.Source = Uri.TryCreate(value, UriKind.Absolute, out var uri) ? new BitmapImage(uri) : null;
    }

    public double Width
    {
        set
        {
            width = value;
            image.Width = value;
            Restretch();
        }
    }

    public double Height
    {
        set
        {
            height = value;
            image.Height = value;
            Restretch();
        }
    }

    private void Restretch()
    {
        image.Stretch = (double.IsNaN(width), double.IsNaN(height)) switch
        {
            (true, true) => Stretch.None,
            (false, false) => Stretch.Fill,
            _ => Stretch.Uniform,
        };
    }
}
