using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
using Weft.WinUI.Protocol;

namespace Weft.WinUI;

/// <summary>A value of a value slot or an event payload (protocol/SPEC.md, "Binary encoding").</summary>
public readonly struct WeftValue : IEquatable<WeftValue>
{
    private enum Kind : byte { Bool, Number, String, Strings }

    private readonly Kind kind;
    private readonly bool boolean;
    private readonly double number;
    // Both reference-backed kinds share a field so scalar slots stay the same size.
    private readonly object? reference;

    private WeftValue(Kind kind, bool boolean = false, double number = 0, object? reference = null)
    {
        this.kind = kind;
        this.boolean = boolean;
        this.number = number;
        this.reference = reference;
    }

    public static WeftValue Of(bool value) => new(Kind.Bool, boolean: value);

    public static WeftValue Of(double value) => new(Kind.Number, number: value);

    public static WeftValue Of(string value) => new(Kind.String, reference: value);

    internal static WeftValue Of(IReadOnlyList<string> value) => new(Kind.Strings, reference: value);

    /// <summary>The value of a <c>Bool</c> slot.</summary>
    public bool Bool => kind == Kind.Bool && boolean;

    /// <summary>The value of an <c>Int</c> slot: a length in DIPs.</summary>
    public double Number => kind == Kind.Number ? number : 0;

    /// <summary>The value of a <c>String</c> slot.</summary>
    public string String => kind == Kind.String ? (string)reference! : "";

    /// <summary>The value of a <c>Strings</c> slot.</summary>
    public IReadOnlyList<string> Strings => kind == Kind.Strings ? (IReadOnlyList<string>)reference! : Array.Empty<string>();

    /// <summary>The value of a <c>Color</c> slot; invalid colors are transparent.</summary>
    public WeftColor Color => WeftColor.Parse(String);

    /// <summary>Appends the binary encoding of an event payload: a tag byte and the data.</summary>
    internal void Encode(List<byte> bytes)
    {
        switch (kind)
        {
            case Kind.Bool:
                bytes.Add((byte)(boolean ? ValueTag.True : ValueTag.False));
                break;
            case Kind.Number:
                bytes.Add((byte)ValueTag.Number);
                Span<byte> f64 = stackalloc byte[8];
                BinaryPrimitives.WriteDoubleLittleEndian(f64, number);
                foreach (var b in f64) bytes.Add(b);
                break;
            case Kind.String:
                bytes.Add((byte)ValueTag.String);
                var utf8 = Encoding.UTF8.GetBytes(String);
                Span<byte> length = stackalloc byte[4];
                BinaryPrimitives.WriteUInt32LittleEndian(length, (uint)utf8.Length);
                foreach (var b in length) bytes.Add(b);
                bytes.AddRange(utf8);
                break;
            case Kind.Strings:
                throw new InvalidOperationException("weft: string lists are not event payloads");
        }
    }

    public bool Equals(WeftValue other)
    {
        if (kind != other.kind) return false;
        return kind switch
        {
            Kind.Bool => boolean == other.boolean,
            Kind.Number => number.Equals(other.number),
            Kind.String => String == other.String,
            Kind.Strings => EqualStrings(Strings, other.Strings),
            _ => false,
        };
    }

    private static bool EqualStrings(IReadOnlyList<string> a, IReadOnlyList<string> b)
    {
        if (ReferenceEquals(a, b)) return true;
        if (a.Count != b.Count) return false;
        for (var i = 0; i < a.Count; i++)
            if (a[i] != b[i]) return false;
        return true;
    }

    public override bool Equals(object? obj) => obj is WeftValue other && Equals(other);

    public override int GetHashCode()
    {
        if (kind != Kind.Strings) return HashCode.Combine(kind, boolean, number, reference);
        var hash = new HashCode();
        hash.Add(kind);
        var strings = Strings;
        for (var i = 0; i < strings.Count; i++) hash.Add(strings[i]);
        return hash.ToHashCode();
    }

    public override string ToString() => kind switch
    {
        Kind.Bool => boolean ? "true" : "false",
        Kind.Number => number.ToString(CultureInfo.InvariantCulture),
        Kind.Strings => $"[{string.Join(", ", Strings)}]",
        _ => String,
    };
}

