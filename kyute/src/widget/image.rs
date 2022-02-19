use crate::{
    cache, composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::PointerEventKind,
    state::Signal,
    style::{BoxStyle, ColorRef, ValueRef},
    theme,
    widget::{Container, Label},
    AssetUri, BoxConstraints, Environment, Event, Measurements, Rect, SideOffsets, Widget,
    WidgetPod,
};
use kyute_shell::application::Application;
use tracing::trace;

#[derive(Clone)]
pub struct Image {
    image: Option<kyute_shell::drawing::Image>,
}

impl Image {
    /// Creates an image widget that displays the image from a specified asset URI.
    #[composable(uncached)]
    pub fn from_uri(uri: &str) -> Image {
        let application = Application::instance();
        let image: kyute_shell::drawing::Image = application
            .asset_loader()
            .load(uri)
            .expect("failed to load image");
        Image { image: Some(image) }
    }

    /// Creates an image widget that loads the image at the specified URI asynchronously,
    /// and displays the image once it is loaded.
    #[composable(uncached)]
    pub fn from_uri_async(uri: &str) -> Image {
        let application = Application::instance();
        let image_future = application
            .asset_loader()
            .load_async::<kyute_shell::drawing::Image>(uri);

        let state = cache::state_async(async move {
            let image_result = image_future.await;
            image_result.ok()
        });

        if let Some(image) = state.get() {
            Image { image }
        } else {
            Image { image: None }
        }
    }
}
