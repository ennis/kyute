use crate::{
    core::Widget,
    widgets::{Align, Clickable},
    Alignment, Ctx, WidgetPtr,
};

/// Extension methods on widgets.
pub trait WidgetExt {
    /*/// Sets the background paint of the widget.
    #[must_use]
    fn background(self, paint: impl Into<Paint>) -> Overlay<Self, Background> {
        Overlay::new(self, Background::new(paint.into()), ZOrder::Below)
    }

    /// Shows an overlay on top of the widget.
    #[must_use]
    fn overlay<W: Widget + 'static>(self, overlay: W) -> Overlay<Self, W> {
        Overlay::new(self, overlay, ZOrder::Above)
    }*/

    /// Makes the widget clickable.
    ///
    /// # Example
    ///
    /// TODO
    #[must_use]
    fn clickable(self, on_click: impl Fn(&mut Ctx) + 'static) -> WidgetPtr;

    #[must_use]
    fn align(self, x: Alignment, y: Alignment) -> WidgetPtr;

    /*#[must_use]
    fn provide_with<T, F>(self, f: F) -> ProvideWith<T, F, Self> {
        ProvideWith::new(f, self)
    }

    #[must_use]
    fn provide<T>(self, value: T) -> Provide<T, Self> {
        Provide::new(value, self)
    }

    /// Disables or enables the widget.
    #[must_use]
    fn disabled<T>(self, disabled: bool) -> ProvideWith<WidgetState, fn(WidgetState) -> WidgetState, Self> {
        // FIXME I'd like to pass a closure but I can't name the closure type, and impl Fn in return position in traits is not stable yet
        let f = if disabled {
            |prev| WidgetState { disabled: true, ..prev }
        } else {
            |prev| WidgetState {
                disabled: false,
                ..prev
            }
        };
        self.provide_with(f)
    }

    /*/// Adds a frame with decorations around the widget.
    #[must_use]
    fn decorate<B>(self, shape_decoration: ShapeDecoration<B>) -> Frame<Self, B> {
        Frame::new(100.percent(), 100.percent(), self).decoration(shape_decoration)
    }*/

    #[must_use]
    fn padding(self, padding: impl Into<Insets>) -> Padding<Self> {
        Padding::new(padding.into(), self)
    }



    #[must_use]
    fn decorate<D: Decoration>(self, decoration: D) -> DecoratedBox<D, Self> {
        DecoratedBox::new(decoration, self)
    }

    #[must_use]
    fn min_size(self, size: Size) -> Constrained<Self> {
        Constrained::new(
            BoxConstraints {
                min: size,
                max: Size::new(f64::INFINITY, f64::INFINITY),
            },
            self,
        )
    }*/
}

impl WidgetExt for WidgetPtr {
    fn clickable(self, on_click: impl Fn(&mut Ctx) + 'static) -> WidgetPtr {
        Clickable::new(self, on_click)
    }

    fn align(self, x: Alignment, y: Alignment) -> WidgetPtr {
        Align::new(x, y, self)
    }
}
