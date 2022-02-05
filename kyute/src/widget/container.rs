use crate::{
    align_boxes, composable,
    style::{Border, NullVisual, PaintCtxExt, Style, Visual},
    Alignment, BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, Offset,
    PaintCtx, Point, Rect, SideOffsets, Size, Widget, WidgetPod,
};

#[derive(Clone)]
pub struct Container<V, Content> {
    alignment: Option<Alignment>,
    width: Option<f64>,
    height: Option<f64>,
    baseline: Option<f64>,
    content_padding: SideOffsets,
    visual: V,
    content: WidgetPod<Content>,
}

impl<Content: Widget + 'static> Container<NullVisual, Content> {
    #[composable(uncached)]
    pub fn new(content: Content) -> Container<NullVisual, Content> {
        Container {
            alignment: None,
            width: None,
            height: None,
            baseline: None,
            content_padding: Default::default(),
            visual: NullVisual,
            content: WidgetPod::new(content),
        }
    }
}

impl<V: Visual, Content: Widget + 'static> Container<V, Content> {
    /// Constrain the width of the container.
    pub fn fix_width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// Constrain the width of the container.
    pub fn fix_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    pub fn fix_size(mut self, size: Size) -> Self {
        self.width = Some(size.width);
        self.height = Some(size.height);
        self
    }

    /// Centers the content in the available space.
    pub fn centered(mut self) -> Self {
        self.alignment = Some(Alignment::CENTER);
        self
    }

    /// Aligns the widget in the available space.
    pub fn aligned(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Aligns the widget in the available space.
    pub fn content_padding(mut self, padding: SideOffsets) -> Self {
        self.content_padding = padding;
        self
    }

    pub fn visual<V2: Visual>(self, visual: V2) -> Container<(V, V2), Content> {
        Container {
            alignment: self.alignment,
            width: self.width,
            height: self.height,
            baseline: self.baseline,
            content_padding: self.content_padding,
            visual: (self.visual, visual),
            content: self.content,
        }
    }
}

impl<V: Visual, Content: Widget> Widget for Container<V, Content> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.event(ctx, event, env)
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        mut constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        // First, measure the child, taking into account the mandatory padding
        let mut content_constraints = constraints;
        content_constraints = content_constraints.deflate(self.content_padding);
        let content_size = self.content.layout(ctx, content_constraints, env);
        let mut content_offset = Offset::new(self.content_padding.left, self.content_padding.top);

        // apply additional w/h sizing constraints to the container
        let mut additional_constraints = match (self.width, self.height) {
            (Some(w), Some(h)) => BoxConstraints::tight(Size::new(w, h)),
            (Some(w), None) => BoxConstraints::new(w..w, ..),
            (None, Some(h)) => BoxConstraints::new(.., h..h),
            (None, None) => BoxConstraints::new(.., ..),
        };
        // ... however the resulting sizing constrains cannot result in a container that is smaller
        // than the content size (the content takes priority when sizing the container).
        additional_constraints.max.width =
            additional_constraints.max.width.max(content_size.width());
        additional_constraints.max.height =
            additional_constraints.max.height.max(content_size.height());

        // finally, merge the additional constraints
        constraints = constraints.enforce(additional_constraints);

        // now determine our width
        // basically: do we stretch or do we size to contents?
        // -> if we have alignment and we are bounded: stretch
        // -> otherwise (if we don't have alignment, or we are unbounded): size to contents
        let size = if let Some(alignment) = self.alignment {
            let w = if constraints.max_width().is_finite() {
                // alignment specified, finite width bounds: expand to fill
                constraints.max_width()
            } else {
                // size to contents
                constraints.constrain_width(content_size.width())
            };
            let h = if constraints.max_height().is_finite() {
                constraints.max_height()
            } else {
                constraints.constrain_height(content_size.height())
            };
            Size::new(w, h)
        } else {
            // no alignment = size to content
            constraints.constrain(content_size.size())
        };

        // Place the contents inside the box according to alignment
        if let Some(alignment) = self.alignment {
            let x = 0.5 * size.width * (1.0 + alignment.x)
                - 0.5 * content_size.width() * (1.0 + alignment.x);
            let y = 0.5 * size.height * (1.0 + alignment.y)
                - 0.5 * content_size.height() * (1.0 + alignment.y);
            content_offset.x += x;
            content_offset.y += y;
        }

        self.content.set_child_offset(content_offset);

        Measurements {
            bounds: Rect {
                origin: Point::origin(),
                size,
            },
            baseline: None,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        ctx.draw_visual(bounds, &self.visual, env);
        self.content.paint(ctx, bounds, env);
    }
}
