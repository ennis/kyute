//! Text editor widget.
use crate::event::Event;
use crate::layout::{BoxConstraints, Measurements, Offset, SideOffsets, Size};
use crate::{
    theme, Bounds, Environment, EventCtx, LayoutCtx, PaintCtx, Point, TypedWidget,
    Visual, Widget, WidgetExt,
};
use kyute_shell::drawing::context::{CompositeMode, FloodImage, InterpolationMode};
use kyute_shell::drawing::{Color, DrawTextOptions, Rect, RectExt, SolidColorBrush};
use kyute_shell::text::{TextFormat, TextFormatBuilder, TextLayout};
use log::trace;
use palette::{Srgb, Srgba};
use std::any::Any;
use std::ops::Range;
use unicode_segmentation::GraphemeCursor;
use winit::event::VirtualKeyCode;

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

impl Default for Selection {
    fn default() -> Self {
        Selection::empty(0)
    }
}

// layout strategy:
// - the text layout is calculated during widget layout, but also when an event causes the text to
//   change
// - update the text layout during painting if necessary

pub struct TextEditVisual {
    /// Formatting information.
    text_format: TextFormat,

    /// The text displayed to the user.
    text: String,

    /// The offset to the content area
    content_offset: Offset,

    /// The size of the content area
    content_size: Size,

    /// The text layout. None if not yet calculated.
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

    /// Flag that indicates that the text must be relayout
    needs_relayout: bool,
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
    }

    /// Inserts text.
    pub fn insert(&mut self, text: &str) {
        let min = self.selection.min();
        let max = self.selection.max();
        self.text.replace_range(min..max, text);
        self.selection = Selection::empty(min + text.len());
        self.needs_relayout = true;
        self.needs_repaint = true;
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

    pub fn select_all(&mut self) {
        self.selection.start = 0;
        self.selection.end = self.text.len();
        self.needs_repaint = true;
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
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {
        let bounds = ctx.bounds();

        // relayout if necessary
        if self.needs_relayout {
            trace!("text relayout");
            self.text_layout = TextLayout::new(
                ctx.platform(),
                &self.text,
                &self.text_format,
                self.content_size,
            )
            .unwrap();
        }

        // fetch colors
        let text_color = env.get(theme::TextColor);
        let caret_color = env.get(theme::TextEditCaretColor);
        let selected_bg_color = env.get(theme::SelectedTextBackgroundColor);
        let selected_text_color = env.get(theme::SelectedTextColor);

        let text_brush = SolidColorBrush::new(ctx, text_color);
        let caret_brush = SolidColorBrush::new(ctx, caret_color);
        let selected_bg_brush = SolidColorBrush::new(ctx, selected_bg_color);
        let selected_text_brush = SolidColorBrush::new(ctx, selected_text_color);

        ctx.save();
        ctx.transform(&bounds.origin.to_vector().to_transform());

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
                .hit_test_text_range(self.selection.min()..self.selection.max(), &bounds.origin)
                .unwrap();
            for sa in selected_areas {
                ctx.fill_rectangle(sa.bounds.round_out(), &selected_bg_brush);
            }
        }

        // text
        ctx.draw_text_layout(
            bounds.origin + self.content_offset,
            &self.text_layout,
            &text_brush,
            DrawTextOptions::ENABLE_COLOR_FONT,
        );

        // caret
        if ctx.is_focused() {
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
        }

        ctx.restore();
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
                let pos = self.position_to_text(p.pointer.position);
                if p.repeat_count == 2 {
                    // double-click selects all
                    self.select_all();
                } else {
                    self.set_cursor(pos);
                }
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
pub struct TextEdit {
    text: String,
}

impl TextEdit {
    pub fn new(text: impl Into<String>) -> TextEdit {
        TextEdit { text: text.into() }
    }
}

// textEdit events:
// - char event received
// - character is inserted
// - updated string is sent to the application

impl TypedWidget for TextEdit {
    type Visual = TextEditVisual;

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<TextEditVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<TextEditVisual>, Measurements) {
        let text = self.text;
        let platform = context.platform();

        // the only thing that we preserve across relayouts is the selection
        let selection = previous_visual.as_ref().map(|v| v.selection).unwrap_or_default();

        // text format
        let font_size = env.get(theme::TextEditFontSize);
        let font_name = env.get(theme::TextEditFontName);
        let text_format = TextFormatBuilder::new(context.platform())
            .family(font_name)
            .size(font_size as f32)
            .build()
            .expect("could not create text format");

        // take all available space
        let size = constraints.biggest();

        // calculate the size available to layout the text
        let padding: SideOffsets = env.get(theme::TextEditPadding);
        let content_size = Size::new(
            size.width - padding.horizontal(),
            size.height - padding.vertical(),
        );

        // content offset
        let content_offset = Offset::new(padding.left, padding.top);

        // determine if we need to relayout the text
        // we relayout the text if:
        // - the text has not actually been layout yet
        // - the text changed
        // - the size available for layout changed
        let text_layout = if let Some(visual) = previous_visual {
            if visual.text == text && visual.content_size == content_size {
                // recycle
                Some(visual.text_layout)
            } else {
                None
            }
        } else {
            None
        };

        let text_layout = text_layout.unwrap_or_else(|| {
            // layout the text within the content area
            TextLayout::new(platform, &text, &text_format, content_size).unwrap()
        });

        // calculate baseline
        let baseline = text_layout
            .line_metrics()
            .first()
            .map(|m| content_offset.y + m.baseline as f64);

        let measurements = Measurements { size, baseline };
        let visual = Box::new(TextEditVisual {
            text_format,
            text,
            content_offset,
            content_size,
            text_layout,
            selection,
            needs_repaint: true,
            needs_relayout: false
        });

        (visual, measurements)
    }
}
