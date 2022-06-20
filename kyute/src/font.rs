use crate::Length;
use kyute_shell::text::{FontStyle, FontWeight};

#[derive(Clone, Debug)]
pub struct Font {
    pub family: String,
    pub weight: FontWeight,
    pub size: Length,
    pub style: FontStyle,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            family: "".to_string(),
            weight: Default::default(),
            size: 14.pt(),
            style: Default::default(),
        }
    }
}

impl Font {
    pub fn new(family: impl Into<String>, size: Length) -> Font {
        Font {
            family: family.into(),
            weight: FontWeight::NORMAL,
            size,
            style: Default::default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.weight = FontWeight::BOLD;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style = FontStyle::Italic;
        self
    }
}

impl_env_value!(Font);
