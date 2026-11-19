using Weft.WinUI;

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
        Assert.AreEqual((byte)0x80, color.A);
        Assert.IsFalse(WeftColor.TryParse("red", out _));
        Assert.IsFalse(WeftColor.TryParse("#12345", out _));
        Assert.IsFalse(WeftColor.TryParse("#ggg", out _));
        // Invalid dynamic colors are transparent.
        Assert.AreEqual(default, WeftValue.Of("nope").Color);
    }
}
