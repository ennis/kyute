//! Text editor widget.
use crate::event::Event;
use crate::layout::{BoxConstraints, EdgeInsets, Layout, Size};
use crate::renderer::Theme;
use crate::visual::{EventCtx, NodeArena, NodeCursor, NodeData, PaintCtx, Visual};
use crate::widget::frame::Frame;
use crate::widget::padding::Padding;
use crate::widget::LayoutCtx;
use crate::{Bounds, BoxedWidget, Point, Widget, WidgetExt};
use bitflags::_core::mem::transmute_copy;
use generational_indextree::NodeId;
use kyute_shell::drawing::{Color, DrawTextOptions, Rect, RectExt, SolidColorBrush};
use kyute_shell::text::{TextFormat, TextFormatBuilder, TextLayout};
use log::trace;
use palette::{Srgb, Srgba};
use std::any::Any;
use std::ops::Range;
use unicode_segmentation::GraphemeCursor;
use winit::event::VirtualKeyCode;
use kyute_shell::drawing::context::{FloodImage, CompositeMode, InterpolationMode};

/// Text selection.
///
/// Start is the start of the selection, end is the end. The caret is at the end of the selection.
/// Note that we don't necessarily have start <= end: a selection with start > end means that the
/// user started the selection gesture from a later point in the text and then went back
/// (right-to-left in LTR languages). In this case, the cursor will appear at the "beginning"
/// (i.e. left, for LTR) of the selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn min(&self) -> usize {
        self.start.min(self.end)
    }
    pub fn max(&self) -> usize {
        self.start.max(self.end)
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    pub fn empty(at: usize) -> Selection {
        Selection { start: at, end: at }
    }
}

// layout strategy:
// - the text layout is calculated during widget layout, but also when an event causes the text to
//   change
// - update the text layout during painting if necessary

pub struct TextEditVisual {
    /// The text displayed to the user.
    text: String,

    /// The text layout.
    ///
    /// FIXME: due to DirectWrite limitations, the text layout contains a copy of the string.
    /// in the future, de-duplicate.
    text_layout: TextLayout,

    /// The currently selected range. If no text is selected, this is a zero-length range
    /// at the cursor position.
    selection: Selection,

    /// Flag that indicates that the text inside the box needs to be relayouted
    needs_text_relayout: bool,

    /// Flag that indicates that the visual needs to be repainted.
    /// Q: Could also be a return value of the methods of visual.
    needs_repaint: bool,
}

pub enum Movement {
    Left,
    Right,
    LeftWord,
    RightWord,
}

fn prev_grapheme_cluster(text: &str, offset: usize) -> Option<usize> {
    let mut c = GraphemeCursor::new(offset, text.len(), true);
    c.prev_boundary(&text, 0).unwrap()
}

fn next_grapheme_cluster(text: &str, offset: usize) -> Option<usize> {
    let mut c = GraphemeCursor::new(offset, text.len(), true);
    c.next_boundary(&text, 0).unwrap()
}

impl TextEditVisual {
    /// Moves the cursor forward or backward.
    pub fn move_cursor(&mut self, movement: Movement, modify_selection: bool) {
        let offset =
            match movement {
                Movement::Left => prev_grapheme_cluster(&self.text, self.selection.end)
                    .unwrap_or(self.selection.end),
                Movement::Right => next_grapheme_cluster(&self.text, self.selection.end)
                    .unwrap_or(self.selection.end),
                Movement::LeftWord | Movement::RightWord => {
                    // TODO word navigation (unicode word segmentation)
                    unimplemented!()
                }
            };

        if modify_selection {
            self.selection.end = offset;
        } else {
            self.selection = Selection::empty(offset);
        }

        self.needs_repaint = true;
        // reset blink
        // need repaint
        // no need layout
    }

    /// Inserts text.
    pub fn insert(&mut self, text: &str) {
        let min = self.selection.min();
        let max = self.selection.max();
        self.text.replace_range(min..max, text);
        self.selection = Selection::empty(min + text.len());
        self.needs_text_relayout = true;
        self.needs_repaint = true;
    }

    /*/// Removes text.
    pub fn delete(&mut self) {
        if self.selection.is_empty() {
            self.move_cursor(Movement::Right, true);
        }
        if self.selection.is_empty() {
            // still empty? we are at the end of the document, nothing to delete
            return;
        }
        self.insert("");
        self.needs_text_relayout = true;
        self.needs_repaint = true;
        // reset blink
        // need layout
    }*/

    /// Sets cursor position.
    pub fn set_cursor(&mut self, pos: usize) {
        if self.selection.is_empty() && self.selection.end == pos {
            return;
        }
        self.selection = Selection::empty(pos);
        self.needs_repaint = true;
        // reset blink
    }

    pub fn set_selection_end(&mut self, pos: usize) {
        if self.selection.end == pos {
            return;
        }
        self.selection.end = pos;
        self.needs_repaint = true;
        // reset blink
    }

    fn position_to_text(&mut self, pos: Point) -> usize {
        let hit = self.text_layout.hit_test_point(pos).unwrap();
        let pos = if hit.is_trailing_hit {
            hit.metrics.text_position + hit.metrics.length
        } else {
            hit.metrics.text_position
        };
        pos
    }
}

impl Visual for TextEditVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let size = ctx.size;

        if self.needs_text_relayout {
            trace!("text relayout");
            // relayout text
            self.text_layout =
                TextLayout::new(ctx.platform(), &self.text, &theme.label_text_format, size)
                    .unwrap();
            self.needs_text_relayout = false;
        }

        let text_color = Color::new(0.0, 0.0, 0.0, 1.0);
        let selected_bg_color = Color::new(0.0, 0.0, 0.0, 1.0);
        let text_brush = SolidColorBrush::new(ctx, text_color);
        let caret_brush = SolidColorBrush::new(ctx, text_color);
        let selected_bg_brush = SolidColorBrush::new(ctx, selected_bg_color);
        let selected_text_brush = SolidColorBrush::new(ctx, Color::new(1.0, 1.0, 1.0, 1.0));

        // text color
        self.text_layout.set_drawing_effect(&text_brush, ..);
        if !self.selection.is_empty() {
            // FIXME slightly changes the layout when the selection straddles a kerning pair?
            self.text_layout.set_drawing_effect(
                &selected_text_brush,
                self.selection.min()..self.selection.max(),
            );
        }

        // selection highlight
        if !self.selection.is_empty() {
            let selected_areas = self
                .text_layout
                .hit_test_text_range(self.selection.min()..self.selection.max(), &Point::origin())
                .unwrap();
            for sa in selected_areas {
                ctx.fill_rectangle(sa.bounds.round_out(), &selected_bg_brush);
            }
        }

        // text
        ctx.draw_text_layout(
            Point::origin(),
            &self.text_layout,
            &text_brush,
            DrawTextOptions::ENABLE_COLOR_FONT,
        );

        // caret
        /*if ctx.is_focused() {
            trace!("has focus");
            let caret_hit_test = self
                .text_layout
                .hit_test_text_position(self.selection.end)
                .unwrap();

            //dbg!(caret_hit_test);
            ctx.fill_rectangle(
                Rect::new(
                    caret_hit_test.point.floor(),
                    Size::new(1.0, caret_hit_test.metrics.bounds.size.height),
                ),
                &caret_brush,
            );
        }*/

        // selection highlight V2 (ugly with cleartype)
        /*if !self.selection.is_empty() {
            let selected_areas = self
                .text_layout
                .hit_test_text_range(self.selection.min()..self.selection.max(), &Point::origin())
                .unwrap();
            for sa in selected_areas {
                let fill = FloodImage::new(ctx, Color::new(1.0, 1.0, 1.0, 1.0));
                let round_bounds = sa.bounds.round_out();
                ctx.draw_image(&fill, round_bounds.origin, Rect::new(Point::origin(), round_bounds.size),
                               InterpolationMode::NearestNeighbor,
                               CompositeMode::MaskInvert);
            }
        }*/

        self.needs_repaint = false;
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::FocusIn => {
                trace!("focus in");
                ctx.request_redraw();
            }
            Event::FocusOut => {
                trace!("focus out");
                let pos = self.selection.end;
                self.set_cursor(pos);
            }
            Event::PointerDown(p) => {
                let pos = self.position_to_text(p.position);
                self.set_cursor(pos);
                ctx.request_focus();
                ctx.capture_pointer();
            }
            Event::PointerMove(p) => {
                // update selection
                if ctx.is_capturing_pointer() {
                    let pos = self.position_to_text(p.position);
                    self.set_selection_end(pos);
                    trace!("selection: {:?}", self.selection)
                }
            }
            Event::PointerUp(p) => {
                // nothing to do (pointer grab automatically ends)
            }
            Event::KeyDown(k) => {
                if let Some(vk) = k.key {
                    match vk {
                        VirtualKeyCode::Back => {
                            if self.selection.is_empty() {
                                self.move_cursor(Movement::Left, true);
                            }
                            self.insert("");
                        }
                        VirtualKeyCode::Delete => {
                            if self.selection.is_empty() {
                                self.move_cursor(Movement::Right, true);
                            }
                            self.insert("");
                        }
                        VirtualKeyCode::Left => {
                            self.move_cursor(Movement::Left, k.modifiers.shift());
                        }
                        VirtualKeyCode::Right => {
                            self.move_cursor(Movement::Right, k.modifiers.shift());
                        }
                        _ => {}
                    }
                }
            }
            Event::Input(input) => {
                // reject control characters (handle in KeyDown instead)
                if !input.character.is_control() {
                    trace!("insert {:?}", input.character);
                    let mut buf = [0u8; 4];
                    self.insert(input.character.encode_utf8(&mut buf[..]));
                }
            }
            _ => {}
        }

        if self.needs_repaint {
            ctx.request_redraw();
            self.needs_repaint = false;
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Text element.
pub struct TextEditBase {
    text: String,
    //text_format: TextFormat,
}

impl<A: 'static> Widget<A> for TextEditBase {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        let text = &self.text;
        let platform = ctx.platform();

        let node_id = cursor.reconcile(nodes, |prev: Option<NodeData<TextEditVisual>>| {
            trace!("new TextEditVisual");

            if let Some(prev) = prev {
                if &prev.visual.text == text {
                    // OK, nothing to update
                    return prev;
                }
            }

            let text_layout = TextLayout::new(
                platform,
                &text,
                &theme.label_text_format,
                constraints.biggest(),
            )
            .unwrap();

            let text_size = text_layout.metrics().bounds.size.ceil();
            let baseline = text_layout
                .line_metrics()
                .first()
                .map(|m| m.baseline.ceil() as f64);

            NodeData::new(
                Layout::new(text_size).with_baseline(baseline),
                None,
                TextEditVisual {
                    text: text.to_owned(),
                    text_layout,
                    selection: Selection::empty(0),
                    needs_repaint: true,
                    needs_text_relayout: false,
                },
            )
        });

        node_id
    }
}

impl TextEditBase {
    pub fn new(text: impl Into<String>) -> TextEditBase {
        TextEditBase { text: text.into() }
    }
}

/// Text edit widget wrapper
pub struct TextEdit {
    text: String,
}

impl TextEdit {
    pub fn new(text: impl Into<String>) -> TextEdit {
        TextEdit { text: text.into() }
    }
}

impl<A: 'static> Widget<A> for TextEdit {
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        nodes: &mut NodeArena,
        cursor: &mut NodeCursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> NodeId {
        Frame {
            border_color: Color::new(0.0, 0.0, 0.0, 1.0),
            border_width: 1.0,
            fill_color: Color::new(1.0, 1.0, 1.0, 1.0),
            inner: Padding::new(EdgeInsets::all(2.0), TextEditBase::new(self.text)).boxed(),
        }
        .layout(ctx, nodes, cursor, constraints, theme)
    }
}
