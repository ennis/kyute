use kyute::application::{run_application, Application};
use kyute::widget::form::Form;
use kyute::widget::textedit::{TextEdit, TextEditBase};
use kyute::{BoxedWidget, WidgetExt};

struct SimpleApplication;

impl Application for SimpleApplication {
    type Action = ();

    fn update(&mut self, _actions: &[()]) {
        unimplemented!()
    }

    fn view(&mut self) -> BoxedWidget<()> {
        use kyute::widget::*;
        Flex::new(Axis::Vertical)
            .push(
                Flex::new(Axis::Horizontal)
                    .push(Baseline::new(20.0, Text::new("Horizontal Flex: ")))
                    .push(Baseline::new(20.0, Button::new("Button A")))
                    .push(Baseline::new(20.0, Button::new("Button B")))
                    .push(Baseline::new(20.0, Text::new("Baseline alignment test"))),
            )
            .push(
                Form::new()
                    .field("Field 1", TextEdit::new("Edit 1"))
                    .field("Field 2", TextEdit::new("Edit 2"))
                    .field("Field 3", TextEdit::new("Edit 3"))
                    .field("Field with a longer label 4", TextEdit::new("Edit 4"))
                    .field("Field 5", TextEdit::new("Edit 4"))
            )
            .push(
                Flex::new(Axis::Vertical)
                    .push(Button::new("Vertical Flex:"))
                    .push(Button::new("Button A"))
                    .push(Button::new("Button B"))
                    .push(Text::new("Hello world"))
                    .push(TextEdit::new("WWWWWWWWWWWWWWWWWWW")),
            )
            .push(Text::new("last"))

            .boxed()
    }
}

fn main() {
    pretty_env_logger::init();

    run_application(SimpleApplication);
}
