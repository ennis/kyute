////////////////////////////////////////////////////////////////////////////////////////////////////

/*
/// Horizontal aligmnent modifier.
pub struct HorizontalAlignment<W>(pub Alignment, pub W);

impl<W: Widget> Widget for HorizontalAlignment<W> {
    type Element = HorizontalAlignmentElement<W::Element>;

    fn id(&self) -> WidgetId {
        self.1.id()
    }

    fn build(self, cx: &mut TreeCtx, env: &Environment) -> Self::Element {
        let inner = self.1.build(cx, env);
        HorizontalAlignmentElement(self.0, inner)
    }

    fn update(self, cx: &mut TreeCtx, node: &mut Self::Element, env: &Environment) -> ChangeFlags {
        if node.0 != self.0 {
            cx.relayout()
        }
        self.1.update(cx, &mut node.1, env)
    }
}

pub struct HorizontalAlignmentElement<T>(pub Alignment, pub T);

impl<T: Element> Element for HorizontalAlignmentElement<T> {
    fn id(&self) -> WidgetId {
        self.1.id()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        let mut geom = self.1.layout(ctx, params);
        geom.x_align = self.0;
        geom
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        self.1.event(ctx, event)
    }

    fn route_event(&mut self, ctx: &mut RouteEventCtx, child: WidgetId, event: &mut Event) -> ChangeFlags {
        self.1.route_event(ctx, child, event)
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        self.1.hit_test(ctx, position)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.1.paint(ctx)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}*/
