//! Text editor widget.
use crate::{
    composable,
    core::Widget,
    env::Environment,
    event::{Event, Modifiers, PointerEventKind},
    state::{Signal, State},
    style::{BoxStyle, PaintCtxExt},
    text::FormattedText,
    theme,
    widget::{text::Text, Container},
    BoxConstraints, Data, EventCtx, LayoutCtx, Measurements, Offset, PaintCtx, Point, Rect, SideOffsets, Size,
    WidgetPod,
};
use keyboard_types::KeyState;
use kyute::text::TextPosition;
use kyute_shell::drawing::Color;
use tracing::trace;
use unicode_segmentation::GraphemeCursor;

/// Text selection.
///
/// Start is the start of the selection, end is the end. The caret is at the end of the selection.
/// Note that we don't necessarily have start <= end: a selection with start > end means that the
/// user started the selection gesture from a later point in the text and then went back
/// (right-to-left in LTR languages). In this case, the cursor will appear at the "beginning"
/// (i.e. left, for LTR) of the selection.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Data)]
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

    /// Current selection.
    selection: Selection,

    /// The offset to the content area
    content_offset: Offset,

    /// The size of the content area
    content_size: Size,

    editing_finished: Signal<FormattedText>,
    text_changed: Signal<FormattedText>,
    selection_changed: Signal<Selection>,

    inner: WidgetPod<Container<Text>>,
}

impl TextEdit {
    /// Creates a new `TextEdit` widget displaying the specified `FormattedText`.
    #[composable]
    pub fn with_selection(formatted_text: impl Into<FormattedText>, selection: Selection) -> TextEdit {
        let formatted_text = formatted_text.into();

        trace!(
            "TextEdit::with_selection: {:?}, {:?}",
            formatted_text.plain_text,
            selection
        );

        let inner = Container::new(Text::new(formatted_text.clone()))
            .box_style(theme::TEXT_EDIT)
            .content_padding(SideOffsets::new_all_same(2.0));

        TextEdit {
            formatted_text,
            selection,
            content_offset: Default::default(),
            content_size: Default::default(),
            selection_changed: Signal::new(),
            editing_finished: Signal::new(),
            text_changed: Signal::new(),
            inner: WidgetPod::new(inner),
        }
    }

    /// Use if you don't care about the selection.
    #[composable]
    pub fn new(formatted_text: impl Into<FormattedText>) -> TextEdit {
        let selection = State::new(|| Selection::empty(0));
        let text_edit = Self::with_selection(formatted_text, selection.get());
        selection.update(text_edit.selection_changed());
        text_edit
    }

    /// Returns whether TODO.
    #[composable]
    pub fn editing_finished(&self) -> Option<FormattedText> {
        self.editing_finished.value()
    }

    /// Returns whether the text has changed.
    #[composable]
    pub fn text_changed(&self) -> Option<FormattedText> {
        self.text_changed.value()
    }

    #[composable]
    pub fn selection_changed(&self) -> Option<Selection> {
        self.selection_changed.value()
    }

    /// Moves the cursor forward or backward. Returns the new selection.
    fn move_cursor(&self, movement: Movement, modify_selection: bool) -> Selection {
        let offset =
            match movement {
                Movement::Left => prev_grapheme_cluster(&self.formatted_text.plain_text, self.selection.end)
                    .unwrap_or(self.selection.end),
                Movement::Right => next_grapheme_cluster(&self.formatted_text.plain_text, self.selection.end)
                    .unwrap_or(self.selection.end),
                Movement::LeftWord | Movement::RightWord => {
                    // TODO word navigation (unicode word segmentation)
                    tracing::warn!("word navigation is unimplemented");
                    self.selection.end
                }
            };

        if modify_selection {
            Selection {
                start: self.selection.start,
                end: offset,
            }
        } else {
            Selection::empty(offset)
        }
    }

    /*//// Inserts text.
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

    /// Returns the position in the text (character offset) that is closest to the given point.
    fn text_position(&self, pos: Point) -> TextPosition {
        let paragraph = self.inner.contents().formatted_paragraph();
        paragraph.glyph_text_position(pos - self.inner.content_offset())
    }

    fn notify_selection_changed(&self, ctx: &mut EventCtx, new_selection: Selection) {
        if new_selection != self.selection {
            eprintln!("notify selection changed {:?}->{:?}", self.selection, new_selection);
            self.selection_changed.signal(ctx, new_selection);
        }
    }

    fn notify_text_changed(&self, ctx: &mut EventCtx, new_text: FormattedText) {
        self.text_changed.signal(ctx, new_text);
    }

    fn notify_editing_finished(&self, ctx: &mut EventCtx, new_text: FormattedText) {
        self.editing_finished.signal(ctx, new_text);
    }
}

impl Widget for TextEdit {
    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        // paint the text
        self.inner.paint(ctx, bounds, env);

        // paint the selection over it
        let offset = self.inner.content_offset();
        let paragraph = self.inner.contents().formatted_paragraph();
        let selection_boxes = paragraph.rects_for_range(self.selection.min()..self.selection.max());
        for mut tb in selection_boxes {
            tb.rect.origin += offset;
            ctx.draw_styled_box(tb.rect, &BoxStyle::new().fill(Color::new(0.0, 0.1, 0.8, 0.5)), env);
        }

        // TODO caret
        // -> move to helper function (format_text_edit): applies the format ranges, splits the text into blocks, returns a SkParagraph
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
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
                        if p.repeat_count == 2 {
                            trace!("text edit: select all");
                            // double-click selects all
                            self.notify_selection_changed(
                                ctx,
                                Selection {
                                    start: 0,
                                    end: self.formatted_text.plain_text.len(),
                                },
                            );
                        } else {
                            let text_pos = self.text_position(p.position);
                            trace!("text edit: move cursor");
                            if self.selection != Selection::empty(text_pos.position) {
                                self.notify_selection_changed(ctx, Selection::empty(text_pos.position));
                            }
                        }
                        ctx.request_redraw();
                        ctx.request_focus();
                        ctx.capture_pointer();
                    }
                    PointerEventKind::PointerMove => {
                        // update selection
                        if ctx.is_capturing_pointer() {
                            trace!("text edit: move cursor");
                            let text_pos = self.text_position(p.position);
                            if self.selection.end != text_pos.position {
                                self.notify_selection_changed(
                                    ctx,
                                    Selection {
                                        start: self.selection.start,
                                        end: text_pos.position,
                                    },
                                )
                            }
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
                        self.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::ArrowRight => {
                        self.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        trace!("insert {:?}", c);
                        //trace!("text edit: character {}", c);
                        //self.insert(&c);
                        //ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        //ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    _ => {}
                },
                KeyState::Up => {}
            },

            Event::Composition(_) => {}
            _ => {}
        }
    }
}
