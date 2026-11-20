using Microsoft.UI.Xaml;
using Weft.WinUI;
using Windows.UI;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class OpReaderTests
{
    [TestMethod]
    public void ParsesColors()
    {
        Assert.IsTrue(WeftColor.TryParse("#3B82f6", out var color));
        Assert.AreEqual(WeftColor.Rgba(0x3b82f6ff), color);
        Assert.IsTrue(WeftColor.TryParse("#abc", out color));
        Assert.AreEqual(WeftColor.Rgba(0xaabbccff), color);
        Assert.IsTrue(WeftColor.TryParse("#12345680", out color));
        Assert.AreEqual(Color.FromArgb(0x80, 0x12, 0x34, 0x56), color.Resolve(null, ElementTheme.Light));
        Assert.IsFalse(WeftColor.TryParse("red", out _));
        Assert.IsFalse(WeftColor.TryParse("#12345", out _));
        Assert.IsFalse(WeftColor.TryParse("#ggg", out _));
        Assert.IsFalse(WeftColor.TryParse("#abc\0\0\0", out _));
        Assert.IsFalse(WeftColor.TryParse("@", out _));
        Assert.IsFalse(WeftColor.TryParse("@accent!", out _));
        // Invalid dynamic colors are transparent.
        Assert.AreEqual(default, WeftValue.Of("nope").Color);
    }

    [TestMethod]
    public void ResolvesThemeColorSlotsAgainstTheSessionTheme()
    {
        var theme = WeftTheme.Register(new() { ["accent"] = (0x0a84ffff, 0x409cff80) });
        var otherTheme = WeftTheme.Register(new() { ["accent"] = (0xaabbccff, 0xaabbccff) });
        var token = WeftValue.Of("@accent").Color;
        Assert.AreEqual(WeftColor.Token("accent"), token);
        Assert.AreEqual(Color.FromArgb(0xff, 0x0a, 0x84, 0xff), token.Resolve(theme, ElementTheme.Light));
        Assert.AreEqual(Color.FromArgb(0x80, 0x40, 0x9c, 0xff), token.Resolve(theme, ElementTheme.Dark));
        Assert.AreEqual(Color.FromArgb(0xff, 0xaa, 0xbb, 0xcc), token.Resolve(otherTheme, ElementTheme.Dark));
        Assert.AreEqual(default, token.Resolve(null, ElementTheme.Dark));
        Assert.AreEqual(default, WeftColor.Token("missing").Resolve(theme, ElementTheme.Dark));
    }
}
