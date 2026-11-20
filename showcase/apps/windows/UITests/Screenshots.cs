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
            using var automation = new UIA3Automation();
            using var app = FlaUI.Core.Application.Launch(start);
            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(10));
                Assert.IsNotNull(window);
                var label = $"page:{id}";
                var button = Retry.WhileNull(() => window.FindAllDescendants(cf => cf.ByControlType(ControlType.Button))
                    .FirstOrDefault(candidate => candidate.Properties.Name.ValueOrDefault == label ||
                        candidate.FindFirstDescendant(cf => cf.ByName(label)) is not null),
                    TimeSpan.FromSeconds(5), ignoreException: true).Result;
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
            }
            finally
            {
                app.Close();
            }
        }
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
