//! Text editor widget.
use crate::{
    core::Widget,
    data::Data,
    env::Environment,
    event::{Event, Modifiers, PointerEventKind},
    style::{State, StyleSet},
    theme, BoxConstraints, CompositionCtx, EnvKey, EventCtx, CallKey, LayoutCtx, Measurements, Offset,
    PaintCtx, Point, Rect, SideOffsets, Size, WidgetDelegate,
};
use keyboard_types::KeyState;
use kyute_shell::{
    drawing::{Brush, Color, DrawTextOptions},
    text::{TextFormat, TextFormatBuilder, TextLayout},
    winit::event::VirtualKeyCode,
};
use std::{any::Any, ops::Range, sync::Arc};
use tracing::trace;
use unicode_segmentation::GraphemeCursor;

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

#[derive(Clone)]
enum TextEditAction {
    TextChanged(String),
    SelectionChanged(Selection),
}

#[derive(Clone, Eq, PartialEq, Hash)]
/// Text editor state
pub struct TextEditState {
    /// The displayed text.
    text: String,

    /// The currently selected range. If no text is selected, this is a zero-length range
    /// at the cursor position.
    selection: Selection,
}

impl Data for TextEditState {
    fn same(&self, other: &Self) -> bool {
        self.text.same(&other.text) && self.selection == other.selection
    }
}

impl TextEditState {
    /// Creates a new text edit state with the given text, the cursor at the beginning, and nothing selected.
    pub fn new(text: String) -> TextEditState {
        TextEditState {
            text,
            selection: Default::default(),
        }
    }

    /// Sets the text.
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        // TODO: truncate selection if necessary
    }

    /// Sets the selection (and cursor position).
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
    }
}

pub struct TextEdit {
    /// Formatting information.
    text_format: TextFormat,

    /// Text editing state.
    state: TextEditState,

    /// The offset to the content area
    content_offset: Offset,

    /// The size of the content area
    content_size: Size,

    /// The text layout. None if not yet calculated.
    ///
    /// FIXME: due to DirectWrite limitations, the text layout contains a copy of the string.
    /// in the future, de-duplicate.
    text_layout: Option<TextLayout>,
}

impl TextEdit {
    pub fn new(state: TextEditState) -> TextEdit {
        TextEdit {
            text_format: TextFormat::builder().size(14.0).build().unwrap(),
            state,
            content_offset: Default::default(),
            content_size: Default::default(),
            text_layout: None,
        }
    }

    pub fn set_state(&mut self, state: TextEditState) {
        if self.state != state {
            self.state = state;
            self.text_layout = None;
        }
    }

    /// Moves the cursor forward or backward.
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
                self.state.selection.end
            }
        };

        if modify_selection {
            self.state.selection.end = offset;
        } else {
            self.state.selection = Selection::empty(offset);
        }
    }

    /// Inserts text.
    // TODO move to EditState
    pub fn insert(&mut self, text: &str) {
        let min = self.state.selection.min();
        let max = self.state.selection.max();
        self.state.text.replace_range(min..max, text);
        self.state.selection = Selection::empty(min + text.len());
    }

    /// Sets cursor position.
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
    }

    // TODO move to EditState
    pub fn select_all(&mut self) {
        self.state.selection.start = 0;
        self.state.selection.end = self.state.text.len();
    }

    fn position_to_text(&self, pos: Point) -> usize {
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
    }
}

impl WidgetDelegate for TextEdit {
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        _children: &mut [Widget],
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let padding = env.get(theme::TEXT_EDIT_PADDING).unwrap_or_default();
        let font_size = self.text_format.font_size() as f64;

        const SELECTION_MAGIC: f64 = 3.0;
        // why default width == 200?
        let size = Size::new(
            constraints.constrain_width(200.0),
            constraints.constrain_height(font_size + SELECTION_MAGIC + padding.vertical()),
        );

        let content_size = Size::new(
            size.width - padding.horizontal(),
            size.height - padding.vertical(),
        );

        let text_layout = TextLayout::new(&self.state.text, &self.text_format, content_size)
            .expect("could not create TextLayout");

        let content_offset = Offset::new(padding.left, padding.top);

        // calculate baseline
        let baseline = text_layout
            .line_metrics()
            .first()
            .map(|m| content_offset.y + m.baseline as f64);

        self.content_size = content_size;
        self.content_offset = content_offset;
        self.text_layout = Some(text_layout);
        Measurements { size, baseline }
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        children: &mut [Widget],
        bounds: Rect,
        env: &Environment,
    ) {
        let bounds = ctx.bounds();
        let text_layout = self
            .text_layout
            .as_mut()
            .expect("paint called before layout");

        let background_style = env.get(theme::TEXT_EDIT_BACKGROUND_STYLE).unwrap();
        background_style.draw_box(ctx, &bounds, State::ACTIVE);

        let text_color = env.get(theme::TEXT_COLOR).unwrap_or_default();
        let selected_text_color = env.get(theme::SELECTED_TEXT_COLOR).unwrap_or_default();
        let selected_background_color = env
            .get(theme::SELECTED_TEXT_BACKGROUND_COLOR)
            .unwrap_or_default();

        let text_brush = Brush::solid_color(ctx, text_color);
        let selected_bg_brush = Brush::solid_color(ctx, selected_background_color);
        let selected_text_brush = Brush::solid_color(ctx, selected_text_color);

        ctx.save();
        ctx.transform(&self.content_offset.to_transform());

        // text color
        text_layout.set_drawing_effect(&text_brush, ..);
        if !self.state.selection.is_empty() {
            // FIXME slightly changes the layout when the selection straddles a kerning pair?
            text_layout.set_drawing_effect(
                &selected_text_brush,
                self.state.selection.min()..self.state.selection.max(),
            );
        }

        // selection highlight
        if !self.state.selection.is_empty() {
            let selected_areas = text_layout
                .hit_test_text_range(
                    self.state.selection.min()..self.state.selection.max(),
                    &bounds.origin,
                )
                .unwrap();
            for sa in selected_areas {
                ctx.fill_rectangle(sa.bounds.round_out(), &selected_bg_brush);
            }
        }

        // text
        ctx.draw_text_layout(
            Point::origin(),
            text_layout,
            &text_brush,
            DrawTextOptions::ENABLE_COLOR_FONT,
        );

        // caret
        if ctx.is_focused() {
            let caret_hit_test = text_layout
                .hit_test_text_position(self.state.selection.end)
                .unwrap();

            //dbg!(caret_hit_test);
            ctx.fill_rectangle(
                Rect::new(
                    caret_hit_test.point.floor(),
                    Size::new(1.0, caret_hit_test.metrics.bounds.size.height),
                ),
                &text_brush,
            );
        }

        ctx.restore();
    }

    fn event(&mut self, ctx: &mut EventCtx, children: &mut [Widget], event: &Event) {
        match event {
            Event::FocusGained => {
                trace!("text edit: focus gained");
                ctx.request_redraw();
            }
            Event::FocusLost => {
                trace!("text edit: focus lost");
                let pos = self.state.selection.end;
                self.set_cursor(pos);
                ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                ctx.request_redraw();
            }
            Event::Pointer(p) => {
                match p.kind {
                    PointerEventKind::PointerDown => {
                        let pos = self.position_to_text(p.position);
                        if p.repeat_count == 2 {
                            // double-click selects all
                            self.select_all();
                            ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        } else {
                            self.set_cursor(pos);
                            ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        }
                        ctx.request_redraw();
                        ctx.request_focus();
                        ctx.capture_pointer();
                    }
                    PointerEventKind::PointerMove => {
                        // update selection
                        if ctx.is_capturing_pointer() {
                            let pos = self.position_to_text(p.position);
                            self.set_selection_end(pos);
                            ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
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
                        if self.state.selection.is_empty() {
                            self.move_cursor(Movement::Left, true);
                        }
                        self.insert("");
                        ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    keyboard_types::Key::Delete => {
                        trace!("text edit: delete");
                        if self.state.selection.is_empty() {
                            self.move_cursor(Movement::Right, true);
                        }
                        self.insert("");
                        ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_relayout();
                    }
                    keyboard_types::Key::ArrowLeft => {
                        self.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::ArrowRight => {
                        self.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        //trace!("insert {:?}", input.character);
                        trace!("text edit: character {}", c);
                        self.insert(&c);
                        ctx.emit_action(TextEditAction::TextChanged(self.state.text.clone()));
                        ctx.emit_action(TextEditAction::SelectionChanged(self.state.selection));
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

/// Describes changes or events that happened on a text edit widget.
#[derive(Clone)]
pub struct TextEditResult(Option<TextEditAction>);

impl TextEditResult {
    /// Calls the specified closure if the edited text has changed.
    pub fn on_text_changed(self, f: impl FnOnce(&str)) -> Self {
        match &self.0 {
            Some(TextEditAction::TextChanged(str)) => f(str),
            _ => {}
        }
        self
    }

    /// Calls the specified closure if the current selection has changed.
    pub fn on_selection_changed(self, f: impl FnOnce(&Selection)) -> Self {
        match &self.0 {
            Some(TextEditAction::SelectionChanged(s)) => f(s),
            _ => {}
        }
        self
    }
}

// the main widget is a text_line_edit_state, which shows an EditState value
// when the text changes, the selection s
pub fn text_line_edit_state(cx: &mut CompositionCtx, state: &TextEditState) -> TextEditResult {
    cx.enter(0);
    let action = cx.emit_node(
        |cx| TextEdit::new(state.clone()),
        |cx, text_edit| {
            text_edit.set_state(state.clone());
        },
        |_| {},
    );
    cx.exit();
    TextEditResult(action.cast())
}

/// Displays a single-line text editor widget.
///
/// TODO generalities (selection state, cursor, etc.)
///
/// The text appearance is controlled by the following environment variables: TODO.
///
/// # Arguments
/// * `text` - the text to display.
///
/// # Return value
/// A [`TextEditResult`] object that describes changes or events that happened on the widget.
///
pub fn text_line_edit(cx: &mut CompositionCtx, text: &str) -> TextEditResult {
    cx.enter(0);
    let r = cx.with_state(
        || TextEditState::new(text.to_string()),
        |cx, state| {
            state.set_text(text.to_string());
            text_line_edit_state(cx, state)
                .on_selection_changed(|selection| state.set_selection(*selection))
        },
    );
    cx.exit();
    r
}
