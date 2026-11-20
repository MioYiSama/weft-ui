using Microsoft.UI.Xaml;
using Weft.WinUI;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class AppearanceTests
{
    [TestMethod]
    public void ContentAspectRatioExcludesPaddingAndBorder()
    {
        var insets = WeftBox.Insets(new Thickness(3, 5, 7, 11), 2);
        Assert.AreEqual(new LayoutInsets(7, 9, 13, 5), insets);
        var bounds = new LayoutBounds(94, double.NaN, 0, double.PositiveInfinity, 0, double.PositiveInfinity, 2, insets);
        var proposal = bounds.Constrain(new LayoutProposal(LayoutLimit.Unbounded, LayoutLimit.Unbounded));
        Assert.AreEqual(new LayoutLimit(94, true), proposal.Width);
        // An 80x40 content box with 20 DIPs of padding and border on the vertical axis.
        Assert.AreEqual(new LayoutLimit(60, true), proposal.Height);
    }

    [TestMethod]
    public void DefaultBorderFollowsAppearance()
    {
        var light = WeftBox.DefaultBorderColor(ElementTheme.Light).Resolve(null, ElementTheme.Light);
        var dark = WeftBox.DefaultBorderColor(ElementTheme.Dark).Resolve(null, ElementTheme.Dark);
        Assert.AreEqual(Windows.UI.Color.FromArgb(0x1f, 0, 0, 0), light);
        Assert.AreEqual(Windows.UI.Color.FromArgb(0x26, 255, 255, 255), dark);
    }

    [TestMethod]
    public void ShadowPresetsMatchWebOffsetsBlurAndAlpha()
    {
        Assert.AreEqual(default(ShadowStyle), ShadowStyle.For(WeftShadow.None));
        Assert.AreEqual(new ShadowStyle(1, 3, 0x1f), ShadowStyle.For(WeftShadow.Small));
        Assert.AreEqual(new ShadowStyle(4, 12, 0x26), ShadowStyle.For(WeftShadow.Medium));
        Assert.AreEqual(new ShadowStyle(12, 32, 0x2e), ShadowStyle.For(WeftShadow.Large));
    }
}
