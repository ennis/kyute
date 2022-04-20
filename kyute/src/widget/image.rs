use crate::{
    cache,
    core::WindowPaintCtx,
    drawing,
    drawing::ToSkia,
    util::fs_watch::watch_path,
    widget::{prelude::*, Null},
    AssetLoader, GpuFrameCtx, SizeI,
};
use std::task::Poll;
use tracing::trace;

#[derive(Clone)]
enum ImageContents<Placeholder> {
    Image(drawing::Image),
    Placeholder(Placeholder),
}

impl<Placeholder: Widget> ImageContents<Placeholder> {
    pub fn new(image: drawing::Image) -> ImageContents<Placeholder> {
        ImageContents::Image(image)
    }

    pub fn placeholder(placeholder: Placeholder) -> ImageContents<Placeholder> {
        ImageContents::Placeholder(placeholder)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Scaling {
    Contain,
    Cover,
}

#[derive(Clone)]
pub struct Image<Placeholder> {
    contents: ImageContents<Placeholder>,
    scaling: Scaling,
}

impl Image<Null> {
    /// Creates an image widget that displays the image from a specified asset URI.
    #[composable]
    pub fn from_uri(uri: &str, scaling: Scaling) -> Image<Null> {
        let image: drawing::Image = AssetLoader::instance().load(uri).expect("failed to load image");
        Image {
            contents: ImageContents::Image(image),
            scaling,
        }
    }

    /// Returns the size of the image in pixels.
    pub fn pixel_size(&self) -> SizeI {
        match self.contents {
            ImageContents::Image(ref image) => image.size(),
            ImageContents::Placeholder(_) => {
                // FIXME: cannot know the size of a placeholder before layout; use LayoutInspector? ensure fixed size?
                SizeI::new(0, 0)
            }
        }
    }

    /// Creates an image widget that loads the image at the specified URI asynchronously,
    /// and displays the image once it is loaded.
    #[composable]
    pub fn from_uri_async(uri: &str, scaling: Scaling) -> Image<Null> {
        let image_future = AssetLoader::instance().load_async::<drawing::Image>(uri);
        let reload = watch_path(uri);
        let uri = uri.to_owned();

        let image = cache::run_async(
            async move {
                let image_result = image_future.await;
                match image_result {
                    Ok(image) => {
                        trace!("image `{}` successfully loaded", uri);
                        Some(image)
                    }
                    Err(err) => {
                        trace!("failed to load image `{}`: {}", uri, err);
                        None
                    }
                }
            },
            reload,
        );

        match image {
            Poll::Ready(Some(image)) => Image {
                contents: ImageContents::Image(image),
                scaling,
            },
            _ => Image {
                contents: ImageContents::Placeholder(Null),
                scaling,
            },
        }
    }

    /*pub fn placeholder<Placeholder: Widget>(self, placeholder: Placeholder) -> Image<Placeholder> {
        match self.contents.into_inner() {
            ImageContents::Image(image) => Image {
                contents: ImageContents::Image(image),
                scaling: Scaling::Cover,
            },
            ImageContents::Placeholder(_) => Image {
                contents: ImageContents::Placeholder(placeholder),
                scaling: Scaling::Cover,
            },
        }
    }*/
}

impl<Placeholder: Widget> Widget for Image<Placeholder> {
    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        match self.contents {
            ImageContents::Image(ref img) => {
                let size_i = img.size();
                let size = Size::new(size_i.width as f64, size_i.height as f64) / ctx.scale_factor;

                let image_aspect_ratio = size.width / size.height;
                let aspect_ratio = constraints.max.width / constraints.max.height;

                let scaled_size = match (
                    self.scaling,
                    /* space is wider than the image */ aspect_ratio > image_aspect_ratio,
                ) {
                    (Scaling::Contain, true) | (Scaling::Cover, false) => {
                        if constraints.max.height.is_finite() {
                            Size::new(constraints.max.height * image_aspect_ratio, constraints.max.height)
                        } else {
                            size
                        }
                    }
                    _ => {
                        if constraints.max.width.is_finite() {
                            Size::new(constraints.max.width, constraints.max.width / image_aspect_ratio)
                        } else {
                            size
                        }
                    }
                };

                Measurements::new(scaled_size)
            }
            ImageContents::Placeholder(ref placeholder) => placeholder.layout(ctx, constraints, env),
        }
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn paint(&self, ctx: &mut PaintCtx) {
        match self.contents {
            ImageContents::Image(ref image) => {
                ctx.surface
                    .canvas()
                    .draw_image(image.to_skia(), Point::origin().to_skia(), None);
            }
            ImageContents::Placeholder(ref placeholder) => placeholder.paint(ctx),
        }
    }
}
