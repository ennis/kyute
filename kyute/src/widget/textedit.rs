//! Text editor widget.
use crate::{
    cache, composable,
    core2::Widget,
    env::Environment,
    event::{Event, Modifiers, PointerEventKind},
    styling::PaintCtxExt,
    theme, BoxConstraints, Cache, Data, EnvKey, EventCtx, Key, LayoutCtx, Measurements, Offset,
    PaintCtx, Point, Rect, SideOffsets, Size, WidgetPod,
};
use keyboard_types::KeyState;
use kyute_shell::{
    drawing::{Color, FromSkia, ToSkia},
    skia as sk,
    winit::event::VirtualKeyCode,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    ops::Range,
    sync::Arc,
};
use tracing::trace;
use unicode_segmentation::GraphemeCursor;
use crate::text::{FormattedTextParagraph, FormattedText, ParagraphStyle};

/// Text selection.
///
/// Start is the start of the selection, end is the end. The caret is at the end of the selection.
/// Note that we don't necessarily have start <= end: a selection with start > end means that the
/// user started the selection gesture from a later point in the text and then went back
/// (right-to-left in LTR languages). In this case, the cursor will appear at the "beginning"
/// (i.e. left, for LTR) of the selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

impl Default for Selection {
    fn default() -> Self {
        Selection::empty(0)
    }
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




/// Text editor widget.
pub struct TextEdit {
    /// Input formatted text.
    formatted_text: FormattedText,

    /// The offset to the content area
    content_offset: Offset,

    /// The size of the content area
    content_size: Size,

    editing_finished: Key<bool>,
    text_changed: Key<bool>,

    /// The formatted paragraph, calculated during layout. `None` if not yet calculated.
    paragraph: RefCell<Option<FormattedTextParagraph>>,
}

impl TextEdit {
    /// Creates a new `TextEdit` widget displaying the specified `FormattedText`.
    #[composable(uncached)]
    pub fn new(
        formatted_text: impl Into<FormattedText>,
    ) -> WidgetPod<TextEdit> {
        let editing_finished = cache::state(|| false);
        let text_changed = cache::state(|| false);

        WidgetPod::new(TextEdit {
            formatted_text: formatted_text.into(),
            content_offset: Default::default(),
            content_size: Default::default(),
            editing_finished,
            text_changed,
            paragraph: RefCell::new(None),
        })
    }

    /// Returns whether TODO.
    #[composable(uncached)]
    pub fn editing_finished(&self) -> bool {
        self.editing_finished.get()
    }

    /// Returns whether the text has changed.
    pub fn text_changed(&self) -> bool {
        self.text_changed.get()
    }

    /*/// Moves the cursor forward or backward.
    // TODO move to EditState
    pub fn move_cursor(&mut self, movement: Movement, modify_selection: bool) {
        let offset = match movement {
            Movement::Left => prev_grapheme_cluster(&self.state.text, self.state.selection.end)
                .unwrap_or(self.state.selection.end),
            Movement::Right => next_grapheme_cluster(&self.state.text, self.state.selection.end)
                .unwrap_or(self.state.selection.end),
            Movement::LeftWord | Movement::RightWord => {
                // TODO word navigation (unicode word segmentation)
                tracing::warn!("word navigation is unimplemented");
                self.state.0.selection.end
            }
        };

        if modify_selection {
            self.state.0.selection.end = offset;
        } else {
            self.state.0.selection = Selection::empty(offset);
        }
    }*/

    /*/// Inserts text.
    // TODO move to EditState
    pub fn insert(&mut self, text: &str) {
        let min = self.state.selection.min();
        let max = self.state.selection.max();
        self.state.text.replace_range(min..max, text);
        self.state.selection = Selection::empty(min + text.len());
    }*/

    /*/// Sets cursor position.
    // TODO move to EditState
    pub fn set_cursor(&mut self, pos: usize) {
        if self.state.selection.is_empty() && self.state.selection.end == pos {
            return;
        }
        self.state.selection = Selection::empty(pos);
        // reset blink
    }

    // TODO move to EditState
    pub fn set_selection_end(&mut self, pos: usize) {
        if self.state.selection.end == pos {
            return;
        }
        self.state.selection.end = pos;
        // reset blink
    }*/

    /*// TODO move to EditState
    pub fn select_all(&mut self) {
        self.state.selection.start = 0;
        self.state.selection.end = self.state.text.len();
    }*/

    /*fn position_to_text(&self, pos: Point) -> usize {
        let hit = self
            .text_layout
            .as_ref()
            .expect("position_to_text called before layout")
            .hit_test_point(pos)
            .unwrap();
        let pos = if hit.is_trailing_hit {
            hit.metrics.text_position + hit.metrics.length
        } else {
            hit.metrics.text_position
        };
        pos
    }*/
}

// Given a FormattedText, create a new one by appending new format ranges.
// FormattedText: immutable?
// -> cheaply clonable
// -> mutable, but with Arc::make_mut under the hood
//  -> provide `with_*` functions
// -> text = ArcStr
//
// 1. rename FormattedText? It's not "formatted" yet. => RichText?
// 2. FormattedText produce a `TextLayout` object, which contains a formatted SkParagraph

impl Widget for TextEdit {
    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        // ???
        const SELECTION_MAGIC: f64 = 3.0;

        // get available size & content size
        let padding = env.get(theme::TEXT_EDIT_PADDING).unwrap_or_default();
        let available_width = constraints.constrain_width(200.0); // not sure why 200?
        let available_height = constraints.max_height();
        let text_available_width = available_width - padding.horizontal();
        let text_available_height = available_height - padding.vertical();

        /*let size = Size::new(
            constraints.constrain_width(200.0),
            constraints.constrain_height(font_size + SELECTION_MAGIC + padding.vertical()),
        );*/
        //let mut style2 = self.formatted_text.paragraph_style.clone();
        //style2.0.set_height(text_available_height as sk::scalar);
        //trace!("TextEdit: paragraph style: {:#?}", style2.0);

        // create the paragraph & layout in content width
        let mut paragraph = self.formatted_text.format();
        paragraph.0.layout(text_available_width as sk::scalar);

        trace!("TextEdit: layout result: {:#?}", paragraph.0);

        // measure the paragraph
        let text_height = paragraph.0.height() as f64;
        let baseline = paragraph.0.alphabetic_baseline() as f64 + padding.top;
        let size = dbg!(Size::new(
            available_width,
            constraints.constrain_height(text_height + padding.vertical()),
        ));

        // stash the laid out paragraph for rendering
        self.paragraph.replace(Some(paragraph));

        Measurements {
            size,
            baseline: Some(baseline),
            is_window: false,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        use crate::styling::*;
        let bounds = ctx.bounds();
        let padding = env.get(theme::TEXT_EDIT_PADDING).unwrap_or_default();

        let mut paragraph = self.paragraph.borrow_mut();
        let paragraph = paragraph.as_mut().expect("paint called before layout");

        // draw background
        //let background_color = env.get(theme::TEXT_EDIT_BACKGROUND_COLOR).unwrap();
        //ctx.draw_styled_box(bounds, rectangle().with(fill(background_color)), env);

        // draw paragraph
        ctx.canvas.save();
        //ctx.canvas
        //    .translate(Offset::new(padding.left, padding.top).to_skia());
        paragraph
            .0
            .paint(&mut ctx.canvas, Point::origin().to_skia());
        ctx.canvas.restore();

        // TODO selection highlight, caret
        // -> move to helper function (format_text_edit): applies the format ranges, splits the text into blocks, returns a SkParagraph
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::FocusGained => {
                trace!("text edit: focus gained");
                ctx.request_redraw();
            }
            Event::FocusLost => {
                trace!("text edit: focus lost");
                //let pos = self.state.selection.end;
                //self.set_cursor(pos);
                //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                ctx.request_redraw();
            }
            Event::Pointer(p) => {
                match p.kind {
                    PointerEventKind::PointerDown => {
                        //let pos = self.position_to_text(p.position);
                        if p.repeat_count == 2 {
                            trace!("text edit: select all");
                            // double-click selects all
                            //self.select_all();
                            //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        } else {
                            trace!("text edit: move cursor");
                            //self.set_cursor(pos);
                            //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        }
                        ctx.request_redraw();
                        ctx.request_focus();
                        ctx.capture_pointer();
                    }
                    PointerEventKind::PointerMove => {
                        // update selection
                        if ctx.is_capturing_pointer() {
                            trace!("text edit: move cursor");
                            //let pos = self.position_to_text(p.position);
                            // self.set_selection_end(pos);
                            // ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                            ctx.request_redraw();
                        }
                    }
                    PointerEventKind::PointerUp => {
                        // nothing to do (pointer grab automatically ends)
                    }
                    _ => {}
                }
            }
            Event::Keyboard(k) => match k.state {
                KeyState::Down => match k.key {
                    keyboard_types::Key::Backspace => {
                        trace!("text edit: backspace");
                        //if self.state.selection.is_empty() {
                        //    self.move_cursor(Movement::Left, true);
                        //}
                        //self.insert("");
                        //ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    keyboard_types::Key::Delete => {
                        trace!("text edit: delete");
                        //if self.state.selection.is_empty() {
                        //    self.move_cursor(Movement::Right, true);
                        //}
                        //self.insert("");
                        //ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    keyboard_types::Key::ArrowLeft => {
                        //self.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::ArrowRight => {
                        // self.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        trace!("insert {:?}", c);
                        //trace!("text edit: character {}", c);
                        //self.insert(&c);
                        //ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        // ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    _ => {}
                },
                KeyState::Up => {}
            },

            Event::Composition(input) => {}
            _ => {}
        }
    }
}
