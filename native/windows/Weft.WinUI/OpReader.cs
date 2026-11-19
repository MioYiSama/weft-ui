using System;
using System.Buffers.Binary;
using System.Text;

namespace Weft.WinUI;

/// <summary>Opcodes of the view protocol (docs/src/content/docs/protocol.md, "Ops").</summary>
internal enum OpCode : byte
{
    Create = 1,
    Set = 2,
    Branch = 3,
    Rows = 4,
    Attach = 5,
}

/// <summary>
/// One decoded op. <see cref="Arg"/> is the field after the scope: the
/// template (create), slot (set) or point (branch / rows / attach);
/// <see cref="Child"/> is the child of branch / attach. The children of
/// <c>rows</c> stay in the batch: read them with <see cref="OpReader.Row"/>.
/// </summary>
internal readonly record struct Op(OpCode Code, uint Scope, uint Arg, uint Child, WeftValue Value, int RowsOffset, int RowsCount);

/// <summary>Why a batch of ops could not be decoded or applied.</summary>
internal sealed class OpException(string message, string? kind = null) : Exception(message)
{
    /// <summary>The protocol error kind for decoding failures; null for application failures.</summary>
    public string? Kind { get; } = kind;
}

/// <summary>Decodes one batch of ops in the binary encoding (version 1).</summary>
internal ref struct OpReader
{
    /// <summary>The encoding version this reader accepts.</summary>
    public const byte Version = 1;

    private static readonly UTF8Encoding StrictUtf8 = new(false, true);

    private readonly ReadOnlySpan<byte> bytes;
    private int offset;

    public OpReader(ReadOnlySpan<byte> bytes)
    {
        if (bytes.IsEmpty) throw new OpException("truncated batch", "truncated");
        if (bytes[0] != Version) throw new OpException($"unknown encoding version {bytes[0]}", "version");
        this.bytes = bytes;
        offset = 1;
    }

    /// <summary>Decodes the next op; false at the end of the batch.</summary>
    public bool Next(out Op op)
    {
        op = default;
        if (offset >= bytes.Length) return false;
        var code = (OpCode)Byte();
        switch (code)
        {
            case OpCode.Create:
                op = new Op(code, U32(), U32(), 0, default, 0, 0);
                break;
            case OpCode.Set:
                op = new Op(code, U32(), U32(), 0, Value(), 0, 0);
                break;
            case OpCode.Branch or OpCode.Attach:
                op = new Op(code, U32(), U32(), U32(), default, 0, 0);
                break;
            case OpCode.Rows:
                var scope = U32();
                var list = U32();
                var count = U32();
                if (count > (uint)(bytes.Length - offset) / 4) throw new OpException("truncated rows", "truncated");
                op = new Op(code, scope, list, 0, default, offset, (int)count);
                offset += (int)count * 4;
                break;
            default:
                throw new OpException($"unknown opcode {(byte)code}", "opcode");
        }
        return true;
    }

    /// <summary>Child <paramref name="index"/> of a <c>rows</c> op.</summary>
    public readonly uint Row(in Op op, int index) =>
        BinaryPrimitives.ReadUInt32LittleEndian(bytes.Slice(op.RowsOffset + index * 4, 4));

    private ReadOnlySpan<byte> Take(uint count)
    {
        if (count > (uint)(bytes.Length - offset)) throw new OpException("truncated batch", "truncated");
        var span = bytes.Slice(offset, (int)count);
        offset += (int)count;
        return span;
    }

    private byte Byte() => Take(1)[0];

    private uint U32() => BinaryPrimitives.ReadUInt32LittleEndian(Take(4));

    private WeftValue Value()
    {
        var tag = Byte();
        return tag switch
        {
            0 => WeftValue.Of(false),
            1 => WeftValue.Of(true),
            2 => WeftValue.Of(BinaryPrimitives.ReadDoubleLittleEndian(Take(8))),
            3 => ReadString(),
            _ => throw new OpException($"unknown value tag {tag}", "tag"),
        };
    }

    private WeftValue ReadString()
    {
        var length = U32();
        var utf8 = Take(length);
        try
        {
            return WeftValue.Of(StrictUtf8.GetString(utf8));
        }
        catch (DecoderFallbackException error)
        {
            throw new OpException(error.Message, "utf8");
        }
    }
}
