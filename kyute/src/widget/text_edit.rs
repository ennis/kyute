//! Text editor widget.
use crate::{
    composable,
    core::Widget,
    drawing::ToSkia,
    env::Environment,
    event::{Event, Modifiers, PointerEventKind},
    state::{Signal, State},
    style::{BoxStyle, PaintCtxExt},
    text::{FormattedText, Selection, TextAffinity, TextPosition},
    theme,
    widget::{Container, Text},
    BoxConstraints, Color, EventCtx, LayoutCtx, Measurements, Offset, PaintCtx, Point, Rect, SideOffsets, Size,
    WidgetId, WidgetPod,
};
use keyboard_types::KeyState;
use skia_safe as sk;
use std::sync::Arc;
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
    c.prev_boundary(&text, 0).unwrap()
}

fn next_grapheme_cluster(text: &str, offset: usize) -> Option<usize> {
    let mut c = GraphemeCursor::new(offset, text.len(), true);
    c.next_boundary(&text, 0).unwrap()
}

/// Text editor widget.
pub struct TextEdit {
    id: WidgetId,

    /// Input formatted text.
    formatted_text: FormattedText,

    /// Current selection.
    selection: Selection,

    /// The offset to the content area
    content_offset: Offset,

    /// The size of the content area
    content_size: Size,

    editing_finished: Signal<Arc<str>>,
    text_changed: Signal<Arc<str>>,
    selection_changed: Signal<Selection>,

    inner: WidgetPod<Container<Text>>,
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

impl TextEdit {
    /// Creates a new `TextEdit` widget displaying the specified `FormattedText`.
    #[composable]
    pub fn with_selection(formatted_text: impl Into<FormattedText>, mut selection: Selection) -> TextEdit {
        let formatted_text = formatted_text.into();

        // clamp selection
        selection.start = selection.start.min(formatted_text.plain_text.len());
        selection.end = selection.end.min(formatted_text.plain_text.len());

        trace!(
            "TextEdit::with_selection: {:?}, {:?}",
            formatted_text.plain_text,
            selection
        );

        let inner = Container::new(Text::new(formatted_text.clone()))
            .box_style(theme::TEXT_EDIT)
            .content_padding(SideOffsets::new_all_same(2.0));

        TextEdit {
            id: WidgetId::here(),
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
    pub fn editing_finished(&self) -> Option<Arc<str>> {
        self.editing_finished.value()
    }

    /// Returns whether the text has changed.
    #[composable]
    pub fn text_changed(&self) -> Option<Arc<str>> {
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

    /// Returns the position in the text (character offset between grapheme clusters) that is closest to the given point.
    fn text_position(&self, pos: Point) -> TextPosition {
        let paragraph = self.inner.widget().contents().paragraph();
        TextPosition {
            position: paragraph.hit_test_point(pos - self.inner.widget().content_offset()).idx,
            affinity: TextAffinity::Upstream,
        }
    }

    fn notify_selection_changed(&self, ctx: &mut EventCtx, new_selection: Selection) {
        if new_selection != self.selection {
            eprintln!("notify selection changed {:?}->{:?}", self.selection, new_selection);
            self.selection_changed.signal(ctx, new_selection);
        }
    }

    fn notify_text_changed(&self, ctx: &mut EventCtx, new_text: Arc<str>) {
        self.text_changed.signal(ctx, new_text);
    }

    fn notify_editing_finished(&self, ctx: &mut EventCtx, new_text: Arc<str>) {
        self.editing_finished.signal(ctx, new_text);
    }
}

impl Widget for TextEdit {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        self.inner.layout(ctx, constraints, env)
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        // paint the text
        self.inner.paint(ctx, bounds, env);

        // paint the selection over it
        let offset = self.inner.widget().content_offset();
        let paragraph = self.inner.widget().contents().paragraph();
        let selection_boxes =
            paragraph.hit_test_text_range(self.selection.min()..self.selection.max(), Point::origin());
        for mut tb in selection_boxes {
            tb.bounds.origin += offset;
            ctx.draw_styled_box(tb.bounds, &BoxStyle::new().fill(Color::new(0.0, 0.1, 0.8, 0.5)), env);
        }

        // paint the caret
        if ctx.has_focus() {
            let caret_hit_test = paragraph.hit_test_text_position(TextPosition {
                position: self.selection.end,
                affinity: TextAffinity::Downstream,
            });

            //dbg!(caret_hit_test);
            let caret_color = env.get(theme::CARET_COLOR).unwrap();
            let paint = sk::Paint::new(caret_color.to_skia(), None);
            let pos = caret_hit_test.point + offset;
            ctx.canvas.draw_rect(
                Rect::new(pos.floor(), Size::new(1.0, caret_hit_test.metrics.bounds.size.height)).to_skia(),
                &paint,
            );
        }
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::FocusGained => {
                trace!("text edit: focus gained");
                ctx.request_redraw();
            }
            Event::FocusLost => {
                trace!("text edit: focus lost");
                let pos = self.selection.end;
                if self.selection.start != self.selection.end {
                    self.notify_selection_changed(ctx, Selection { start: pos, end: pos })
                }
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
                            self.notify_selection_changed(
                                ctx,
                                Selection {
                                    start: self.selection.start,
                                    end: text_pos.position,
                                },
                            );
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
                        let selection = if self.selection.is_empty() {
                            self.move_cursor(Movement::Left, true)
                        } else {
                            self.selection
                        };
                        let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, selection, "");
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
                        ctx.request_relayout();
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
                    }
                    keyboard_types::Key::ArrowLeft => {
                        let selection = self.move_cursor(Movement::Left, k.modifiers.contains(Modifiers::SHIFT));
                        self.notify_selection_changed(ctx, selection);
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::ArrowRight => {
                        let selection = self.move_cursor(Movement::Right, k.modifiers.contains(Modifiers::SHIFT));
                        self.notify_selection_changed(ctx, selection);
                        ctx.request_redraw();
                    }
                    keyboard_types::Key::Character(ref c) => {
                        // reject control characters (handle in KeyDown instead)
                        trace!("insert {:?}", c);
                        let (new_text, new_selection) = edit_text(&self.formatted_text.plain_text, self.selection, c);
                        self.notify_text_changed(ctx, new_text);
                        self.notify_selection_changed(ctx, new_selection);
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
