using System;
using System.Collections.Generic;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;

namespace Weft.WinUI;

// Controlled edits dispatch synchronously. When the app refuses an edit, show the
// value slot again. Values applied by set ops are never reported back.

/// <summary>The shared controlled editing path for <c>Input</c> and <c>TextArea</c>.</summary>
public abstract class WeftTextInput : WeftElement
{
    // The normal intrinsic width is about 20 characters, not a fixed layout constraint.
    private const double InputWidth = 200;
    private protected TextBox Box { get; } = new();
    private readonly Func<string, bool> edit;
    private string value = "";
    private bool composing;

    private protected WeftTextInput(Func<string, bool> edit)
    {
        this.edit = edit;
        Box.SetValue(WeftLayout.PreferredWidthProperty, InputWidth);
        Box.TextChanging += (_, args) =>
        {
            if (args.IsContentChanging) Edited();
        };
        // Keep IME composition local until the text is committed.
        Box.TextCompositionStarted += (_, _) => composing = true;
        Box.TextCompositionEnded += (_, _) =>
        {
            composing = false;
            Edited();
        };
    }

    protected sealed override FrameworkElement Control => Box;

    /// <summary>The controlled value slot.</summary>
    public string Value
    {
        set
        {
            this.value = Normalize(value);
            if (!composing && Normalize(Box.Text) != this.value) Show();
        }
    }

    public string Placeholder { set => Box.PlaceholderText = value; }
    public bool Disabled { set => Box.IsEnabled = !value; }

    private string Normalize(string text) => Box.AcceptsReturn
        ? text.Replace("\r\n", "\n").Replace('\r', '\n')
        : text;

    private void Edited()
    {
        if (composing) return;
        var text = Normalize(Box.Text);
        if (text != value && !edit(text)) Show();
    }

    /// <summary>Replaces the text, keeping the caret where the new text allows.</summary>
    private void Show()
    {
        var caret = Box.SelectionStart;
        Box.Text = value;
        Box.SelectionStart = Math.Min(caret, Box.Text.Length);
    }
}

/// <summary><c>Input</c>: a single-line <see cref="TextBox"/>.</summary>
public sealed class WeftInput(Func<string, bool> edit) : WeftTextInput(edit) { }

/// <summary><c>TextArea</c>: a wrapping <see cref="TextBox"/> with a three-line intrinsic height.</summary>
public sealed class WeftTextArea : WeftTextInput
{
    private const int Rows = 3;
    private const double LineHeightMultiplier = 1.4;

    public WeftTextArea(Func<string, bool> edit) : base(edit)
    {
        Box.AcceptsReturn = true;
        Box.TextWrapping = TextWrapping.Wrap;
        ScrollViewer.SetVerticalScrollBarVisibility(Box, ScrollBarVisibility.Auto);
        Box.RegisterPropertyChangedCallback(TextBox.FontSizeProperty, (_, _) => UpdatePreferredHeight());
        Box.RegisterPropertyChangedCallback(TextBox.PaddingProperty, (_, _) => UpdatePreferredHeight());
        UpdatePreferredHeight();
    }

    private void UpdatePreferredHeight() => Box.SetValue(WeftLayout.PreferredHeightProperty,
        Box.FontSize * LineHeightMultiplier * Rows + Box.Padding.Top + Box.Padding.Bottom);
}

/// <summary><c>Checkbox</c>: a <see cref="CheckBox"/> without a label.</summary>
public sealed class WeftCheckbox : WeftElement
{
    private readonly CheckBox box = new() { MinWidth = 0 };
    private readonly Func<bool, bool> edit;
    private bool value;

    public WeftCheckbox(Func<bool, bool> edit)
    {
        this.edit = edit;
        box.IsChecked = false;
        box.Checked += (_, _) => Edited(true);
        box.Unchecked += (_, _) => Edited(false);
    }

    protected override FrameworkElement Control => box;

    /// <summary>The controlled value slot.</summary>
    public bool Checked
    {
        set
        {
            this.value = value;
            box.IsChecked = value;
        }
    }

    public bool Disabled
    {
        set => box.IsEnabled = !value;
    }

    private void Edited(bool now)
    {
        if (now == value) return;
        if (!edit(now)) box.IsChecked = value;
    }
}

/// <summary><c>Switch</c>: a <see cref="ToggleSwitch"/> without on/off labels.</summary>
public sealed class WeftSwitch : WeftElement
{
    private readonly ToggleSwitch toggle = new() { OnContent = "", OffContent = "", MinWidth = 0 };
    private bool value;

    public WeftSwitch(Func<bool, bool> edit) => toggle.Toggled += (_, _) =>
    {
        var now = toggle.IsOn;
        if (now != value && !edit(now)) toggle.IsOn = value;
    };

    protected override FrameworkElement Control => toggle;
    public bool Checked { set { this.value = value; toggle.IsOn = value; } }
    public bool Disabled { set => toggle.IsEnabled = !value; }
}

/// <summary><c>Slider</c>: an integer-valued <see cref="Slider"/>.</summary>
public sealed class WeftSlider : WeftElement
{
    private readonly Slider slider = new() { Maximum = 100, StepFrequency = 1, SmallChange = 1, SnapsTo = SliderSnapsTo.StepValues };
    private double value, minimum, maximum = 100, step = 1;
    private bool updating;

    public WeftSlider(Func<double, bool> edit)
    {
        slider.SetValue(WeftLayout.PreferredWidthProperty, 128.0);
        slider.ValueChanged += (_, _) =>
        {
            if (updating) return;
            var now = RoundValue(slider.Value, minimum, slider.Maximum, step);
            if (now == VisibleValue || !edit(now)) Show();
        };
    }

    protected override FrameworkElement Control => slider;
    public double Value { set { this.value = value; Show(); } }
    public double Min { set { minimum = value; UpdateRange(); } }
    public double Max { set { maximum = value; UpdateRange(); } }
    public double Step { set { step = Math.Max(1, value); UpdateRange(); } }
    public bool Disabled { set => slider.IsEnabled = !value; }

    private double VisibleValue => RoundValue(value, minimum, slider.Maximum, step);

    // Keep the requested value and bounds even when the native Slider coerces a slot.
    // Slots arrive independently, so changing the range must reapply the current value.
    private void UpdateRange()
    {
        updating = true;
        var high = Math.Max(minimum, maximum);
        if (minimum > slider.Maximum) slider.Maximum = high;
        slider.Minimum = minimum;
        slider.Maximum = high;
        slider.StepFrequency = step;
        slider.SmallChange = step;
        Show();
        updating = false;
    }

    private void Show()
    {
        var visible = VisibleValue;
        if (slider.Value != visible) slider.Value = visible;
    }

    internal static double RoundValue(double value, double min, double max, double step)
    {
        var index = (Math.Clamp(value, min, max) - min) / step;
        // Step values are anchored at min, with ties rounded toward the larger value.
        return min + Math.Min(Math.Floor(index + 0.5), Math.Floor((max - min) / step)) * step;
    }
}

/// <summary><c>Picker</c>: a <see cref="ComboBox"/> showing string options.</summary>
public sealed class WeftPicker : WeftElement
{
    private readonly ComboBox combo = new();
    private double selected = -1;
    private int count;
    private bool updatingOptions;

    public WeftPicker(Func<double, bool> edit) => combo.SelectionChanged += (_, _) =>
    {
        if (updatingOptions) return;
        var now = combo.SelectedIndex;
        if (now != VisibleIndex && (now < 0 || !edit(now))) Show();
    };

    protected override FrameworkElement Control => combo;
    public double Selected { set { selected = value; Show(); } }
    public string Placeholder { set => combo.PlaceholderText = value; }
    public bool Disabled { set => combo.IsEnabled = !value; }

    public IReadOnlyList<string> Options
    {
        set
        {
            updatingOptions = true;
            count = value.Count;
            combo.ItemsSource = value;
            Show();
            updatingOptions = false;
        }
    }

    private int VisibleIndex => selected >= 0 && selected < count ? (int)selected : -1;

    private void Show()
    {
        var index = VisibleIndex;
        if (combo.SelectedIndex != index) combo.SelectedIndex = index;
    }
}
