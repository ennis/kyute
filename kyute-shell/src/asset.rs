use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fs::File,
    hash::{Hash, Hasher},
    io,
    marker::PhantomData,
};
use thiserror::Error;

#[derive(Copy, Clone, Debug)]
pub struct RawAssetId<'a> {
    pub uri: &'a str,
    pub data: Option<&'static [u8]>,
}

impl<'a> PartialEq for RawAssetId<'a> {
    fn eq(&self, other: &Self) -> bool {
        // the second part is not necessary if we rely on uri to uniquely identify the asset
        self.uri == other.uri /*&& self.data.map(|d| d.as_ptr()) == other.data.map(|d| d.as_ptr())*/
    }
}

impl<'a> Eq for RawAssetId<'a> {}

impl<'a> Hash for RawAssetId<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.uri, state);
    }
}

/// Statically identifies an asset.
#[derive(Debug, Eq, PartialEq)]
pub struct AssetId<T> {
    pub raw: RawAssetId<'static>,
    _type: PhantomData<T>,
}

impl<T> Clone for AssetId<T> {
    fn clone(&self) -> Self {
        AssetId {
            raw: self.raw,
            _type: PhantomData,
        }
    }
}

impl<T> Copy for AssetId<T> {}

impl<T> Hash for AssetId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.raw, state)
    }
}

impl<T> AssetId<T> {
    pub const fn new(uri: &'static str) -> AssetId<T> {
        AssetId {
            raw: RawAssetId { uri, data: None },
            _type: PhantomData,
        }
    }

    pub const fn with_data(uri: &'static str, data: &'static [u8]) -> AssetId<T> {
        AssetId {
            raw: RawAssetId {
                uri,
                data: Some(data),
            },
            _type: PhantomData,
        }
    }
}

pub trait Asset: Any + Clone {
    type LoadError: std::error::Error;

    fn load(reader: &mut dyn io::Read) -> Result<Self, Self::LoadError>;

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, Self::LoadError> {
        let mut bytes = bytes;
        Self::load(&mut bytes)
    }
}

struct Resolvers;

impl Resolvers {
    /// Resolves an asset URI to a reader
    pub fn resolve(&self, asset_id: &RawAssetId) -> io::Result<Box<dyn io::Read>> {
        if let Some(data) = asset_id.data {
            let reader = data;
            Ok(Box::new(reader))
        } else {
            // resolve from filesystem
            // TODO pluggable schemes / search paths
            let file = File::open(asset_id.uri)?;
            Ok(Box::new(file))
        }
    }
}

pub struct AssetLoader {
    cache: RefCell<HashMap<String, Box<dyn Any>>>,
    resolvers: Resolvers,
}

impl Default for AssetLoader {
    fn default() -> Self {
        AssetLoader::new()
    }
}

#[derive(Error)]
pub enum AssetLoadError<E: Error> {
    #[error("I/O error")]
    Io(io::Error),
    #[error("asset error")]
    Asset(#[from] E),
    #[error("type mismatch for cached asset")]
    CachedAssetTypeMismatch,
}

impl AssetLoader {
    pub fn new() -> AssetLoader {
        AssetLoader {
            cache: RefCell::new(Default::default()),
            resolvers: Resolvers,
        }
    }

    /// Loads an asset from a raw asset id.
    pub fn load_raw<T: Asset>(
        &self,
        raw_asset_id: RawAssetId,
    ) -> Result<T, AssetLoadError<T::LoadError>> {
        {
            let cache = self.cache.borrow();
            if let Some(entry) = cache.get(raw_asset_id.uri) {
                return Ok(entry
                    .downcast_ref::<T>()
                    .ok_or(AssetLoadError::CachedAssetTypeMismatch)?
                    .clone());
            }
        }

        let mut reader = self
            .resolvers
            .resolve(&raw_asset_id)
            .map_err(AssetLoadError::Io)?;
        // FIXME should call load_from_bytes when possible
        let value = T::load(&mut reader).map_err(AssetLoadError::Asset)?;
        self.cache
            .borrow_mut()
            .insert(raw_asset_id.uri.to_string(), Box::new(value.clone()));
        Ok(value)
    }

    /// Loads an asset.
    pub fn load<T: Asset>(&self, asset_id: AssetId<T>) -> Result<T, AssetLoadError<T::LoadError>> {
        self.load_raw(asset_id.raw)
    }

    /*pub fn load_from_file<T: Asset>(&self, path: &str) -> io::Result<T> {
        std::fs::canonicalize(path)
    }*/
}
