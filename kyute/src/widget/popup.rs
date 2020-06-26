//! Popup windows
use crate::{TypedWidget, BoxConstraints, Measurements, LayoutCtx, Environment, BoxedWidget, WidgetExt};
use crate::window::Window;
use winit::window::WindowBuilder;
use winit::dpi::LogicalSize;
use crate::widget::DummyWidget;
use winit::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};

pub struct Popup<'a> {
    contents: BoxedWidget<'a>,
    on_close: Option<Box<dyn FnMut()>>
}

impl<'a> Popup<'a> {
    pub fn new() -> Popup<'a> {
        Popup {
            contents: DummyWidget.boxed(),
            on_close: None
        }
    }
}

impl<'a> TypedWidget for Popup<'a>
{
    type Visual = <Window<'a> as TypedWidget>::Visual;

    fn layout(self,
              context: &mut LayoutCtx,
              previous_visual: Option<Box<Self::Visual>>,
              constraints: &BoxConstraints,
              env: Environment) -> (Box<Self::Visual>, Measurements)
    {
        // measure the contents
        //let max_width = 600.0;
        let constraints = constraints.enforce(&BoxConstraints::new(100.0..600.0, 100.0..));

        let window_builder = WindowBuilder::new()
            .with_decorations(false)
            .with_always_on_top(true)
            .with_popup(true)
            .with_transparent(true)
            .with_resizable(false);

        Window::new(window_builder)
            .parent_window(context.parent_window().expect("cannot create a popup without a parent window"))
            .on_focus_lost(|| eprintln!("Popup focus lost"))
            .on_close_requested(|| eprintln!("Popup close requested"))
            .contents(self.contents)
            .layout(context, previous_visual, &constraints, env)
    }
}