//! TODO: copied code from kyute-shell: at some point, move this into a shared "common types" crate
//!
use serde::Serialize;
use std::{error::Error, fmt};

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}

impl Color {
    pub fn new(red: f64, green: f64, blue: f64, alpha: f64) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn from_rgb_u8(red: u8, green: u8, blue: u8) -> Color {
        Color::new(
            (red as f64) / 255.0,
            (green as f64) / 255.0,
            (blue as f64) / 255.0,
            1.0,
        )
    }

    pub fn from_rgba_u8(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
        Color::new(
            (red as f64) / 255.0,
            (green as f64) / 255.0,
            (blue as f64) / 255.0,
            (alpha as f64) / 255.0,
        )
    }

    /// Creates a new color from an hex code.
    pub fn from_hex(hex: &str) -> Result<Color, ColorParseError> {
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
