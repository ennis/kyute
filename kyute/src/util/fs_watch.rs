//! File system watcher
use crate::{composable, environment, memoize, state, EnvKey};
use notify::{RecommendedWatcher, Watcher};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct WatchSubscription(Arc<()>);

struct WatchEntry {
    /// Store the original path because that's how the `notify` crate identifies watched paths.
    original_path: PathBuf,
    /// Same as `original_path` but canonicalized.
    canonical_path: PathBuf,
    callbacks: Vec<Box<dyn FnMut(notify::Event) + Send>>,
    subscription: Arc<()>,
}

impl WatchEntry {
    fn new(original_path: PathBuf, canonical_path: PathBuf) -> WatchEntry {
        WatchEntry {
            original_path,
            canonical_path,
            callbacks: vec![],
            subscription: Arc::new(()),
        }
    }
}

struct FileWatcherInner {
    watcher: Mutex<RecommendedWatcher>,
    entries: Arc<Mutex<HashMap<PathBuf, WatchEntry>>>,
}

#[derive(Clone)]
pub struct FileSystemWatcher(Arc<FileWatcherInner>);

impl_env_value!(FileSystemWatcher);

pub(crate) const FILE_SYSTEM_WATCHER: EnvKey<FileSystemWatcher> = EnvKey::new("kyute.file-system-watcher");

impl FileSystemWatcher {
    /// Creates a new FileWatcher.
    pub(crate) fn new() -> FileSystemWatcher {
        let watch_entries: Arc<Mutex<HashMap<PathBuf, WatchEntry>>> = Arc::new(Mutex::new(HashMap::new()));
        let watch_entries_clone = watch_entries.clone();

        let watcher = Mutex::new(
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        let mut entries = watch_entries_clone.lock().unwrap();

                        eprintln!("file event: `{:?}`", event);
                        for path in event.paths.iter() {
                            if let Ok(canonical_path) = fs::canonicalize(path) {
                                // see if there's a watcher for this path
                                if let Some(entry) = entries.get_mut(&canonical_path) {
                                    // OK, invoke callbacks
                                    for cb in entry.callbacks.iter_mut() {
                                        (cb)(event.clone())
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => error!("watch error: {}", e),
                }
            })
            .expect("failed to create filesystem watcher"),
        );

        FileSystemWatcher(Arc::new(FileWatcherInner {
            watcher,
            entries: watch_entries,
        }))
    }

    /// Returns the FileWatcher instance from the current environment.
    pub fn instance() -> FileSystemWatcher {
        environment()
            .get(FILE_SYSTEM_WATCHER)
            .expect("could not find a FileWatcher in the current environment")
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
        let mut watch_list = self.0.entries.lock().unwrap();
        // ... first, remove watch list entries that have expired (subscription dropped), and unwatch them
        watch_list.retain(|p, w| {
            if Arc::strong_count(&w.subscription) > 0 {
                true
            } else {
                eprintln!("removing watcher for path `{:?}`", p);
                watcher.unwatch(&w.canonical_path);
                false
            }
        });
        // ... then add the new handler to the map
        eprintln!(
            "watching `{}` (`{}`)",
            original_path.display(),
            canonical_path.display()
        );
        let entry = watch_list
            .entry(canonical_path.clone())
            .or_insert_with(|| WatchEntry::new(original_path, canonical_path));
        entry.callbacks.push(Box::new(callback));
        let subscription = entry.subscription.clone();
        Ok(WatchSubscription(subscription))
    }
}

/// A composable function that returns true when a change has been detected on the specified path.
#[composable]
pub fn watch_path(path: impl AsRef<Path>) -> bool {
    let changed = state(|| false);
    let changed_2 = changed.clone();

    memoize(path.as_ref().to_owned(), || {
        FileSystemWatcher::instance()
            .watch(path, false, move |_event| {
                changed_2.set(true);
            })
            .ok()
    });

    changed.update(false).unwrap_or(false)
}
