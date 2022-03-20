use crate::widget::{prelude::*, LayoutWrapper};

pub struct Align<W> {
    alignment: Alignment,
    width_factor: Option<f64>,
    height_factor: Option<f64>,
    inner: LayoutWrapper<W>,
}

impl<W: Widget + 'static> Align<W> {
    pub fn new(alignment: Alignment, inner: W) -> Align<W> {
        Align {
            alignment,
            width_factor: None,
            height_factor: None,
            inner: LayoutWrapper::new(inner),
        }
    }

    pub fn width_factor(mut self, w: f64) -> Self {
        self.width_factor = Some(w);
        self
    }

    pub fn height_factor(mut self, h: f64) -> Self {
        self.height_factor = Some(h);
        self
    }

    /// Returns a reference to the inner widget.
    pub fn inner(&self) -> &W {
        self.inner.inner()
    }

    /// Returns a mutable reference to the inner widget.
    pub fn inner_mut(&mut self) -> &mut W {
        self.inner.inner_mut()
    }
}

impl<W: Widget> Widget for Align<W> {
    fn widget_id(&self) -> Option<WidgetId> {
        // inherit the identity of the contents
        self.inner.widget_id()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.inner.event(ctx, event, env);
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // measure child
        let child = self.inner.layout(ctx, constraints.loosen(), env);

        let mut size = child.size();

        if let Some(width_factor) = self.width_factor {
            size.width *= width_factor;
        } else if constraints.max_width().is_finite() {
            size.width = constraints.max_width();
        }

        if let Some(height_factor) = self.height_factor {
            size.height *= height_factor;
        } else if constraints.max_height().is_finite() {
            size.height = constraints.max_height();
        }

        // now place the child inside
        let x = 0.5 * size.width * (1.0 + self.alignment.x) - 0.5 * child.width() * (1.0 + self.alignment.x);
        let y = 0.5 * size.height * (1.0 + self.alignment.y) - 0.5 * child.height() * (1.0 + self.alignment.y);
        let baseline = child.baseline.map(|b| b + y);

        self.inner.set_offset(Offset::new(x, y));

        let bounds = Rect {
            origin: child.bounds.origin,
            size,
        };

        Measurements {
            bounds,
            // FIXME
            clip_bounds: bounds,
            baseline,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        self.inner.paint(ctx, env)
    }
}
