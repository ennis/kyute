//! Text editor widget.
use crate::{
    core::Widget,
    drawing::ToSkia,
    event::{Event, Modifiers},
    text::{Selection, TextSpan, TextStyle},
    State, WidgetCtx,
};
use keyboard_types::KeyState;
use kurbo::Point;
use skia_safe as sk;
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tracing::{trace, warn};
use unicode_segmentation::GraphemeCursor;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Movement {
    Left,
    Right,
    LeftWord,
    RightWord,
}

fn prev_grapheme_cluster(text: &str, offset: usize) -> Option<usize> {
    let mut c = GraphemeCursor::new(offset, text.len(), true);
    c.prev_boundary(text, 0).unwrap()
}

fn next_grapheme_cluster(text: &str, offset: usize) -> Option<usize> {
    let mut c = GraphemeCursor::new(offset, text.len(), true);
    c.next_boundary(text, 0).unwrap()
}

// Current state of a text editor widget.
pub struct EditableTextStateInner {
    text: String,
    selection: Selection,
    // composing: TextRange
}

pub struct EditableTextState(State<EditableTextStateInner>);

impl EditableTextState {
    // get current text
    // get current selection
    // set current text
    // set current selection
    // move cursor
    // expand current selection
    // delete selected text
    // insert text at cursor
    // move cursor to next line

    // scroll / move to cursor
}

// Reacting to changes in current selection? -> depend on EditableTextState
// Reacting to changes in current text? -> depend on EditableTextState

/// Text editor widget.
pub struct BaseTextEdit {
    state: State<TextEditingState>,
    style: TextStyle,
    on_editing_finished: Box<dyn FnMut(&mut WidgetCtx, &Self)>,
    text_changed: Box<dyn FnMut(&mut WidgetCtx, &Self)>,
    selection_changed: Box<dyn FnMut(&mut WidgetCtx, &Self)>,
    /// Whether the text edit is focused.
    focused: bool,
    horizontal_offset: f64,
    paragraph: Option<sk::textlayout::Paragraph>,
}

impl BaseTextEdit {
    /*pub fn new() -> BaseTextEdit {
        let text = TextSpan::new("".to_string(), Default::default());
        Self::with_text(text)
    }*/

    /// Use if you don't care about the selection.
    pub fn new(state: State<TextEditingState>) -> BaseTextEdit {
        BaseTextEdit {
            state,
            style: Default::default(),
            on_editing_finished: Box::new(|_, _| {}),
            text_changed: Box::new(|_, _| {}),
            selection_changed: Box::new(|_, _| {}),
            focused: false,
            horizontal_offset: 0.0,
            paragraph: None,
        }
    }
}

/// Helper function that creates a new string with the text under `selection` replaced by the specified string.
///
/// Returns the edited string and the new selection that results from the editing operation.
fn edit_text(text: &str, selection: Selection, replace_with: &str) -> (Arc<str>, Selection) {
    let min = selection.min();
    let max = selection.max();
    // FIXME don't copy to a string just to call `replace_range`
    let mut string = text.to_string();
    string.replace_range(min..max, replace_with);
    let text = Arc::from(string);
    (text, Selection::empty(min + replace_with.len()))
}

impl BaseTextEdit {
    /// Moves the cursor forward or backward. Returns the new selection.
    fn move_cursor(&self, movement: Movement, modify_selection: bool) -> Selection {
        let offset = match movement {
            Movement::Left => {
                prev_grapheme_cluster(&self.formatted_text.text, self.selection.end).unwrap_or(self.selection.end)
            }
            Movement::Right => {
                next_grapheme_cluster(&self.formatted_text.text, self.selection.end).unwrap_or(self.selection.end)
            }
            Movement::LeftWord | Movement::RightWord => {
                // TODO word navigation (unicode word segmentation)
                warn!("word navigation is unimplemented");
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

    /// Returns the position in the text (character offset between grapheme clusters) that is closest to the given point.
    fn text_position(&self, mut pos: Point) -> TextPosition {
        pos.x -= self.horizontal_offset.get();
        TextPosition {
            position: paragraph.hit_test_point(pos).idx,
            affinity: TextAffinity::Upstream,
        }
    }

    fn notify_selection_changed(&self, _ctx: &mut EventCtx, new_selection: Selection) {
        if new_selection != self.selection {
            eprintln!("notify selection changed {:?}->{:?}", self.selection, new_selection);
            self.selection_changed.signal(new_selection);
        }
    }

    fn notify_text_changed(&self, _ctx: &mut EventCtx, new_text: Arc<str>) {
        self.text_changed.signal(new_text);
    }

    fn notify_editing_finished(&self, _ctx: &mut EventCtx, new_text: Arc<str>) {
        self.editing_finished.signal(new_text);
    }
}

impl Widget for BaseTextEdit {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        // relax text constraints
        let text_constraints = LayoutParams {
            min: Size::zero(),
            max: Size::new(f64::INFINITY, f64::INFINITY),
            ..*constraints
        };
        let child_layout = self.inner.layout(ctx, &text_constraints, env);

        let width = constraints
            .finite_max_width()
            .unwrap_or(child_layout.measurements.width());
        let height = constraints
            .finite_max_height()
            .unwrap_or(child_layout.measurements.height());

        if !ctx.speculative {
            // update the horizontal offset if the cursor position
            // overflows the available space
            let mut h_offset = self.horizontal_offset.get();
            let paragraph = self.inner.inner().paragraph();
            let cursor_hit = paragraph.hit_test_text_position(TextPosition {
                position: self.selection.end,
                affinity: TextAffinity::Upstream,
            });

            if cursor_hit.point.x + h_offset > width {
                trace!("cursor pos overflow to the right");
                h_offset = -cursor_hit.point.x + width;
            } else if cursor_hit.point.x + h_offset < 0.0 {
                trace!("cursor pos overflow to the left");
                h_offset = -cursor_hit.point.x;
            }

            self.inner.set_offset(Offset::new(h_offset, 0.0));
            self.horizontal_offset.set_without_invalidation(h_offset);
        }

        Geometry {
            x_align: Default::default(),
            y_align: Default::default(),
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements {
                size: Size::new(width, height),
                clip_bounds: None,
                baseline: child_layout.measurements.baseline,
            },
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::FocusGained => {
                trace!("text edit: focus gained");
                self.focused_changed.signal(true);
            }
            Event::FocusLost => {
                trace!("text edit: focus lost");
                let pos = self.selection.end;
                if self.selection.start != self.selection.end {
                    self.notify_selection_changed(ctx, Selection { start: pos, end: pos });
                }
                self.notify_editing_finished(ctx, self.formatted_text.plain_text.clone());
                self.focused_changed.signal(false);
            }
            Event::Pointer(p) => {
                match p.kind {
                    PointerEventKind::PointerOver => ctx.set_cursor_icon(CursorIcon::Text),
                    PointerEventKind::PointerOut => ctx.set_cursor_icon(CursorIcon::Default),
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
                        ctx.request_focus();
                        ctx.capture_pointer();
                        ctx.set_handled();
                    }
                    PointerEventKind::PointerMove => {
                        // update selection
                        if ctx.is_capturing_pointer() {
                            trace!("text edit: move cursor");
                            let text_pos = self.text_position(p.position);
                            self.notify_selection_changed(
                                ctx,
                                Selection {
                                    start: self.selection.start,
                                    end: text_pos.position,
                                },
                            );
                            ctx.set_handled();
                        }
                    }
                    PointerEventKind::PointerUp => {
                        // nothing to do (pointer grab automatically ends)
                        ctx.set_handled();
                    }
                    _ => {}
                }
            }
            Event::Keyboard(k) => match k.state {
                KeyState::Down => match k.key {
                    keyboard_types::Key::Backspace => {
                        trace!("text edit: backspace");
                        let selection = if self.selection.is_empty() {
                            self.move_cursor(Movement::Left, true)
                        } else {
                            self.selection
                        };
                        let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, selection, "");
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
                        ctx.request_relayout();
                        ctx.set_handled();
                    }
                    keyboard_types::Key::Delete => {
                        trace!("text edit: delete");
                        let selection = if self.selection.is_empty() {
                            self.move_cursor(Movement::Right, true)
                        } else {
                            self.selection
                        };
                        let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, selection, "");
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
                        ctx.request_relayout();
                        ctx.set_handled();
                    }
                    keyboard_types::Key::ArrowLeft => {
                        let selection = self.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        self.notify_selection_changed(ctx, selection);
                        ctx.set_handled();
                    }
                    keyboard_types::Key::ArrowRight => {
                        let selection = self.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        self.notify_selection_changed(ctx, selection);
                        ctx.set_handled();
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, self.selection, c);
                        trace!("insert {:?}; text after = {}", c, new_text);
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
                        ctx.request_relayout();
                        ctx.set_handled();
                    }
                    keyboard_types::Key::Enter => {
                        // enter validates
                        self.notify_editing_finished(ctx, self.formatted_text.plain_text.clone());
                        ctx.set_handled();
                    }
                    _ => {}
                },
                KeyState::Up => {
                    ctx.set_handled();
                }
            },

            Event::Composition(_) => {}
            _ => {}
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        use skia_safe as sk;

        // paint the text
        self.inner.paint(ctx);

        let h_offset = self.horizontal_offset.get();

        // paint the selection over it
        let paragraph = self.inner.inner().paragraph();
        let selection_boxes =
            paragraph.hit_test_text_range(self.selection.min()..self.selection.max(), Point::origin());

        {
            // TODO color from environment or theme
            let mut paint = sk::Paint::new(Color::new(0.0, 0.8, 0.8, 0.5).to_skia(), None);
            for mut sb in selection_boxes {
                let canvas = ctx.surface.canvas();
                let offset_sb_bounds = sb.bounds.translate(Offset::new(h_offset, 0.0));
                let rect = offset_sb_bounds.to_skia();
                canvas.draw_rect(rect, &paint);
            }
        }

        // paint the caret
        if self.focused {
            let caret_hit_test = paragraph.hit_test_text_position(TextPosition {
                position: self.selection.end,
                affinity: TextAffinity::Downstream,
            });

            // TODO color from environment or theme
            let caret_color = Color::new(1.0, 1.0, 1.0, 1.0);
            let paint = sk::Paint::new(caret_color.to_skia(), None);
            let mut pos = caret_hit_test.point;
            pos.x += h_offset;
            let canvas = ctx.surface.canvas();
            canvas.draw_rect(
                Rect::new(pos.floor(), Size::new(1.0, caret_hit_test.metrics.bounds.size.height)).to_skia(),
                &paint,
            );
        }
    }
}

/// The built-in text edit style, compatible with light & dark modes.
const TEXT_EDIT_STYLE: &str = r#"
border-radius: 3px;
padding: 2px;
width: 100%;
min-height: 1.5em;
background: $text-background-color;
"#;

#[derive(Widget)]
pub struct TextEdit {
    inner: StyledBox<BaseTextEdit>,
}

impl TextEdit {
    /// Creates a new `TextEditInner` widgets displaying the specified `FormattedText`.
    #[composable]
    pub fn with_selection(formatted_text: impl Into<FormattedText>, mut selection: Selection) -> TextEdit {
        let mut base = BaseTextEdit::with_selection(formatted_text, selection);
        TextEdit {
            inner: base.style(TEXT_EDIT_STYLE),
        }
    }

    /// Use if you don't care about the selection.
    #[composable]
    pub fn new(formatted_text: impl Into<FormattedText>) -> TextEdit {
        #[state]
        let mut selection = Selection::empty(0);
        Self::with_selection(formatted_text, selection).on_selection_changed(|s| selection = s)
    }

    /// Returns whether TODO.
    pub fn editing_finished(&self) -> Option<Arc<str>> {
        self.inner.inner().editing_finished.value()
    }

    pub fn on_editing_finished(self, f: impl FnOnce(Arc<str>)) -> Self {
        if let Some(text) = self.editing_finished() {
            f(text)
        }
        self
    }

    /// Returns whether the text has changed.
    pub fn text_changed(&self) -> Option<Arc<str>> {
        self.inner.inner().text_changed.value()
    }

    pub fn on_text_changed(self, f: impl FnOnce(Arc<str>)) -> Self {
        if let Some(text) = self.text_changed() {
            f(text)
        }
        self
    }

    pub fn selection_changed(&self) -> Option<Selection> {
        self.inner.inner().selection_changed.value()
    }

    pub fn on_selection_changed(self, f: impl FnOnce(Selection)) -> Self {
        if let Some(selection) = self.selection_changed() {
            f(selection)
        }
        self
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Text fields
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TextField {
    label: Text,
    edit: TextEdit,
}

impl TextField {
    #[composable]
    pub fn new(label: impl Into<FormattedText>, text: impl Into<FormattedText>) -> TextField {
        let label = Text::new(label);
        let edit = TextEdit::new(text);
        TextField { label, edit }
    }

    /// Returns whether TODO.
    pub fn editing_finished(&self) -> Option<Arc<str>> {
        self.edit.editing_finished()
    }

    pub fn on_editing_finished(self, f: impl FnOnce(Arc<str>)) -> Self {
        if let Some(text) = self.editing_finished() {
            f(text)
        }
        self
    }

    /// Returns whether the text has changed.
    pub fn text_changed(&self) -> Option<Arc<str>> {
        self.edit.text_changed()
    }

    pub fn on_text_changed(self, f: impl FnOnce(Arc<str>)) -> Self {
        if let Some(text) = self.text_changed() {
            f(text)
        }
        self
    }

    pub fn selection_changed(&self) -> Option<Selection> {
        self.edit.selection_changed()
    }

    pub fn on_selection_changed(self, f: impl FnOnce(Selection)) -> Self {
        if let Some(selection) = self.selection_changed() {
            f(selection)
        }
        self
    }
}

impl From<TextField> for form::Row {
    fn from(field: TextField) -> Self {
        form::Row::Field {
            label: field.label.vertical_alignment(Alignment::FirstBaseline).arc_pod(),
            content: field.edit.vertical_alignment(Alignment::FirstBaseline).arc_pod(),
            swap_content_and_label: false,
        }
    }
}
