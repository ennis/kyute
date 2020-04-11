//! Text editor widget.
use crate::event::Event;
use crate::layout::{BoxConstraints, Layout, PaintLayout, Size};
use crate::renderer::Theme;
use crate::visual::{EventCtx, Node, PaintCtx, Visual};
use crate::widget::LayoutCtx;
use crate::{Bounds, Point, Widget};
use kyute_shell::drawing::{Color, DrawTextOptions, Rect, RectExt};
use kyute_shell::text::TextLayout;
use log::trace;
use palette::{Srgb, Srgba};
use std::any::Any;
use std::ops::Range;
use unicode_segmentation::GraphemeCursor;

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
        self.needs_repaint = true;
    }

    /// Removes text.
    pub fn delete(&mut self) {
        if self.selection.is_empty() {
            self.move_cursor(Movement::Right, true);
        }
        if self.selection.is_empty() {
            // still empty? we are at the end of the document, nothing to delete
            return;
        }
        self.insert("");
        self.needs_repaint = true;
        // reset blink
        // need layout
    }

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
}

impl Visual for TextEditVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        let size = ctx.size;

        let bg_color: Color = Srgb::from_format(palette::named::WHITE).into();
        let border_color: Color = Srgb::from_format(palette::named::BLACK).into();

        let rect = ctx.bounds();
        // box background
        ctx.fill_rectangle(rect.stroke_inset(1.0), bg_color);
        // border
        ctx.draw_rectangle(rect.stroke_inset(1.0), border_color, 1.0);

        // text
        ctx.draw_text_layout(
            Point::origin(),
            &self.text_layout,
            border_color,
            DrawTextOptions::empty(),
        );

        // caret
        eprintln!("selection={:?}", self.selection);
        let caret_hit_test = self
            .text_layout
            .hit_test_text_position(self.selection.end)
            .unwrap();
        ctx.fill_rectangle(
            Rect::new(caret_hit_test.point, Size::new(1.0, 14.0)),
            Color::new(0.0, 0.0, 0.0, 1.0),
        );

        self.needs_repaint = false;
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerDown(p) => {
                let hit = self.text_layout.hit_test_point(p.position).unwrap();
                eprintln!("{:?}", hit);

                let pos = if hit.is_trailing_hit {
                    hit.metrics.text_position + hit.metrics.length
                } else {
                    hit.metrics.text_position
                };

                self.set_cursor(pos);

                ctx.capture_pointer();
            }
            Event::PointerMove(p) => {}
            Event::PointerUp(p) => {}
            _ => {}
        }

        if self.needs_repaint {
            ctx.request_redraw();
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
pub struct TextEdit {
    text: String,
}

impl<A: 'static> Widget<A> for TextEdit {
    type Visual = TextEditVisual;

    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        node: Option<Node<Self::Visual>>,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) -> Node<Self::Visual> {
        let text = &self.text;
        let platform = ctx.platform();

        let mut node = node.unwrap_or_else(|| {
            Node::new(
                Layout::default(),
                None,
                TextEditVisual {
                    text: text.to_owned(),
                    text_layout: TextLayout::new(
                        platform,
                        &text,
                        &theme.label_text_format,
                        constraints.biggest(),
                    )
                    .unwrap(),
                    selection: Selection::empty(0),
                    needs_repaint: false,
                },
            )
        });

        if &node.visual.text != text {
            // text changed, relayout
            node.visual.text_layout = TextLayout::new(
                ctx.platform(),
                &text,
                &theme.label_text_format,
                constraints.biggest(),
            )
            .unwrap();
        }

        let text_size = node.visual.text_layout.metrics().bounds.size.ceil();

        let baseline = node
            .visual
            .text_layout
            .line_metrics()
            .first()
            .map(|m| m.baseline.ceil() as f64);

        node.layout = Layout::new(text_size).with_baseline(baseline);

        node
    }
}

impl TextEdit {
    pub fn new(text: impl Into<String>) -> TextEdit {
        TextEdit { text: text.into() }
    }
}
