using System.Reflection;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Definitions;
using FlaUI.Core.Tools;
using FlaUI.UIA3;

namespace UITests;

/// <summary>
/// The example's click flow over every built-in element; run with
/// `pnpm --filter example test:windows` from an interactive desktop session
/// after `pnpm --filter example build:windows`.
/// </summary>
[TestClass]
public sealed class UITests
{
    private static readonly TimeSpan Timeout = TimeSpan.FromSeconds(5);

    [TestMethod]
    public void ClickFlow()
    {
        using var automation = new UIA3Automation();
        using var app = Application.Launch(AppPath());
        try
        {
            var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(10));
            Assert.IsNotNull(window);

            // Button and `if`.
            ButtonWithText(window, "Click me").Invoke();
            WaitForText(window, "You clicked!");

            // A controlled input: drafts longer than 12 characters are refused and rolled back.
            var field = window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Edit))?.AsTextBox()
                ?? throw new AssertFailedException("no input");
            var add = ButtonWithText(window, "Add");
            Assert.IsFalse(add.IsEnabled);
            Type(field, "Buy milk and bread");
            Assert.AreEqual("Buy milk and", field.Text);
            Assert.IsTrue(add.IsEnabled);

            // `for` rows.
            add.Invoke();
            WaitForText(window, "Buy milk and");
            WaitFor(() => field.Text == "", () => $"the input shows \"{field.Text}\"");
            Type(field, "Eggs");
            WaitFor(() => add.IsEnabled, () => "Add stays disabled");
            add.Invoke();
            WaitForText(window, "2 todos");

            // Checkboxes in the rows.
            var first = CheckBox(window, 0);
            Assert.AreEqual(ToggleState.Off, first.ToggleState);
            first.Toggle();
            WaitFor(() => first.ToggleState == ToggleState.On, () => "the first checkbox stays unchecked");
            Assert.AreEqual(ToggleState.Off, CheckBox(window, 1).ToggleState);

            // Removing the first row keeps the second.
            ButtonsWithText(window, "Remove")[0].Invoke();
            WaitForText(window, "1 todos");
            Assert.IsNull(Text(window, "Buy milk and"));
            Assert.IsNotNull(Text(window, "Eggs"));
            Assert.AreEqual(ToggleState.Off, CheckBox(window, 0).ToggleState);
        }
        finally
        {
            app.Close();
        }
    }

    /// <summary>
    /// Types <paramref name="text"/> one character at a time, each an edit of
    /// the input's current value. UI Automation sets the value directly, so the
    /// edits do not depend on the keyboard layout or input method of the desktop.
    /// </summary>
    private static void Type(TextBox field, string text)
    {
        foreach (var c in text) field.Text += c;
    }

    private static AutomationElement? Text(Window window, string text) =>
        window.FindFirstDescendant(cf => cf.ByControlType(ControlType.Text).And(cf.ByName(text)));

    private static void WaitForText(Window window, string text) =>
        WaitFor(() => Text(window, text) is not null, () => $"no text \"{text}\"");

    private static void WaitFor(Func<bool> condition, Func<string> failure)
    {
        if (!Retry.WhileFalse(condition, Timeout, ignoreException: true).Result) Assert.Fail(failure());
    }

    private static CheckBox CheckBox(Window window, int index) =>
        window.FindAllDescendants(cf => cf.ByControlType(ControlType.CheckBox))[index].AsCheckBox();

    /// <summary>Buttons whose content holds the text <paramref name="text"/>, in order.</summary>
    private static Button[] ButtonsWithText(Window window, string text) =>
        window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button))
            .Where(button => button.FindFirstDescendant(cf => cf.ByName(text)) is not null)
            .Select(button => button.AsButton())
            .ToArray();

    private static Button ButtonWithText(Window window, string text) =>
        ButtonsWithText(window, text).FirstOrDefault() ?? throw new AssertFailedException($"no button \"{text}\"");

    /// <summary>The most recently built App.exe of the app project.</summary>
    private static string AppPath()
    {
        var project = typeof(UITests).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>()
            .Single(a => a.Key == "AppProject").Value!;
        return new DirectoryInfo(Path.Combine(project, "bin"))
            .EnumerateFiles("App.exe", SearchOption.AllDirectories)
            .MaxBy(file => file.LastWriteTimeUtc)?.FullName
            ?? throw new FileNotFoundException($"App.exe is not built in {project}; build the app first");
    }
}
