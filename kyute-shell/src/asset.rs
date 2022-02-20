use notify::{RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    error::Error,
    fmt, fs,
    fs::File,
    future::Future,
    hash::{Hash, Hasher},
    io,
    marker::PhantomData,
    path::Path,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use tokio::task;
use tracing::trace;

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
            raw: AssetUri {
                uri,
                data: Some(data),
            },
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

type EventTxMap = HashMap<String, tokio::sync::mpsc::Sender<notify::Event>>;
type EventRx = tokio::sync::mpsc::Receiver<notify::Event>;

fn handle_filesystem_event(event: notify::Event, watchers: &EventTxMap) {
    //trace!("handle_filesystem_event {:?}", event);
    for path in event.paths.iter() {
        if let Some(s) = path.to_str() {
            for (watched_path, tx) in watchers {
                if let Ok(watched_canonical) = fs::canonicalize(watched_path) {
                    if let Ok(event_canonical) = fs::canonicalize(s) {
                        if watched_canonical == event_canonical {
                            trace!("changed: {:?}", watched_path);
                            tx.try_send(event.clone());
                        }
                    }
                }
            }
        }
    }
}

struct Resolvers {
    filesystem_watcher: Mutex<notify::RecommendedWatcher>,
    event_txs: Arc<Mutex<EventTxMap>>,
}

impl Resolvers {
    fn new() -> Resolvers {
        let event_txs = Arc::new(Mutex::new(EventTxMap::new()));
        let event_txs_clone = event_txs.clone();
        let filesystem_watcher = Mutex::new(
            notify::recommended_watcher(move |res| match res {
                Ok(event) => {
                    let w = event_txs_clone.lock().unwrap();
                    handle_filesystem_event(event, &w);
                }
                Err(e) => tracing::error!("watch error: {}", e),
            })
            .expect("failed to create filesystem watcher"),
        );

        Resolvers {
            filesystem_watcher,
            event_txs,
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
    fn watch_changes(&self, uri: &str, recursive: bool) -> EventRx {
        let (tx, rx) = tokio::sync::mpsc::channel(50);
        let mut txs = self.event_txs.lock().unwrap();

        txs.insert(uri.to_string(), tx);
        self.filesystem_watcher
            .lock()
            .unwrap()
            .watch(
                Path::new(uri),
                if recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                },
            )
            .expect("failed to watch for file changes");

        tracing::trace!("watching `{}`, txs: {:?}", uri, txs);
        rx
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

/*
We have:
 - typed asset IDs, generated by the "resource compiler" or whatever
 - asset URIs

Should the assetLoader be in charge of caching the loaded results?
- for all assets?
- only for typed asset IDs?

If not the asset loader, then what should cache the loaded assets?
- store asset objects in lazy_statics
- an "asset cache" wrapper on top of the asset loader
- something else?

Is there a difference between assets and resources?

What about hot-reloading?
- AssetLoader should provide a way to watch for file changes

Watch asset paths for changes.
    - Async?
    - recomp when something changes
*/
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
    pub fn watch_changes(&self, uri: &str) -> impl Future<Output = ()> {
        let mut rx = self.resolvers.watch_changes(uri, false);
        async move {
            let event = rx.recv().await;
            trace!("watch_changes: event={:?}", event);
        }
    }

    /*pub fn load_from_file<T: Asset>(&self, path: &str) -> io::Result<T> {
        std::fs::canonicalize(path)
    }*/
}
