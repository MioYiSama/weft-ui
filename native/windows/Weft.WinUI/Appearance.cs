using System;
using System.Numerics;
using Microsoft.UI;
using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Hosting;
using Microsoft.UI.Xaml.Media;
using Windows.Foundation;

namespace Weft.WinUI;

/// <summary>A Weft border box with a separately styled background, border, and content.</summary>
public abstract class WeftDecoratedElement : WeftElement
{
    private protected WeftBox Box { get; } = new();
    protected sealed override FrameworkElement Control => Box;
    private protected override LayoutInsets ContentInsets => Box.ContentInsets;

    private WeftSession? session;
    private WeftBrush background, borderColor;

    private protected WeftTheme? Theme => session?.Theme;
    private protected ElementTheme Appearance => session?.ActualTheme ?? Box.ActualTheme;

    internal void AttachTheme(WeftSession owner) => session = owner;
    internal virtual void ReapplyColors()
    {
        background.Reapply(Theme, Appearance);
        borderColor.Reapply(Theme, Appearance);
    }

    public WeftColor Background
    {
        set => Box.Paint.Background = background.Set(value, Theme, Appearance);
    }

    public double Radius { set => Box.Radius = Math.Max(0, value); }

    public double Border
    {
        set
        {
            var width = Math.Max(0, value);
            if (Box.Border.BorderThickness.Left == width) return;
            Box.SetBorderWidth(width);
            Resize();
        }
    }

    public WeftColor BorderColor { set => Box.PaintBorderBrush = borderColor.Set(value, Theme, Appearance); }
    public WeftShadow Shadow { set => Box.ShadowPreset = value; }
    public bool Clip { set => Box.ClipContent = value; }
}

// Each property retains its own token and reuses its brush across updates.
internal struct WeftBrush
{
    private WeftColor color;
    private SolidColorBrush? brush;

    internal SolidColorBrush Set(WeftColor value, WeftTheme? theme, ElementTheme appearance)
    {
        color = value;
        var resolved = value.Resolve(theme, appearance);
        if (brush is null) brush = new SolidColorBrush(resolved);
        else brush.Color = resolved;
        return brush;
    }

    internal readonly void Reapply(WeftTheme? theme, ElementTheme appearance)
    {
        if (color.IsToken) brush!.Color = color.Resolve(theme, appearance);
    }
}

// Keep the layout Border distinct from a shadow behind it. When clipping is off,
// a paint-only Border rounds the chrome without implicitly clipping its sibling content.
internal sealed class WeftBox : Grid
{
    internal Border Border { get; } = new() { BackgroundSizing = BackgroundSizing.OuterBorderEdge };
    internal Border Paint => useChrome ? chrome! : Border;
    private Border? chrome;
    private bool useChrome;
    private bool clip;
    private double radius;
    private RectangleGeometry? contentClip;
    private SolidColorBrush? defaultBorder;
    private bool customBorder;
    private WeftBoxShadow? shadow;

    internal WeftBox() => Children.Add(Border);

    internal LayoutInsets ContentInsets => Insets(Border.Padding, Border.BorderThickness.Left);

    internal static LayoutInsets Insets(Thickness padding, double border) =>
        new(padding.Top + border, padding.Right + border, padding.Bottom + border, padding.Left + border);

    internal double Radius
    {
        set
        {
            if (radius == value) return;
            radius = value;
            UpdateChrome();
            if (shadow is not null) shadow.Radius = radius;
        }
    }

    internal bool ClipContent
    {
        set
        {
            if (clip == value) return;
            clip = value;
            UpdateChrome();
            var content = (FrameworkElement)Border.Child;
            if (clip)
            {
                contentClip ??= new RectangleGeometry();
                content.Clip = contentClip;
                Border.SizeChanged += ClipSizeChanged;
                UpdateClip();
            }
            else
            {
                Border.SizeChanged -= ClipSizeChanged;
                content.Clip = null;
            }
        }
    }

    internal SolidColorBrush PaintBorderBrush
    {
        set
        {
            customBorder = true;
            if (defaultBorder is not null) ActualThemeChanged -= UpdateDefaultBorder;
            defaultBorder = null;
            Paint.BorderBrush = value;
        }
    }

    internal void SetBorderWidth(double width)
    {
        var thickness = new Thickness(width);
        Border.BorderThickness = thickness;
        if (useChrome) chrome!.BorderThickness = thickness;
        if (width > 0 && !customBorder && defaultBorder is null)
        {
            defaultBorder = new SolidColorBrush();
            UpdateDefaultBorder(this, null);
            Paint.BorderBrush = defaultBorder;
            ActualThemeChanged += UpdateDefaultBorder;
        }
        UpdateClip();
    }

    internal static WeftColor DefaultBorderColor(ElementTheme appearance) =>
        WeftColor.Rgba(appearance == ElementTheme.Dark ? 0xffffff26 : 0x0000001f);

    private void UpdateDefaultBorder(FrameworkElement sender, object? args) =>
        defaultBorder!.Color = DefaultBorderColor(ActualTheme).Resolve(null, ActualTheme);

    private void UpdateChrome()
    {
        var needsChrome = radius > 0 && !clip;
        if (needsChrome != useChrome)
        {
            var previous = Paint;
            if (needsChrome)
            {
                chrome ??= new Border { BackgroundSizing = BackgroundSizing.OuterBorderEdge, IsHitTestVisible = false };
                chrome.BorderThickness = Border.BorderThickness;
                Children.Insert(Children.Count - 1, chrome);
            }
            else
            {
                Children.Remove(chrome!);
            }
            useChrome = needsChrome;
            Paint.Background = previous.Background;
            Paint.BorderBrush = previous.BorderBrush;
            previous.Background = null;
            previous.BorderBrush = null;
            previous.CornerRadius = default;
        }
        Paint.CornerRadius = new CornerRadius(radius);
    }

    private void ClipSizeChanged(object sender, SizeChangedEventArgs args) => UpdateClip();

    internal void UpdateClip()
    {
        if (!clip) return;
        // The child begins after the padding; the clip includes that padding but not the border.
        var padding = Border.Padding;
        var border = Border.BorderThickness.Left;
        contentClip!.Rect = new Rect(-padding.Left, -padding.Top,
            Math.Max(0, Border.ActualWidth - 2 * border), Math.Max(0, Border.ActualHeight - 2 * border));
    }

    internal WeftShadow ShadowPreset
    {
        set
        {
            if (shadow is null && value == WeftShadow.None) return;
            if (shadow is null)
            {
                shadow = new WeftBoxShadow(this, radius);
                Children.Insert(0, shadow.Host);
            }
            shadow.Preset = value;
        }
    }
}

internal readonly record struct ShadowStyle(float Y, float Blur, byte Alpha)
{
    internal static ShadowStyle For(WeftShadow shadow) => shadow switch
    {
        WeftShadow.Small => new(1, 3, 0x1f),
        WeftShadow.Medium => new(4, 12, 0x26),
        WeftShadow.Large => new(12, 32, 0x2e),
        _ => default,
    };
}

// The Canvas hosts the composition output without affecting layout or hit testing.
internal sealed class WeftBoxShadow
{
    // Leave space for the largest blur and offset on each side of the border box.
    private const float Bleed = 64;

    internal Canvas Host { get; } = new() { IsHitTestVisible = false };
    private readonly CompositionRoundedRectangleGeometry silhouette, outline;
    private readonly CompositionVisualSurface maskSurface, shadowSurface, outlineSurface;
    private readonly ShapeVisual maskShape, outlineShape;
    private readonly SpriteVisual caster, output;
    private readonly DropShadow cast;
    private float radius;

    internal WeftBoxShadow(WeftBox box, double radius)
    {
        var compositor = ElementCompositionPreview.GetElementVisual(Host).Compositor;
        var solid = compositor.CreateColorBrush(Colors.Black);

        // An invisible XAML Rectangle's GetAlphaMask rendered a square shadow on WinUI.
        // Rasterize the rounded silhouette directly instead.
        silhouette = compositor.CreateRoundedRectangleGeometry();
        var fill = compositor.CreateSpriteShape(silhouette);
        fill.FillBrush = solid;
        maskShape = compositor.CreateShapeVisual();
        maskShape.Shapes.Add(fill);
        maskSurface = compositor.CreateVisualSurface();
        maskSurface.SourceVisual = maskShape;
        cast = compositor.CreateDropShadow();
        cast.Color = Colors.Black;
        cast.Mask = compositor.CreateSurfaceBrush(maskSurface);
        caster = compositor.CreateSpriteVisual();
        caster.Shadow = cast;

        // Knock out the shadow inside the border box, even if it has no background.
        // The inner edge of a thick outline forms the rounded opening in the mask.
        outline = compositor.CreateRoundedRectangleGeometry();
        outline.Offset = new Vector2(Bleed / 2);
        var stroke = compositor.CreateSpriteShape(outline);
        stroke.StrokeBrush = solid;
        stroke.StrokeThickness = Bleed;
        outlineShape = compositor.CreateShapeVisual();
        outlineShape.Shapes.Add(stroke);
        outlineSurface = compositor.CreateVisualSurface();
        outlineSurface.SourceVisual = outlineShape;

        shadowSurface = compositor.CreateVisualSurface();
        shadowSurface.SourceVisual = caster;
        shadowSurface.SourceOffset = new Vector2(-Bleed);
        var mask = compositor.CreateMaskBrush();
        mask.Source = compositor.CreateSurfaceBrush(shadowSurface);
        mask.Mask = compositor.CreateSurfaceBrush(outlineSurface);
        output = compositor.CreateSpriteVisual();
        output.Offset = new Vector3(-Bleed, -Bleed, 0);
        output.Brush = mask;
        ElementCompositionPreview.SetElementChildVisual(Host, output);

        this.radius = (float)radius;
        box.SizeChanged += (_, args) => Resize(args.NewSize);
        Resize(box.RenderSize);
    }

    internal double Radius
    {
        set
        {
            radius = (float)value;
            UpdateCorners(caster.Size);
        }
    }

    internal WeftShadow Preset
    {
        set
        {
            output.IsVisible = value != WeftShadow.None;
            if (value == WeftShadow.None) return;
            var preset = ShadowStyle.For(value);
            cast.BlurRadius = preset.Blur;
            cast.Offset = new Vector3(0, preset.Y, 0);
            cast.Opacity = preset.Alpha / 255f;
        }
    }

    private void Resize(Size size)
    {
        var box = new Vector2((float)size.Width, (float)size.Height);
        var expanded = box + new Vector2(2 * Bleed);
        maskShape.Size = maskSurface.SourceSize = caster.Size = silhouette.Size = box;
        outline.Size = box + new Vector2(Bleed);
        outlineShape.Size = outlineSurface.SourceSize = shadowSurface.SourceSize = output.Size = expanded;
        UpdateCorners(box);
    }

    private void UpdateCorners(Vector2 box)
    {
        var clamped = new Vector2(Math.Min(radius, Math.Min(box.X, box.Y) / 2));
        silhouette.CornerRadius = clamped;
        outline.CornerRadius = clamped + new Vector2(Bleed / 2);
    }
}
