//! File system watcher
use crate::{application::ExtEvent, cache::event_loop_proxy, composable, environment, memoize, state, EnvKey};
use notify::{RecommendedWatcher, Watcher};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

#[derive(Clone)]
pub struct WatchSubscription(Arc<()>);

struct WatchHandler {
    original_path: PathBuf,
    canonical_path: PathBuf,
    callback: Box<dyn FnMut(notify::Event) + Send>,
    subscription: Weak<()>,
}

struct FileWatcherInner {
    watcher: Mutex<RecommendedWatcher>,
    watch_handlers: Arc<Mutex<HashMap<PathBuf, WatchHandler>>>,
}

#[derive(Clone)]
pub struct FileSystemWatcher(Arc<FileWatcherInner>);

impl_env_value!(FileSystemWatcher);

pub(crate) const FILE_SYSTEM_WATCHER: EnvKey<FileSystemWatcher> = EnvKey::new("kyute.file-system-watcher");

impl FileSystemWatcher {
    /// Creates a new FileWatcher.
    pub(crate) fn new() -> FileSystemWatcher {
        let watch_handlers: Arc<Mutex<HashMap<PathBuf, WatchHandler>>> = Arc::new(Mutex::new(HashMap::new()));
        let watch_handlers_clone = watch_handlers.clone();

        let watcher = Mutex::new(
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
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

        FileSystemWatcher(Arc::new(FileWatcherInner {
            watcher,
            watch_handlers,
        }))
    }

    /// Returns the FileWatcher instance from the current environment.
    pub fn instance() -> FileSystemWatcher {
        environment()
            .get(FILE_SYSTEM_WATCHER)
            .expect("could not find a FileWatcher in the current environment")
            .clone()
    }

    /// Watches for changes to a specified path.
    pub fn watch(
        &self,
        path: impl AsRef<Path>,
        recursive: bool,
        callback: impl FnMut(notify::Event) + Send + 'static,
    ) -> io::Result<WatchSubscription> {
        let mode = if recursive {
            notify::RecursiveMode::Recursive
        } else {
            notify::RecursiveMode::NonRecursive
        };

        // canonicalize the path first
        let original_path = path.as_ref().to_owned();
        let canonical_path = fs::canonicalize(&original_path)?;

        // watch the canonicalized path
        let mut watcher = self.0.watcher.lock().unwrap();
        watcher
            .watch(&canonical_path, mode)
            .expect("failed to watch for file changes");

        // update the list of handlers...
        let mut watch_list = self.0.watch_handlers.lock().unwrap();
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
            original_path,
            canonical_path: canonical_path.clone(),
            callback: Box::new(callback),
            subscription: Arc::downgrade(&subscription_token),
        };
        tracing::trace!(
            "watching `{}` (`{}`)",
            handler.original_path.display(),
            handler.canonical_path.display()
        );
        watch_list.insert(canonical_path.clone(), handler);

        Ok(WatchSubscription(subscription_token))
    }
}

/// A composable function that returns true when a change has been detected on the specified path.
#[composable]
pub fn watch_path(path: impl AsRef<Path>) -> bool {
    let changed = state(|| false);
    let event_loop_proxy = event_loop_proxy();

    memoize(path.as_ref().to_owned(), || {
        FileSystemWatcher::instance()
            .watch(path, false, move |_event| {
                event_loop_proxy
                    .send_event(ExtEvent::Recompose {
                        cache_fn: Box::new(move |cache| cache.set_state(changed, true)),
                    })
                    .unwrap();
            })
            .ok()
    });

    changed.update(false)
}
