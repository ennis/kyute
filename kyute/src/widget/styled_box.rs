use crate::{
    drawing,
    drawing::{BlendMode, Paint, PaintCtxExt, RoundedRect, Shape},
    style,
    style::Style,
    widget::prelude::*,
};
use kyute_common::SideOffsets;
use std::convert::TryInto;

pub struct StyledBox<Inner> {
    style: Style,
    computed: LayoutCache<style::ComputedStyle>,
    inner: WidgetPod<Inner>,
}

impl<Inner: Widget + 'static> StyledBox<Inner> {
    pub fn new(inner: Inner, style: impl TryInto<Style>) -> Self {
        StyledBox {
            style: style.try_into().unwrap_or_else(|_| {
                warn!("Failed to parse style");
                Style::default()
            }),
            computed: Default::default(),
            inner: WidgetPod::new(inner),
        }
    }
}

impl<Inner: Widget + 'static> Widget for StyledBox<Inner> {
    fn widget_id(&self) -> Option<WidgetId> {
        self.inner.widget_id()
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Layout {
        let computed = self
            .computed
            .update(ctx, constraints, |_| self.style.compute(constraints));

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

        trace!("computed styles: {:#?}", computed);

        // compute actual min/max heights
        let mut min_width = computed
            .layout
            .min_width
            .unwrap_or(constraints.min.width)
            .max(constraints.min.width);
        let mut max_width = computed
            .layout
            .max_width
            .unwrap_or(constraints.max.width)
            .min(constraints.max.width);
        let mut min_height = computed
            .layout
            .min_height
            .unwrap_or(constraints.min.height)
            .max(constraints.min.height);
        let mut max_height = computed
            .layout
            .max_height
            .unwrap_or(constraints.max.height)
            .min(constraints.max.height);
        if let Some(width) = computed.layout.width {
            let w = width.clamp(min_width, max_width);
            min_width = w;
            max_width = w;
        }
        if let Some(height) = computed.layout.height {
            let h = height.clamp(min_height, max_height);
            min_height = h;
            max_height = h;
        }

        let content_max_width = (max_width - padding_h).max(0.0);
        let content_max_height = (max_height - padding_v).max(0.0);

        trace!(
            "max: {}x{}, content_max: {}x{}",
            max_width,
            max_height,
            content_max_width,
            content_max_height
        );

        // layout contents with modified constraints
        let sublayout = self.inner.layout(
            ctx,
            &LayoutConstraints {
                min: Size::zero(),
                max: Size::new(content_max_width, content_max_height),
                ..*constraints
            },
            env,
        );

        // the content may include extra padding, in addition to the padding specified by this widget
        let content_size = sublayout.padding_box_size();

        //---------------------------------
        // compute our box size
        let width = (content_size.width + padding_h).clamp(min_width, max_width);
        let height = (content_size.height + padding_v).clamp(min_height, max_height);
        trace!(
            "content_size={:?}, sublayout={:?}, final size={}x{}",
            content_size,
            sublayout,
            width,
            height
        );

        if !ctx.speculative {
            // position the contents inside the "content area box", which is the final box minus
            // the padding. The "content area box" may be different from the "content box" if
            // the width and height constraints force this widget to be bigger than the content + padding.
            let content_area_size = Size::new(width - padding_h, height - padding_v);
            let offset = sublayout.content_box_offset(content_area_size)
                + Offset::new(
                    computed.layout.padding_left + computed.border.border_left_width,
                    computed.layout.padding_top + computed.border.border_top_width,
                );
            trace!("content offset={:?}", offset);
            self.inner.set_offset(offset);
        }

        let mut layout = Layout::new(Size::new(width, height));

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

        trace!("final layout ={:#?}", layout);

        layout
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
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

        // TODO: clip to inner border rect
        self.inner.paint(ctx);
    }
}
