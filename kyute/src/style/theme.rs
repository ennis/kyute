//! Sets of environment keys loaded as a group.
use crate::{EnvKey, EnvValue, Environment};
use kyute_shell::{application::Application, asset::AssetLoadError, Asset, AssetId};
use serde::de::DeserializeOwned;
use serde_json as json;
use std::{io, io::Read};
use thiserror::Error;

/// Compile time snake to kebab-case 🥙
const fn comptime_snake_to_kebab<const LEN: usize>(s: &str) -> [u8; LEN] {
    let s = s.as_bytes();
    let mut data = [0u8; LEN];
    let mut i = 0usize;
    while i < s.len() {
        data[i] = match s[i] {
            b'_' => b'-',
            ch => ch.to_ascii_lowercase(),
        };
        i += 1;
    }
    data
}

#[derive(Debug, Error)]
pub enum ThemeLoadError {
    #[error("I/O error")]
    Io(#[from] io::Error),
    #[error("asset not found")]
    AssetNotFound,
    #[error("JSON error")]
    JsonError(#[from] serde_json::Error),
    #[error("invalid JSON structure")]
    InvalidJsonStructure,
    #[error("theme property not found")]
    PropertyNotFound,
}

/// Contains JSON theme data.
#[derive(Clone)]
pub struct ThemeData {
    json: json::Value,
}

impl Asset for ThemeData {
    type LoadError = ThemeLoadError;

    fn load(reader: &mut dyn Read) -> Result<Self, Self::LoadError> {
        Ok(ThemeData {
            json: json::from_reader(reader).map_err(|e| ThemeLoadError::JsonError(e))?,
        })
    }
}

impl ThemeData {
    /// Loads theme data from a JSON resource.
    pub fn load(id: AssetId<ThemeData>) -> Result<ThemeData, AssetLoadError<ThemeLoadError>> {
        let application = Application::instance();
        let asset_loader = application.asset_loader();
        asset_loader.load(id)
    }

    /// Reads a named property from the theme into the specified environment.
    pub fn load_property<T: DeserializeOwned + EnvValue>(
        &self,
        key: EnvKey<T>,
        env: &mut Environment,
    ) -> Result<(), ThemeLoadError> {
        let map = self
            .json
            .as_object()
            .ok_or(ThemeLoadError::InvalidJsonStructure)?;
        let prop = map
            .get(key.name())
            .ok_or(ThemeLoadError::PropertyNotFound)?;
        let value = serde_json::from_value(prop.clone())?;
        env.set(key, value);
        Ok(())
    }
}

#[macro_export]
macro_rules! define_theme {
    (
        $(#[$outer_meta:meta])*
        $v:vis $theme:ident [ $theme_name:literal ] {
            $( $(#[$inner_meta:meta])* const $key:ident : $t:ty ; )*
        }
    ) => {
        $(#[$outer_meta])*
        $v mod $theme {
            use super::*;

            const fn from_utf8(v: &[u8]) -> &str {
                match std::str::from_utf8(v) {
                    Ok(str) => str,
                    Err(e) => {
                        panic!("from_utf8 failed")
                    }
                }
            }

            $( $(#[$inner_meta])* $v const $key : $crate::EnvKey<$t> = $crate::EnvKey::new(from_utf8(&$crate::style::theme::comptime_snake_to_kebab::<{::std::concat!($theme_name, ".", ::std::stringify!($key)).len()}>(::std::concat!($theme_name, ".", ::std::stringify!($key))))); )*

            pub fn load(env: &mut $crate::Environment, id: $crate::shell::AssetId<ThemeData>) -> Result<(), $crate::shell::AssetLoadError<$crate::style::ThemeLoadError>> {
                let data = $crate::style::ThemeData::load(id)?;
                $(data.load_property($key, env);)*
                Ok(())
            }
        }
    };
}

pub use define_theme;
