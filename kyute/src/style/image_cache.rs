use crate::EnvKey;
use kyute_shell::{application::Application, AssetLoadError};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
struct Entry {
    image: kyute_shell::drawing::Image,
}

struct Inner {
    entries: HashMap<String, Entry>,
}

#[derive(Clone)]
pub struct ImageCache(Arc<Mutex<Inner>>);

impl ImageCache {
    pub fn new() -> ImageCache {
        ImageCache(Arc::new(Mutex::new(Inner {
            entries: Default::default(),
        })))
    }

    pub fn load(&self, uri: &str) -> Result<kyute_shell::drawing::Image, AssetLoadError<io::Error>> {
        let mut cache = self.0.lock().unwrap();

        if let Some(entry) = cache.entries.get(uri) {
            return Ok(entry.image.clone());
        }

        let image = Application::instance()
            .asset_loader()
            .load::<kyute_shell::drawing::Image>(uri)?;
        cache.entries.insert(uri.to_owned(), Entry { image: image.clone() });
        Ok(image)
    }
}

impl_env_value!(ImageCache);

pub const IMAGE_CACHE: EnvKey<ImageCache> = EnvKey::new("kyute.image-cache");
