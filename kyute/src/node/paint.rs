use crate::event::InputState;
use crate::node::event::FocusState;
use crate::node::NodeTree;
use crate::{env, Point, Rect, Size};
use crate::{style, Measurements, Offset};
use generational_indextree::NodeId;
use kyute_shell::drawing::{Brush, Color, DrawContext, Transform};
use kyute_shell::platform::Platform;
use std::ops::{Deref, DerefMut};

/// Context passed to [`Visual::paint`].
pub struct PaintCtx<'a> {
    platform: &'a Platform,
    pub(crate) draw_ctx: &'a mut DrawContext,
    style_collection: &'a style::StyleCollection,
    window_bounds: Rect,
    node_id: NodeId,
    focus_state: &'a FocusState,
    input_state: &'a InputState,
    hover: bool,
    focus: bool,
}

impl<'a> PaintCtx<'a> {
    pub fn platform(&self) -> &Platform {
        self.platform
    }

    /// Returns the window bounds of the node
    pub fn window_bounds(&self) -> Rect {
        self.window_bounds
    }

    /// Returns the bounds of the node.
    pub fn bounds(&self) -> Rect {
        Rect::new(Point::origin(), self.window_bounds.size)
    }

    /// Returns the size of the node.
    pub fn size(&self) -> Size {
        self.window_bounds.size
    }

    pub fn is_hovering(&self) -> bool {
        self.hover
    }

    pub fn is_focused(&self) -> bool {
        self.focus
    }

    pub fn is_capturing_pointer(&self) -> bool {
        self.focus_state.pointer_grab == Some(self.node_id)
    }

    /// Returns the style collection.
    pub fn style_collection(&self) -> &style::StyleCollection {
        self.style_collection
    }

    /// Draws in the bounds using the given style set.
    pub fn draw_styled_box_in_bounds(
        &mut self,
        style_set: &str,
        bounds: Rect,
        palette: style::PaletteIndex,
    ) {
        let mut state_bits = style::State::empty();
        if self.focus {
            state_bits |= style::State::FOCUS;
        }
        if self.hover {
            state_bits |= style::State::HOVER;
        }
        self.style_collection.draw(
            self.platform,
            self.draw_ctx,
            bounds,
            style_set,
            state_bits,
            palette,
        );
    }

    /// Draws in the bounds using the given style set.
    pub fn draw_styled_box(&mut self, style_set: &str, palette: style::PaletteIndex) {
        self.draw_styled_box_in_bounds(style_set, self.bounds(), palette)
    }
}

// PaintCtx auto-derefs to a DrawContext
impl<'a> Deref for PaintCtx<'a> {
    type Target = DrawContext;

    fn deref(&self) -> &Self::Target {
        self.draw_ctx
    }
}

impl<'a> DerefMut for PaintCtx<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draw_ctx
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DebugLayout {
    None,
    All,
    Hover,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PaintOptions {
    pub debug_draw_bounds: DebugLayout,
}

impl Default for PaintOptions {
    fn default() -> Self {
        PaintOptions {
            debug_draw_bounds: DebugLayout::None,
        }
    }
}

/// Draws a rectangle that represents the given bounds.
fn draw_layout(draw_context: &mut DrawContext, m: Measurements) {
    let px = 1.0 / draw_context.scale_factor();
    let brush = Brush::new_solid_color(draw_context, Color::new(1.0, 0.0, 0.0, 1.0));
    let baseline_brush = Brush::new_solid_color(draw_context, Color::new(0.0, 1.0, 0.0, 1.0));
    let rect = Rect::from_size(m.size);
    draw_context.draw_rectangle(rect.inflate(-0.5, -0.5), &brush, 1.0 * px);
    if let Some(baseline) = m.baseline {
        let baseline_rect = Rect::new(
            rect.origin + Offset::new(1.0 * px, baseline),
            Size::new(m.size.width - 2.0 * px, px),
        );
        draw_context.fill_rectangle(baseline_rect, &baseline_brush);
    }
}

impl NodeTree {
    /// Painting.
    pub fn paint(
        &mut self,
        platform: &Platform,
        draw_context: &mut DrawContext,
        style_collection: &style::StyleCollection,
        input_state: &InputState,
        options: &PaintOptions,
    ) {
        self.paint_node(
            platform,
            draw_context,
            style_collection,
            Offset::zero(),
            input_state,
            self.root,
            options,
        )
    }

    /// Draws the node in the specified drawing context.
    ///
    /// Effectively, it applies the transform of the node (which, right now, is only an offset relative to the parent),
    /// and calls [`Visual::paint`] on `self.visual`.
    fn paint_node(
        &mut self,
        platform: &Platform,
        draw_context: &mut DrawContext,
        style_collection: &style::StyleCollection,
        offset: Offset,
        input_state: &InputState,
        node_id: NodeId,
        options: &PaintOptions,
    ) {
        let node = self.arena[node_id].get_mut();
        let node_offset = node.offset;
        let node_measurements = node.measurements;
        let node_size = node_measurements.size;
        let window_bounds = Rect::new(Point::origin() + offset + node_offset, node_size);

        let hover = input_state
            .pointers
            .iter()
            .any(|(_, state)| window_bounds.contains(state.position));

        draw_context.save();
        draw_context.transform(&node_offset.to_transform());

        {
            let mut ctx = PaintCtx {
                platform,
                draw_ctx: draw_context,
                style_collection,
                window_bounds,
                node_id,
                focus_state: &self.focus,
                input_state,
                hover,
                focus: self.focus.focus == Some(node_id),
            };
            let env = &node.env;
            node.visual.as_mut().map(|v| v.paint(&mut ctx, &env));
        }

        // paint children
        let mut child_id = self.arena[node_id].first_child();
        while let Some(id) = child_id {
            self.paint_node(
                platform,
                draw_context,
                style_collection,
                offset + node_offset,
                input_state,
                id,
                options,
            );
            child_id = self.arena[id].next_sibling();
        }

        // bounds debugging
        match options.debug_draw_bounds {
            DebugLayout::Hover if hover => {
                draw_layout(draw_context, node_measurements);
            }
            DebugLayout::All => {
                draw_layout(draw_context, node_measurements);
            }
            _ => {}
        }

        draw_context.restore();
    }
}
