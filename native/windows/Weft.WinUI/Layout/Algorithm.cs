using System;

namespace Weft.WinUI;

internal readonly record struct LayoutSize(double Width, double Height);
internal readonly record struct LayoutFrame(double X, double Y, double Width, double Height);
internal readonly record struct LayoutInsets(double Top, double Right, double Bottom, double Left)
{
    public double Width => Left + Right;
    public double Height => Top + Bottom;
}

// AtMost(+∞) is unbounded. Exact sizes are always finite.
internal readonly record struct LayoutLimit(double Value, bool IsExact = false)
{
    public static LayoutLimit Unbounded => new(double.PositiveInfinity);
    public bool IsBounded => double.IsFinite(Value);
    public LayoutLimit Inset(double amount) => new(Math.Max(0, Value - amount), IsExact);
    public double Resolve(double natural) => IsExact ? Value : Math.Min(natural, Value);
}

internal readonly record struct LayoutProposal(LayoutLimit Width, LayoutLimit Height)
{
    public static LayoutProposal Exact(LayoutSize size) => new(new(size.Width, true), new(size.Height, true));
    public LayoutProposal Inset(LayoutInsets padding) => new(Width.Inset(padding.Width), Height.Inset(padding.Height));
}

internal readonly record struct LayoutBounds(
    double Width, double Height, double MinWidth, double MaxWidth, double MinHeight, double MaxHeight,
    double AspectRatio, LayoutInsets Padding)
{
    public bool HasWidth => !double.IsNaN(Width);
    public bool HasHeight => !double.IsNaN(Height);

    public LayoutProposal Constrain(LayoutProposal proposal)
    {
        var width = HasWidth ? new(Width, true) : proposal.Width;
        var height = HasHeight ? new(Height, true) : proposal.Height;
        if (double.IsFinite(AspectRatio) && AspectRatio > 0)
        {
            if (width.IsExact && !HasHeight)
                height = new(Math.Max(0, width.Value - Padding.Width) / AspectRatio + Padding.Height, true);
            else if (height.IsExact && !width.IsExact)
                width = new(Math.Max(0, height.Value - Padding.Height) * AspectRatio + Padding.Width, true);
        }
        return new(Constrain(width, MinWidth, MaxWidth), Constrain(height, MinHeight, MaxHeight));
    }

    public LayoutSize Resolve(LayoutProposal constrained, LayoutSize natural) => new(
        Math.Max(MinWidth, constrained.Width.Resolve(natural.Width)),
        Math.Max(MinHeight, constrained.Height.Resolve(natural.Height)));

    private static LayoutLimit Constrain(LayoutLimit limit, double min, double max) =>
        new(Math.Max(min, Math.Min(limit.Value, max)), limit.IsExact);
}

internal readonly record struct LayoutChild(
    LayoutBounds Bounds, LayoutInsets Margin, double Grow, bool Fill, bool IsSpacer, double? SpacerSize)
{
    public double Weight => IsSpacer ? (SpacerSize.HasValue ? 0 : 1) : Math.Max(0, Grow);
}

// The caller supplies measurements and storage for the result; neither algorithm depends on WinUI controls.
internal static class WeftLayoutAlgorithm
{
    public static LayoutSize Linear(
        WeftAxis axis, double gap, WeftCrossAlign align, WeftJustify justify,
        LayoutInsets padding, LayoutProposal proposal, ReadOnlySpan<LayoutChild> children,
        Func<int, LayoutProposal, LayoutSize> measure, Span<LayoutFrame> frames)
    {
        var vertical = axis == WeftAxis.Vertical;
        var available = proposal.Inset(padding);
        var main = vertical ? available.Height : available.Width;
        var cross = vertical ? available.Width : available.Height;
        var used = gap * Math.Max(0, children.Length - 1);
        var naturalCross = 0.0;
        var weight = 0.0;
        var lastGrow = -1;
        for (var i = 0; i < children.Length; i++)
        {
            ref readonly var child = ref children[i];
            used += vertical ? child.Margin.Height : child.Margin.Width;
            if (main.IsBounded && child.Weight > 0)
            {
                weight += child.Weight;
                lastGrow = i;
                continue;
            }
            var size = MeasureLinear(i, child, LayoutLimit.Unbounded, cross, vertical, align, measure);
            frames[i] = new(0, 0, size.Width, size.Height);
            used += vertical ? size.Height : size.Width;
            naturalCross = Math.Max(naturalCross, vertical ? size.Width + child.Margin.Width : size.Height + child.Margin.Height);
        }

        if (lastGrow >= 0)
        {
            var free = Math.Max(0, main.Value - used);
            var assigned = 0.0;
            for (var i = 0; i <= lastGrow; i++)
            {
                ref readonly var child = ref children[i];
                if (child.Weight <= 0) continue;
                var share = i == lastGrow ? free - assigned : free * (child.Weight / weight);
                assigned += share;
                var size = MeasureLinear(i, child, new(Math.Max(0, share), true), cross, vertical, align, measure);
                frames[i] = new(0, 0, size.Width, size.Height);
                used += vertical ? size.Height : size.Width;
                naturalCross = Math.Max(naturalCross, vertical ? size.Width + child.Margin.Width : size.Height + child.Margin.Height);
            }
        }

        var mainSize = main.Resolve(used);
        var crossSize = cross.Resolve(naturalCross);
        var remaining = mainSize - used;
        var count = children.Length;
        var (cursor, spacing) = justify switch
        {
            WeftJustify.Center => (remaining / 2, 0.0),
            WeftJustify.End => (remaining, 0.0),
            WeftJustify.SpaceBetween when count > 1 => (0.0, remaining / (count - 1)),
            WeftJustify.SpaceAround when count > 0 => (remaining / (2 * count), remaining / count),
            WeftJustify.SpaceEvenly => (remaining / (count + 1), remaining / (count + 1)),
            _ => (0.0, 0.0),
        };
        for (var i = 0; i < count; i++)
        {
            ref readonly var child = ref children[i];
            var margin = child.Margin;
            var frame = frames[i];
            var extra = vertical ? crossSize - frame.Width - margin.Width : crossSize - frame.Height - margin.Height;
            var offset = CrossExact(child, cross, vertical, align) ? 0 : align switch
            {
                WeftCrossAlign.Center => extra / 2,
                WeftCrossAlign.End => extra,
                _ => 0,
            };
            frames[i] = vertical
                ? frame with { X = padding.Left + margin.Left + offset, Y = padding.Top + cursor + margin.Top }
                : frame with { X = padding.Left + cursor + margin.Left, Y = padding.Top + margin.Top + offset };
            cursor += (vertical ? frame.Height + margin.Height : frame.Width + margin.Width) + gap + spacing;
        }

        return vertical ? new(crossSize + padding.Width, mainSize + padding.Height)
                        : new(mainSize + padding.Width, crossSize + padding.Height);
    }

    private static bool CrossExact(LayoutChild child, LayoutLimit cross, bool vertical, WeftCrossAlign align) =>
        !child.IsSpacer && cross.IsBounded && (child.Fill || align == WeftCrossAlign.Stretch)
            && !(vertical ? child.Bounds.HasWidth : child.Bounds.HasHeight);

    private static LayoutSize MeasureLinear(
        int index, LayoutChild child, LayoutLimit main, LayoutLimit cross, bool vertical, WeftCrossAlign align,
        Func<int, LayoutProposal, LayoutSize> measure)
    {
        if (child.IsSpacer)
        {
            var length = child.SpacerSize ?? main.Resolve(0);
            return vertical ? new(0, length) : new(length, 0);
        }
        var limit = new LayoutLimit(cross.Inset(vertical ? child.Margin.Width : child.Margin.Height).Value,
            CrossExact(child, cross, vertical, align));
        return measure(index, vertical ? new(limit, main) : new(main, limit));
    }

    public static LayoutSize Stack(
        WeftAlignment alignment, LayoutInsets padding, LayoutProposal proposal, ReadOnlySpan<LayoutChild> children,
        Func<int, LayoutProposal, LayoutSize> measure, Span<LayoutFrame> frames)
    {
        var available = proposal.Inset(padding);
        var natural = new LayoutSize();
        for (var i = 0; i < children.Length; i++)
        {
            ref readonly var child = ref children[i];
            if (child.Fill) continue;
            var bounds = available.Inset(child.Margin);
            var size = measure(i, new(new(bounds.Width.Value), new(bounds.Height.Value)));
            frames[i] = new(0, 0, size.Width, size.Height);
            natural = new(Math.Max(natural.Width, size.Width + child.Margin.Width),
                          Math.Max(natural.Height, size.Height + child.Margin.Height));
        }
        var content = new LayoutSize(available.Width.Resolve(natural.Width), available.Height.Resolve(natural.Height));
        var exact = LayoutProposal.Exact(content);
        var (x, y) = alignment switch
        {
            WeftAlignment.Top => (0.5, 0.0),
            WeftAlignment.TopEnd => (1.0, 0.0),
            WeftAlignment.Start => (0.0, 0.5),
            WeftAlignment.Center => (0.5, 0.5),
            WeftAlignment.End => (1.0, 0.5),
            WeftAlignment.BottomStart => (0.0, 1.0),
            WeftAlignment.Bottom => (0.5, 1.0),
            WeftAlignment.BottomEnd => (1.0, 1.0),
            _ => (0.0, 0.0),
        };
        for (var i = 0; i < children.Length; i++)
        {
            ref readonly var child = ref children[i];
            var frame = frames[i];
            if (child.Fill)
            {
                var size = measure(i, exact.Inset(child.Margin));
                frame = new(0, 0, size.Width, size.Height);
            }
            frames[i] = frame with {
                X = padding.Left + child.Margin.Left + x * (content.Width - frame.Width - child.Margin.Width),
                Y = padding.Top + child.Margin.Top + y * (content.Height - frame.Height - child.Margin.Height),
            };
        }
        return new(content.Width + padding.Width, content.Height + padding.Height);
    }
}
