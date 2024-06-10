//! Text editor widget.
use crate::{
    core::Widget,
    drawing::{FromSkia, ToSkia},
    event::Event,
    text::{Selection, TextSpan, TextStyle},
    widgets::viewport::Viewport,
    Binding, BoxConstraints, Ctx, Geometry, HitTestResult, LayoutCtx, PaintCtx, State, WidgetPod, WidgetPtr,
};
use keyboard_types::{KeyState, Modifiers};
use kurbo::{Point, Rect, Size, Vec2};
use skia_safe as sk;
use skia_safe::textlayout::{Paragraph, RectHeightStyle, RectWidthStyle};
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tracing::{trace, warn};
use unicode_segmentation::GraphemeCursor;

/// Cursor movement.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Movement {
    /// Move the cursor left (next character).
    Left,
    /// Move the cursor right (previous character).
    Right,
    /// Move the cursor left by a word.
    LeftWord,
    /// Move the cursor right by a word.
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
pub struct TextEditingState {
    pub text: String,
    pub selection: Selection,
}

pub type TextAffinity = sk::textlayout::Affinity;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextPosition {
    pub position: usize,
    pub affinity: TextAffinity,
}

impl TextEditingState {
    // get current text
    // get current selection
    // set current text
    // set current selection
    // delete selected text
    // insert text at cursor
    // move cursor to next line
    // copy to clipboard
    // paste from clipboard

    // (action) scroll / move to cursor

    /// Returns the current text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the current selection.
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// Sets the current selection.
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }

    pub fn cursor_pos(&self) -> usize {
        self.selection.end
    }

    pub fn select_all(&mut self) {
        self.selection = Selection {
            start: 0,
            end: self.text.len(),
        };
    }

    /// Moves the cursor forward or backward, possibly modifying the anchor of the selection. Returns the new selection.
    ///
    /// # Arguments
    ///
    /// - `movement`: the direction to move the cursor
    /// - `anchor`: whether to keep the anchor of the selection (if false, the selection will be collapsed to the cursor position)
    pub fn move_cursor(&self, movement: Movement, keep_anchor: bool) -> Selection {
        let offset = match movement {
            Movement::Left => prev_grapheme_cluster(&self.text, self.selection.end).unwrap_or(self.selection.end),
            Movement::Right => next_grapheme_cluster(&self.text, self.selection.end).unwrap_or(self.selection.end),
            Movement::LeftWord | Movement::RightWord => {
                // TODO word navigation (unicode word segmentation)
                warn!("word navigation is unimplemented");
                self.selection.end
            }
        };

        if keep_anchor {
            Selection {
                start: self.selection.start,
                end: offset,
            }
        } else {
            Selection::empty(offset)
        }
    }

    /// Deletes the selected text, if any.
    pub fn delete_selected(&mut self) {
        if self.selection.is_empty() {
            return;
        }

        let min = self.selection.min();
        let max = self.selection.max();
        self.text.replace_range(min..max, "");
        self.selection = Selection::empty(min);
    }

    /// Copies the current selection to the clipboard.
    pub fn copy_to_clipboard(&self) {
        // TODO
    }

    pub fn cut_to_clipboard(&mut self) {
        // TODO
    }

    /// Replaces the current selection with the clipboard contents.
    pub fn paste_from_clipboard(&mut self) {
        // TODO
    }

    /// Clears the text.
    pub fn clear(&mut self) {
        self.text.clear();
        self.selection = Selection::empty(0);
    }

    /*/// Returns the position in the text (character offset between grapheme clusters) that is closest to the given point.
    ///
    /// # Arguments
    ///
    /// - `position`: the point to test, **in window coordinates**.
    // NOTE: we specify the window_position in window coordinates because EditableTextState
    // isn't a widget itself, so it has no local coordinate space.
    // Technically, we could associate the EditableTextState to the local coordinate space of the widget
    // that contains it, but since this method could be called from anywhere, it would be complicated
    // for the caller to keep track of the correct coordinate space and apply the correct transformation.
    pub fn text_position(&self, window_position: Point) -> TextPosition {
        if let Some(ref paragraph) = self.paragraph {
            paragraph.
        }

        TextPosition {
            position: paragraph.hit_test_point(pos).idx,
            affinity: TextAffinity::Upstream,
        }
    }*/
}

// Issue: TextEditState is both:
// - a way to control the text edit widget
// - a way to retrieve the current state of the text edit widget

// Reacting to changes in current selection? -> depend on EditableTextState
// Reacting to changes in current text? -> depend on EditableTextState

/// Text editor widget.
pub struct BaseTextEdit {
    state: State<TextEditingState>,
    //style: TextStyle,
    editing_finished: Box<dyn FnMut(&mut Ctx, String)>,
    text_changed: Box<dyn FnMut(&mut Ctx, String)>,
    selection_changed: Box<dyn FnMut(&mut Ctx, Selection)>,
    /// Whether the text edit is focused.
    focused: bool,
    paragraph: Option<Paragraph>,
    viewport: Viewport<CoreTextEdit>,
}

impl BaseTextEdit {
    /// Use if you don't care about the selection.
    pub fn new(state: State<TextEditingState>) -> BaseTextEdit {
        BaseTextEdit {
            state,
            editing_finished: Box::new(|_, _| {}),
            text_changed: Box::new(|_, _| {}),
            selection_changed: Box::new(|_, _| {}),
            focused: false,
            //horizontal_offset: 0.0,
            paragraph: None,
            viewport: Viewport::new(CoreTextEdit::new(state.clone(), Arc::new(TextStyle::default()))),
        }
    }
}

/// Helper function that creates a new string with the text under `selection` replaced by the specified string.
///
/// Returns the edited string and the new selection that results from the editing operation.
fn edit_text(text: &mut String, selection: Selection, replace_with: &str) -> Selection {
    let min = selection.min();
    let max = selection.max();
    // FIXME don't copy to a string just to call `replace_range`
    text.replace_range(min..max, replace_with);
    Selection::empty(min + replace_with.len())
}

impl Widget for BaseTextEdit {
    fn mount(&mut self, cx: &mut Ctx) {
        self.viewport.mount(cx);
    }

    fn update(&mut self, cx: &mut Ctx) {
        // the state has changed
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {}

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {
        self.inner.hit_test(result, position)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        let geometry = self.inner.layout(ctx, constraints);
        let state = self.state.get_untracked();
        //let core_text_size = self.inner.content_size();
        // ensure that the cursor is in view by scrolling if necessary
        let cursor_pos = state.cursor_pos();
        let cursor_rect = self
            .inner
            .inner()
            .paragraph
            .as_ref()
            .unwrap()
            .get_rects_for_range(cursor_pos..cursor_pos, RectHeightStyle::Tight, RectWidthStyle::Max)
            .first()
            .unwrap();
        let cursor_rect = Rect::from_skia(cursor_rect.rect);
        self.inner.horizontal_scroll_to(cursor_rect.x0);
        geometry
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // paint the text
        self.inner.paint(ctx);

        /*let h_offset = self.horizontal_offset.get();

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
        }*/
    }
}

/*
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
*/

//
// TextEdit
//   -> State<TextEditingState>
//      -> BaseTextEdit(state): event handlers
//          -> CoreTextEdit(state): layouts and draws the text, selection and cursor
// -> can swap out the CoreTextEdit with something else to change the rendering

/// Implements the core logic of the text edit widget.
///
/// This widget is responsible for handling text input, cursor movement, selection, and rendering.
/// The `TextEdit` widget above wraps this to add a scrolling viewport and is responsible for requesting
/// relayouts and propagating updates to the text.
pub struct CoreTextEdit {
    state: State<TextEditingState>,
    style: Arc<TextStyle>,
    paragraph: Paragraph,
    single_line: bool,
    editing_finished: Box<dyn FnMut(&mut Ctx, String)>,
    text_changed: Box<dyn FnMut(&mut Ctx, String)>,
    selection_changed: Box<dyn FnMut(&mut Ctx, Selection)>,
}

// Option 1: pass State<TextEditingState>
// On input:
// - update the state (this queues a call to CoreTextEdit::update)

// Main issue:
// how to rect to changes in the cursor position? and I mean the actual coordinates on the screen, not the current selection
// this is necessary for the viewport to scroll to the cursor position.
// Flutter does that via "Scrollables" I think;
// -> basically, CoreTextEdit itself requests a scroll to a particular location

pub struct ScrollState {
    pub offset: Vec2,
}

impl ScrollState {
    pub fn scroll_to(&mut self, offset: Vec2) {
        // issue: scroll_to is called with `offset` in the caller's coordinate space,
        // but we need to convert it to the viewport's coordinate space.
        // Furthermore, the coordinates passed to `Widget::event` may not be the same as the coordinates
        // of the owning `WidgetPod` (modifier widgets like `Padding` can transform events without introducing a WidgetPod).
        //
        // Possible solutions:
        // `ReferenceFrame`: a combination of a `WidgetPtr` and a transform to the modified widget inside the reference frame.
        // This can be obtained from the current `Ctx`.
        // Alternatively, just store the transform from the widget to window space.
        //
        // - This *must* take a `Ctx` in order to identify which widget is calling, and thus
        // the coordinate space to convert from.
        // - We need to find a way to convert between coordinate spaces. This is tricky because
        //   there's no generic transform hierarchy available.
        // self.widget.transform_to();
        self.offset = offset;
    }
}

impl CoreTextEdit {
    pub fn new(state: State<TextEditingState>, style: Arc<TextStyle>) -> WidgetPtr<CoreTextEdit> {
        let w = WidgetPod::new(CoreTextEdit {
            state,
            editing_finished: Box::new(|_, _| {}),
            text_changed: Box::new(|_, _| {}),
            selection_changed: Box::new(|_, _| {}),
            style,
            paragraph: TextSpan::default().build_paragraph(),
            single_line: true,
        });
        w
    }

    /*pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.to_string();
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }*/

    /// Returns the text position for the given cursor coordinates.
    fn text_position(&self, coordinate: Point) -> TextPosition {
        let coord = coordinate.to_skia();
        let text_position = self.paragraph.get_glyph_position_at_coordinate(coord); //hit_test_point(position.to_skia()).idx;
        TextPosition {
            position: text_position.position as usize,
            affinity: text_position.affinity,
        }
    }
}

impl Widget for CoreTextEdit {
    fn mount(&mut self, cx: &mut Ctx) {}

    fn update(&mut self, cx: &mut Ctx) {}

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {
        let mut state = self.state.get();
        let mut editing_finished = false;
        let mut text_changed = false;
        let mut selection_changed = false;

        let Some(ref mut paragraph) = self.paragraph else {
            // no layout yet
            return;
        };

        match event {
            Event::FocusGained => {
                //trace!("text edit: focus gained");
            }
            Event::FocusLost => {
                //trace!("text edit: focus lost");
                //let pos = self.state.get().selection.end;
                //if state.selection.start != state.selection.end {
                //    selection_changed = true;
                //}
                //editing_finished = true;
            }
            Event::PointerOver(p) => {}
            Event::PointerOut(p) => {}
            Event::PointerDown(p) => {
                if p.repeat_count == 2 {
                    trace!("text edit: select all");
                    // double-click selects all
                    self.state.update(cx, TextEditingState::select_all);
                } else {
                    trace!("text edit: move cursor");
                    let tp = self.text_position(p.position);
                    self.state.update(cx, |state| {
                        state.set_cursor(tp.position, false);
                    });

                    selection_changed = true;
                }

                // TODO
                //ctx.request_focus();
                //ctx.capture_pointer();
                //ctx.set_handled();
            }
            Event::PointerMove(p) => {
                // update selection
                /*if ctx.is_capturing_pointer() {
                    trace!("text edit: move cursor");
                    let text_pos = self.text_position(p.position);
                    selection_changed = true;
                    /*self.notify_selection_changed(
                        ctx,
                        Selection {
                            start: self.selection.start,
                            end: text_pos.position,
                        },
                    );*/
                }*/
            }
            Event::PointerUp(p) => {
                // nothing to do (pointer grab automatically ends)
                //ctx.set_handled();
            }
            Event::Keyboard(k) => match k.state {
                KeyState::Down => match k.key {
                    keyboard_types::Key::Backspace => {
                        trace!("text edit: backspace");
                        if state.selection.is_empty() {
                            state.move_cursor(Movement::Left, true)
                        }
                        edit_text(&mut state.text, state.selection, "");
                        // TODO
                        /*let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, selection, "");
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
                        ctx.request_relayout();
                        ctx.set_handled();*/
                    }
                    keyboard_types::Key::Delete => {
                        trace!("text edit: delete");
                        if state.selection.is_empty() {
                            state.move_cursor(Movement::Right, true)
                        }
                        state.delete_selected();
                        selection_changed = true;
                        text_changed = true;
                    }
                    keyboard_types::Key::ArrowLeft => {
                        state.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        selection_changed = true;
                    }
                    keyboard_types::Key::ArrowRight => {
                        state.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        selection_changed = true;
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        edit_text(&mut state.text, state.selection, c);
                        text_changed = true;
                    }
                    keyboard_types::Key::Enter => {
                        editing_finished = true;
                    }
                    _ => {}
                },
                KeyState::Up => {}
            },

            //Event::Composition(_) => {}
            _ => {}
        }

        drop(state);
        if editing_finished {
            // TODO: avoid expensive cloning here
            let text = self.state.get_untracked().text().to_string();
            (self.editing_finished)(cx, text);
        }
        if text_changed {
            let text = self.state.get_untracked().text().to_string();
            (self.text_changed)(cx, text);
        }
        if selection_changed {
            let selection = self.state.get_untracked().selection();
            (self.selection_changed)(cx, selection);
        }
    }

    fn hit_test(&mut self, result: &mut HitTestResult, position: Point) -> bool {}

    fn layout(&mut self, cx: &mut LayoutCtx, bc: &BoxConstraints) -> Geometry {
        // TODO: cache constraints and only relayout if they changed

        // relax text constraints
        let text_edit_state = self.state.get_untracked();
        let text = text_edit_state.text();

        // determine the available space
        let available_width = if self.single_line { f64::INFINITY } else { todo!() };

        // create the paragraph
        let mut paragraph = TextSpan::new(text, Arc::new(self.style.clone())).build_paragraph();
        paragraph.layout(available_width as sk::scalar);
        let w = paragraph.longest_line() as f64;
        let h = paragraph.height() as f64;
        let alphabetic_baseline = paragraph.alphabetic_baseline();
        let unconstrained_size = Size::new(w, h);
        let size = bc.constrain(unconstrained_size);
        Geometry {
            size,
            baseline: Some(alphabetic_baseline as f64),
            bounding_rect: size.to_rect(),
            paint_bounding_rect: size.to_rect(),
        }
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        cx.with_canvas(|canvas| {
            self.paragraph.paint(canvas, Point::ZERO.to_skia());
        })
    }
}
