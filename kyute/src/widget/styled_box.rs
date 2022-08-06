use crate::{
    cache, drawing,
    drawing::{BlendMode, Paint, PaintCtxExt, RoundedRect, Shape, ToSkia},
    style,
    style::{Style, WidgetState},
    widget::prelude::*,
    PointerEventKind, SideOffsets, State,
};
use skia_safe as sk;
use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
};

pub struct StyledBox<Inner> {
    style: Style,
    computed: LayoutCache<style::ComputedStyle>,
    inner: WidgetPod<Inner>,
    hovered: State<bool>,
}

impl<Inner: Widget + 'static> StyledBox<Inner> {
    #[composable]
    pub fn new(inner: Inner, style: impl TryInto<Style>) -> Self {
        StyledBox {
            style: style.try_into().unwrap_or_else(|_| {
                warn!("Failed to parse style");
                Style::default()
            }),
            computed: Default::default(),
            inner: WidgetPod::new(inner),
            hovered: cache::state(|| false),
        }
    }

    pub fn inner(&self) -> &Inner {
        self.inner.inner()
    }

    pub fn inner_mut(&mut self) -> &mut Inner {
        self.inner.inner_mut()
    }
}

impl<Inner> Deref for StyledBox<Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        self.inner.inner()
    }
}

impl<Inner> DerefMut for StyledBox<Inner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.inner_mut()
    }
}

impl<Inner: Widget + 'static> Widget for StyledBox<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> BoxLayout {
        let _span = trace_span!("StyledBox layout", widget_id = ?self.widget_id_dbg()).entered();

        let mut widget_state = params.widget_state;
        widget_state.set(WidgetState::HOVER, self.hovered.get());

        // TODO layout cache not enough here (doesn't take into account widget state)
        let computed = if ctx.speculative {
            self.style.compute(widget_state, params, env)
        } else {
            self.computed.invalidate();
            self.computed
                .update(ctx, params, |ctx| self.style.compute(widget_state, params, env))
        };

        trace!("=== [{:?}] StyledBox layout ===", self.inner.widget_id());

        // horizontal & vertical padding, including border widths
        let padding_h = computed.layout.padding_right
            + computed.layout.padding_left
            + computed.border.border_left_width
            + computed.border.border_right_width;
        let padding_v = computed.layout.padding_top
            + computed.layout.padding_bottom
            + computed.border.border_top_width
            + computed.border.border_bottom_width;

        //dbg!(constraints);
        //trace!("computed styles: {:#?}", computed);

        // compute min/max w/h constraints

        let (min, max) = {
            let mut min_width = computed.layout.min_width.unwrap_or(params.min.width);
            let mut max_width = computed.layout.max_width.unwrap_or(params.max.width);
            let mut min_height = computed.layout.min_height.unwrap_or(params.min.height);
            let mut max_height = computed.layout.max_height.unwrap_or(params.max.height);

            // explicit width/height declarations
            if let Some(w) = computed.layout.width {
                min_width = w;
                max_width = w;
            }
            if let Some(h) = computed.layout.height {
                min_height = h;
                max_height = h;
            }

            // clamp to parent constraints & sanitize
            min_width = params.constrain_width(min_width);
            max_width = params.constrain_width(max_width);
            min_height = params.constrain_height(min_height);
            max_height = params.constrain_height(max_height);
            if min_width >= max_width {
                min_width = max_width;
            }
            if min_height >= max_height {
                min_height = max_height;
            }

            (Size::new(min_width, min_height), Size::new(max_width, max_height))
        };

        let content_max_width = (max.width - padding_h).max(0.0);
        let content_max_height = (max.height - padding_v).max(0.0);
        let content_max = Size::new(content_max_width, content_max_height);

        trace!("min: {:?}, max: {:?}, content_max: {:?}", min, max, content_max);

        // layout contents with modified constraints
        let sublayout = {
            let mut sublayout = self.inner.layout(
                ctx,
                &LayoutParams {
                    min: Size::zero(),
                    max: content_max,
                    ..*params
                },
                env,
            );

            // apply our additional padding + borders to the child box layout
            sublayout.padding_left += computed.layout.padding_left + computed.border.border_left_width;
            sublayout.padding_right += computed.layout.padding_right + computed.border.border_right_width;
            sublayout.padding_top += computed.layout.padding_top + computed.border.border_top_width;
            sublayout.padding_bottom += computed.layout.padding_bottom + computed.border.border_bottom_width;
            sublayout
        };

        // size of contents + padding + border padding
        // => adjusted content box
        let content_plus_padding = sublayout.padding_box_size();

        //---------------------------------
        // compute our box size
        let final_size = content_plus_padding.clamp(min, max);
        /*trace!(
            "content_size={:?}, sublayout={:?}, final size={}x{}",
            content_size,
            sublayout,
            width,
            height
        );*/

        let mut layout = BoxLayout::new(final_size);

        // position the adjusted content box
        let offset = sublayout.place_into(&Measurements::new(final_size));
        layout.measurements.baseline = sublayout.measurements.baseline.map(|b| b + offset.y);

        trace!("content offset={:?}", offset);
        if !ctx.speculative {
            self.inner.set_offset(offset);
        }

        // propagate positioning constraints upwards
        if let Some(top) = computed.layout.top {
            layout.y_align = Alignment::START;
            layout.padding_top = top;
        }
        if let Some(bottom) = computed.layout.bottom {
            layout.y_align = Alignment::END;
            layout.padding_bottom = bottom;
        }
        if let Some(left) = computed.layout.left {
            layout.x_align = Alignment::START;
            layout.padding_left = left;
        }
        if let Some(right) = computed.layout.right {
            layout.x_align = Alignment::END;
            layout.padding_right = right;
        }

        trace!("final layout = {:?}", layout);
        layout
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            // track pointer hover
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerOver => {
                    self.hovered.set_without_invalidation(true);
                    // TODO: only request a repaint if no layout-affecting style is touched
                    ctx.request_relayout();
                }
                PointerEventKind::PointerOut => {
                    self.hovered.set_without_invalidation(false);
                    ctx.request_relayout();
                }
                _ => {}
            },
            _ => {}
        }

        self.inner.route_event(ctx, event, env)
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        let style = self.computed.get_cached();

        let border_widths = [
            style.border.border_top_width,
            style.border.border_right_width,
            style.border.border_bottom_width,
            style.border.border_left_width,
        ];

        let outer_border_rrect = RoundedRect {
            rect: ctx.bounds,
            radii: [
                Offset::new(style.border.border_top_left_radius, style.border.border_top_left_radius),
                Offset::new(
                    style.border.border_top_right_radius,
                    style.border.border_top_right_radius,
                ),
                Offset::new(
                    style.border.border_bottom_right_radius,
                    style.border.border_bottom_right_radius,
                ),
                Offset::new(
                    style.border.border_bottom_left_radius,
                    style.border.border_bottom_left_radius,
                ),
            ],
        };
        let inner_border_rrect = outer_border_rrect.contract(border_widths);
        let outer_border_shape = Shape::RoundedRect(outer_border_rrect);
        let inner_border_shape = Shape::RoundedRect(inner_border_rrect);

        // draw drop shadows
        for box_shadow in style.box_shadow.box_shadows.iter() {
            if !box_shadow.inset {
                ctx.draw_box_shadow(&outer_border_shape, box_shadow);
            }
        }

        // fill shape with background paint
        ctx.fill_shape(&inner_border_shape, &style.background.background_image);

        // draw inset shadows
        for box_shadow in style.box_shadow.box_shadows.iter() {
            if box_shadow.inset {
                ctx.draw_box_shadow(&inner_border_shape, box_shadow);
            }
        }

        if let Some(border_style) = style.border.border_style {
            let border = drawing::Border {
                widths: border_widths,
                // TODO: support border-image and nonuniform colors
                paint: Paint::Color(style.border.border_top_color),
                line_style: border_style,
                blend_mode: BlendMode::SrcOver,
            };
            ctx.draw_border(&outer_border_shape, &border);
        }

        // draw the contents, clipped by the inner border rounded rect
        ctx.surface.canvas().save();
        ctx.surface
            .canvas()
            .clip_rrect(inner_border_rrect.to_skia(), sk::ClipOp::Intersect, true);
        self.inner.paint(ctx);
        ctx.surface.canvas().restore();
    }
}
