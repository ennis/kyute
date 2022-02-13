use crate::drawing::{FromSkia, ToSkia};
use palette::Shade;
use std::{error::Error, fmt, marker::PhantomData};

/// Color spec.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        (Ok(a), Ok(b)) => Ok((a << 4) + b),
        _ => Err(ColorParseError),
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::new(0.0, 0.0, 0.0, 0.0)
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

    /// Replaces alpha value.
    pub const fn with_alpha(self, alpha: f64) -> Color {
        Color(palette::Srgba {
            color: self.0.color,
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

    pub const fn to_rgba_u8(&self) -> (u8, u8, u8, u8) {
        (
            (self.0.color.red * 255.0) as u8,
            (self.0.color.green * 255.0) as u8,
            (self.0.color.blue * 255.0) as u8,
            (self.0.alpha * 255.0) as u8,
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

    /// TODO documentation
    pub fn lighten(&self, amount: f64) -> Color {
        Color(Shade::lighten(&self.0.into_linear(), amount as f32).into_encoding())
    }

    /// TODO documentation
    pub fn darken(&self, amount: f64) -> Color {
        Color(Shade::darken(&self.0.into_linear(), amount as f32).into_encoding())
    }

    pub fn to_hex(&self) -> String {
        match self.to_rgba_u8() {
            (r, g, b, 255) => {
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            }
            (r, g, b, a) => {
                format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
            }
        }
    }

    /// Creates a new color from an hex code.
    pub const fn from_hex(hex: &str) -> Color {
        match Self::try_from_hex(hex) {
            Ok(color) => color,
            Err(_) => {
                panic!("invalid hex color")
            }
        }
    }

    /// Creates a new color from an hex code.
    pub const fn try_from_hex(hex: &str) -> Result<Color, ColorParseError> {
        match hex.as_bytes() {
            // #RRGGBB, RRGGBB
            &[b'#', r0, r1, g0, g1, b0, b1] | &[r0, r1, g0, g1, b0, b1] => {
                match (
                    byte_from_ascii(r0, r1),
                    byte_from_ascii(g0, g1),
                    byte_from_ascii(b0, b1),
                ) {
                    (Ok(r), Ok(g), Ok(b)) => Ok(Color::from_rgb_u8(r, g, b)),
                    _ => Err(ColorParseError),
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
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Ok(Color::from_rgba_u8(r, g, b, a)),
                    _ => Err(ColorParseError),
                }
            }
            // #RGB, RGB
            &[b'#', r, g, b] | &[r, g, b] => {
                match (
                    nibble_from_ascii(r),
                    nibble_from_ascii(g),
                    nibble_from_ascii(b),
                ) {
                    (Ok(r), Ok(g), Ok(b)) => Ok(Color::from_rgb_u8(r, g, b)),
                    _ => Err(ColorParseError),
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
                    (Ok(r), Ok(g), Ok(b), Ok(a)) => Ok(Color::from_rgba_u8(r, g, b, a)),
                    _ => Err(ColorParseError),
                }
            }
            _ => Err(ColorParseError),
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
