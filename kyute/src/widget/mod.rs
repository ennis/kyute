//! `Widget` base trait and built-in widgets.
pub mod baseline;
pub mod button;
pub mod dummy;
pub mod expand;
pub mod flex;
pub mod id;
pub mod map;
pub mod text;
pub mod textedit;
pub mod padding;

// re-export common widgets
pub use baseline::Baseline;
pub use button::Button;
pub use dummy::DummyWidget;
pub use expand::Expand;
pub use flex::Axis;
pub use flex::Flex;
pub use map::Map;
pub use text::Text;

use crate::application::WindowCtx;
use crate::layout::BoxConstraints;
use crate::renderer::Theme;
use crate::visual::{Visual, NodeReplacer, NodeList};
use crate::visual::Node;
use kyute_shell::platform::Platform;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use crate::Layout;

/// Objects that receive actions.
pub trait ActionSink<A> {
    fn emit(&self, action: A);
}

pub(crate) struct ActionCollector<A> {
    pub(crate) actions: RefCell<Vec<A>>,
}

impl<A> ActionCollector<A> {
    pub fn new() -> ActionCollector<A> {
        ActionCollector {
            actions: RefCell::new(Vec::new()),
        }
    }
}

impl<A> ActionSink<A> for ActionCollector<A> {
    fn emit(&self, action: A) {
        unimplemented!()
    }
}

struct ActionMapper<B, F> {
    parent: Rc<dyn ActionSink<B>>,
    map: F,
}

impl<A: 'static, B: 'static, F: Fn(A) -> B + 'static> ActionSink<A> for ActionMapper<B, F> {
    fn emit(&self, action: A) {
        self.parent.emit((self.map)(action))
    }
}

/// Context passed to [`Widget::layout`].
pub struct LayoutCtx<'a, 'ctx, A> {
    pub(crate) win_ctx: &'a mut WindowCtx<'ctx>,
    pub(crate) action_sink: Rc<dyn ActionSink<A>>,
}

impl<'a, 'ctx, A> LayoutCtx<'a,'ctx,A>
{
}

impl<'a, 'ctx, A: 'static> LayoutCtx<'a, 'ctx, A> {
    pub fn platform(&self) -> &'ctx Platform {
        self.win_ctx.platform
    }

    pub fn map<B: 'static, F: Fn(B) -> A + 'static>(&mut self, f: F) -> LayoutCtx<'_, 'ctx, B> {
        LayoutCtx {
            win_ctx: self.win_ctx,
            action_sink: Rc::new(ActionMapper {
                parent: self.action_sink.clone(),
                map: f,
            }),
        }
    }
}


// ctx.register_window(window)

/// Trait representing a widget before layout.
///
/// First, the user builds a tree of [`Widget`]s which represents the user interface. Then, the
/// widgets are laid out by calling [`Widget::layout`], which consumes the widgets and produces a tree
/// of [`Node`]s, which represent a tree of laid-out visual elements on the screen.
///
/// ## Details
///
/// The tree of [`Node`]s can be cached and reused to handle events and repaints, as long as the
/// layout does not need to changed. In contrast, the widget tree is much more short-lived, and thus
/// can easily borrow things from the application.
///
/// This is useful for widgets that create child widgets on-demand, based on layout information or
/// retained state: an example of this would be list views, which typically
/// only display a subset of the elements at a time, depending on the scroll position and the
/// available size. For lists with a lot of elements, it can be wasteful to
/// create a child widget for every element in the list up front when we know that most of them will
/// be discarded during layout. To solve this, we can pass a "widget-provider" object (typically,
/// a closure) from which the list widget could request widgets "on-demand". However, in most cases,
/// widgets are generated from application data, which means that the provider would need to borrow
/// the data to create the child widget. This is main reason behind the distinction between Nodes
/// and Widgets: if there was only one retained tree, it would borrow the application state for too
/// long, making the usage impractical.
///
/// See also [Inside Flutter - Building widgets on demand](https://flutter.dev/docs/resources/inside-flutter#building-widgets-on-demand).
pub trait Widget<A> {
    type Visual: Visual;

    /// Performs layout, consuming the widget.
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<Self::Visual>;

    ///
    fn layout_single_child<'a>(self,
                           ctx: &mut LayoutCtx<A>,
                           node_list: &'a mut NodeList,
                           constraints: &BoxConstraints,
                           theme: &Theme
    ) -> RefMut<'a, Node<Self::Visual>>
    {
        node_list.replacer().replace_or_create_with(None, move |node| self.layout(ctx, node, constraints, theme));
        RefMut::map(node_list.list.first_mut().unwrap().borrow_mut(), |node| node.downcast_mut().unwrap())
    }

}

/// Widget with a type-erased visual.
pub trait AnyWidget<A>
{
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        placer: &mut NodeReplacer,
        constraints: &BoxConstraints,
        theme: &Theme);

    fn layout_single_child<'a>(self,
                               ctx: &mut LayoutCtx<A>,
                               node_list: &'a mut NodeList,
                               constraints: &BoxConstraints,
                               theme: &Theme
    ) -> RefMut<'a, Node<dyn Visual>>
    {
        self.layout(ctx, &mut node_list.replacer(), constraints, theme);
        node_list.list.first_mut().unwrap().borrow_mut()
    }
}

struct AnyWidgetWrapper<W> {
    inner: W,
}

impl<A, W> AnyWidget<A> for AnyWidgetWrapper<W>
    where W: Widget<A>
{
    fn layout(self,
              ctx: &mut LayoutCtx<A>,
              placer: &mut NodeReplacer,
              constraints: &BoxConstraints,
              theme: &Theme)
    {
        // TODO key?
        placer.replace_or_create_with(None, move |node| {
            self.inner.layout(ctx, node, constraints, theme)
        });
    }


}

/// A widget wrapped in a box, that produce a visual wrapped in a box as well.
pub type BoxedWidget<A> = Box<dyn AnyWidget<A>>;


/// Extension methods for [`Widget`].
pub trait WidgetExt<A: 'static>: Widget<A> {
    /// TODO
    fn map<B, F>(self, f: F) -> Map<A, Self, F>
    where
        Self: Sized,
        F: Fn(A) -> B,
    {
        Map::new(self, f)
    }

    /// Turns this widget into a type-erased boxed representation.
    fn boxed<'a>(self) -> Box<dyn AnyWidget<A> + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(AnyWidgetWrapper { inner: self })
    }
}

impl<A: 'static, W: Widget<A>> WidgetExt<A> for W {}
