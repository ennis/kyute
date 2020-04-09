use crate::event::{Event, EventCtx};
use crate::layout::{BoxConstraints, PaintLayout};
use crate::renderer::Theme;
use crate::visual::{Node, Visual};
use crate::widget::{ActionSink, LayoutCtx};
use crate::{Point, Widget};
use std::marker::PhantomData;
use std::rc::Rc;

/// Map one action to another.
pub struct Map<A, W, F> {
    inner: W,
    map: F,
    _phantom: PhantomData<A>,
}

impl<A, W, F> Map<A, W, F> {
    pub fn new(inner: W, map: F) -> Map<A, W, F> {
        Map {
            inner,
            map,
            _phantom: PhantomData,
        }
    }
}

impl<A: 'static, B: 'static, W: Widget<A>, F: Fn(A) -> B + 'static> Widget<B> for Map<A, W, F> {

    type Visual = W::Visual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<B>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme
    ) -> Node<Self::Visual>
    {
        let mut ctx = ctx.map(self.map);
        self.inner.layout(&mut ctx, node, constraints, theme)
    }
}
