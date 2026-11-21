using System;
using Microsoft.UI;
using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Automation;
using Microsoft.UI.Xaml.Automation.Peers;
using Microsoft.UI.Xaml.Automation.Provider;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Windows.Foundation;
using Windows.System;

namespace Weft.WinUI;

internal sealed partial class WeftBox
{
    private WeftTap? tap;

    internal void EnableTap(Action invoke) => tap = new WeftTap(this, invoke);

    internal void SetOpacity(double value)
    {
        if (tap is null) Opacity = value;
        else tap.SetOpacity(value);
    }

    protected override AutomationPeer OnCreateAutomationPeer() =>
        tap is null ? base.OnCreateAutomationPeer() : new TapPeer(this, tap);

    private sealed class TapPeer(WeftBox box, WeftTap tap) : FrameworkElementAutomationPeer(box), IInvokeProvider
    {
        protected override AutomationControlType GetAutomationControlTypeCore() => AutomationControlType.Button;
        protected override string GetClassNameCore() => nameof(WeftTap);
        protected override object GetPatternCore(PatternInterface patternInterface) =>
            patternInterface == PatternInterface.Invoke ? this : base.GetPatternCore(patternInterface);

        protected override string GetNameCore()
        {
            var explicitName = base.GetNameCore();
            return string.IsNullOrEmpty(explicitName) ? Label(box.Border) ?? "" : explicitName;
        }

        private static string? Label(DependencyObject visual)
        {
            // Nested buttons have their own names; only non-interactive content labels this one.
            if (visual is Control or WeftBox { tap: not null } or UIElement { Visibility: Visibility.Collapsed })
                return null;
            if (visual is TextBlock { Text.Length: > 0 } text) return text.Text;
            for (var i = 0; i < VisualTreeHelper.GetChildrenCount(visual); i++)
                if (Label(VisualTreeHelper.GetChild(visual, i)) is { } label) return label;
            return null;
        }

        public void Invoke() => box.DispatcherQueue.TryEnqueue(tap.Invoke);
    }
}

// Only boxes with a handler need a hit surface, input state or a button peer.
internal sealed class WeftTap
{
    private readonly WeftBox box;
    private readonly Action invoke;
    private double opacity = 1;
    private TapPress press;

    internal WeftTap(WeftBox box, Action invoke)
    {
        this.box = box;
        this.invoke = invoke;
        box.Background = new SolidColorBrush(Colors.Transparent);
        box.IsTabStop = true;
        box.UseSystemFocusVisuals = true;
        AutomationProperties.SetAccessibilityView(box, AccessibilityView.Content);
        box.PointerPressed += PointerPressed;
        box.PointerReleased += PointerReleased;
        box.PointerCaptureLost += PointerCanceled;
        box.PointerCanceled += PointerCanceled;
        box.KeyDown += KeyDown;
        box.KeyUp += KeyUp;
        box.LostFocus += (_, _) =>
        {
            if (press.ReleaseKey()) RenderOpacity();
        };
    }

    internal void SetOpacity(double value)
    {
        opacity = value;
        RenderOpacity();
    }

    internal void Invoke() => invoke();

    private void RenderOpacity() => box.Opacity = press.ApplyOpacity(opacity);

    private void PointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (e.Pointer.PointerDeviceType == PointerDeviceType.Mouse &&
            !e.GetCurrentPoint(box).Properties.IsLeftButtonPressed) return;
        // Handled child events (especially ButtonBase's) never reach this handler.
        e.Handled = true;
        if (press.HasPointer || !box.CapturePointer(e.Pointer)) return;
        press.PressPointer(e.Pointer.PointerId);
        RenderOpacity();
    }

    private void PointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (!press.ReleasePointer(e.Pointer.PointerId)) return;
        e.Handled = true;
        var inside = new Rect(0, 0, box.ActualWidth, box.ActualHeight).Contains(e.GetCurrentPoint(box).Position);
        RenderOpacity();
        box.ReleasePointerCapture(e.Pointer);
        if (inside) Invoke();
    }

    private void PointerCanceled(object sender, PointerRoutedEventArgs e)
    {
        if (!press.ReleasePointer(e.Pointer.PointerId)) return;
        e.Handled = true;
        RenderOpacity();
    }

    private void KeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key is not (VirtualKey.Enter or VirtualKey.Space) ||
            FocusManager.GetFocusedElement(box.XamlRoot) != box) return;
        e.Handled = true;
        if (e.KeyStatus.WasKeyDown) return;
        press.PressKey(e.Key);
        RenderOpacity();
    }

    private void KeyUp(object sender, KeyRoutedEventArgs e)
    {
        if (!press.ReleaseKey(e.Key)) return;
        e.Handled = true;
        RenderOpacity();
        Invoke();
    }
}

internal struct TapPress
{
    private uint? pointer;
    private VirtualKey? key;

    internal readonly bool HasPointer => pointer.HasValue;
    internal readonly double ApplyOpacity(double opacity) => pointer.HasValue || key.HasValue ? opacity * 0.6 : opacity;

    internal void PressPointer(uint id) => pointer = id;

    internal bool ReleasePointer(uint id)
    {
        if (pointer != id) return false;
        pointer = null;
        return true;
    }

    internal void PressKey(VirtualKey value) => key ??= value;
    internal bool ReleaseKey(VirtualKey value) => key == value && ReleaseKey();

    internal bool ReleaseKey()
    {
        if (!key.HasValue) return false;
        key = null;
        return true;
    }
}
