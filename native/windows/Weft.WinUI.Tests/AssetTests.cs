using Weft.WinUI;
using Windows.Foundation;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class AssetTests
{
    private static readonly WeftAssets Assets = new(new()
    {
        ["logo"] = [new("logo.scale-100.png", 1), new("logo.scale-200.png", 2), new("logo.scale-300.jpg", 3)],
        ["only_high"] = [new("only_high.scale-200.png", 2)],
        ["star"] = [new("star.svg", 1)],
    }, [
        new("Inter-Regular.ttf", "Inter", 400, false),
        new("Inter SemiBold-SemiBold.ttf", "Inter", 600, false),
        new("Inter-Bold.ttf", "Inter", 700, false),
        new("Inter-RegularItalic.ttf", "Inter", 400, true),
    ]);

    [TestMethod]
    [DataRow("logo", 1d, "logo.scale-100.png")]
    [DataRow("logo", 1.25d, "logo.scale-200.png")]
    [DataRow("logo", 2d, "logo.scale-200.png")]
    [DataRow("logo", 2.25d, "logo.scale-300.jpg")]
    [DataRow("logo", 4d, "logo.scale-300.jpg")]
    [DataRow("only_high", 1d, "only_high.scale-200.png")]
    [DataRow("star", 2d, "star.svg")]
    [DataRow("missing", 1d, null)]
    public void SelectsAssetOrLeavesPlaceholder(string name, double scale, string? file) =>
        Assert.AreEqual(file, Assets.FindImage(name, scale)?.File);

    [TestMethod]
    public void SelectsCustomFaceByWeightAndStyle()
    {
        Assert.AreEqual("ms-appx:///weft/assets/fonts/Inter%20SemiBold-SemiBold.ttf#Inter", Assets.FindFont("Inter", 600, false)?.Source);
        Assert.AreEqual("Inter-Bold.ttf", Assets.FindFont("inter", 700, false)?.File);
        Assert.AreEqual("Inter-RegularItalic.ttf", Assets.FindFont("Inter", 600, true)?.File);
        Assert.AreEqual("Inter-Regular.ttf", Assets.FindFont("Inter", 500, false)?.File);
        Assert.IsNull(Assets.FindFont("Missing", 400, false));
    }

    [TestMethod]
    public void CoverPlacesTheOverflowOnBothSidesOfTheViewport()
    {
        Assert.AreEqual(new Rect(-40, 0, 160, 80), WeftImageContent.CoverBounds(new Size(80, 80), new Size(200, 100)));
        Assert.AreEqual(new Rect(0, -40, 80, 160), WeftImageContent.CoverBounds(new Size(80, 80), new Size(100, 200)));
    }

    [TestMethod]
    public void SizesImageByItsLogicalDimensionsAndPreservesAspectRatioForOneExactSide()
    {
        var intrinsic = new Size(64, 40);
        var unbounded = new LayoutProposal(LayoutLimit.Unbounded, LayoutLimit.Unbounded);
        Assert.AreEqual(intrinsic, WeftImageContent.MeasureImage(unbounded, intrinsic));
        Assert.AreEqual(new Size(32, 20), WeftImageContent.MeasureImage(unbounded with { Width = new(32, true) }, intrinsic));
        Assert.AreEqual(new Size(32, 20), WeftImageContent.MeasureImage(unbounded with { Height = new(20, true) }, intrinsic));
        var exact = LayoutProposal.Exact(new(80, 60));
        Assert.AreEqual(new Size(80, 60), WeftImageContent.MeasureImage(exact, intrinsic));
        // Without intrinsic dimensions the placeholder keeps the declared content box.
        Assert.AreEqual(new Size(80, 60), WeftImageContent.MeasureImage(exact, default));
    }
}
