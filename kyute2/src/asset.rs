use std::{
    error::Error,
    fmt,
    fs::File,
    future::Future,
    hash::{Hash, Hasher},
    io,
    marker::PhantomData,
    sync::Arc,
};
use thiserror::Error;

/// URI for an asset.
///
/// Could be a path on the filesystem, an identifier inside a resource bundle, or a pointer to static data.
#[derive(Copy, Clone, Debug)]
pub struct AssetUri<'a> {
    pub uri: &'a str,
    pub data: Option<&'static [u8]>,
}

impl<'a> PartialEq for AssetUri<'a> {
    fn eq(&self, other: &Self) -> bool {
        // the second part is not necessary if we rely on uri to uniquely identify the asset
        self.uri == other.uri /*&& self.data.map(|d| d.as_ptr()) == other.data.map(|d| d.as_ptr())*/
    }
}

impl<'a> Eq for AssetUri<'a> {}

impl<'a> Hash for AssetUri<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.uri, state);
    }
}

/// Statically identifies an asset.
///
/// TODO currently unused, but eventually will be generated by some kind of resource compiler / bundler.
#[derive(Debug, Eq)]
pub struct AssetId<T> {
    pub raw: AssetUri<'static>,
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

impl<T> PartialEq for AssetId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw.eq(&other.raw)
    }
}

impl<T> Hash for AssetId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.raw, state)
    }
}

impl<T> AssetId<T> {
    pub const fn new(uri: &'static str) -> AssetId<T> {
        AssetId {
            raw: AssetUri { uri, data: None },
            _type: PhantomData,
        }
    }

    pub const fn with_data(uri: &'static str, data: &'static [u8]) -> AssetId<T> {
        AssetId {
            raw: AssetUri { uri, data: Some(data) },
            _type: PhantomData,
        }
    }
}

/// Trait for objects that can be loaded by an `AssetLoader`.
pub trait Asset: Sized + Send {
    type LoadError: std::error::Error + Send + 'static;

    fn load(reader: &mut dyn io::Read) -> Result<Self, Self::LoadError>;

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, Self::LoadError> {
        let mut bytes = bytes;
        Self::load(&mut bytes)
    }
}

/// In charge of resolving paths.
///
/// Right now it only handles filesystem paths.
struct Resolvers;

impl Resolvers {
    /// Resolves an asset URI to a reader
    fn open(&self, uri: &str) -> io::Result<Box<dyn io::Read>> {
        // resolve from filesystem
        // TODO pluggable schemes / search paths
        let file = File::open(uri)?;
        Ok(Box::new(file))
    }
}

//pub(crate) const ASSET_LOADER: EnvKey<AssetLoader> = builtin_env_key!("kyute.asset-loader");

/// Object responsible for resolving asset URIs to streams of data.
#[derive(Clone)]
pub struct AssetLoader {
    resolvers: Arc<Resolvers>,
}

//impl_env_value!(AssetLoader);

#[derive(Debug, Error)]
pub enum AssetLoadError<E: Error + fmt::Debug> {
    #[error("I/O error")]
    Io(io::Error),
    #[error("asset error")]
    Asset(#[from] E),
}

impl AssetLoader {
    /// Creates a new `AssetLoader`.
    ///
    /// Note that one is created by default in the default environment.
    pub fn new() -> AssetLoader {
        AssetLoader {
            resolvers: Arc::new(Resolvers),
        }
    }

    /// Returns the `AssetLoader` instance from the current environment.
    pub fn instance() -> AssetLoader {
        todo!()
        /*cache::environment()
        .get(&ASSET_LOADER)
        .expect("could not find an AssetLoader instance in the current environment")*/
    }

    /// Loads an asset from an URI.
    pub fn load<T: Asset>(&self, uri: &str) -> Result<T, AssetLoadError<T::LoadError>> {
        // open reader
        let mut reader = self.resolvers.open(uri).map_err(AssetLoadError::Io)?;
        // FIXME should call load_from_bytes when possible
        // load the asset from the reader
        let value = T::load(&mut reader).map_err(AssetLoadError::Asset)?;
        Ok(value)
    }

    /*/// Loads an asset asynchronously.
    pub fn load_async<T: Asset + 'static>(
        &self,
        uri: &str,
    ) -> impl Future<Output = Result<T, AssetLoadError<T::LoadError>>> {
        let resolvers = self.resolvers.clone();
        let uri = uri.to_string();
        async move {
            task::spawn_blocking(move || {
                let mut reader = resolvers.open(&uri).map_err(AssetLoadError::Io)?;
                let value = T::load(&mut reader).map_err(AssetLoadError::Asset)?;
                Ok(value)
            })
            .await
            .expect("failed to await")
        }
    }*/
}

impl Default for AssetLoader {
    fn default() -> Self {
        Self::new()
    }
}
