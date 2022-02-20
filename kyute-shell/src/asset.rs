use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    error::Error,
    fmt, fs,
    fs::File,
    future::Future,
    hash::{Hash, Hasher},
    io,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};
use thiserror::Error;
use tokio::task;
use tracing::trace;

pub use notify::Event;

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
#[derive(Debug, Eq, PartialEq)]
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

pub trait Asset: Sized + Send {
    type LoadError: std::error::Error + Send + 'static;

    fn load(reader: &mut dyn io::Read) -> Result<Self, Self::LoadError>;

    fn load_from_bytes(bytes: &[u8]) -> Result<Self, Self::LoadError> {
        let mut bytes = bytes;
        Self::load(&mut bytes)
    }
}

#[derive(Clone)]
pub struct WatchSubscription(Arc<()>);

struct WatchHandler {
    original_path: PathBuf,
    canonical_path: PathBuf,
    callback: Box<dyn FnMut(Event) + Send>,
    subscription: Weak<()>,
}

struct Resolvers {
    watcher: Mutex<RecommendedWatcher>,
    watch_handlers: Arc<Mutex<HashMap<PathBuf, WatchHandler>>>,
}

impl Resolvers {
    fn new() -> Resolvers {
        let watch_handlers : Arc<Mutex<HashMap<PathBuf, WatchHandler>>> = Arc::new(Mutex::new(HashMap::new()));
        let watch_handlers_clone = watch_handlers.clone();

        let watcher = Mutex::new(
            notify::recommended_watcher(move |res: notify::Result<Event>| {
                match res {
                    Ok(event) => {
                        let mut handlers = watch_handlers_clone.lock().unwrap();
                        for path in event.paths.iter() {
                            if let Ok(canonical_path) = fs::canonicalize(path) {
                                // see if there's a watcher for this path
                                if let Some(entry) = handlers.get_mut(&canonical_path) {
                                    // OK, invoke callback
                                    (entry.callback)(event.clone())
                                }
                            }
                        }
                    }
                    Err(e) => tracing::error!("watch error: {}", e),
                }
            })
            .expect("failed to create filesystem watcher"),
        );

        Resolvers {
            watcher,
            watch_handlers,
        }
    }

    /// Resolves an asset URI to a reader
    fn open(&self, uri: &str) -> io::Result<Box<dyn io::Read>> {
        // resolve from filesystem
        // TODO pluggable schemes / search paths
        let file = File::open(uri)?;
        Ok(Box::new(file))
    }

    /// Watches for changes to a specified URI.
    fn watch_changes(
        &self,
        uri: &str,
        recursive: bool,
        callback: impl FnMut(Event) + Send + 'static,
    ) -> io::Result<WatchSubscription> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        // canonicalize the path first
        let canonical_path = fs::canonicalize(uri)?;

        // watch the canonicalized path
        let mut watcher = self.watcher.lock().unwrap();
        watcher
            .watch(&canonical_path, mode)
            .expect("failed to watch for file changes");

        // update the list of handlers...
        let mut watch_list = self.watch_handlers.lock().unwrap();
        // ... first, remove watch list entries that have expired (subscription dropped), and unwatch them
        watch_list.retain(|p, w| {
            if w.subscription.strong_count() > 0 {
                true
            } else {
                tracing::trace!("removing watcher for path `{:?}`", p);
                watcher.unwatch(&w.original_path);
                false
            }
        });
        // ... then add the new handler to the map
        let subscription_token = Arc::new(());
        let handler = WatchHandler {
            original_path: uri.into(),
            canonical_path: canonical_path.clone(),
            callback: Box::new(callback),
            subscription: Arc::downgrade(&subscription_token),
        };
        watch_list.insert(canonical_path.clone(), handler);

        tracing::trace!("watching `{}`", uri);
        Ok(WatchSubscription(subscription_token))
    }
}

#[derive(Clone)]
pub struct AssetLoader {
    resolvers: Arc<Resolvers>,
}

impl Default for AssetLoader {
    fn default() -> Self {
        AssetLoader::new()
    }
}

#[derive(Debug, Error)]
pub enum AssetLoadError<E: Error + fmt::Debug> {
    #[error("I/O error")]
    Io(io::Error),
    #[error("asset error")]
    Asset(#[from] E),
}

impl AssetLoader {
    pub fn new() -> AssetLoader {
        AssetLoader {
            resolvers: Arc::new(Resolvers::new()),
        }
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

    /// Loads an asset asynchronously.
    pub fn load_async<T: Asset + 'static>(
        &self,
        uri: &str,
    ) -> impl Future<Output = Result<T, AssetLoadError<T::LoadError>>> {
        let resolvers = self.resolvers.clone();
        let uri = uri.to_string();
        async move {
            trace!("load_async {:?}", uri);
            task::spawn_blocking(move || {
                trace!("load_async(worker) {:?}", uri);
                let mut reader = resolvers.open(&uri).map_err(AssetLoadError::Io)?;
                let value = T::load(&mut reader).map_err(AssetLoadError::Asset)?;
                Ok(value)
            })
            .await
            .expect("failed to await")
        }
    }

    /// Watches for changes to an asset file.
    pub fn watch_changes(
        &self,
        uri: &str,
        recursive: bool,
        callback: impl FnMut(Event) + Send  + 'static,
    ) -> io::Result<WatchSubscription> {
        self.resolvers.watch_changes(uri, recursive, callback)
    }
}
