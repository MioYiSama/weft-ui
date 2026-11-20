using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace Weft.WinUI;

// Controlled `Input` and `Checkbox`. The control reports every user edit
// through `edit`, which dispatches it synchronously; when the app refuses
// it (`edit` returns false) the control shows its value slot again. Values
// written by `set` ops are recognized by comparing with the slot, so they are
// never reported back. Text being composed by an input method is not
// reported, so it is never rolled back mid-composition; the committed text is.

/// <summary><c>Input</c>: a <see cref="TextBox"/>.</summary>
public sealed class WeftInput : WeftElement
{
    // The normal intrinsic width is about 20 characters, not a fixed layout constraint.
    private const double InputWidth = 200;
    private readonly TextBox box = new();
    private readonly Func<string, bool> edit;
    private string value = "";
    private bool composing;

    public WeftInput(Func<string, bool> edit)
    {
        this.edit = edit;
        box.SetValue(WeftLayout.PreferredWidthProperty, InputWidth);
        box.TextChanging += (_, args) =>
        {
            if (args.IsContentChanging) Edited();
        };
        box.TextCompositionStarted += (_, _) => composing = true;
        box.TextCompositionEnded += (_, _) =>
        {
            composing = false;
            Edited();
        };
    }

    protected override FrameworkElement Control => box;

    /// <summary>The controlled value slot.</summary>
    public string Value
    {
        set
        {
            this.value = value;
            if (!composing && box.Text != value) Show(value);
        }
    }

    public string Placeholder
    {
        set => box.PlaceholderText = value;
    }

    public bool Disabled
    {
        set => box.IsEnabled = !value;
    }

    private void Edited()
    {
        if (composing || box.Text == value) return;
        if (!edit(box.Text)) Show(value);
    }

    /// <summary>Replaces the text, keeping the caret where it was as far as the new text allows.</summary>
    private void Show(string text)
    {
        var caret = box.SelectionStart;
        box.Text = text;
        box.SelectionStart = Math.Min(caret, text.Length);
    }
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
