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

/// <summary>Why a batch of ops could not be decoded.</summary>
internal sealed class OpException(string message) : Exception(message);

/// <summary>Decodes one batch of ops in the binary encoding (version 1).</summary>
internal ref struct OpReader
{
    /// <summary>The encoding version this reader accepts.</summary>
    public const byte Version = 1;

    private readonly ReadOnlySpan<byte> bytes;
    private int offset;

    public OpReader(ReadOnlySpan<byte> bytes)
    {
        if (bytes.IsEmpty) throw new OpException("truncated batch");
        if (bytes[0] != Version) throw new OpException($"unknown encoding version {bytes[0]}");
        this.bytes = bytes;
        offset = 1;
    }

    /// <summary>Decodes the next op; false at the end of the batch.</summary>
    public bool Next(out Op op)
    {
        op = default;
        if (offset >= bytes.Length) return false;
        var code = (OpCode)Byte();
        var scope = U32();
        switch (code)
        {
            case OpCode.Create:
                op = new Op(code, scope, U32(), 0, default, 0, 0);
                break;
            case OpCode.Set:
                var slot = U32();
                op = new Op(code, scope, slot, 0, Value(), 0, 0);
                break;
            case OpCode.Branch or OpCode.Attach:
                var point = U32();
                op = new Op(code, scope, point, U32(), default, 0, 0);
                break;
            case OpCode.Rows:
                var list = U32();
                var count = U32();
                if (count > (uint)(bytes.Length - offset) / 4) throw new OpException("truncated rows");
                op = new Op(code, scope, list, 0, default, offset, (int)count);
                offset += (int)count * 4;
                break;
            default:
                throw new OpException($"unknown opcode {(byte)code}");
        }
        return true;
    }

    /// <summary>Child <paramref name="index"/> of a <c>rows</c> op.</summary>
    public readonly uint Row(in Op op, int index) =>
        BinaryPrimitives.ReadUInt32LittleEndian(bytes.Slice(op.RowsOffset + index * 4, 4));

    private ReadOnlySpan<byte> Take(int count)
    {
        if (count > bytes.Length - offset) throw new OpException("truncated batch");
        var span = bytes.Slice(offset, count);
        offset += count;
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
            3 => WeftValue.Of(Encoding.UTF8.GetString(Take(checked((int)U32())))),
            _ => throw new OpException($"unknown value tag {tag}"),
        };
    }
}
