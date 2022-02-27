use crate::{
    composable,
    drawing::ToSkia,
    text::{FormattedText, FormattedTextParagraph},
    BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx, Point, Rect, Size, Widget,
    WidgetId,
};
use std::cell::{Ref, RefCell};

/// Displays formatted text.
pub struct TextV1 {
    /// Input formatted text.
    formatted_text: FormattedText,
    /// The formatted paragraph, calculated during layout. `None` if not yet calculated.
    paragraph: RefCell<Option<FormattedTextParagraph>>,
}

impl TextV1 {
    /// Creates a new text element.
    #[composable]
    #[deprecated(note = "Use `Text` instead")]
    pub fn new(formatted_text: impl Into<FormattedText>) -> TextV1 {
        let formatted_text = formatted_text.into();
        TextV1 {
            formatted_text,
            paragraph: RefCell::new(None),
        }
    }

    /// Returns a reference to the formatted text paragraph.
    pub fn formatted_paragraph(&self) -> Ref<FormattedTextParagraph> {
        Ref::map(self.paragraph.borrow(), |x| {
            x.as_ref().expect("`Text::formatted_paragraph` called before layout")
        })
    }
}

impl Widget for TextV1 {
    fn widget_id(&self) -> Option<WidgetId> {
        // no need for a stable identity
        None
    }

    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, _env: &Environment) -> Measurements {
        let available_width = constraints.max_width();
        //let available_height = constraints.max_height();
        let paragraph = self.formatted_text.format(available_width);

        // measure the paragraph
        let text_height = paragraph.0.height() as f64;
        let baseline = paragraph.0.alphabetic_baseline() as f64;
        let size = Size::new(available_width, constraints.constrain_height(text_height)); // TODO?

        // stash the laid out paragraph for rendering
        self.paragraph.replace(Some(paragraph));

        Measurements {
            bounds: size.into(),
            baseline: Some(baseline),
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, _env: &Environment) {
        let mut paragraph = self.paragraph.borrow_mut();
        let paragraph = paragraph.as_mut().expect("paint called before layout");
        paragraph.0.paint(&mut ctx.canvas, Point::origin().to_skia());
    }
}
