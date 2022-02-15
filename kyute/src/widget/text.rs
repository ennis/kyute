use crate::{
    composable,
    text::{FormattedText, FormattedTextParagraph},
    BoxConstraints, Environment, Event, EventCtx, LayoutCtx, Measurements, PaintCtx, Point,
    Rect, Size, Widget,
};
use kyute_shell::drawing::ToSkia;
use std::cell::{Ref, RefCell};

/// Displays formatted text.
pub struct Text {
    /// Input formatted text.
    formatted_text: FormattedText,
    /// The formatted paragraph, calculated during layout. `None` if not yet calculated.
    paragraph: RefCell<Option<FormattedTextParagraph>>,
}

impl Text {
    /// Creates a new text element.
    #[composable(uncached)]
    pub fn new(formatted_text: impl Into<FormattedText>) -> Text {
        let formatted_text = formatted_text.into();
        Text {
            formatted_text,
            paragraph: RefCell::new(None),
        }
    }

    /// Returns a reference to the formatted text paragraph.
    pub fn formatted_paragraph(&self) -> Ref<FormattedTextParagraph> {
        Ref::map(self.paragraph.borrow(), |x| {
            x.as_ref()
                .expect("`Text::formatted_paragraph` called before layout")
        })
    }
}

impl Widget for Text {
    fn event(&self, _ctx: &mut EventCtx, _event: &mut Event, _env: &Environment) {}

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
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

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        let mut paragraph = self.paragraph.borrow_mut();
        let paragraph = paragraph.as_mut().expect("paint called before layout");
        paragraph
            .0
            .paint(&mut ctx.canvas, Point::origin().to_skia());
    }
}
