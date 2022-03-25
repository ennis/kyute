use crate::{widget::prelude::*, Length, Transform};
use std::sync::Arc;

pub enum PositioningMode {
    /// Position relative to center.
    Center,
    /// Position relative to leading edge (
    Leading,
    Trailing,
    SizeRelative,
}

#[derive(Clone)]
struct CanvasItem {
    offset_x: Length,
    offset_y: Length,
    widget: Arc<WidgetPod>,
    anchor: Alignment,
}

#[derive(Clone)]
pub struct Canvas {
    id: WidgetId,
    transform: Transform,
    left: Length,
    right: Length,
    top: Length,
    bottom: Length,
    items: Vec<CanvasItem>,
}

impl Canvas {
    #[composable]
    pub fn new() -> Canvas {
        Canvas {
            id: WidgetId::here(),
            transform: Transform::identity(),
            left: Length::Dip(f64::NEG_INFINITY),
            right: Length::Dip(f64::INFINITY),
            top: Length::Dip(f64::NEG_INFINITY),
            bottom: Length::Dip(f64::INFINITY),
            items: vec![],
        }
    }

    pub fn bounds(
        mut self,
        left: impl Into<Length>,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
    ) -> Self {
        self.set_bounds(left, top, right, bottom);
        self
    }

    pub fn set_bounds(
        &mut self,
        left: impl Into<Length>,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
    ) {
        self.left = left.into();
        self.top = top.into();
        self.right = right.into();
        self.bottom = bottom.into();
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn item(
        mut self,
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        widget: impl Widget + 'static,
    ) -> Canvas {
        self.add_item(offset_x, offset_y, widget);
        self
    }

    pub fn add_item(
        &mut self,
        offset_x: impl Into<Length>,
        offset_y: impl Into<Length>,
        widget: impl Widget + 'static,
    ) {
        self.items.push(CanvasItem {
            anchor: Alignment::CENTER,
            offset_x: offset_x.into(),
            offset_y: offset_y.into(),
            widget: Arc::new(WidgetPod::new(widget)),
        });
    }
}

impl Widget for Canvas {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        for item in self.items.iter() {
            item.widget.event(ctx, event, env)
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // a canvas always takes the maximum available space
        let width = constraints.finite_max_width().unwrap_or(0.0);
        let height = constraints.finite_max_height().unwrap_or(0.0);

        let left = self.left.to_dips(ctx.scale_factor, width);
        let top = self.top.to_dips(ctx.scale_factor, height);
        let right = self.right.to_dips(ctx.scale_factor, width);
        let bottom = self.bottom.to_dips(ctx.scale_factor, height);

        // place the items in the canvas
        for item in self.items.iter() {
            let measurements = item.widget.layout(ctx, BoxConstraints::new(.., ..), env);
            let mut offset = Offset::new(
                item.offset_x.to_dips(ctx.scale_factor, width),
                item.offset_y.to_dips(ctx.scale_factor, height),
            );

            // prevent item from going out of bounds
            // FIXME: this assumes that the top-left corner is the origin of the item
            // this is always the case *for now*
            offset.x = offset.x.clamp(left, right - measurements.width());
            offset.y = offset.y.clamp(top, bottom - measurements.height());

            let transform = offset.to_transform().then(&self.transform);
            item.widget.set_transform(transform);
        }

        //trace!("canvas size: {}x{}", width, height);
        Measurements::new(Rect::new(Point::origin(), Size::new(width, height)))
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        for item in self.items.iter() {
            item.widget.paint(ctx, env)
        }
    }
}

#[derive(Clone)]
pub struct Viewport<Contents> {
    contents: WidgetPod<Contents>,
    transform: Transform,
    constrain_width: bool,
    constrain_height: bool,
}

impl<Contents: Widget + 'static> Viewport<Contents> {
    #[composable]
    pub fn new(contents: Contents) -> Viewport<Contents> {
        Viewport {
            transform: Transform::identity(),
            contents: WidgetPod::new(contents),
            constrain_width: false,
            constrain_height: false,
        }
    }

    pub fn transform(mut self, transform: Transform) -> Self {
        self.set_transform(transform);
        self
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// Constrain the max width of the viewport to the parent.
    pub fn constrain_width(mut self) -> Self {
        self.constrain_width = true;
        self
    }

    pub fn constrain_height(mut self) -> Self {
        self.constrain_height = true;
        self
    }

    pub fn contents(&self) -> &Contents {
        self.contents.widget()
    }
}

impl<Contents: Widget + 'static> Widget for Viewport<Contents> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.contents.widget().widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.contents.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let mut child_constraints = constraints;
        if !self.constrain_width {
            child_constraints.min.width = 0.0;
            child_constraints.max.width = f64::INFINITY;
        }
        if !self.constrain_height {
            child_constraints.min.height = 0.0;
            child_constraints.max.height = f64::INFINITY;
        }

        // unconstrained
        self.contents.layout(ctx, child_constraints, env);
        self.contents.set_transform(self.transform);

        // always take the maximum available space
        let width = constraints.finite_max_width().unwrap_or(0.0);
        let height = constraints.finite_max_height().unwrap_or(0.0);
        Measurements::from(Size::new(width, height))
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.contents.paint(ctx, env)
    }
}
