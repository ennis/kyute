use kyute_common::Color;
use kyute_shell::text::{Attribute, FormattedText};
use std::{error::Error, fmt::Display, str::FromStr};

/// Validation result.
pub enum ValidationResult {
    /// The input is valid.
    Valid,
    /// The input is invalid.
    Invalid,
    /// The input is invalid as-is, but possibly because the user hasn't finished inputting the value.
    Incomplete,
}

/// Format, validates, and parses text input.
pub trait Formatter<T> {
    /// Formats the given value.
    fn format(&self, value: &T) -> FormattedText;

    /// Formats the given partial input.
    fn format_partial_input(&self, text: &str) -> FormattedText;

    /// Validates the given input.
    fn validate_partial_input(&self, text: &str) -> ValidationResult;

    /// Parses the given input.
    fn parse(&self, text: &str) -> Result<T, anyhow::Error>;
}

/// Formatter using the `FromStr` and `Display` traits.
pub struct DisplayFormatter;

impl<T> Formatter<T> for DisplayFormatter
where
    T: Display + FromStr,
    <T as FromStr>::Err: Error + Send + Sync + 'static,
{
    fn format(&self, value: &T) -> FormattedText {
        format!("{}", value).into()
    }

    fn format_partial_input(&self, text: &str) -> FormattedText {
        match text.parse::<T>() {
            Ok(_) => text.into(),
            Err(_) => {
                // highlight in red if not a valid number
                FormattedText::from(text).attribute(.., Attribute::Color(Color::from_hex("#DC143C")))
            }
        }
    }

    fn validate_partial_input(&self, text: &str) -> ValidationResult {
        match text.parse::<T>() {
            Ok(_) => ValidationResult::Valid,
            Err(_) => ValidationResult::Invalid,
        }
    }

    fn parse(&self, text: &str) -> Result<T, anyhow::Error> {
        Ok(text.parse::<T>()?)
    }
}

/// Formatter for floating-point values.
pub struct FloatingPointNumberFormatter {
    precision: usize,
}

impl FloatingPointNumberFormatter {
    /// Creates a new instance of this formatter.
    ///
    /// # Arguments
    /// * precision the maximum number of digits to print after the dot
    pub fn new(precision: usize) -> FloatingPointNumberFormatter {
        FloatingPointNumberFormatter { precision }
    }
}

macro_rules! impl_float_formatter {
    ($t:ty) => {
        impl Formatter<$t> for FloatingPointNumberFormatter {
            fn format(&self, value: &$t) -> FormattedText {
                format!("{:.*}", self.precision, value).into()
            }

            fn format_partial_input(&self, text: &str) -> FormattedText {
                match text.parse::<$t>() {
                    Ok(_) => text.into(),
                    Err(_) => {
                        // highlight in red if not a valid number
                        // TODO this probably shouldn't be done by the formatter
                        FormattedText::from(text).attribute(.., Attribute::Color(Color::from_hex("#DC143C")))
                    }
                }
            }

            fn validate_partial_input(&self, text: &str) -> ValidationResult {
                match text.parse::<$t>() {
                    Ok(_) => ValidationResult::Valid,
                    Err(_) => ValidationResult::Invalid,
                }
            }

            fn parse(&self, text: &str) -> Result<$t, anyhow::Error> {
                Ok(text.parse::<$t>()?)
            }
        }
    };
}

impl_float_formatter!(f32);
impl_float_formatter!(f64);
