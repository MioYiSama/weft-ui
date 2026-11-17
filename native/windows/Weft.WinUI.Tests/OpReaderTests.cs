using Weft.WinUI;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class OpReaderTests
{
    private static byte[] U32(uint value) => BitConverter.GetBytes(value);

    private static byte[] Encode(WeftValue value)
    {
        var bytes = new List<byte>();
        value.Encode(bytes);
        return [.. bytes];
    }

    private static List<string> Decode(byte[] bytes)
    {
        var ops = new List<string>();
        var reader = new OpReader(bytes);
        while (reader.Next(out var op))
        {
            var rows = new uint[op.RowsCount];
            for (var i = 0; i < rows.Length; i++) rows[i] = reader.Row(op, i);
            ops.Add(op.Code switch
            {
                OpCode.Create => $"create {op.Scope} {op.Arg}",
                OpCode.Set => $"set {op.Scope} {op.Arg} {op.Value}",
                OpCode.Rows => $"rows {op.Scope} {op.Arg} [{string.Join(" ", rows)}]",
                _ => $"{op.Code.ToString().ToLowerInvariant()} {op.Scope} {op.Arg} {op.Child}",
            });
        }
        return ops;
    }

    [TestMethod]
    public void DecodesEveryOpAndValue()
    {
        byte[] bytes =
        [
            1,
            1, .. U32(3), .. U32(2),
            2, .. U32(3), .. U32(0), .. Encode(WeftValue.Of("mïlk")),
            2, .. U32(3), .. U32(1), .. Encode(WeftValue.Of(-2.5)),
            2, .. U32(3), .. U32(2), 1,
            4, .. U32(2), .. U32(0), .. U32(2), .. U32(3), .. U32(4),
            3, .. U32(1), .. U32(0), .. U32(0),
            5, .. U32(0), .. U32(0), .. U32(1),
        ];
        CollectionAssert.AreEqual(
            new[] { "create 3 2", "set 3 0 mïlk", "set 3 1 -2.5", "set 3 2 true", "rows 2 0 [3 4]", "branch 1 0 0", "attach 0 0 1" },
            Decode(bytes));
    }

    [TestMethod]
    public void RejectsMalformedBatches()
    {
        Assert.ThrowsExactly<OpException>(() => Decode([2]));
        Assert.ThrowsExactly<OpException>(() => Decode([1, 1, .. U32(3)]));
        // A row count larger than the rest of the batch.
        Assert.ThrowsExactly<OpException>(() => Decode([1, 4, .. U32(2), .. U32(0), .. U32(1000)]));
        Assert.ThrowsExactly<OpException>(() => Decode([1, 9, .. U32(0)]));
        Assert.ThrowsExactly<OpException>(() => Decode([1, 2, .. U32(1), .. U32(0), 7]));
    }

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
