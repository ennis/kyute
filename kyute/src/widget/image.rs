use crate::{
    application::ExtEvent,
    cache, drawing,
    drawing::ToSkia,
    widget::{prelude::*, Null},
    AssetLoader,
};
use std::task::Poll;
use tracing::trace;

#[derive(Clone)]
enum ImageContents<Placeholder> {
    Image(drawing::Image),
    Placeholder(Placeholder),
}

#[derive(Clone)]
pub struct Image<Placeholder> {
    contents: ImageContents<Placeholder>,
}

impl Image<Null> {
    /// Creates an image widget that displays the image from a specified asset URI.
    #[composable]
    pub fn from_uri(uri: &str) -> Image<Null> {
        let image: drawing::Image = AssetLoader::instance().load(uri).expect("failed to load image");

        Image {
            contents: ImageContents::Image(image),
        }
    }
}

/// A composable function that returns true when the asset at the specified URI should be reloaded.
#[composable]
fn watch_file_changes(uri: &str) -> bool {
    let changed = cache::state(|| false);
    let event_loop_proxy = cache::event_loop_proxy();

    cache::memoize(uri.to_owned(), || {
        AssetLoader::instance()
            .watch_changes(uri, false, move |_event| {
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

impl<Placeholder: Widget> Image<Placeholder> {
    /// Creates an image widget that loads the image at the specified URI asynchronously,
    /// and displays the image once it is loaded.
    #[composable]
    pub fn from_uri_async(uri: &str, placeholder: Placeholder) -> Image<Placeholder> {
        let image_future = AssetLoader::instance().load_async::<drawing::Image>(uri);
        let reload = watch_file_changes(uri);

        let image = cache::run_async(
            async move {
                let image_result = image_future.await;
                trace!("Image::from_uri_async {:?}", image_result);
                image_result.ok()
            },
            reload,
        );

        match image {
            Poll::Ready(Some(image)) => Image {
                contents: ImageContents::Image(image),
            },
            _ => Image {
                contents: ImageContents::Placeholder(placeholder),
            },
        }
    }
}

impl<Placeholder: Widget> Widget for Image<Placeholder> {
    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        match self.contents {
            ImageContents::Image(ref img) => {
                // TODO DPI
                let size_i = img.size();
                Measurements::new(Rect::new(
                    Point::origin(),
                    Size::new(size_i.width as f64, size_i.height as f64),
                ))
            }
            ImageContents::Placeholder(ref placeholder) => placeholder.layout(ctx, constraints, env),
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        match self.contents {
            ImageContents::Image(ref img) => {
                ctx.canvas.draw_image(img.to_skia(), Point::origin().to_skia(), None);
            }
            ImageContents::Placeholder(ref placeholder) => placeholder.paint(ctx, bounds, env),
        }
    }
}
