use crate::{
    style::{BoxStyle, PaintCtxExt},
    widget::{prelude::*, LayoutWrapper},
    Length, SideOffsets, UnitExt, ValueRef,
};

#[derive(Clone)]
pub struct Container<Content> {
    alignment: Option<Alignment>,
    min_width: Option<ValueRef<Length>>,
    min_height: Option<ValueRef<Length>>,
    max_width: Option<ValueRef<Length>>,
    max_height: Option<ValueRef<Length>>,
    baseline: Option<ValueRef<Length>>,
    content_padding: ValueRef<SideOffsets>,
    box_style: ValueRef<BoxStyle>,
    content: LayoutWrapper<Content>,
}

impl<Content: Widget + 'static> Container<Content> {
    #[composable]
    pub fn new(content: Content) -> Container<Content> {
        Container {
            alignment: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            baseline: None,
            content_padding: Default::default(),
            box_style: BoxStyle::default().into(),
            content: LayoutWrapper::new(content),
        }
    }

    /// Returns the offset of the contents after layout.
    ///
    /// The returned value is unspecified if this function is called before layout.
    pub fn content_offset(&self) -> Offset {
        self.content.offset()
    }

    /// Returns a reference to the contents.
    pub fn contents(&self) -> &Content {
        self.content.inner()
    }

    /// Returns a mutable reference to the contents.
    pub fn contents_mut(&mut self) -> &mut Content {
        self.content.inner_mut()
    }
}

impl<Content: Widget + 'static> Container<Content> {
    /// Sets the baseline of the content.
    pub fn baseline(mut self, baseline: impl Into<ValueRef<Length>>) -> Self {
        self.set_baseline(baseline);
        self
    }

    /// Sets the baseline of the content.
    pub fn set_baseline(&mut self, baseline: impl Into<ValueRef<Length>>) {
        self.baseline = Some(baseline.into());
    }

    /// Constrain the minimum width of the container.
    pub fn min_width(mut self, width: impl Into<ValueRef<Length>>) -> Self {
        self.set_min_width(width);
        self
    }

    /// Constrain the minimum width of the container.
    pub fn set_min_width(&mut self, width: impl Into<ValueRef<Length>>) {
        self.min_width = Some(width.into());
    }

    /// Constrain the minimum height of the container.
    pub fn min_height(mut self, height: impl Into<ValueRef<Length>>) -> Self {
        self.set_min_height(height);
        self
    }

    /// Constrain the minimum height of the container.
    pub fn set_min_height(&mut self, height: impl Into<ValueRef<Length>>) {
        self.min_height = Some(height.into());
    }

    /// Constrain the width of the container.
    pub fn fixed_width(mut self, width: impl Into<ValueRef<Length>>) -> Self {
        self.set_fixed_width(width);
        self
    }

    /// Constrain the width of the container.
    pub fn set_fixed_width(&mut self, width: impl Into<ValueRef<Length>>) {
        let w = width.into();
        self.min_width = Some(w);
        self.max_width = Some(w);
    }

    /// Constrain the width of the container.
    pub fn fixed_height(mut self, height: impl Into<ValueRef<Length>>) -> Self {
        self.set_fixed_height(height);
        self
    }

    /// Constrain the width of the container.
    pub fn set_fixed_height(&mut self, height: impl Into<ValueRef<Length>>) {
        let h = height.into();
        self.min_height = Some(h);
        self.max_height = Some(h);
    }

    pub fn fix_size(mut self, size: Size) -> Self {
        self.set_fixed_size(size);
        self
    }

    pub fn set_fixed_size(&mut self, size: Size) {
        self.min_width = Some(size.width.dip().into());
        self.max_width = Some(size.width.dip().into());
        self.min_height = Some(size.height.dip().into());
        self.max_height = Some(size.height.dip().into());
    }

    /// Centers the content in the available space.
    pub fn centered(mut self) -> Self {
        self.set_centered();
        self
    }

    /// Centers the content in the available space.
    pub fn set_centered(&mut self) {
        self.alignment = Some(Alignment::CENTER);
    }

    /// Aligns the widget in the available space.
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Aligns the widget in the available space.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = Some(alignment);
    }

    /// Aligns the widget in the available space.
    pub fn content_padding(mut self, padding: impl Into<ValueRef<SideOffsets>>) -> Self {
        self.set_content_padding(padding);
        self
    }

    /// Aligns the widget in the available space.
    pub fn set_content_padding(&mut self, padding: impl Into<ValueRef<SideOffsets>>) {
        self.content_padding = padding.into();
    }

    /// Sets the style used to paint the box of the container.
    pub fn box_style(mut self, box_style: impl Into<ValueRef<BoxStyle>>) -> Self {
        self.set_box_style(box_style);
        self
    }

    /// Sets the style used to paint the box of the container.
    pub fn set_box_style(&mut self, box_style: impl Into<ValueRef<BoxStyle>>) {
        self.box_style = box_style.into();
    }
}

impl<Content: Widget> Widget for Container<Content> {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        self.content.event(ctx, event, env)
    }

    fn layout(&self, ctx: &mut LayoutCtx, mut constraints: BoxConstraints, env: &Environment) -> Measurements {
        // First, measure the child, taking into account the mandatory padding
        let content_padding = self.content_padding.resolve(env).unwrap();
        let content_constraints = constraints.deflate(content_padding);
        let mut content_size = self.content.layout(ctx, content_constraints, env);
        content_size.bounds = content_size.bounds.outer_rect(content_padding);
        let mut content_offset = Offset::new(content_padding.left, content_padding.top);

        // adjust content baseline so that `baseline = adjusted_content_baseline + padding.top`.
        if let Some(baseline) = self.baseline {
            let baseline = (baseline.resolve(env).unwrap().to_dips(ctx.scale_factor) - content_offset.y).max(0.0);
            let offset = baseline - content_size.baseline.unwrap_or(content_size.bounds.size.height).round();
            content_offset.y += offset;
            content_size.bounds.size.height += offset;
        }

        // measure text
        // add padding
        // baseline adjustment:

        // the sizing constraints cannot result in a container that is smaller
        // than the content size (the content takes priority when sizing the container).
        /* additional_constraints.max.width =
            additional_constraints.max.width.max(content_size.width());
        additional_constraints.max.height =
            additional_constraints.max.height.max(content_size.height());*/

        fn clamp(x: f64, min: f64, max: f64) -> f64 {
            x.max(min).min(max)
        }

        constraints.min.width = clamp(content_size.width(), constraints.min.width, constraints.max.width);
        constraints.min.height = clamp(content_size.height(), constraints.min.height, constraints.max.height);

        // apply additional w/h sizing constraints to the container
        //let mut additional_constraints = BoxConstraints::new(..,..);
        if let Some(w) = self.min_width {
            let w = w.resolve(env).unwrap().to_dips(ctx.scale_factor);
            constraints.min.width = clamp(w, constraints.min.width, constraints.max.width);
        }
        if let Some(w) = self.max_width {
            let w = w.resolve(env).unwrap().to_dips(ctx.scale_factor);
            constraints.max.width = clamp(w, constraints.min.width, constraints.max.width);
        }
        if let Some(h) = self.min_height {
            let h = h.resolve(env).unwrap().to_dips(ctx.scale_factor);
            constraints.min.height = clamp(h, constraints.min.height, constraints.max.height);
        }
        if let Some(h) = self.max_height {
            let h = h.resolve(env).unwrap().to_dips(ctx.scale_factor);
            constraints.max.height = clamp(h, constraints.min.height, constraints.max.height);
        }

        // now determine our width and height
        // basically: do we stretch or do we size to contents?
        // -> if we have alignment and we are bounded: stretch
        // -> otherwise (if we don't have alignment, or we are unbounded): size to contents
        let size = if self.alignment.is_some() {
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
            let x = 0.5 * size.width * (1.0 + alignment.x) - 0.5 * content_size.width() * (1.0 + alignment.x);
            let y = 0.5 * size.height * (1.0 + alignment.y) - 0.5 * content_size.height() * (1.0 + alignment.y);
            content_offset.x += x;
            content_offset.y += y;
        }

        self.content.set_offset(content_offset);

        Measurements {
            bounds: Rect {
                origin: Point::origin(),
                size,
            },
            baseline: content_size.baseline.map(|b| b + content_offset.y),
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let style = self.box_style.resolve(env).unwrap();
        ctx.draw_styled_box(bounds, &style, env);
        self.content.paint(ctx, bounds, env);
    }
}
