use futures::channel::mpsc::Sender;
use futures::SinkExt;
use kyute::application::run;
use kyute::component::{CommandSink, Component, State};
use kyute::widget::constrained::ConstrainedBox;
use kyute::widget::form::Form;
use kyute::widget::slider::Slider;
use kyute::widget::textedit::TextEdit;
use kyute::widget::Button;
use kyute::window::Window;
use kyute::{BoxConstraints, BoxedWidget, Update, WidgetExt};
use std::marker::PhantomData;
use winit::window::WindowBuilder;

/*
struct SimpleComponent;

impl Component for SimpleComponent {

    // internal commands
    type Cmd = i32;

    fn command(&mut self, msg: i32) -> EventResult {

    }

    fn view(&mut self, mq: Messages<i32>) -> BoxedWidget
    {
        Button::new("Click Me").on_click(mq.emit(0)).boxed()
    }
}

struct ColorPicker {
    current: Color,
    palette: Vec<Color>,
}

enum InternalCommand {
    AddToPalette,
    RemoveFromPalette,
    ClearPalette,
    ColorChanged(Color)
}

// use case: replace the current document
// - menu > open document
// - dialog box: do you want to save the current document?
// - dialog box: file picker
// - update document view

// Component: DocumentView
// - on command received, return a Future<

impl Component for ColorPicker {

    // on command received:
    // - emit another internal command
    // - call an event handler
    // - spawn an async task

    // -> spawn a task for each component
    // -> awaits commands from:
    //      - its view
    //      - commands from other tasks
    // -> invokes callbacks (can be a one-shot future)


    fn command(&mut self, cmd: InternalCommand) -> Option<ColorPickerCommand> {
        match cmd {
            InternalCommand::ColorChanged(c) => {
                // update internal state
                self.on_color_changed(c);
            },
            InternalCommand::FileOpen => {

                //
                async {
                    let r = MessageBox::save_or_discard().await;
                    if r == Save {
                        // save document
                    }

                    // close document
                    cq.emit(DocumentView::Close).await;
                    // control yields to main event loop
                    //

                }

                // control returns to caller of command() (NodeTree::event())
                // for each emitted command:
                // - if it's an internal command, run it immediately
                // - spawned task: add it to the global queue
                // control returns to the main loop
                // - poll all the tasks
                //      - control enters the "FileOpen" async task
                //      - enter MessageBox, spawn another task
                //      - yield
                // - continue polling
                //      - control enters the MessageBox procedure
                //      - opens a window, add it to the global list
                //      - yield
                // - tasks are stalled, wait until next event
                // - next event received
                //      - routed to message box
                //      - routed to button
                //      - button emits events (callback)
                // - continue polling
                //      - MessageBox procedure unblocked because button emitted

            }
            _ => None
        }
    }

    fn view(&mut self, params: &PaletteParams, cmd: Commands<InternalCommand>) -> BoxedWidget {
        // display a grid of colors for every item in the palette,
        Flex::new()
            .push(ColorPickerCanvas::new().on_change(cmd.map(InternalCommand::ColorChanged)))
            .push(Button::new().on_click(cmd.emit(InternalCommand::AddToPalette)))
            .push(Button::new().on_click(cmd.emit(InternalCommand::ClearPalette)))

        ColorPicker::new()
            .on_click(cmd.emit(InternalCommand::AddToPalette))
            .content();

        wrap_component::<ColorPicker>(ColorPickerParams {
            on_click: cmd.emit(InternalCommand::AddToPalette)
            .. Default::default()
        });     // -> BoxedWidget
    }
}
*/

struct SimpleComponentState {
    counter: i32,
}

impl State for SimpleComponentState {
    type Cmd = ();

    fn command(&mut self, cmd: ()) -> Update {
        eprintln!("button clicked");
        self.counter += 1;
        Update::Relayout
    }
}

struct SimpleComponentParams {
    big: String,
}

struct SimpleComponent<'a>(&'a SimpleComponentParams);

impl<'a> Component for SimpleComponent<'a> {
    type State = SimpleComponentState;

    fn view(
        &self,
        state: &mut SimpleComponentState,
        commands: CommandSink<Self::State>,
    ) -> BoxedWidget {
        Button::new(format!("{}.{}", self.0.big, state.counter))
            .on_click(commands.emit(()))
            .boxed()
    }

    fn mount(&self) -> SimpleComponentState {
        SimpleComponentState { counter: 0 }
    }
}

struct SimpleApplicationState {
    data: SimpleComponentParams,
}

impl State for SimpleApplicationState {
    type Cmd = ();

    fn command(&mut self, command: ()) -> Update {
        Update::None
    }
}

// application state?
// - within a node, maybe wrapped in Arc
// -

#[derive(Copy, Clone, Debug)]
struct SimpleApplication;

impl Component for SimpleApplication {
    type State = SimpleApplicationState;

    fn view<'a>(
        &'a self,
        state: &'a mut SimpleApplicationState,
        cmds: CommandSink<Self::State>,
    ) -> BoxedWidget<'a> {
        use kyute::widget::*;
        Window::new(WindowBuilder::new())
            .contents(
                Flex::new(Axis::Vertical)
                    .push(
                        Flex::new(Axis::Horizontal)
                            .push(Baseline::new(20.0, Text::new("Horizontal Flex: ")))
                            .push(Baseline::new(
                                20.0,
                                Button::new("Button A").on_click(|_| eprintln!("clicked button A")),
                            ))
                            .push(Baseline::new(
                                20.0,
                                Button::new("Button B").on_click(|_| eprintln!("clicked button B")),
                            ))
                            .push(Baseline::new(20.0, Text::new("Baseline alignment test"))),
                    )
                    .push(ConstrainedBox::new(
                        BoxConstraints::new(0.0..400.0, ..),
                        Form::new()
                            .field("Field 1", TextEdit::new("Edit 1"))
                            .field("Field 2", TextEdit::new("Edit 2"))
                            .field("Field 3", TextEdit::new("Edit 3"))
                            .field("Field with a longer label 4", TextEdit::new("Edit 4"))
                            .field("Field 5", TextEdit::new("Edit 4"))
                            .field("Slider", Slider::new(0.5).min(0.0).max(1.0)),
                    ))
                    .push(
                        Flex::new(Axis::Vertical)
                            .push(Button::new("Vertical Flex:"))
                            .push(
                                Button::new("Button A").on_click(|_| eprintln!("clicked button A")),
                            )
                            .push(
                                Button::new("Button B").on_click(|_| eprintln!("clicked button B")),
                            )
                            .push(Text::new("Hello world"))
                            .push(TextEdit::new("WWWWWWWWWWWWWWWWWWW")),
                    )
                    .push(Text::new("last"))
                    .push(Window::new(WindowBuilder::new().with_title("child window")))
                    .push(SimpleComponent(&state.data)),
            )
            .boxed()
    }

    fn mount(&self) -> SimpleApplicationState {
        SimpleApplicationState {
            data: SimpleComponentParams {
                big: "hello world".to_string(),
            },
        }
    }
}

fn main() {
    pretty_env_logger::init();
    run(SimpleApplication);
}
