#![feature(unsized_locals)]

//! Kyute widget toolkit
pub mod layout;
pub mod renderer;
pub mod state;
pub mod visual;
pub mod widget;
pub mod event;
pub mod application;

// re-exports

pub use self::renderer::Painter;
pub use self::renderer::Renderer;

pub use self::visual::Node;
pub use self::visual::Visual;

pub use self::widget::BoxedWidget;
pub use self::widget::Widget;
pub use self::widget::WidgetExt;

pub use self::layout::PaintLayout;
pub use self::layout::Alignment;
pub use self::layout::Bounds;
pub use self::layout::BoxConstraints;
pub use self::layout::Layout;
pub use self::layout::Point;
pub use self::layout::Size;
use kyute_shell::platform::{PlatformWindow, Platform};


///
pub struct Cache {
    /// Cached visual tree.
    tree: Option<Node<Box<dyn Visual>>>,
}

/*
impl<A: 'static> Cache<A> {
    /// Creates a new `Cache`.
    pub fn new() -> Cache<A> {
        Cache { tree: None }
    }

    /// Layouts the widget and paints it.
    pub fn paint(&mut self, painter: &mut Painter, widget: impl Widget<A>) {
        // fill the available space
        let size = painter.size();
        let mut tree = widget.layout(painter.renderer(), &BoxConstraints::loose(painter.size()));
        let root_layout = Layout::new(size);
        tree.paint(
            painter,
            &PaintLayout::new(Point::new(0.0, 0.0), &root_layout),
        );
        self.tree.replace(tree.boxed());
    }

    /// If the geometry of the canvas has changed since the last time, layout needs to be done again, returns false.
    pub fn paint_cached(&mut self, _painter: &Painter) -> bool {
        // for now, always invalidate
        false
    }
}
*/