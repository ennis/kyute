use crate::drawing::{FromSkia, ToSkia};
use std::{error::Error, fmt, marker::PhantomData};

/// Color spec.
#[derive(Copy, Clone, Debug)]
pub struct Color(palette::Srgba);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ColorParseError;

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid hex color string")
    }
}

impl Error for ColorParseError {}

// taken from druid
const fn nibble_from_ascii(b: u8) -> Result<u8, ColorParseError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(ColorParseError),
    }
}

const fn byte_from_ascii(b0: u8, b1: u8) -> Result<u8, ColorParseError> {
    match (nibble_from_ascii(b0), nibble_from_ascii(b1)) {
        (Ok(b0), Ok(b1)) => Ok(b0 << 4 + b1),
        _ => Err(ColorParseError),
    }
}

impl Color {
    /// Creates a new color from RGBA values.
    pub const fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Color {
        Color(palette::Srgba {
            color: palette::Srgb {
                red: red as f32,
                green: green as f32,
                blue: blue as f32,
                standard: PhantomData,
            },
            alpha: alpha as f32,
        })
    }

    /// TODO documentation
    pub const fn from_rgb_u8(red: u8, green: u8, blue: u8) -> Color {
        Color::new(
            (red as f64) / 255.0,
            (green as f64) / 255.0,
            (blue as f64) / 255.0,
            1.0,
        )
    }

    /// TODO documentation
    pub const fn from_rgba_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color::new(
            (red as f64) / 255.0,
            (green as f64) / 255.0,
            (blue as f64) / 255.0,
            (alpha as f64) / 255.0,
        )
    }

    /// Creates a new color from an hex code.
    ///
    /// Panics if invalid.
    pub const fn from_hex(hex: &str) -> Color {
        match hex.as_bytes() {
            // #RRGGBB, RRGGBB
            &[b'#', r0, r1, g0, g1, b0, b1] | &[r0, r1, g0, g1, b0, b1] => {
                match (
                    byte_from_ascii(r0, r1),
                    byte_from_ascii(g0, g1),
                    byte_from_ascii(b0, b1),
                ) {
                    (Ok(r), Ok(g), Ok(b)) => Color::from_rgb_u8(r, g, b),
                    _ => panic!("invalid hex color"),
                }
            }
            // #RRGGBBAA, RRGGBBAA
            &[b'#', r0, r1, g0, g1, b0, b1, a0, a1] | &[r0, r1, g0, g1, b0, b1, a0, a1] => {
                match (
                    byte_from_ascii(r0, r1),
                    byte_from_ascii(g0, g1),
                    byte_from_ascii(b0, b1),
                    byte_from_ascii(a0, a1),
                ) {
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Color::from_rgba_u8(r, g, b, a),
                    _ => panic!("invalid hex color"),
                }
            }
            // #RGB, RGB
            &[b'#', r, g, b] | &[r, g, b] => {
                match (
                    nibble_from_ascii(r),
                    nibble_from_ascii(g),
                    nibble_from_ascii(b),
                ) {
                    (Ok(r), Ok(g), Ok(b)) => Color::from_rgb_u8(r, g, b),
                    _ => panic!("invalid hex color"),
                }
            }
            // #RGBA, RGBA
            &[b'#', r, g, b, a] | &[r, g, b, a] => {
                match (
                    nibble_from_ascii(r),
                    nibble_from_ascii(g),
                    nibble_from_ascii(b),
                    nibble_from_ascii(a),
                ) {
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Color::from_rgba_u8(r, g, b, a),
                    _ => panic!("invalid hex color"),
                }
            }
            _ => panic!("invalid hex color"),
        }
    }
}

impl ToSkia for Color {
    type Target = skia_safe::Color4f;

    fn to_skia(&self) -> Self::Target {
        skia_safe::Color4f {
            r: self.0.red,
            g: self.0.green,
            b: self.0.blue,
            a: self.0.alpha,
        }
    }

}

impl FromSkia for Color {
    type Source = skia_safe::Color4f;

    fn from_skia(value: Self::Source) -> Self {
        Color(palette::Srgba {
            color: palette::Srgb {
                red: value.r,
                green: value.g,
                blue: value.b,
                standard: PhantomData,
            },
            alpha: value.a,
        })
    }
}
