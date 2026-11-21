using Weft.WinUI;
using Weft.WinUI.Protocol;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class ControlTests
{
    [TestMethod]
    public void PickerOptionsDecodeAsAnOrderedStringList()
    {
        byte[] batch =
        [
            WeftProtocol.Version, (byte)OpCode.Set, 1, 0, 0, 0, 2, 0, 0, 0,
            (byte)ValueTag.Strings, 3, 0, 0, 0,
            2, 0, 0, 0, 0xc3, 0xa9,
            0, 0, 0, 0,
            2, 0, 0, 0, 0xc3, 0xa9,
        ];
        var ops = new List<Op>();
        OpReader.ReadAll(batch, ops);
        CollectionAssert.AreEqual(new[] { "é", "", "é" }, ops.Single().Value.Strings.ToArray());
        Assert.AreEqual(0, WeftValue.Of("not a list").Strings.Count);
    }

    [TestMethod]
    [DataRow("ab😀c", 5, "ab😀c")]
    [DataRow("ab😀c", 4, "ab😀")]
    [DataRow("ab😀c", 3, "ab")]
    [DataRow("😀", 1, "")]
    [DataRow("abc", 0, "")]
    [DataRow("a\nb😀", 4, "a\nb")]
    public void InputLimitsUseUtf16WithoutSplittingEmoji(string text, int limit, string expected) =>
        Assert.AreEqual(expected, WeftTextInput.LimitEdit(text, limit));

    [TestMethod]
    public void InputLimitsDropAnAlreadySplitNativeEdit() =>
        Assert.AreEqual("ab", WeftTextInput.LimitEdit("ab\ud83d", 3));

    [TestMethod]
    [DataRow(17.4, 3.0, 23.0, 5.0, 18.0)]
    [DataRow(-7.5, -10.0, 0.0, 5.0, -5.0)]
    [DataRow(11.0, 3.0, 11.0, 5.0, 8.0)]
    [DataRow(-1.0, 3.0, 11.0, 5.0, 3.0)]
    public void SliderRoundsFromItsMinimum(double raw, double min, double max, double step, double expected) =>
        Assert.AreEqual(expected, WeftSlider.RoundValue(raw, min, max, step));
}
