//! winit-based application wrapper.
//!
//! Provides the `run_application` function that opens the main window and translates the incoming
//! events from winit into the events expected by a kyute [`NodeTree`](crate::node::NodeTree).
use crate::event::{
    Event, InputEvent, InputState, KeyboardEvent, PointerButton, PointerButtonEvent,
    PointerButtons, PointerEvent, PointerState, WheelDeltaMode, WheelEvent,
};
use crate::layout::Size;
use crate::node::{NodeTree, RepaintRequest, DebugLayout, PaintOptions};
use crate::{Bounds, BoxConstraints, BoxedWidget, Environment, Measurements, Point, Visual, Widget, style};
use anyhow::Result;
use kyute_shell::drawing::Color;
use kyute_shell::platform::Platform;
use kyute_shell::window::{PlatformWindow, WindowDrawContext};
use log::trace;
use log::warn;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::{Rc, Weak};
use std::time::Duration;
use std::time::Instant;
use winit::event::{WindowEvent, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::window::{WindowBuilder, WindowId};
use crate::style::{StyleCollection, Shape, Brush, Border, StateFilter, ColorRef};
use config::FileFormat;
use std::fs::File;
use std::io::Write;

/// Encapsulates the behavior of an application.
pub trait Application {
    /// The type of actions emitted by the view and handled by the application.
    type Action: 'static;

    /// Update state
    /// TODO rethink
    fn update(&mut self);

    /// Returns the view (widget tree) to display to the user.
    fn view(&mut self) -> BoxedWidget;

    // Called whenever an OpenGL viewport needs rendering.
    // Used with `OpenGlViewportWidget`.
    // fn render_gl(&mut self, framebuffer: GLuint, viewport_id: ViewportId, bounds: Bounds);
}

/// Context needed to open a window.
pub struct WindowCtx<'a> {
    pub(crate) platform: &'a Platform,
    event_loop: &'a EventLoopWindowTarget<()>,
    new_windows: Vec<Rc<RefCell<Window>>>,
}

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: winit::event::DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

/// A window managed by kyute with a cached visual node.
struct Window {
    window: PlatformWindow,
    tree: NodeTree,
    inputs: InputState,
    // for double-click detection
    last_click: Option<LastClick>,
    /// Widget styles for the window.
    style_collection: Rc<StyleCollection>,
    debug_layout: DebugLayout
}

impl Window {
    /// Opens a window and registers the window into the event loop.
    pub fn open(ctx: &mut WindowCtx, builder: WindowBuilder, style_collection: Rc<StyleCollection>) -> Result<Rc<RefCell<Window>>> {
        // create the platform window
        let window = PlatformWindow::new(ctx.event_loop, builder, ctx.platform, true)?;
        //let size: (f64, f64) = window.window().inner_size().to_logical::<f64>(1.0).into();

        // create the default visual
        let tree = NodeTree::new();

        let window = Window {
            window,
            tree,
            inputs: InputState::default(),
            last_click: None,
            style_collection,
            debug_layout: DebugLayout::None,
        };
        let window = Rc::new(RefCell::new(window));

        ctx.new_windows.push(window.clone());
        Ok(window)
    }

    /// Returns the ID of the window.
    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    /// deliver window event
    fn window_event(&mut self, ctx: &mut WindowCtx, window_event: &WindowEvent) {
        let event_result = match window_event {
            WindowEvent::Resized(size) => {
                self.window.resize((*size).into());
                return;
            }
            WindowEvent::ModifiersChanged(m) => {
                self.inputs.mods = *m;
                return;
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                // update pointer state
                let pointer_state = self
                    .inputs
                    .pointers
                    .entry(*device_id)
                    .or_insert(PointerState::default());
                let button = match button {
                    winit::event::MouseButton::Left => PointerButton::LEFT,
                    winit::event::MouseButton::Right => PointerButton::RIGHT,
                    winit::event::MouseButton::Middle => PointerButton::MIDDLE,
                    winit::event::MouseButton::Other(3) => PointerButton::X1,
                    winit::event::MouseButton::Other(4) => PointerButton::X2,
                    winit::event::MouseButton::Other(b) => PointerButton(*b as u16),
                };
                match state {
                    winit::event::ElementState::Pressed => pointer_state.buttons.set(button),
                    winit::event::ElementState::Released => pointer_state.buttons.reset(button),
                };

                let click_time = Instant::now();
                let position = pointer_state.position;

                // determine the repeat count (double-click, triple-click, etc.) for button down event
                let repeat_count = match &mut self.last_click {
                    Some(ref mut last)
                        if last.device_id == *device_id
                            && last.button == button
                            && last.position == position
                            && (click_time - last.time) < ctx.platform.double_click_time() =>
                    {
                        // same device, button, position, and within the platform specified double-click time
                        match state {
                            winit::event::ElementState::Pressed => {
                                last.repeat_count += 1;
                                last.repeat_count
                            }
                            winit::event::ElementState::Released => {
                                // no repeat for release events (although that could be possible?),
                                1
                            }
                        }
                    }
                    other => {
                        // no match, reset
                        match state {
                            winit::event::ElementState::Pressed => {
                                *other = Some(LastClick {
                                    device_id: *device_id,
                                    button,
                                    position,
                                    time: click_time,
                                    repeat_count: 1,
                                });
                            }
                            winit::event::ElementState::Released => {
                                *other = None;
                            }
                        };
                        1
                    }
                };

                let p = PointerButtonEvent {
                    pointer: PointerEvent {
                        position,
                        window_position: position,
                        modifiers: self.inputs.mods,
                        buttons: pointer_state.buttons,
                        pointer_id: *device_id,
                    },
                    button: Some(button),
                    repeat_count,
                };

                let e = match state {
                    winit::event::ElementState::Pressed => Event::PointerDown(p),
                    winit::event::ElementState::Released => Event::PointerUp(p),
                };

                self.tree.event(ctx, &self.window, &self.inputs, &e)
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                let logical = position.to_logical::<f64>(self.window.window().scale_factor());
                let logical = Point::new(logical.x, logical.y);

                let pointer_state = self
                    .inputs
                    .pointers
                    .entry(*device_id)
                    .or_insert(PointerState::default());
                pointer_state.position = logical;

                let p = PointerEvent {
                    position: logical,
                    window_position: logical,
                    modifiers: self.inputs.mods,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                };

                let result = self.tree
                    .event(ctx, &self.window, &self.inputs, &Event::PointerMove(p));

                // force redraw if bounds debugging mode is on
                if self.debug_layout != DebugLayout::None {
                    RepaintRequest::Repaint
                } else {
                    result
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => {
                let pointer = self.inputs.synthetic_pointer_event(*device_id);
                if let Some(pointer) = pointer {
                    let wheel_event = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => WheelEvent {
                            pointer,
                            delta_x: *x as f64,
                            delta_y: *y as f64,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Line,
                        },
                        winit::event::MouseScrollDelta::PixelDelta(pos) => WheelEvent {
                            pointer,
                            delta_x: pos.x,
                            delta_y: pos.y,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Pixel,
                        },
                    };
                    self.tree
                        .event(ctx, &self.window, &self.inputs, &Event::Wheel(wheel_event))
                } else {
                    warn!("wheel event received but pointer position is not yet known");
                    return;
                }
            }
            WindowEvent::ReceivedCharacter(char) => self.tree.event(
                ctx,
                &self.window,
                &self.inputs,
                &Event::Input(InputEvent { character: *char }),
            ),
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                let keyboard_event = KeyboardEvent {
                    scan_code: input.scancode,
                    key: input.virtual_keycode,
                    repeat: false, // TODO
                    modifiers: self.inputs.mods,
                };

                let event = match input.state {
                    winit::event::ElementState::Pressed => Event::KeyDown(keyboard_event),
                    winit::event::ElementState::Released => Event::KeyUp(keyboard_event),
                };

                // Ctrl+F12 cycles through bounds debugging modes
                if input.state == winit::event::ElementState::Pressed && input.virtual_keycode == Some(VirtualKeyCode::F12) && self.inputs.mods.ctrl() {
                    self.debug_layout = match self.debug_layout {
                        DebugLayout::None => DebugLayout::Hover,
                        DebugLayout::Hover => DebugLayout::All,
                        DebugLayout::All => DebugLayout::None,
                    };
                    RepaintRequest::Repaint
                }
                else {
                    self.tree.event(ctx, &self.window, &self.inputs, &event)
                }
            }

            _ => {
                return;
            }
        };

        // handle follow-up actions
        match event_result {
            RepaintRequest::Repaint | RepaintRequest::Relayout => {
                // TODO ask for relayout
                self.window.window().request_redraw();
            }
            _ => {}
        }
    }

    /// Updates the current visual tree for this stage.
    fn relayout(&mut self, window_ctx: &mut WindowCtx, env: Environment, widget: BoxedWidget) {
        // get window logical size
        let size: (f64, f64) = self
            .window
            .window()
            .inner_size()
            .to_logical::<f64>(1.0)
            .into();
        let size: Size = size.into();
        dbg!(size);
        // perform layout, update the visual node
        self.tree
            .layout(widget, size, &BoxConstraints::loose(size), env, window_ctx);
        // request a redraw of this window
        self.window.window().request_redraw()
    }

    /// Called when the window needs to be repainted.
    fn paint(&mut self, platform: &Platform) {
        {
            let mut wdc = WindowDrawContext::new(&mut self.window);
            wdc.clear(Color::new(0.326, 0.326, 0.326, 1.0));
            let options = PaintOptions {
                debug_draw_bounds: self.debug_layout
            };
            self.tree.paint(platform, &mut wdc, &self.style_collection, &self.inputs, &options);
        }
        self.window.present();
    }
}


/// Loads the application style.
fn load_application_style(cfg: &config::Config) -> Rc<StyleCollection> {
    let path = cfg.get_str("app_style").expect("no `app_style` file specified in application settings (`Settings.toml`)");
    // load RON
    let f = File::open(&path).expect("failed to open the application style");
    let app_style : StyleCollection = ron::de::from_reader(f).expect("failed to load the application style");
    Rc::new(app_style)
}

/// Creates a default application style
fn write_default_application_style()
{
    use style::{Length, Angle, Shape, StyleSet, Brush, GradientType, Style, State, Palette, PaletteIndex, ColorRef, BlendMode, BorderPosition};

    const FRAME_BG_SUNKEN_COLOR: PaletteIndex =  PaletteIndex(0);
    const FRAME_BG_NORMAL_COLOR: PaletteIndex =  PaletteIndex(1);
    const FRAME_BG_RAISED_COLOR: PaletteIndex =  PaletteIndex(2);
    const FRAME_FOCUS_COLOR: PaletteIndex =  PaletteIndex(3);
    const FRAME_BORDER_COLOR: PaletteIndex =  PaletteIndex(4);
    const BUTTON_BACKGROUND_TOP_COLOR: PaletteIndex =  PaletteIndex(5);
    const BUTTON_BACKGROUND_BOTTOM_COLOR: PaletteIndex =  PaletteIndex(6);
    const BUTTON_BACKGROUND_TOP_COLOR_HOVER: PaletteIndex =  PaletteIndex(7);
    const BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER: PaletteIndex =  PaletteIndex(8);
    const BUTTON_BORDER_BOTTOM_COLOR: PaletteIndex =  PaletteIndex(9);
    const BUTTON_BORDER_TOP_COLOR: PaletteIndex =  PaletteIndex(10);
    const WIDGET_OUTER_GROOVE_BOTTOM_COLOR: PaletteIndex =  PaletteIndex(11);
    const WIDGET_OUTER_GROOVE_TOP_COLOR: PaletteIndex =  PaletteIndex(12);
    const FRAME_BG_SUNKEN_COLOR_HOVER: PaletteIndex =  PaletteIndex(13);

    let palette = Palette {
        entries: vec![
            Color::new(0.227, 0.227, 0.227, 1.0), // FRAME_BG_SUNKEN_COLOR
            Color::new(0.326, 0.326, 0.326, 1.0), // FRAME_BG_NORMAL_COLOR
            Color::new(0.424, 0.424, 0.424, 1.0), // FRAME_BG_RAISED_COLOR
            Color::new(0.600, 0.600, 0.900, 1.0), // FRAME_FOCUS_COLOR
            Color::new(0.130, 0.130, 0.130, 1.0), // FRAME_BORDER_COLOR
            Color::new(0.450, 0.450, 0.450, 1.0), // BUTTON_BACKGROUND_TOP_COLOR
            Color::new(0.400, 0.400, 0.400, 1.0), // BUTTON_BACKGROUND_BOTTOM_COLOR
            Color::new(0.500, 0.500, 0.500, 1.0), // BUTTON_BACKGROUND_TOP_COLOR_HOVER
            Color::new(0.450, 0.450, 0.450, 1.0), // BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER
            Color::new(0.100, 0.100, 0.100, 1.0), // BUTTON_BORDER_BOTTOM_COLOR
            Color::new(0.180, 0.180, 0.180, 1.0), // BUTTON_BORDER_TOP_COLOR
            Color::new(1.000, 1.000, 1.000, 0.2), // WIDGET_OUTER_GROOVE_BOTTOM_COLOR
            Color::new(1.000, 1.000, 1.000, 0.0), // WIDGET_OUTER_GROOVE_TOP_COLOR
            Color::new(0.180, 0.180, 0.180, 1.0), // FRAME_BG_SUNKEN_COLOR_HOVER
        ]
    };

    let button_style_set = style::StyleSet {
        shape: Shape::RoundedRect(style::Length::Dip(2.0)),
        styles: vec![
            style::Style {
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR)),
                    ],
                    reverse: false
                }),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }, Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                            (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                        ],
                        reverse: false
                    },
                }],
                .. Style::default()
            },

            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR_HOVER)),
                    ],
                    reverse: false
                }),
                .. Style::default()
            }
        ]
    };

    let slider_knob = style::StyleSet {
        shape: Shape::Path("M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z".to_string()),
        styles: vec![
            style::Style {
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR)),
                    ],
                    reverse: false
                }),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }],
                .. Style::default()
            },

            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::Gradient {
                    angle: Angle::degrees(90.0),
                    ty: GradientType::Linear,
                    stops: vec![
                        (0.0, ColorRef::Palette(BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER)),
                        (1.0, ColorRef::Palette(BUTTON_BACKGROUND_TOP_COLOR_HOVER)),
                    ],
                    reverse: false
                }),
                .. Style::default()
            }
        ]
    };

    let slider_track = style::StyleSet {
        shape: Shape::RoundedRect(style::Length::Dip(2.0)),
        styles: vec![
            style::Style {
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }, Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                            (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                        ],
                        reverse: false
                    },
                }],
                .. Style::default()
            },
        ]
    };

    let text_box_style_set = style::StyleSet {
        shape: Shape::Rect,
        styles: vec![
            style::Style {
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }, Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Outside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::Gradient {
                        angle: Angle::degrees(90.0),
                        ty: GradientType::Linear,
                        stops: vec![
                            (0.0, ColorRef::Palette(WIDGET_OUTER_GROOVE_BOTTOM_COLOR)),
                            (0.3, ColorRef::Palette(WIDGET_OUTER_GROOVE_TOP_COLOR)),
                        ],
                        reverse: false
                    },
                }],
                .. Style::default()
            },

            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                .. Style::default()
            }
        ]
    };

    let mut style_sets = HashMap::new();
    style_sets.insert("button".to_string(), button_style_set);
    style_sets.insert("text_box".to_string(), text_box_style_set);
    style_sets.insert("slider_knob".to_string(), slider_knob);
    style_sets.insert("slider_track".to_string(), slider_track);

    let style_collection = StyleCollection {
        style_sets,
        palettes: vec![palette]
    };

    // serialize
    let s = ron::ser::to_string_pretty(&style_collection,ron::ser::PrettyConfig::new()).expect("failed to export string");
    let mut file = File::create("default_style.ron").expect("could not open default_style.ron");
    file.write_all(s.as_bytes()).expect("could not write default style");
    log::info!("default style written to default_style.ron");
}


/// Runs the specified application.
pub fn run_application<A: Application + 'static>(mut app: A) -> ! {
    // winit event loop
    let event_loop = winit::event_loop::EventLoop::new();
    // platform-specific, window-independent states
    let platform = unsafe { Platform::init() };

    // load application settings
    let mut cfg = config::Config::new();
    cfg.merge(config::File::with_name("Settings").format(config::FileFormat::Toml));
    write_default_application_style();
    let app_style = load_application_style(&cfg);

    // create a window to render the main view.
    let mut win_ctx = WindowCtx {
        platform: &platform,
        event_loop: &event_loop,
        new_windows: Vec::new(),
    };
    let main_window = Window::open(&mut win_ctx, WindowBuilder::new().with_title("Default"), app_style)
        .expect("failed to create main window");

    // ID -> Weak<Window>
    let mut open_windows = HashMap::new();
    open_windows.insert(main_window.borrow().id(), Rc::downgrade(&main_window));

    // perform the initial layout
    main_window
        .borrow_mut()
        .relayout(&mut win_ctx, Environment::new(), app.view());

    event_loop.run(move |event, elwt, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut win_ctx = WindowCtx {
            platform: &platform,
            event_loop: elwt,
            new_windows: Vec::new(),
        };

        match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                // deliver event to the window
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().window_event(&mut win_ctx, &event);
                    }
                }

                if let WindowEvent::Resized(size) = event {
                    // A window has been resized.
                    // Note that, currently, we have no way of relayouting each window separately
                    // (since Application::view() returns the widget tree for all windows).
                    // For now, just relayout everything.

                    // get the widget tree
                    let widget = app.view();

                    // the root of the widget tree is the main window, update it:
                    // this will also send a redraw request for all affected windows.
                    main_window
                        .borrow_mut()
                        .relayout(&mut win_ctx, Environment::new(), widget);
                }
            }

            winit::event::Event::RedrawRequested(window_id) => {
                // A window needs to be repainted
                if let Some(window) = open_windows.get(&window_id) {
                    if let Some(window) = window.upgrade() {
                        window.borrow_mut().paint(&platform);
                    }
                }
            }
            _ => (),
        }

        // remove (and close) windows that were dropped
        open_windows.retain(|_, window| window.strong_count() != 0);

        // add the newly-created windows to the list of managed windows
        open_windows.extend(
            mem::take(&mut win_ctx.new_windows)
                .drain(..)
                .map(|v| (v.borrow().id(), Rc::downgrade(&v))),
        );
    })
}
