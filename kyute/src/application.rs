//! winit-based application wrapper.
//!
//! Provides the `run_application` function that opens the main window and translates the incoming
//! events from winit into the events expected by a kyute [`NodeTree`](crate::node::NodeTree).
use crate::{
    component::{Action, Component},
    event::{
        Event, InputEvent, InputState, KeyboardEvent, PointerButton, PointerButtonEvent,
        PointerButtons, PointerEvent, PointerState, WheelDeltaMode, WheelEvent,
    },
    node::{DebugLayout, NodeTree, PaintOptions, RepaintRequest},
    style,
    style::{Border, Brush, ColorRef, Shape, StateFilter, StyleCollection},
    BoxConstraints, BoxedWidget, Environment, Measurements, Point, Rect, Size, Update, Visual,
    Widget, WidgetExt,
};
use anyhow::Result;
use config::FileFormat;
use futures::{
    channel::mpsc::{unbounded, Receiver, Sender, UnboundedReceiver},
    executor::{LocalPool, LocalSpawner},
    task::LocalSpawnExt,
    SinkExt, StreamExt,
};
use generational_indextree::NodeId;
use kyute_shell::{
    drawing::Color,
    platform::Platform,
    window::{PlatformWindow, WindowDrawContext},
};
use log::{trace, warn};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    mem,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};
use winit::{
    event::{VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};

/// Loads the application style.
fn load_application_style(cfg: &config::Config) -> Rc<StyleCollection> {
    let path = cfg
        .get_str("app_style")
        .unwrap_or("default_style.ron".to_string());
    // load RON
    let f = File::open(&path).expect("failed to open the application style");
    let app_style: StyleCollection =
        ron::de::from_reader(f).expect("failed to load the application style");
    Rc::new(app_style)
}

/// Creates a default application style
fn write_default_application_style() {
    use style::{
        Angle, BlendMode, BorderPosition, Brush, ColorRef, GradientType, Length, Palette,
        PaletteIndex, Shape, State, Style, StyleSet,
    };

    const FRAME_BG_SUNKEN_COLOR: PaletteIndex = PaletteIndex(0);
    const FRAME_BG_NORMAL_COLOR: PaletteIndex = PaletteIndex(1);
    const FRAME_BG_RAISED_COLOR: PaletteIndex = PaletteIndex(2);
    const FRAME_FOCUS_COLOR: PaletteIndex = PaletteIndex(3);
    const FRAME_BORDER_COLOR: PaletteIndex = PaletteIndex(4);
    const BUTTON_BACKGROUND_TOP_COLOR: PaletteIndex = PaletteIndex(5);
    const BUTTON_BACKGROUND_BOTTOM_COLOR: PaletteIndex = PaletteIndex(6);
    const BUTTON_BACKGROUND_TOP_COLOR_HOVER: PaletteIndex = PaletteIndex(7);
    const BUTTON_BACKGROUND_BOTTOM_COLOR_HOVER: PaletteIndex = PaletteIndex(8);
    const BUTTON_BORDER_BOTTOM_COLOR: PaletteIndex = PaletteIndex(9);
    const BUTTON_BORDER_TOP_COLOR: PaletteIndex = PaletteIndex(10);
    const WIDGET_OUTER_GROOVE_BOTTOM_COLOR: PaletteIndex = PaletteIndex(11);
    const WIDGET_OUTER_GROOVE_TOP_COLOR: PaletteIndex = PaletteIndex(12);
    const FRAME_BG_SUNKEN_COLOR_HOVER: PaletteIndex = PaletteIndex(13);

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
        ],
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
                    reverse: false,
                }),
                borders: vec![
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Inside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                    },
                    Border {
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
                            reverse: false,
                        },
                    },
                ],
                ..Style::default()
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
                    reverse: false,
                }),
                ..Style::default()
            },
        ],
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
                    reverse: false,
                }),
                borders: vec![Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                }],
                ..Style::default()
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
                    reverse: false,
                }),
                ..Style::default()
            },
        ],
    };

    let slider_track = style::StyleSet {
        shape: Shape::RoundedRect(style::Length::Dip(2.0)),
        styles: vec![style::Style {
            fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
            borders: vec![
                Border {
                    opacity: 1.0,
                    blend_mode: BlendMode::Normal,
                    position: BorderPosition::Inside(Length::zero()),
                    width: Length::Dip(1.0),
                    brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                },
                Border {
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
                        reverse: false,
                    },
                },
            ],
            ..Style::default()
        }],
    };

    let text_box_style_set = style::StyleSet {
        shape: Shape::Rect,
        styles: vec![
            style::Style {
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                borders: vec![
                    Border {
                        opacity: 1.0,
                        blend_mode: BlendMode::Normal,
                        position: BorderPosition::Inside(Length::zero()),
                        width: Length::Dip(1.0),
                        brush: Brush::SolidColor(ColorRef::Palette(BUTTON_BORDER_BOTTOM_COLOR)),
                    },
                    Border {
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
                            reverse: false,
                        },
                    },
                ],
                ..Style::default()
            },
            style::Style {
                state_filter: StateFilter {
                    value: State::HOVER,
                    mask: State::HOVER,
                },
                fill: Some(Brush::SolidColor(ColorRef::Palette(FRAME_BG_SUNKEN_COLOR))),
                ..Style::default()
            },
        ],
    };

    let mut style_sets = HashMap::new();
    style_sets.insert("button".to_string(), button_style_set);
    style_sets.insert("text_box".to_string(), text_box_style_set);
    style_sets.insert("slider_knob".to_string(), slider_knob);
    style_sets.insert("slider_track".to_string(), slider_track);

    let style_collection = StyleCollection {
        style_sets,
        palettes: vec![palette],
    };

    // serialize
    let s = ron::ser::to_string_pretty(&style_collection, ron::ser::PrettyConfig::new())
        .expect("failed to export string");
    let mut file = File::create("default_style.ron").expect("could not open default_style.ron");
    file.write_all(s.as_bytes())
        .expect("could not write default style");
    log::info!("default style written to default_style.ron");
}

/// Context needed to open a window.
pub struct AppCtx {
    /// Platform services (text, drawing, etc.).
    pub(crate) platform: Platform,
    /// The style collection of the application.
    pub(crate) style: Rc<StyleCollection>,
    /// Open windows, mapped to their corresponding node in the node tree.
    pub(crate) windows: HashMap<WindowId, NodeId>,
    /// Spawner to spawn the tasks in charge of forwarding comments to the components.
    pub(crate) spawner: LocalSpawner,
    /// Pending actions
    pub(crate) actions: Vec<Action>,
    /// Root widget generator
    pub(crate) root_widget: Box<dyn FnMut() -> BoxedWidget<'static> + 'static>,
}

impl AppCtx {
    fn new(
        platform: Platform,
        style_collection: Rc<StyleCollection>,
        spawner: LocalSpawner,
        root_widget: impl FnMut() -> BoxedWidget<'static> + 'static,
    ) -> AppCtx {
        AppCtx {
            platform,
            style: style_collection,
            windows: HashMap::new(),
            spawner,
            actions: Vec::new(),
            root_widget: Box::new(root_widget),
        }
    }

    fn dispatch_actions(&mut self, tree: &mut NodeTree, event_loop: &EventLoopWindowTarget<()>) {
        let actions = mem::replace(&mut self.actions, Vec::new());
        // set of windows to repaint
        let mut relayout = false;
        let mut nodes_to_repaint = Vec::new();

        for action in actions {
            let mut node_data = if let Some(node) = tree.arena.get_mut(action.target) {
                node.get_mut()
            } else {
                log::warn!("action targets deleted node");
                return;
            };

            let update = (action.run)(node_data);

            match update {
                Update::Repaint => {
                    nodes_to_repaint.push(action.target);
                }
                Update::Relayout => {
                    relayout = true;
                }
                Update::None => {}
            }
        }

        if relayout {
            self.layout(tree, event_loop)
        } else {
            let mut windows_to_repaint = HashMap::new();
            // determine windows to repaint
            for node_to_repaint in nodes_to_repaint {
                // repaint the window that owns the action target
                if let Some(parent_window) = tree.find_parent_window(node_to_repaint) {
                    windows_to_repaint.insert(parent_window.id(), parent_window);
                }
            }

            for window in windows_to_repaint.values() {
                window.window().request_redraw()
            }
        }
    }

    fn layout(&mut self, tree: &mut NodeTree, event_loop: &EventLoopWindowTarget<()>) {
        // the root available space is infinite, technically, this can produce infinitely big visuals,
        // but this is never the case for visuals under windows (they are constrained by the size of the window).
        // Note that there's no window created by default. The user should create window widgets to
        // have something to render to.
        tree.layout(
            (self.root_widget)(),
            Size::zero(),
            &BoxConstraints::new(.., ..),
            Environment::new(),
            self,
            event_loop,
        );
    }

    fn run(mut self, mut tree: NodeTree, event_loop: EventLoop<()>) {
        // run event loop
        event_loop.run(move |event, elwt, control_flow| {
            *control_flow = ControlFlow::Wait;

            // helper to extract a visual from the node tree
            fn replace_visual(
                tree: &mut NodeTree,
                node_id: NodeId,
                with: Option<Box<dyn Visual>>,
            ) -> Option<Box<dyn Visual>> {
                tree.arena
                    .get_mut(node_id)
                    .and_then(|n| mem::replace(&mut n.get_mut().visual, with))
            }

            match event {
                winit::event::Event::WindowEvent { window_id, event } => {
                    // deliver event to the target window in the node tree
                    if let Some(node_id) = self.windows.get(&window_id).cloned() {
                        // see RedrawRequested for more comments
                        let mut v = replace_visual(&mut tree, node_id, None);
                        v.as_mut()
                            .and_then(|v| v.window_handler_mut())
                            .map(|v| v.window_event(&mut self, &event, &mut tree, node_id));
                        replace_visual(&mut tree, node_id, v);
                    }

                    if let WindowEvent::Resized(size) = event {
                        self.layout(&mut tree, elwt)
                    }

                    self.dispatch_actions(&mut tree, elwt)
                }

                winit::event::Event::RedrawRequested(window_id) => {
                    // A window needs to be repainted
                    if let Some(node_id) = self.windows.get(&window_id).cloned() {
                        // sleight-of-hand: extract the visual from the tree, then put it back
                        // so that we can give a mut ref to the tree to Visual::window_paint
                        let mut v = replace_visual(&mut tree, node_id, None);
                        // now call the `window_paint` procedure on the visual, if it exists
                        v.as_mut()
                            .and_then(|v| v.window_handler_mut())
                            .map(|v| v.window_paint(&mut self, &mut tree, node_id));
                        // put back the visual
                        replace_visual(&mut tree, node_id, v);
                    } else {
                        log::warn!("repaint for unregistered window")
                    }
                }
                _ => (),
            }
        })
    }
}

pub fn run(mut root_widget: impl Widget + Clone + 'static) {
    // winit event loop
    let event_loop = EventLoop::new();
    // platform-specific, window-independent initialization
    let platform = unsafe { Platform::init() };

    // load application settings
    let mut cfg = config::Config::new();
    cfg.merge(config::File::with_name("Settings").format(config::FileFormat::Toml));
    write_default_application_style();
    let app_style = load_application_style(&cfg);

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    let mut tree = NodeTree::new();

    let mut app_ctx = AppCtx::new(platform, app_style, spawner, move || {
        root_widget.clone().boxed()
    });
    // perform the initial layout
    app_ctx.layout(&mut tree, &event_loop);
    // enter the main event loop
    app_ctx.run(tree, event_loop);
}
