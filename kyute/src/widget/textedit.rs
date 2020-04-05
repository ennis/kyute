//! Text editor widget.
use crate::event::{Event, EventCtx};
use crate::layout::{BoxConstraints, Layout, PaintLayout, Size};
use crate::renderer::Theme;
use crate::visual::{Cursor, Node, PaintCtx, Visual};
use crate::widget::LayoutCtx;
use crate::{Bounds, Point, Widget};
use kyute_shell::text::TextLayout;
use log::trace;
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
#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub struct Selection
{
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

    pub fn empty(at: usize) ->  Selection {
        Selection {start: at, end: at}
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
}

// PointerDown (mouse grab on)
//  - hit test
//  - set cursor
// PointerMove
//  - hit test
//  - set selection end
// PointerUp
//  - hit test
//  - set selection end

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
    c.prev_boundary(&text, 0).unwrap()
}

impl TextEditVisual {

    /// Moves the cursor forward or backward.
    pub fn move_cursor(&mut self, movement: Movement, modify_selection: bool)
    {
        let offset = match movement {
            Movement::Left => {
                prev_grapheme_cluster(&self.text, self.selection.end).unwrap_or(self.selection.end)
            },
            Movement::Right => {
                next_grapheme_cluster(&self.text, self.selection.end).unwrap_or(self.selection.end)
            },
            Movement::LeftWord | Movement::RightWord =>  {
                // TODO word navigation (unicode word segmentation)
                unimplemented!()
            }
        };

        if modify_selection {
            self.selection.end = offset;
        } else {
            self.selection = Selection::empty(offset);
        }

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

        // reset blink
        // need layout
        // need repaint
    }

    /// Sets cursor position.
    pub fn set_cursor(&mut self, pos: usize) {
        self.selection = Selection::empty(pos);

        // reset blink
        // need repaint
    }

    pub fn set_selection_end(&mut self, pos: usize) {
        if self.selection.end == pos {
            return;
        }

        self.selection.end = pos;

        // reset blink
        // need repaint
    }
}

// focus:
// - on pointer down, gain focus, send Event::FocusOn
//
// tab navigation:
// - move focus to sibling
//      - node.parent, go to next node that has tab navigation
//      - if no next, then go up, recurse
//
// Node:
// - tab_navigation: bool
// -

impl Visual for TextEditVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, theme: &Theme) {
        //ctx.painter.draw_text_with_selection_range(ctx.bounds.origin, &self.text_layout, self.selection.clone());
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, ctx: &EventCtx, event: &Event) {
        match event {
            Event::PointerDown(p) => {

                //ctx.grab_pointer(); // redirect all pointer events to this widget until pointerUp
            },
            Event::PointerMove(p) => {

            },
            Event::PointerUp(p) => {

            }
            _ => {}
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
    fn layout(
        self,
        ctx: &mut LayoutCtx<A>,
        cursor: &mut Cursor,
        constraints: &BoxConstraints,
        theme: &Theme,
    ) {
        // TODO
        let text = &self.text;
        let platform = ctx.platform();

        let mut node = cursor.open(None, move || TextEditVisual {
            text: text.to_owned(),
            text_layout: TextLayout::new(
                platform,
                &text,
                &theme.label_text_format,
                constraints.biggest(),
            )
            .unwrap(),
            selection: Selection::empty(0),
        });
        let node = &mut node;

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
    }
}

impl TextEdit {
    pub fn new(text: impl Into<String>) -> TextEdit {
        TextEdit { text: text.into() }
    }
}
