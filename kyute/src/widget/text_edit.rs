//! Text editor widget.
use crate::{
    cache, composable,
    core::Widget,
    drawing::ToSkia,
    env::Environment,
    event::{Event, Modifiers, PointerEventKind},
    widget::{prelude::*, StyledBox, Text},
    State,
};
use keyboard_types::KeyState;
use kyute_common::Color;
use kyute_shell::{
    text::{FormattedText, Selection, TextAffinity, TextPosition},
    winit::window::CursorIcon,
};
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tracing::trace;
use unicode_segmentation::GraphemeCursor;

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

/// Text editor widget.
pub struct BaseTextEdit {
    id: WidgetId,
    /// Input formatted text.
    formatted_text: FormattedText,
    /// Current selection.
    selection: Selection,
    editing_finished: Signal<Arc<str>>,
    text_changed: Signal<Arc<str>>,
    selection_changed: Signal<Selection>,
    focused_changed: Signal<bool>,
    focused: bool,
    inner: WidgetPod<Text>,
    horizontal_offset: State<f64>,
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
    /// Creates a new `TextEditInner` widget displaying the specified `FormattedText`.
    #[composable]
    pub fn with_selection(formatted_text: impl Into<FormattedText>, mut selection: Selection) -> BaseTextEdit {
        #[state]
        let mut focused = false;

        let formatted_text = formatted_text.into();

        // clamp selection
        selection.start = selection.start.min(formatted_text.plain_text.len());
        selection.end = selection.end.min(formatted_text.plain_text.len());
        let inner = WidgetPod::new(Text::new(formatted_text.clone()));

        // handle focus changes
        let focused_changed = Signal::new();
        if let Some(f) = focused_changed.value() {
            focused = f;
        }

        BaseTextEdit {
            id: WidgetId::here(),
            formatted_text,
            selection,
            selection_changed: Signal::new(),
            editing_finished: Signal::new(),
            text_changed: Signal::new(),
            focused,
            focused_changed,
            inner,
            horizontal_offset: cache::state(|| 0.0),
        }
    }

    /// Use if you don't care about the selection.
    #[composable]
    pub fn new(formatted_text: impl Into<FormattedText>) -> BaseTextEdit {
        #[state]
        let mut selection = Selection::empty(0);
        Self::with_selection(formatted_text, selection).on_selection_changed(|s| selection = s)
    }

    /// Returns whether TODO.
    pub fn editing_finished(&self) -> Option<Arc<str>> {
        self.editing_finished.value()
    }

    pub fn on_editing_finished(self, f: impl FnOnce(Arc<str>)) -> Self {
        self.editing_finished.map(f);
        self
    }

    /// Returns whether the text has changed.
    pub fn text_changed(&self) -> Option<Arc<str>> {
        self.text_changed.value()
    }

    pub fn on_text_changed(self, f: impl FnOnce(Arc<str>)) -> Self {
        self.text_changed.map(f);
        self
    }

    pub fn selection_changed(&self) -> Option<Selection> {
        self.selection_changed.value()
    }

    pub fn on_selection_changed(self, f: impl FnOnce(Selection)) -> Self {
        self.selection_changed.map(f);
        self
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

    /// Returns the position in the text (character offset between grapheme clusters) that is closest to the given point.
    fn text_position(&self, mut pos: Point) -> TextPosition {
        let paragraph = self.inner.inner().paragraph();
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> BoxLayout {
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

        BoxLayout::new(Size::new(width, height))
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

#[derive(Widget)]
pub struct TextEdit {
    inner: StyledBox<BaseTextEdit>,
}

impl TextEdit {
    /// Creates a new `TextEditInner` widget displaying the specified `FormattedText`.
    #[composable]
    pub fn with_selection(formatted_text: impl Into<FormattedText>, mut selection: Selection) -> TextEdit {
        let mut base = BaseTextEdit::with_selection(formatted_text, selection);

        TextEdit {
            inner: base.style(
                "border-radius: 3px;\
                 padding: 2px;\
                 width: 100%;\
                 border: solid 1px rgb(30 30 30);\
                 background: rgb(40 40 40);",
            ),
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
