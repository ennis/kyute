use crate::{
    animation::layer::Layer,
    style::{BoxStyle, Paint, PaintCtxExt, VisualState},
    widget::prelude::*,
    Length, RoundToPixel, SideOffsets, UnitExt, ValueRef,
};

struct ContainerLayerDelegate {
    box_style: BoxStyle,
}

impl ContainerLayerDelegate {
    fn new(box_style: BoxStyle) -> ContainerLayerDelegate {
        ContainerLayerDelegate { box_style }
    }
}

impl LayerDelegate for ContainerLayerDelegate {
    fn draw(&self, ctx: &mut PaintCtx) {
        ctx.draw_styled_box(ctx.bounds, &self.box_style)
    }
}

#[derive(Clone)]
pub struct Container<Content> {
    layer: LayerHandle,
    alignment: Option<Alignment>,
    min_width: Option<Length>,
    min_height: Option<Length>,
    max_width: Option<Length>,
    max_height: Option<Length>,
    baseline: Option<Length>,
    padding_top: Length,
    padding_right: Length,
    padding_bottom: Length,
    padding_left: Length,
    box_style: ValueRef<BoxStyle>,
    alternate_box_styles: Vec<(VisualState, ValueRef<BoxStyle>)>,
    redraw_on_hover: bool,
    content: Content,
}

impl<Content: Widget + 'static> Container<Content> {
    #[composable]
    pub fn new(content: Content) -> Container<Content> {
        Container {
            layer: Layer::new(),
            alignment: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            baseline: None,
            padding_top: Length::zero(),
            padding_right: Length::zero(),
            padding_bottom: Length::zero(),
            padding_left: Length::zero(),
            box_style: BoxStyle::default().into(),
            alternate_box_styles: vec![],
            redraw_on_hover: false,
            content,
        }
    }

    /// Returns the offset of the contents after layout.
    ///
    /// The returned value is unspecified if this function is called before layout.
    pub fn content_offset(&self) -> Offset {
        let transform = self.content.layer().transform();
        Offset::new(transform.m31, transform.m32)
    }

    /// Returns a reference to the contents.
    pub fn inner(&self) -> &Content {
        &self.content
    }

    /// Returns a mutable reference to the contents.
    pub fn inner_mut(&mut self) -> &mut Content {
        &mut self.content
    }
}

impl<Content: Widget + 'static> Container<Content> {
    /// Sets the baseline of the content.
    #[must_use]
    pub fn baseline(mut self, baseline: impl Into<Length>) -> Self {
        self.set_baseline(baseline);
        self
    }

    /// Sets the baseline of the content.
    pub fn set_baseline(&mut self, baseline: impl Into<Length>) {
        self.baseline = Some(baseline.into());
    }

    /// Constrain the minimum width of the container.
    #[must_use]
    pub fn min_width(mut self, width: impl Into<Length>) -> Self {
        self.set_min_width(width);
        self
    }

    /// Constrain the minimum width of the container.
    pub fn set_min_width(&mut self, width: impl Into<Length>) {
        self.min_width = Some(width.into());
    }

    /// Constrain the minimum height of the container.
    #[must_use]
    pub fn min_height(mut self, height: impl Into<Length>) -> Self {
        self.set_min_height(height);
        self
    }

    /// Constrain the minimum height of the container.
    pub fn set_min_height(&mut self, height: impl Into<Length>) {
        self.min_height = Some(height.into());
    }

    /// Constrain the width of the container.
    #[must_use]
    pub fn fixed_width(mut self, width: impl Into<Length>) -> Self {
        self.set_fixed_width(width);
        self
    }

    /// Constrain the width of the container.
    pub fn set_fixed_width(&mut self, width: impl Into<Length>) {
        let w = width.into();
        self.min_width = Some(w);
        self.max_width = Some(w);
    }

    /// Constrain the width of the container.
    #[must_use]
    pub fn fixed_height(mut self, height: impl Into<Length>) -> Self {
        self.set_fixed_height(height);
        self
    }

    /// Constrain the width of the container.
    pub fn set_fixed_height(&mut self, height: impl Into<Length>) {
        let h = height.into();
        self.min_height = Some(h);
        self.max_height = Some(h);
    }

    #[must_use]
    pub fn fix_size(mut self, size: Size) -> Self {
        self.set_fixed_size(size);
        self
    }

    pub fn set_fixed_size(&mut self, size: Size) {
        self.min_width = Some(size.width.dip());
        self.max_width = Some(size.width.dip());
        self.min_height = Some(size.height.dip());
        self.max_height = Some(size.height.dip());
    }

    /// Fills the available space.
    ///
    /// Equivalent to `self.fixed_width(100.percent()).fixed_height(100.percent())`
    #[must_use]
    pub fn fill(mut self) -> Self {
        self.set_fixed_width(100.percent());
        self.set_fixed_height(100.percent());
        self
    }

    /// Centers the content in the available space.
    #[must_use]
    pub fn centered(mut self) -> Self {
        self.set_centered();
        self
    }

    /// Centers the content in the available space.
    pub fn set_centered(&mut self) {
        self.alignment = Some(Alignment::CENTER);
    }

    /// Aligns the widget in the available space.
    #[must_use]
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Aligns the widget in the available space.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = Some(alignment);
    }

    /// Aligns the widget in the available space.
    #[must_use]
    pub fn content_padding(mut self, top: Length, right: Length, bottom: Length, left: Length) -> Self {
        self.set_content_padding(top, right, bottom, left);
        self
    }

    /// Aligns the widget in the available space.
    pub fn set_content_padding(&mut self, top: Length, right: Length, bottom: Length, left: Length) {
        self.padding_top = top;
        self.padding_right = right;
        self.padding_bottom = bottom;
        self.padding_left = left;
    }

    /// Sets the background color. Overrides any previously set box style.
    pub fn background(mut self, paint: impl Into<Paint>) -> Self {
        self.set_box_style(BoxStyle::new().fill(paint));
        self
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

    /// Adds an alternate style, which replaces the main style when the widget is in the specified state.
    pub fn alternate_box_style(mut self, state: VisualState, box_style: impl Into<ValueRef<BoxStyle>>) -> Self {
        self.push_alternate_box_style(state, box_style);
        self
    }

    /// Sets the overlay style, only active when the widget is in the specified state.
    pub fn push_alternate_box_style(&mut self, state: VisualState, box_style: impl Into<ValueRef<BoxStyle>>) {
        self.alternate_box_styles.push((state, box_style.into()));
        if state.contains(VisualState::HOVER) {
            self.redraw_on_hover = true;
        }
    }
}

impl<Content: Widget> Widget for Container<Content> {
    fn widget_id(&self) -> Option<WidgetId> {
        // inherit the identity of the contents
        self.content.widget_id()
    }

    fn layer(&self) -> &LayerHandle {
        &self.layer
    }

    fn layout(&self, ctx: &mut LayoutCtx, mut constraints: BoxConstraints, env: &Environment) -> Measurements {
        let box_style = self.box_style.resolve(env).unwrap();

        // Base size for proportional length calculations
        let base_width = constraints.finite_max_width().unwrap_or(0.0);
        let base_height = constraints.finite_max_height().unwrap_or(0.0);

        // First, measure the child, taking into account the mandatory padding
        let mut content_padding = SideOffsets::new(
            self.padding_top.to_dips(ctx.scale_factor, base_height),
            self.padding_right.to_dips(ctx.scale_factor, base_width),
            self.padding_bottom.to_dips(ctx.scale_factor, base_height),
            self.padding_left.to_dips(ctx.scale_factor, base_width),
        );

        // Around-borders should be taken into account in the layout (they affect the size of the item).
        content_padding =
            content_padding + box_style.border_side_offsets(ctx.scale_factor, Size::new(base_width, base_height));

        let content_constraints = constraints.deflate(content_padding);

        let mut content_size = self.content.layout(ctx, content_constraints, env);
        content_size.size = content_size.local_bounds().outer_rect(content_padding).size;

        let mut content_offset = Offset::new(content_padding.left, content_padding.top);

        // adjust content baseline so that `baseline = adjusted_content_baseline + padding.top`.
        if let Some(baseline) = self.baseline {
            // TODO do size-relative baselines make sense?
            let baseline = (baseline.to_dips(ctx.scale_factor, base_height) - content_offset.y).max(0.0);
            let offset = baseline - content_size.baseline.unwrap_or(content_size.size.height).round();
            content_offset.y += offset;
            content_size.size.height += offset;
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

        constraints.min.width = clamp(content_size.size.width, constraints.min.width, constraints.max.width);
        constraints.min.height = clamp(content_size.size.height, constraints.min.height, constraints.max.height);

        // apply additional w/h sizing constraints to the container
        //let mut additional_constraints = BoxConstraints::new(..,..);
        if let Some(w) = self.min_width {
            let w = w.to_dips(ctx.scale_factor, base_width);
            constraints.min.width = clamp(w, constraints.min.width, constraints.max.width);
        }
        if let Some(w) = self.max_width {
            let w = w.to_dips(ctx.scale_factor, base_width);
            constraints.max.width = clamp(w, constraints.min.width, constraints.max.width);
        }
        if let Some(h) = self.min_height {
            let h = h.to_dips(ctx.scale_factor, base_height);
            constraints.min.height = clamp(h, constraints.min.height, constraints.max.height);
        }
        if let Some(h) = self.max_height {
            let h = h.to_dips(ctx.scale_factor, base_height);
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
            constraints.constrain(content_size.size)
        };

        // Place the contents inside the box according to alignment
        if let Some(alignment) = self.alignment {
            let x = 0.5 * size.width * (1.0 + alignment.x) - 0.5 * content_size.width() * (1.0 + alignment.x);
            let y = 0.5 * size.height * (1.0 + alignment.y) - 0.5 * content_size.height() * (1.0 + alignment.y);
            content_offset.x += x;
            content_offset.y += y;
        }

        // finally, round to pixel boundaries
        content_offset = content_offset.round_to_pixel(ctx.scale_factor);

        self.content.layer().set_offset(content_offset);

        let box_style = self.box_style.resolve(env).unwrap();
        let clip_bounds = box_style.clip_bounds(Rect::new(Point::origin(), size), ctx.scale_factor);

        // update layer
        self.layer.set_size(size);
        self.layer.add_child(self.content.layer());
        // TODO update_delegate
        self.layer.set_delegate(ContainerLayerDelegate { box_style });

        Measurements {
            size,
            clip_bounds,
            baseline: content_size.baseline.map(|b| b + content_offset.y),
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // if one of our alternate styles depend on HOVER, we must request a repaint on pointer over/out
        /*let redraw_on_hover = self
            .alternate_box_styles
            .iter()
            .any(|(state, _)| state.contains(VisualState::HOVER));

        if redraw_on_hover
            && matches!(
                event,
                Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerOver | PointerEventKind::PointerOut,
                    ..
                })
            )
        {
            ctx.request_redraw();
        }*/

        // route_event:
        // - by default, applies the layer transform, handle routing, and calls event
        // - for WidgetPods: also apply filter

        //ctx.route_event(&self.content, event, env);
        self.content.route_event(ctx, event, env)
    }
}
