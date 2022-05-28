use crate::{
    cache,
    core::{DebugNode},
    drawing,
    drawing::ToSkia,
    util::fs_watch::watch_path,
    widget::{prelude::*, Null},
    AssetLoader, SizeI,
};
use kyute_common::Color;
use skia_safe as sk;
use std::task::Poll;
use tracing::trace;

#[derive(Clone)]
enum ImageContents<Placeholder> {
    Image { uri: Option<String>, image: drawing::Image },
    Placeholder(Placeholder),
}

impl<Placeholder: Widget> ImageContents<Placeholder> {
    pub fn new(uri: Option<String>, image: drawing::Image) -> ImageContents<Placeholder> {
        ImageContents::Image { uri, image }
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
    colorize: Option<Color>,
}

impl<Placeholder> Image<Placeholder> {
    pub fn colorize(mut self, color: Color) -> Self {
        self.colorize = Some(color);
        self
    }
}

impl Image<Null> {
    /// Creates an image widget that displays the image from a specified asset URI.
    #[composable]
    pub fn from_uri(uri: &str, scaling: Scaling) -> Image<Null> {
        let image: drawing::Image = AssetLoader::instance().load(uri).expect("failed to load image");
        Image {
            contents: ImageContents::new(uri.to_string().into(), image),
            scaling,
            colorize: None,
        }
    }

    /// Returns the size of the image in pixels.
    pub fn pixel_size(&self) -> SizeI {
        match self.contents {
            ImageContents::Image { ref image, .. } => image.size(),
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
        let uri_owned = uri.to_owned();

        let image = cache::run_async(
            async move {
                let image_result = image_future.await;
                match image_result {
                    Ok(image) => {
                        trace!("image `{}` successfully loaded", uri_owned);
                        Some(image)
                    }
                    Err(err) => {
                        trace!("failed to load image `{}`: {}", uri_owned, err);
                        None
                    }
                }
            },
            reload,
        );

        match image {
            Poll::Ready(Some(image)) => Image {
                contents: ImageContents::new(Some(uri.to_string()), image),
                scaling,
                colorize: None,
            },
            _ => Image {
                contents: ImageContents::Placeholder(Null),
                scaling,
                colorize: None,
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
            ImageContents::Image { ref image, .. } => {
                let size_i = image.size();
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
            ImageContents::Image { ref image, .. } => {
                let mut paint;
                let paint = if let Some(color) = self.colorize {
                    paint = sk::Paint::default();
                    paint.set_color_filter(sk::color_filters::blend(
                        color.to_skia().to_color(),
                        sk::BlendMode::SrcIn,
                    ));
                    Some(&paint)
                } else {
                    None
                };

                ctx.surface
                    .canvas()
                    .draw_image(image.to_skia(), Point::origin().to_skia(), paint);
            }
            ImageContents::Placeholder(ref placeholder) => placeholder.paint(ctx),
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(match self.contents {
            ImageContents::Image { ref image, ref uri } => {
                let mut msg = format!("{}px x {}px image", image.size().width, image.size().height);
                if let Some(ref uri) = uri {
                    msg += &format!("({})", uri);
                }
                msg
            }
            ImageContents::Placeholder(ref placeholder) => "placeholder".to_string(),
        })
    }
}
