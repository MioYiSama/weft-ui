//! Hexadecimal RGBA colors shared by semantic analysis and view backends.

use std::fmt;

/// An RGBA color with eight-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Parses `#RGB`, `#RRGGBB`, or `#RRGGBBAA`, accepting either hex case.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        fn digit(byte: u8) -> Option<u8> {
            match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                b'A'..=b'F' => Some(byte - b'A' + 10),
                _ => None,
            }
        }
        fn channel(hi: u8, lo: u8) -> Option<u8> {
            Some(digit(hi)? * 16 + digit(lo)?)
        }
        let (r, g, b, a) = match value.as_bytes() {
            [b'#', r, g, b] => (digit(*r)? * 17, digit(*g)? * 17, digit(*b)? * 17, 255),
            [b'#', r1, r2, g1, g2, b1, b2] => (
                channel(*r1, *r2)?,
                channel(*g1, *g2)?,
                channel(*b1, *b2)?,
                255,
            ),
            [b'#', r1, r2, g1, g2, b1, b2, a1, a2] => (
                channel(*r1, *r2)?,
                channel(*g1, *g2)?,
                channel(*b1, *b2)?,
                channel(*a1, *a2)?,
            ),
            _ => return None,
        };
        Some(Self { r, g, b, a })
    }

    /// Theme tables always carry all four RGBA channels, including opaque alpha.
    pub(crate) fn rgba_hex(self) -> impl fmt::Display {
        fmt::from_fn(move |out| {
            write!(
                out,
                "#{:02x}{:02x}{:02x}{:02x}",
                self.r, self.g, self.b, self.a
            )
        })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)?;
        if self.a != 255 {
            write!(f, "{:02x}", self.a)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn parse_and_format() {
        for (input, channels, normalized) in [
            ("#000", [0, 0, 0, 255], "#000000"),
            ("#fAf", [255, 170, 255, 255], "#ffaaff"),
            ("#01aBfF", [1, 171, 255, 255], "#01abff"),
            ("#123456FF", [18, 52, 86, 255], "#123456"),
            ("#ABCDEF00", [171, 205, 239, 0], "#abcdef00"),
            ("#12345680", [18, 52, 86, 128], "#12345680"),
        ] {
            let color = Color::parse(input).expect("valid hex color");
            assert_eq!([color.r, color.g, color.b, color.a], channels, "{input}");
            assert_eq!(color.to_string(), normalized, "{input}");
            assert_eq!(Color::parse(normalized), Some(color), "{input}");
        }
    }

    #[test]
    fn rejects_other_formats() {
        for input in [
            "",
            "#",
            "fff",
            "gray",
            "transparent",
            "#12",
            "#1234",
            "#12345",
            "#1234567",
            "#123456789",
            "#ggg",
            "#12345g",
            "#123456fg",
            " #fff",
            "#fff ",
            "#fff\n",
            "#１２３",
            "#éa",
        ] {
            assert_eq!(Color::parse(input), None, "{input:?}");
        }
    }
}
