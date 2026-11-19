using System.Buffers.Binary;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Weft.WinUI;

namespace Weft.WinUI.Tests;

[TestClass]
public sealed class CorpusTests
{
    [TestMethod]
    public void DecodesOpsAndReportsErrorKinds()
    {
        using var corpus = Load("ops.json");
        foreach (var row in corpus.RootElement.GetProperty("cases").EnumerateArray())
        {
            var name = Name(row);
            var bytes = Hex(row);
            if (row.TryGetProperty("error", out var kind))
            {
                var error = Assert.ThrowsExactly<OpException>(() => ReadBatch(bytes), name);
                Assert.AreEqual(kind.GetString(), error.Kind, name);
                continue;
            }

            var reader = new OpReader(bytes);
            foreach (var expected in row.GetProperty("ops").EnumerateArray())
            {
                Assert.IsTrue(reader.Next(out var op), name);
                AssertOp(ref reader, op, expected, name);
            }
            Assert.IsFalse(reader.Next(out _), name);
        }
    }

    [TestMethod]
    public void EncodesValues()
    {
        using var corpus = Load("values.json");
        foreach (var row in corpus.RootElement.GetProperty("cases").EnumerateArray())
        {
            var actual = new List<byte>();
            Value(row.GetProperty("value")).Encode(actual);
            Assert.IsTrue(Hex(row).SequenceEqual(actual), Name(row));
        }
    }

    [TestMethod]
    public void UnframesRequests()
    {
        using var corpus = Load("frames.json");
        foreach (var row in corpus.RootElement.GetProperty("requests").EnumerateArray())
        {
            var (head, body) = PlatformModules.Unframe(Hex(row));
            AssertFrame(row, head, body);
        }
    }

    [TestMethod]
    public void FramesResponses()
    {
        using var corpus = Load("frames.json");
        foreach (var row in corpus.RootElement.GetProperty("responses").EnumerateArray())
        {
            var encoded = PlatformModules.Frame(Head(row).AsObject(), Body(row)).AsSpan();
            Assert.IsTrue(encoded.Length >= 4, Name(row));
            var headLength = BinaryPrimitives.ReadUInt32LittleEndian(encoded);
            Assert.IsTrue(headLength <= (uint)(encoded.Length - 4), Name(row));
            var head = JsonNode.Parse(encoded.Slice(4, (int)headLength));
            AssertFrame(row, head, encoded[(4 + (int)headLength)..]);
        }
    }

    [TestMethod]
    public void RejectsMalformedFrames()
    {
        using var corpus = Load("frames.json");
        foreach (var row in corpus.RootElement.GetProperty("malformed").EnumerateArray())
        {
            var bytes = Hex(row);
            Assert.ThrowsExactly<InvalidOperationException>(() => PlatformModules.Unframe(bytes), Name(row));
        }
    }

    private static JsonDocument Load(string file) =>
        JsonDocument.Parse(File.ReadAllBytes(Path.Combine(AppContext.BaseDirectory, "corpus", file)));

    private static string Name(JsonElement row) => row.GetProperty("name").GetString()!;

    private static byte[] Hex(JsonElement row) =>
        Convert.FromHexString(string.Concat(row.GetProperty("hex").GetString()!.Where(c => !char.IsWhiteSpace(c))));

    private static JsonNode Head(JsonElement row) => JsonNode.Parse(row.GetProperty("head").GetRawText())!;

    private static byte[] Body(JsonElement row) => Encoding.UTF8.GetBytes(row.GetProperty("body").GetString()!);

    private static void AssertFrame(JsonElement row, JsonNode? head, ReadOnlySpan<byte> body)
    {
        Assert.IsTrue(JsonNode.DeepEquals(Head(row), head), Name(row));
        Assert.IsTrue(body.SequenceEqual(Body(row)), Name(row));
    }

    private static WeftValue Value(JsonElement value)
    {
        if (value.TryGetProperty("bool", out var boolean)) return WeftValue.Of(boolean.GetBoolean());
        if (value.TryGetProperty("number", out var number)) return WeftValue.Of(number.GetDouble());
        return WeftValue.Of(value.GetProperty("string").GetString()!);
    }

    private static void ReadBatch(byte[] bytes)
    {
        var reader = new OpReader(bytes);
        while (reader.Next(out _)) { }
    }

    private static void AssertOp(ref OpReader reader, in Op op, JsonElement expected, string name)
    {
        var code = expected.GetProperty("op").GetString() switch
        {
            "create" => OpCode.Create,
            "set" => OpCode.Set,
            "branch" => OpCode.Branch,
            "rows" => OpCode.Rows,
            "attach" => OpCode.Attach,
            var unknown => throw new InvalidOperationException($"unknown corpus op {unknown}"),
        };
        Assert.AreEqual(code, op.Code, name);
        Assert.AreEqual(expected.GetProperty("scope").GetUInt32(), op.Scope, name);
        switch (code)
        {
            case OpCode.Create:
                Assert.AreEqual(expected.GetProperty("template").GetUInt32(), op.Arg, name);
                break;
            case OpCode.Set:
                Assert.AreEqual(expected.GetProperty("slot").GetUInt32(), op.Arg, name);
                Assert.AreEqual(Value(expected.GetProperty("value")), op.Value, name);
                break;
            case OpCode.Branch or OpCode.Attach:
                Assert.AreEqual(expected.GetProperty("point").GetUInt32(), op.Arg, name);
                Assert.AreEqual(expected.GetProperty("child").GetUInt32(), op.Child, name);
                break;
            case OpCode.Rows:
                Assert.AreEqual(expected.GetProperty("point").GetUInt32(), op.Arg, name);
                var children = expected.GetProperty("children");
                Assert.AreEqual(children.GetArrayLength(), op.RowsCount, name);
                var i = 0;
                foreach (var child in children.EnumerateArray())
                {
                    Assert.AreEqual(child.GetUInt32(), reader.Row(op, i++), name);
                }
                break;
        }
    }
}
