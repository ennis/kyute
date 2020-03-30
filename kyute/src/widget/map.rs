use crate::layout::{PaintLayout, BoxConstraints};
use crate::renderer::{Painter, Renderer};
use crate::visual::{Node, Visual, Cursor};
use crate::{Widget, Point};
use std::marker::PhantomData;
use crate::event::{Event, EventCtx};
use crate::widget::{LayoutCtx, ActionSink};
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

    fn layout(self, ctx: &mut LayoutCtx<B>, cursor: &mut Cursor, constraints: &BoxConstraints)
    {
        let mut ctx = ctx.map(self.map);
        self.inner.layout(&mut ctx, cursor, constraints);
    }
}
