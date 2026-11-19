using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;
using System.Text;
using Windows.UI;

namespace Weft.WinUI;

/// <summary>A value of a value slot or an event payload (docs/src/content/docs/protocol.md, "Binary encoding").</summary>
public readonly struct WeftValue : IEquatable<WeftValue>
{
    private enum Kind : byte { Bool, Number, String }

    private readonly Kind kind;
    private readonly bool boolean;
    private readonly double number;
    private readonly string? text;

    private WeftValue(Kind kind, bool boolean = false, double number = 0, string? text = null)
    {
        this.kind = kind;
        this.boolean = boolean;
        this.number = number;
        this.text = text;
    }

    public static WeftValue Of(bool value) => new(Kind.Bool, boolean: value);

    public static WeftValue Of(double value) => new(Kind.Number, number: value);

    public static WeftValue Of(string value) => new(Kind.String, text: value);

    /// <summary>The value of a <c>Bool</c> slot.</summary>
    public bool Bool => kind == Kind.Bool && boolean;

    /// <summary>The value of an <c>Int</c> slot: a length in DIPs.</summary>
    public double Number => kind == Kind.Number ? number : 0;

    /// <summary>The value of a <c>String</c> slot.</summary>
    public string String => kind == Kind.String ? text! : "";

    /// <summary>The value of a <c>Color</c> slot; invalid colors are transparent.</summary>
    public Color Color
    {
        get
        {
            if (WeftColor.TryParse(String, out var color)) return color;
#if DEBUG
            WeftLog.Write(Native.LogWarn, $"weft: invalid color {String}, using transparent");
#endif
            return default;
        }
    }

    /// <summary>Appends the binary encoding: a tag byte and the data.</summary>
    internal void Encode(List<byte> bytes)
    {
        switch (kind)
        {
            case Kind.Bool:
                bytes.Add(boolean ? (byte)1 : (byte)0);
                break;
            case Kind.Number:
                bytes.Add(2);
                Span<byte> f64 = stackalloc byte[8];
                BinaryPrimitives.WriteDoubleLittleEndian(f64, number);
                foreach (var b in f64) bytes.Add(b);
                break;
            default:
                bytes.Add(3);
                var utf8 = Encoding.UTF8.GetBytes(text!);
                Span<byte> length = stackalloc byte[4];
                BinaryPrimitives.WriteUInt32LittleEndian(length, (uint)utf8.Length);
                foreach (var b in length) bytes.Add(b);
                bytes.AddRange(utf8);
                break;
        }
    }

    public bool Equals(WeftValue other) =>
        kind == other.kind && boolean == other.boolean && number.Equals(other.number) && text == other.text;

    public override bool Equals(object? obj) => obj is WeftValue other && Equals(other);

    public override int GetHashCode() => HashCode.Combine(kind, boolean, number, text);

    public override string ToString() => kind switch
    {
        Kind.Bool => boolean ? "true" : "false",
        Kind.Number => number.ToString(CultureInfo.InvariantCulture),
        _ => text!,
    };
}

/// <summary>Colors of the views.</summary>
public static class WeftColor
{
    /// <summary>A constant color of the generated views, written as <c>0xRRGGBBAA</c>.</summary>
    public static Color Rgba(uint rgba) =>
        Color.FromArgb((byte)rgba, (byte)(rgba >> 24), (byte)(rgba >> 16), (byte)(rgba >> 8));

    /// <summary>Parses <c>#RGB</c>, <c>#RRGGBB</c> or <c>#RRGGBBAA</c> in either hex case.</summary>
    public static bool TryParse(string text, out Color color)
    {
        color = default;
        if (text.Length is not (4 or 7 or 9) || text[0] != '#') return false;
        var digits = text.AsSpan(1);
        foreach (var c in digits)
        {
            if (!char.IsAsciiHexDigit(c)) return false;
        }
        var value = uint.Parse(digits, NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture);
        color = digits.Length switch
        {
            3 => Rgba((value >> 8 & 0xf) * 17 << 24 | (value >> 4 & 0xf) * 17 << 16 | (value & 0xf) * 17 << 8 | 0xff),
            6 => Rgba(value << 8 | 0xff),
            _ => Rgba(value),
        };
        return true;
    }
}
