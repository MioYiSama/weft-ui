using Weft.WinUI;
using Windows.System;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class TapTests
{
    [TestMethod]
    public void OnlyTheCapturedPointerEndsThePress()
    {
        var press = new TapPress();
        press.PressPointer(1);
        Assert.AreEqual(0.48, press.ApplyOpacity(0.8), 1e-9);
        Assert.IsFalse(press.ReleasePointer(2));
        Assert.IsTrue(press.HasPointer);
        // A value update under capture keeps the same press multiplier.
        Assert.AreEqual(0.3, press.ApplyOpacity(0.5), 1e-9);
        Assert.IsTrue(press.ReleasePointer(1));
        Assert.AreEqual(0.5, press.ApplyOpacity(0.5));
        Assert.IsFalse(press.ReleasePointer(1));
    }

    [TestMethod]
    public void OverlappingKeyboardAndPointerPressesDoNotDimTwice()
    {
        var press = new TapPress();
        press.PressPointer(1);
        press.PressKey(VirtualKey.Enter);
        Assert.AreEqual(0.6, press.ApplyOpacity(1));
        Assert.IsFalse(press.ReleaseKey(VirtualKey.Space));
        Assert.IsTrue(press.ReleasePointer(1));
        Assert.AreEqual(0.6, press.ApplyOpacity(1));
        Assert.IsTrue(press.ReleaseKey());
        Assert.IsFalse(press.ReleaseKey(VirtualKey.Enter));
        Assert.AreEqual(1.0, press.ApplyOpacity(1));
    }
}
