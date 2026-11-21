using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Capturing;
using FlaUI.Core.Definitions;
using FlaUI.Core.Tools;
using FlaUI.UIA3;

namespace UITests;

[TestClass]
public sealed class Screenshots
{
    [TestMethod]
    public void Pages()
    {
        var pages = Environment.GetEnvironmentVariable("WEFT_PAGES");
        var directory = Environment.GetEnvironmentVariable("WEFT_SHOT_DIR");
        if (pages is null || directory is null)
        {
            Assert.Inconclusive("Run pnpm --filter showcase shot:windows");
            return;
        }

        foreach (var page in pages.Split(','))
        {
            var parts = page.Split(':');
            var id = parts[0];
            var scheme = parts[1];
            var start = new ProcessStartInfo(UITests.AppPath()) { UseShellExecute = false };
            start.Environment["WEFT_SHOT_APPEARANCE"] = scheme;
            start.Environment["WEFT_SHOT_PAGE"] = id;
            using var automation = new UIA3Automation();
            using var app = FlaUI.Core.Application.Launch(start);
            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(10));
                Assert.IsNotNull(window);
                var label = $"page:{id}";
                var button = FindButton(window, label);
                if (button is null)
                {
                    var tree = string.Join("\n", window.FindAllDescendants().Select(element =>
                        $"{element.Properties.ControlType.ValueOrDefault}: {element.Properties.Name.ValueOrDefault}"));
                    Assert.Fail($"No catalog button {label}. Window elements:\n{tree}");
                }
                button.Patterns.ScrollItem.PatternOrDefault?.ScrollIntoView();
                button.AsButton().Invoke();
                Thread.Sleep(TimeSpan.FromSeconds(1));

                var suffix = scheme == "dark" ? "-dark" : "";
                CaptureWindow(window, Path.Combine(directory, $"{id}{suffix}.png"));
                if (id == "appearance") VerifyVisibility(window);
                if (id == "text") VerifyTypography(window);
                if (id == "controls") VerifyControls(window);
            }
            finally
            {
                app.Close();
            }
        }
    }

    private static AutomationElement? FindButton(Window window, string label) =>
        Retry.WhileNull(() => window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button))
            .FirstOrDefault(candidate => candidate.Properties.Name.ValueOrDefault == label ||
                candidate.FindFirstDescendant(cf => cf.ByName(label)) is not null),
            TimeSpan.FromSeconds(5), ignoreException: true).Result;

    private static void VerifyVisibility(Window window)
    {
        const string cardLabel = "This card leaves no gap when hidden.";
        var hide = FindButton(window, "Hide card");
        var card = window.FindFirstDescendant(cf => cf.ByName(cardLabel));
        var end = window.FindFirstDescendant(cf => cf.ByName("End of grid"));
        Assert.IsNotNull(hide);
        Assert.IsNotNull(card);
        Assert.IsNotNull(end);
        var fixedGap = end.BoundingRectangle.Top - card.BoundingRectangle.Bottom;
        var before = end.BoundingRectangle.Top;

        hide.AsButton().Invoke();
        var show = FindButton(window, "Show card");
        Assert.IsNotNull(show);
        var after = Retry.While(() => end.BoundingRectangle.Top, y => y >= before, TimeSpan.FromSeconds(5));
        Assert.IsFalse(after.TimedOut, "The last label must move up when the card collapses");
        Assert.IsNull(window.FindFirstDescendant(cf => cf.ByName(cardLabel)));
        Assert.IsTrue(after.Result - show.BoundingRectangle.Bottom < fixedGap,
            "The collapsed card must not leave its own gap after the button");

        show.AsButton().Invoke();
        Assert.IsNotNull(FindButton(window, "Hide card"));
        Assert.IsNotNull(Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByName(cardLabel)),
            TimeSpan.FromSeconds(5)).Result);
    }

    private static void VerifyTypography(Window window)
    {
        var texts = window.FindAllDescendants(cf => cf.ByControlType(ControlType.Text));
        AutomationElement Text(string prefix)
        {
            var matches = texts.Where(element =>
                element.Properties.Name.ValueOrDefault?.StartsWith(prefix, StringComparison.Ordinal) == true).ToArray();
            Assert.AreEqual(1, matches.Length, $"Expected one text block beginning with {prefix}");
            return matches[0];
        }

        Assert.AreEqual("Read closely, pause often, then turn the page.",
            Text("Read closely").Properties.Name.ValueOrDefault, "Spans must read as one text block");

        var oneLine = Text("Italic").BoundingRectangle.Height;
        Assert.IsTrue(oneLine > 0);
        var limited = Text("The wind lifted").BoundingRectangle.Height;
        Assert.AreEqual(2.0 * oneLine, limited, 2.0, "The limited paragraph must occupy two lines");

        var compact = Text("18 px").BoundingRectangle.Height;
        var expanded = Text("28 px").BoundingRectangle.Height;
        Assert.IsTrue(compact > 0);
        Assert.AreEqual(28.0 / 18.0, (double)expanded / compact, 0.08, "Both lines must use the requested height");
    }

    private static void VerifyControls(Window window)
    {
        AutomationElement[] Elements(string className, int count)
        {
            var matches = window.FindAllDescendants(cf => cf.ByClassName(className));
            Assert.AreEqual(count, matches.Length, $"Expected {count} {className} controls");
            return matches;
        }

        var switches = Elements("ToggleSwitch", 4);
        var locked = switches[0].Patterns.Toggle.Pattern;
        var alerts = switches[1].Patterns.Toggle.Pattern;
        var sounds = switches[2].Patterns.Toggle.Pattern;
        var checkboxes = Elements("CheckBox", 2);
        var checkbox = checkboxes[0].AsCheckBox();
        var sliders = Elements("Slider", 2);
        var slider = sliders[0].AsSlider();
        var pickers = Elements("ComboBox", 3);
        var region = pickers[0].AsComboBox();
        var channel = pickers[1].AsComboBox();
        var textAreas = Elements("TextBox", 2);
        var notes = textAreas[0].AsTextBox();
        Assert.IsTrue(notes.BoundingRectangle.Height > region.BoundingRectangle.Height * 1.5,
            "The multiline box should reserve three lines even while empty");
        foreach (var disabled in new[] { switches[3], checkboxes[1], sliders[1], pickers[2], textAreas[1] })
            Assert.IsFalse(disabled.IsEnabled, $"{disabled.Properties.ClassName.Value} should be disabled");

        void Volume(double value)
        {
            Assert.AreEqual(value, slider.Value, "The slider thumb must match the controlled value");
            Assert.IsNotNull(window.FindFirstDescendant(cf => cf.ByName($"音量 · {value}%")),
                "The Weft state must also hold the stepped value");
        }

        void Select(ComboBox picker, string label)
        {
            picker.Expand();
            var item = Retry.WhileNull(() => picker.FindFirstDescendant(cf => cf.ByName(label)
                .And(cf.ByControlType(ControlType.ListItem))), TimeSpan.FromSeconds(3)).Result;
            Assert.IsNotNull(item, $"No picker option {label}");
            item.Patterns.SelectionItem.Pattern.Select();
            picker.Collapse();
        }

        string Notes() => notes.Text.Replace("\r\n", "\n").Replace('\r', '\n');

        Assert.AreEqual(ToggleState.Off, locked.ToggleState.Value);
        Assert.AreEqual(ToggleState.On, alerts.ToggleState.Value);
        Assert.AreEqual(ToggleState.Off, sounds.ToggleState.Value);
        Assert.AreEqual(ToggleState.On, checkbox.ToggleState);
        Assert.AreEqual("本地", region.SelectedItem?.Text);
        Assert.IsNull(channel.SelectedItem, "An out-of-range value must show the placeholder");
        Volume(40);

        alerts.Toggle();
        sounds.Toggle();
        checkbox.Toggle();
        slider.Value = 66;
        Select(region, "远程");
        notes.Text = "first\r\nsecond";
        Assert.AreEqual(ToggleState.Off, alerts.ToggleState.Value);
        Assert.AreEqual(ToggleState.On, sounds.ToggleState.Value);
        Assert.AreEqual(ToggleState.Off, checkbox.ToggleState);
        Volume(65);
        Assert.AreEqual("远程", region.SelectedItem?.Text);
        Assert.AreEqual("first\nsecond", Notes());

        locked.Toggle();
        Assert.AreEqual(ToggleState.On, locked.ToggleState.Value);
        alerts.Toggle();
        sounds.Toggle();
        checkbox.Toggle();
        slider.Value = 40;
        Select(region, "本地");
        Select(channel, "测试版");
        notes.Text = "rejected";
        Assert.AreEqual(ToggleState.Off, alerts.ToggleState.Value);
        Assert.AreEqual(ToggleState.On, sounds.ToggleState.Value);
        Assert.AreEqual(ToggleState.Off, checkbox.ToggleState);
        Volume(65);
        Assert.AreEqual("远程", region.SelectedItem?.Text);
        Assert.IsNull(channel.SelectedItem, "A refused selection must bring back the placeholder");
        Assert.AreEqual("first\nsecond", Notes());

        locked.Toggle();
        Select(channel, "测试版");
        Assert.AreEqual("测试版", channel.SelectedItem?.Text);
    }

    private static void CaptureWindow(Window window, string path)
    {
        // FlaUI crops the desktop by a window's bounds. Use physical pixels for both.
        const int perMonitorAwareV2 = -4;
        var previous = SetThreadDpiAwarenessContext(perMonitorAwareV2);
        if (previous == 0) throw new Win32Exception(Marshal.GetLastWin32Error());
        try
        {
            Capture.Element(window).ToFile(path);
        }
        finally
        {
            SetThreadDpiAwarenessContext(previous);
        }
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern nint SetThreadDpiAwarenessContext(nint dpiContext);
}
