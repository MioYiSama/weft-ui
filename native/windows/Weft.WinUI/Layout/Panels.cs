using System;
using System.Collections.Generic;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.Foundation;

namespace Weft.WinUI;

internal sealed class WeftLayout : DependencyObject
{
    public static readonly DependencyProperty GrowProperty = Register("Grow", 0.0);
    public static readonly DependencyProperty FillProperty = Register("Fill", false);
    public static readonly DependencyProperty IsSpacerProperty = Register("IsSpacer", false);
    public static readonly DependencyProperty SpacerSizeProperty = Register("SpacerSize", double.NaN);
    public static readonly DependencyProperty AspectRatioProperty = Register("AspectRatio", 0.0);
    public static readonly DependencyProperty PreferredWidthProperty = Register("PreferredWidth", double.NaN);
    public static readonly DependencyProperty PreferredHeightProperty = Register("PreferredHeight", double.NaN);

    private static DependencyProperty Register<T>(string name, T fallback) where T : struct =>
        DependencyProperty.RegisterAttached(name, typeof(T), typeof(WeftLayout), new PropertyMetadata(fallback, Changed));

    private static void Changed(DependencyObject child, DependencyPropertyChangedEventArgs args)
    {
        if (VisualTreeHelper.GetParent(child) is WeftPanel parent) parent.InvalidateMeasure();
    }
}

internal abstract class WeftPanel : Panel
{
    private readonly List<FrameworkElement> visible = [];
    private readonly Func<int, LayoutProposal, LayoutSize> measure;
    private readonly Func<int, LayoutProposal, LayoutSize> reuseMeasurement;
    private LayoutChild[] children = [];
    private LayoutFrame[] frames = [];
    private LayoutSize[] measuredChildren = [];
    private LayoutSize measured;
    private bool exactWidth;
    private bool exactHeight;

    protected WeftPanel()
    {
        measure = MeasureChild;
        reuseMeasurement = ReuseMeasurement;
    }

    protected virtual bool FlexibleChildren => true;

    internal void SetProposal(LayoutProposal proposal)
    {
        if (proposal.Width.IsExact == exactWidth && proposal.Height.IsExact == exactHeight) return;
        (exactWidth, exactHeight) = (proposal.Width.IsExact, proposal.Height.IsExact);
        InvalidateMeasure();
    }

    protected void SetLayout<T>(ref T field, T value) where T : struct
    {
        if (EqualityComparer<T>.Default.Equals(field, value)) return;
        field = value;
        InvalidateMeasure();
    }

    protected virtual LayoutProposal MeasureProposal(Size available) => new(
        new(available.Width, exactWidth || !double.IsNaN(Width)),
        new(available.Height, exactHeight || !double.IsNaN(Height)));

    protected virtual LayoutProposal ArrangeProposal(Size final) => LayoutProposal.Exact(new(final.Width, final.Height));

    protected override Size MeasureOverride(Size availableSize)
    {
        var count = Children.Count;
        if (children.Length < count)
        {
            var capacity = Math.Max(count, 2 * children.Length);
            children = new LayoutChild[capacity];
            frames = new LayoutFrame[capacity];
            measuredChildren = new LayoutSize[capacity];
            visible.EnsureCapacity(capacity);
        }
        visible.Clear();
        var flexible = FlexibleChildren;
        for (var i = 0; i < count; i++)
        {
            if (Children[i] is not FrameworkElement child || child.Visibility == Visibility.Collapsed) continue;
            var insets = child is WeftBox box ? box.ContentInsets : default;
            var bounds = new LayoutBounds(child.Width, child.Height, child.MinWidth, child.MaxWidth, child.MinHeight,
                child.MaxHeight, (double)child.GetValue(WeftLayout.AspectRatioProperty), insets);
            var isSpacer = (bool)child.GetValue(WeftLayout.IsSpacerProperty);
            var spacer = isSpacer ? (double)child.GetValue(WeftLayout.SpacerSizeProperty) : double.NaN;
            if (isSpacer) child.Measure(new Size());
            var margin = child.Margin;
            children[visible.Count] = new(bounds, new LayoutInsets(margin.Top, margin.Right, margin.Bottom, margin.Left),
                flexible ? (double)child.GetValue(WeftLayout.GrowProperty) : 0,
                flexible && (bool)child.GetValue(WeftLayout.FillProperty), isSpacer, double.IsNaN(spacer) ? null : spacer);
            visible.Add(child);
        }
        measured = Compute(MeasureProposal(availableSize), measure);
        return new Size(measured.Width, measured.Height);
    }

    protected override Size ArrangeOverride(Size finalSize)
    {
        // WinUI can assign a different slot during arrange. Reuse desired sizes: native
        // measurements here would invalidate layout from within its arrange pass.
        if (finalSize.Width != (float)measured.Width || finalSize.Height != (float)measured.Height)
            Compute(ArrangeProposal(finalSize), reuseMeasurement);
        for (var i = 0; i < visible.Count; i++)
        {
            var frame = frames[i];
            var margin = children[i].Margin;
            // The algorithm positions the border box; WinUI expects the margin box.
            visible[i].Arrange(new Rect(frame.X - margin.Left, frame.Y - margin.Top,
                Math.Max(0, frame.Width + margin.Width), Math.Max(0, frame.Height + margin.Height)));
        }
        return finalSize;
    }

    protected abstract LayoutSize Compute(LayoutProposal proposal, ReadOnlySpan<LayoutChild> children,
        Func<int, LayoutProposal, LayoutSize> measure, Span<LayoutFrame> frames);

    private LayoutSize Compute(LayoutProposal proposal, Func<int, LayoutProposal, LayoutSize> measureChild) => Compute(
        proposal, children.AsSpan(0, visible.Count), measureChild, frames.AsSpan(0, visible.Count));

    private LayoutSize MeasureChild(int index, LayoutProposal proposal)
    {
        var child = visible[index];
        ref readonly var data = ref children[index];
        var constrained = Constrain(index, proposal);
        if (child is WeftBox box && box.Border.Child is WeftPanel panel) panel.SetProposal(constrained);
        child.Measure(new Size(Math.Max(0, constrained.Width.Value + data.Margin.Width),
            Math.Max(0, constrained.Height.Value + data.Margin.Height)));
        var desired = child.DesiredSize;
        var natural = new LayoutSize(Math.Max(0, desired.Width - data.Margin.Width), Math.Max(0, desired.Height - data.Margin.Height));
        measuredChildren[index] = natural;
        return data.Bounds.Resolve(constrained, natural);
    }

    private LayoutSize ReuseMeasurement(int index, LayoutProposal proposal) =>
        children[index].Bounds.Resolve(Constrain(index, proposal), measuredChildren[index]);

    private LayoutProposal Constrain(int index, LayoutProposal proposal)
    {
        var bounds = children[index].Bounds;
        var constrained = bounds.Constrain(proposal);
        if (!constrained.Width.IsExact)
        {
            var preferred = (double)visible[index].GetValue(WeftLayout.PreferredWidthProperty);
            if (!double.IsNaN(preferred))
                constrained = constrained with { Width = new(Math.Max(bounds.MinWidth, constrained.Width.Resolve(preferred)), true) };
        }
        if (!constrained.Height.IsExact)
        {
            var preferred = (double)visible[index].GetValue(WeftLayout.PreferredHeightProperty);
            if (!double.IsNaN(preferred))
                constrained = constrained with { Height = new(Math.Max(bounds.MinHeight, constrained.Height.Resolve(preferred)), true) };
        }
        return constrained;
    }
}

internal sealed class WeftLinearPanel(
    WeftAxis axis, bool scrolling = false, bool flexibleChildren = true, bool fillAvailable = false) : WeftPanel
{
    private double gap;
    private WeftCrossAlign align;
    private WeftJustify justify;

    internal bool IsRoot => fillAvailable;

    // The root is a focus sink for a blur request, not a normal tab stop.
    internal void TakeInputFocus()
    {
        IsTabStop = true;
        if (Focus(FocusState.Programmatic)) LostFocus += StopTakingInputFocus;
        else IsTabStop = false;
    }

    private void StopTakingInputFocus(object sender, RoutedEventArgs args)
    {
        if (args.OriginalSource != this) return;
        LostFocus -= StopTakingInputFocus;
        IsTabStop = false;
    }

    internal double Gap { set => SetLayout(ref gap, Math.Max(0, value)); }
    internal WeftCrossAlign Align { set => SetLayout(ref align, value); }
    internal WeftJustify Justify { set => SetLayout(ref justify, value); }

    protected override bool FlexibleChildren => flexibleChildren;

    protected override LayoutProposal MeasureProposal(Size available) => fillAvailable
        ? new(new(available.Width, double.IsFinite(available.Width)), new(available.Height, double.IsFinite(available.Height)))
        : UnboundScrollAxis(base.MeasureProposal(available));

    protected override LayoutProposal ArrangeProposal(Size final) => UnboundScrollAxis(base.ArrangeProposal(final));

    private LayoutProposal UnboundScrollAxis(LayoutProposal proposal)
    {
        // Content can overflow the viewport only on the scrolling axis, including during arrange.
        if (!scrolling) return proposal;
        return axis == WeftAxis.Vertical ? proposal with { Height = LayoutLimit.Unbounded }
                                         : proposal with { Width = LayoutLimit.Unbounded };
    }

    protected override LayoutSize Compute(LayoutProposal proposal, ReadOnlySpan<LayoutChild> children,
        Func<int, LayoutProposal, LayoutSize> measure, Span<LayoutFrame> frames) =>
        WeftLayoutAlgorithm.Linear(axis, gap, align, justify, default, proposal, children, measure, frames);
}

internal sealed class WeftStackPanel : WeftPanel
{
    private WeftAlignment alignment;

    internal WeftAlignment Alignment { set => SetLayout(ref alignment, value); }

    protected override LayoutSize Compute(LayoutProposal proposal, ReadOnlySpan<LayoutChild> children,
        Func<int, LayoutProposal, LayoutSize> measure, Span<LayoutFrame> frames) =>
        WeftLayoutAlgorithm.Stack(alignment, default, proposal, children, measure, frames);
}
