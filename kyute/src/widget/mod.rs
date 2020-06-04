//! `Widget` base trait and built-in widgets.
pub mod align;
pub mod baseline;
pub mod button;
pub mod constrained;
pub mod dummy;
pub mod expand;
pub mod flex;
pub mod form;
pub mod frame;
pub mod id;
pub mod map;
pub mod padding;
pub mod slider;
pub mod text;
pub mod textedit;

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
use crate::visual::Visual;
use crate::{visual, LayoutCtx, Measurements, env};
use generational_indextree::NodeId;
use kyute_shell::platform::Platform;
use std::any::TypeId;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use crate::env::Environment;

/// Receivers of actions emitted by widgets.
pub trait ActionSink<A> {
    fn emit(&self, action: A);
}

///
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
    /// Returns the key of the widget, used to match the widget to the node tree.
    fn key(&self) -> Option<u64> {
        None
    }

    /// Returns the typeid of the visual that this widget produces.
    ///
    /// The reconciliation algorithm uses both the key and the visual type ID to match a widget with
    /// a node in the node tree.
    fn visual_type_id(&self) -> TypeId;

    /// Performs layout, consuming the widget.
    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<dyn Visual>, Measurements);
}

/// A widget wrapped in a box, that produce a visual wrapped in a box as well.
pub type BoxedWidget<A> = Box<dyn Widget<A>>;

impl<A> Widget<A> for Box<dyn Widget<A>> {
    fn key(&self) -> Option<u64> {
        self.as_ref().key()
    }

    fn visual_type_id(&self) -> TypeId {
        self.as_ref().visual_type_id()
    }

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> (Box<dyn Visual>, Measurements) {
        (*self).layout(context, previous_visual, constraints, theme)
    }
}

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
    fn boxed<'a>(self) -> Box<dyn Widget<A> + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }
}

impl<A: 'static, W: Widget<A>> WidgetExt<A> for W {}

pub trait TypedWidget<A: 'static> {
    type Visual: Visual;

    fn key(&self) -> Option<u64> { None }

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<Self::Visual>>,
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> (Box<Self::Visual>, Measurements);
}

// FIXME impl may overlap with impl Widget for Box<dyn Widget> because of a possible impl of TypedWidget in a downstream crate
// (not sure that the orphan rules even allow it...).
// Possible fixes:
// - don't impl Widget for Box<Widget<A>>
// - remove the `A` trait param and replace with associated type
//      - may need to parameterize some types on A where it's not needed right now
//          (e.g. widgets that don't emit actions)
// - remove the `A` trait param and design another mechanism to emit actions

impl<A: 'static, T> Widget<A> for T
where
    T: TypedWidget<A>,
{
    fn key(&self) -> Option<u64> {
        self.key()
    }

    fn visual_type_id(&self) -> TypeId {
        TypeId::of::<Self::Visual>()
    }

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<dyn Visual>>,
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> (Box<dyn Visual>, Measurements) {
        let (visual, measurements) = TypedWidget::layout(
            self,
            context,
            previous_visual.map(|v| v.downcast().expect("unexpected visual type")),
            constraints,
            theme,
        );
        (visual, measurements)
    }
}
