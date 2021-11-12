//! Core types and traits (Widget, Node and Contexts)

use crate::{
    application::AppCtx,
    bloom::Bloom,
    composition::{ActionResult, CompositionSlot},
    event::{Event, InputState, MoveFocusDirection, PointerButton, PointerEvent, PointerEventKind},
    layout::{BoxConstraints, Measurements},
    region::Region,
    util::Counter,
    Environment, CallKey, Offset, PhysicalSize, Point, Rect, Size,
};
use keyboard_types::{KeyState, KeyboardEvent};
use kyute_shell::{
    drawing::{Color, DrawContext},
    platform::Platform,
    window::{PlatformWindow, WindowDrawContext},
    winit,
    winit::{
        event::{DeviceId, VirtualKeyCode, WindowEvent},
        event_loop::EventLoopWindowTarget,
        window::WindowId,
    },
};
use std::{
    any::Any,
    cell::Cell,
    fmt,
    num::NonZeroU64,
    ops::{Deref, DerefMut},
    time::Instant,
};
use tracing::{trace, trace_span, warn};

/// ID of a node in the tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(NonZeroU64);

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0.get())
    }
}

static NODE_ID_COUNTER: Counter = Counter::new();

impl NodeId {
    /// Generates a new node ID unique for this execution of the program.
    pub fn next() -> NodeId {
        let val = NODE_ID_COUNTER.next_nonzero();
        NodeId(val)
    }
}

/// Context passed to window widgets.
///
/// See [`Widget::paint`]
pub struct WindowPaintCtx<'a> {
    app_ctx: &'a mut AppCtx,
    window_state: &'a WindowState,
    node_id: NodeId,
}

/// Context passed to widgets during rendering so that they can draw themselves.
///
/// See [`Widget::paint`]
pub struct PaintCtx<'a, 'pctx> {
    pub(crate) draw_ctx: &'a mut DrawContext<'pctx>,
    pub(crate) node_id: NodeId,
    pub(crate) window_bounds: Rect,
    focus: Option<NodeId>,
    pointer_grab: Option<NodeId>,
    hot: Option<NodeId>,
    inputs: &'a InputState,
    scale_factor: f64,
    invalid: &'a Region,
    hover: bool,
}

impl<'a, 'pctx> PaintCtx<'a, 'pctx> {
    /// Returns the window bounds of the node
    pub fn window_bounds(&self) -> Rect {
        self.window_bounds
    }

    /// Returns the bounds of the node.
    pub fn bounds(&self) -> Rect {
        // FIXME: is the local origin always on the top-left corner?
        Rect::new(Point::origin(), self.window_bounds.size)
    }

    /// Returns the size of the node.
    pub fn size(&self) -> Size {
        self.window_bounds.size
    }

    pub fn is_hovering(&self) -> bool {
        self.hover
    }

    pub fn is_focused(&self) -> bool {
        self.focus == Some(self.node_id)
    }

    pub fn is_capturing_pointer(&self) -> bool {
        self.pointer_grab == Some(self.node_id)
    }
}

// PaintCtx auto-derefs to a DrawContext
impl<'a, 'pctx> Deref for PaintCtx<'a, 'pctx> {
    type Target = DrawContext<'pctx>;

    fn deref(&self) -> &Self::Target {
        self.draw_ctx
    }
}

impl<'a, 'pctx> DerefMut for PaintCtx<'a, 'pctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.draw_ctx
    }
}

/// Context passed to widgets during the layout pass.
///
/// See [`Widget::layout`].
pub struct LayoutCtx<'a> {
    pub(crate) app_ctx: &'a AppCtx,
    window_state: Option<&'a WindowState>,
}

impl<'a> LayoutCtx<'a> {
    /// Returns the size of the parent window.
    pub fn parent_window_size(&self) -> Size {
        self.window_state
            .map(|w| w.logical_size())
            .unwrap_or(Size::zero())
    }
}

/// What to do after an event or a layout operation.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RepaintRequest {
    /// Do nothing
    None,
    /// Repaint the widgets
    Repaint,
    /// Relayout and repaint the widgets
    Relayout,
}

/// Focus-related action.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FocusAction {
    /// Keep the focus, or do nothing if the node does not have it.
    Keep,
    /// Acquire focus, if the node does not have it already
    Acquire,
    /// Release the focus, if the node has it
    Release,
    /// Move the focus backward or forward in tab order.
    Move(MoveFocusDirection),
}

/// Stores information about the last click (for double-click handling)
pub(crate) struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

fn key_code_from_winit(
    input: &winit::event::KeyboardInput,
) -> (keyboard_types::Key, keyboard_types::Code) {
    use keyboard_types::{Code, Key};

    let code = match input.scancode {
        0x0029 => Code::Backquote,
        0x002B => Code::Backslash,
        0x000E => Code::Backspace,
        0x001A => Code::BracketLeft,
        0x001B => Code::BracketRight,
        0x0033 => Code::Comma,
        0x000B => Code::Digit0,
        0x0002 => Code::Digit1,
        0x0003 => Code::Digit2,
        0x0004 => Code::Digit3,
        0x0005 => Code::Digit4,
        0x0006 => Code::Digit5,
        0x0007 => Code::Digit6,
        0x0008 => Code::Digit7,
        0x0009 => Code::Digit8,
        0x000A => Code::Digit9,
        0x000D => Code::Equal,
        0x0056 => Code::IntlBackslash,
        0x0073 => Code::IntlRo,
        0x007D => Code::IntlYen,
        0x001E => Code::KeyA,
        0x0030 => Code::KeyB,
        0x002E => Code::KeyC,
        0x0020 => Code::KeyD,
        0x0012 => Code::KeyE,
        0x0021 => Code::KeyF,
        0x0022 => Code::KeyG,
        0x0023 => Code::KeyH,
        0x0017 => Code::KeyI,
        0x0024 => Code::KeyJ,
        0x0025 => Code::KeyK,
        0x0026 => Code::KeyL,
        0x0032 => Code::KeyM,
        0x0031 => Code::KeyN,
        0x0018 => Code::KeyO,
        0x0019 => Code::KeyP,
        0x0010 => Code::KeyQ,
        0x0013 => Code::KeyR,
        0x001F => Code::KeyS,
        0x0014 => Code::KeyT,
        0x0016 => Code::KeyU,
        0x002F => Code::KeyV,
        0x0011 => Code::KeyW,
        0x002D => Code::KeyX,
        0x0015 => Code::KeyY,
        0x002C => Code::KeyZ,
        0x000C => Code::Minus,
        0x0034 => Code::Period,
        0x0028 => Code::Quote,
        0x0027 => Code::Semicolon,
        0x0035 => Code::Slash,
        0x0038 => Code::AltLeft,
        0xE038 => Code::AltRight,
        0x003A => Code::CapsLock,
        0xE05D => Code::ContextMenu,
        0x001D => Code::ControlLeft,
        0xE01D => Code::ControlRight,
        0x001C => Code::Enter,
        0xE05B => Code::Super,
        0xE05C => Code::Super,
        0x002A => Code::ShiftLeft,
        0x0036 => Code::ShiftRight,
        0x0039 => Code::Space,
        0x000F => Code::Tab,
        0x0079 => Code::Convert,
        0x0072 => Code::Lang1,
        0xE0F2 => Code::Lang1,
        0x0071 => Code::Lang2,
        0xE0F1 => Code::Lang2,
        0x0070 => Code::KanaMode,
        0x007B => Code::NonConvert,
        0xE053 => Code::Delete,
        0xE04F => Code::End,
        0xE047 => Code::Home,
        0xE052 => Code::Insert,
        0xE051 => Code::PageDown,
        0xE049 => Code::PageUp,
        0xE050 => Code::ArrowDown,
        0xE04B => Code::ArrowLeft,
        0xE04D => Code::ArrowRight,
        0xE048 => Code::ArrowUp,
        0xE045 => Code::NumLock,
        0x0052 => Code::Numpad0,
        0x004F => Code::Numpad1,
        0x0050 => Code::Numpad2,
        0x0051 => Code::Numpad3,
        0x004B => Code::Numpad4,
        0x004C => Code::Numpad5,
        0x004D => Code::Numpad6,
        0x0047 => Code::Numpad7,
        0x0048 => Code::Numpad8,
        0x0049 => Code::Numpad9,
        0x004E => Code::NumpadAdd,
        0x007E => Code::NumpadComma,
        0x0053 => Code::NumpadDecimal,
        0xE035 => Code::NumpadDivide,
        0xE01C => Code::NumpadEnter,
        0x0059 => Code::NumpadEqual,
        0x0037 => Code::NumpadMultiply,
        0x004A => Code::NumpadSubtract,
        0x0001 => Code::Escape,
        0x003B => Code::F1,
        0x003C => Code::F2,
        0x003D => Code::F3,
        0x003E => Code::F4,
        0x003F => Code::F5,
        0x0040 => Code::F6,
        0x0041 => Code::F7,
        0x0042 => Code::F8,
        0x0043 => Code::F9,
        0x0044 => Code::F10,
        0x0057 => Code::F11,
        0x0058 => Code::F12,
        0xE037 => Code::PrintScreen,
        0x0054 => Code::PrintScreen,
        0x0046 => Code::ScrollLock,
        0x0045 => Code::Pause,
        0xE046 => Code::Pause,
        0xE06A => Code::BrowserBack,
        0xE066 => Code::BrowserFavorites,
        0xE069 => Code::BrowserForward,
        0xE032 => Code::BrowserHome,
        0xE067 => Code::BrowserRefresh,
        0xE065 => Code::BrowserSearch,
        0xE068 => Code::BrowserStop,
        0xE06B => Code::LaunchApp1,
        0xE021 => Code::LaunchApp2,
        0xE06C => Code::LaunchMail,
        0xE022 => Code::MediaPlayPause,
        0xE06D => Code::MediaSelect,
        0xE024 => Code::MediaStop,
        0xE019 => Code::MediaTrackNext,
        0xE010 => Code::MediaTrackPrevious,
        0xE05E => Code::Power,
        0xE02E => Code::AudioVolumeDown,
        0xE020 => Code::AudioVolumeMute,
        0xE030 => Code::AudioVolumeUp,
        _ => Code::Unidentified,
    };

    let key = if let Some(vk) = input.virtual_keycode {
        match vk {
            VirtualKeyCode::Key1 => Key::Unidentified,
            VirtualKeyCode::Key2 => Key::Unidentified,
            VirtualKeyCode::Key3 => Key::Unidentified,
            VirtualKeyCode::Key4 => Key::Unidentified,
            VirtualKeyCode::Key5 => Key::Unidentified,
            VirtualKeyCode::Key6 => Key::Unidentified,
            VirtualKeyCode::Key7 => Key::Unidentified,
            VirtualKeyCode::Key8 => Key::Unidentified,
            VirtualKeyCode::Key9 => Key::Unidentified,
            VirtualKeyCode::Key0 => Key::Unidentified,
            VirtualKeyCode::A => Key::Unidentified,
            VirtualKeyCode::B => Key::Unidentified,
            VirtualKeyCode::C => Key::Unidentified,
            VirtualKeyCode::D => Key::Unidentified,
            VirtualKeyCode::E => Key::Unidentified,
            VirtualKeyCode::F => Key::Unidentified,
            VirtualKeyCode::G => Key::Unidentified,
            VirtualKeyCode::H => Key::Unidentified,
            VirtualKeyCode::I => Key::Unidentified,
            VirtualKeyCode::J => Key::Unidentified,
            VirtualKeyCode::K => Key::Unidentified,
            VirtualKeyCode::L => Key::Unidentified,
            VirtualKeyCode::M => Key::Unidentified,
            VirtualKeyCode::N => Key::Unidentified,
            VirtualKeyCode::O => Key::Unidentified,
            VirtualKeyCode::P => Key::Unidentified,
            VirtualKeyCode::Q => Key::Unidentified,
            VirtualKeyCode::R => Key::Unidentified,
            VirtualKeyCode::S => Key::Unidentified,
            VirtualKeyCode::T => Key::Unidentified,
            VirtualKeyCode::U => Key::Unidentified,
            VirtualKeyCode::V => Key::Unidentified,
            VirtualKeyCode::W => Key::Unidentified,
            VirtualKeyCode::X => Key::Unidentified,
            VirtualKeyCode::Y => Key::Unidentified,
            VirtualKeyCode::Z => Key::Unidentified,
            VirtualKeyCode::Escape => Key::Escape,
            VirtualKeyCode::F1 => Key::F1,
            VirtualKeyCode::F2 => Key::F2,
            VirtualKeyCode::F3 => Key::F3,
            VirtualKeyCode::F4 => Key::F4,
            VirtualKeyCode::F5 => Key::F5,
            VirtualKeyCode::F6 => Key::F6,
            VirtualKeyCode::F7 => Key::F7,
            VirtualKeyCode::F8 => Key::F8,
            VirtualKeyCode::F9 => Key::F9,
            VirtualKeyCode::F10 => Key::F10,
            VirtualKeyCode::F11 => Key::F11,
            VirtualKeyCode::F12 => Key::F12,
            VirtualKeyCode::Pause => Key::Pause,
            VirtualKeyCode::Insert => Key::Insert,
            VirtualKeyCode::Home => Key::Home,
            VirtualKeyCode::Delete => Key::Delete,
            VirtualKeyCode::End => Key::End,
            VirtualKeyCode::PageDown => Key::PageDown,
            VirtualKeyCode::PageUp => Key::PageUp,
            VirtualKeyCode::Left => Key::ArrowLeft,
            VirtualKeyCode::Up => Key::ArrowUp,
            VirtualKeyCode::Right => Key::ArrowRight,
            VirtualKeyCode::Down => Key::ArrowDown,
            VirtualKeyCode::Return => Key::Enter,
            VirtualKeyCode::Space => Key::Unidentified,
            VirtualKeyCode::Compose => Key::Compose,
            VirtualKeyCode::Caret => Key::Unidentified,
            VirtualKeyCode::Numlock => Key::NumLock,
            VirtualKeyCode::Numpad0 => Key::Unidentified,
            VirtualKeyCode::Numpad1 => Key::Unidentified,
            VirtualKeyCode::Numpad2 => Key::Unidentified,
            VirtualKeyCode::Numpad3 => Key::Unidentified,
            VirtualKeyCode::Numpad4 => Key::Unidentified,
            VirtualKeyCode::Numpad5 => Key::Unidentified,
            VirtualKeyCode::Numpad6 => Key::Unidentified,
            VirtualKeyCode::Numpad7 => Key::Unidentified,
            VirtualKeyCode::Numpad8 => Key::Unidentified,
            VirtualKeyCode::Numpad9 => Key::Unidentified,
            VirtualKeyCode::Backslash => Key::Unidentified,
            VirtualKeyCode::Capital => Key::Unidentified,
            VirtualKeyCode::Colon => Key::Unidentified,
            VirtualKeyCode::Comma => Key::Unidentified,
            VirtualKeyCode::Convert => Key::Convert,
            VirtualKeyCode::Decimal => Key::Unidentified,
            VirtualKeyCode::Divide => Key::Unidentified,
            VirtualKeyCode::Equals => Key::Unidentified,
            VirtualKeyCode::Grave => Key::Unidentified,
            VirtualKeyCode::Kana => Key::KanaMode,
            VirtualKeyCode::Kanji => Key::KanjiMode,
            VirtualKeyCode::LAlt => Key::Alt,
            VirtualKeyCode::LBracket => Key::Unidentified,
            VirtualKeyCode::LControl => Key::Control,
            VirtualKeyCode::LShift => Key::Shift,
            VirtualKeyCode::LWin => Key::Super,
            VirtualKeyCode::Mail => Key::LaunchMail,
            VirtualKeyCode::MediaSelect => Key::Unidentified,
            VirtualKeyCode::MediaStop => Key::MediaStop,
            VirtualKeyCode::Minus => Key::Unidentified,
            VirtualKeyCode::Multiply => Key::Unidentified,
            VirtualKeyCode::Mute => Key::AudioVolumeMute,
            VirtualKeyCode::MyComputer => Key::Unidentified,
            VirtualKeyCode::NavigateForward => Key::BrowserForward,
            VirtualKeyCode::NavigateBackward => Key::BrowserBack,
            VirtualKeyCode::NextTrack => Key::MediaTrackNext,
            VirtualKeyCode::NoConvert => Key::NonConvert,
            VirtualKeyCode::NumpadComma => Key::Unidentified,
            VirtualKeyCode::NumpadEnter => Key::Enter,
            VirtualKeyCode::Period => Key::Unidentified,
            VirtualKeyCode::PlayPause => Key::MediaPlayPause,
            VirtualKeyCode::Power => Key::Power,
            VirtualKeyCode::PrevTrack => Key::MediaTrackPrevious,
            VirtualKeyCode::RAlt => Key::Alt,
            VirtualKeyCode::RBracket => Key::Unidentified,
            VirtualKeyCode::RControl => Key::Control,
            VirtualKeyCode::RShift => Key::Shift,
            VirtualKeyCode::Semicolon => Key::Unidentified,
            VirtualKeyCode::Slash => Key::Unidentified,
            VirtualKeyCode::Sleep => Key::Unidentified,
            VirtualKeyCode::Tab => Key::Tab,
            VirtualKeyCode::VolumeDown => Key::AudioVolumeDown,
            VirtualKeyCode::VolumeUp => Key::AudioVolumeUp,
            VirtualKeyCode::Copy => Key::Copy,
            VirtualKeyCode::Paste => Key::Paste,
            VirtualKeyCode::Cut => Key::Cut,
            VirtualKeyCode::Back => Key::Backspace,
            _ => Key::Unidentified,
        }
    } else {
        Key::Unidentified
    };

    (key, code)
}

struct WindowState {
    window: PlatformWindow,
    focus: Option<NodeId>,
    pointer_grab: Option<NodeId>,
    hot: Option<NodeId>,
    inputs: InputState,
    last_click: Option<LastClick>,
    scale_factor: f64,
    invalid: Region,
    needs_layout: bool,
}

impl WindowState {
    fn new(window: PlatformWindow) -> Self {
        WindowState {
            window,
            focus: None,
            pointer_grab: None,
            hot: None,
            inputs: Default::default(),
            last_click: None,
            scale_factor: 1.0,
            invalid: Default::default(),
            needs_layout: true,
        }
    }

    fn release_focus(&mut self) {
        self.focus = None;
    }

    fn release_pointer_grab(&mut self) {
        self.pointer_grab = None;
    }

    fn acquire_focus(&mut self, node: NodeId) {
        self.focus = Some(node);
    }

    fn acquire_pointer_grab(&mut self, node: NodeId) {
        self.pointer_grab = Some(node);
    }

    /// Window event processing.
    fn process_window_event(
        &mut self,
        app_ctx: &mut AppCtx,
        widget: &mut dyn WidgetDelegate,
        children: &mut [Widget],
        window_event: &winit::event::WindowEvent,
    ) {
        let _span = trace_span!("process_window_event", ?window_event).entered();

        // ---------------------------------------
        // Default window event processing: update scale factor, input states (pointer pos, keyboard mods).
        // Some input events (pointer, keyboard) are also converted to normal events delivered
        // to the widgets within the window.
        let event = match window_event {
            // don't send Character events for control characters
            WindowEvent::ReceivedCharacter(c) if !c.is_control() => {
                Some(Event::Keyboard(KeyboardEvent {
                    state: KeyState::Down,
                    key: keyboard_types::Key::Character(c.to_string()),
                    code: keyboard_types::Code::Unidentified,
                    location: keyboard_types::Location::Standard,
                    modifiers: self.inputs.modifiers,
                    // TODO
                    repeat: false,
                    is_composing: false,
                }))
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = *scale_factor;
                // TODO
                None
            }
            WindowEvent::Resized(size) => {
                self.window
                    .resize(PhysicalSize::new(size.width as f64, size.height as f64));
                self.needs_layout = true;
                None
            }
            WindowEvent::Focused(true) => {
                // TODO
                None
            }
            WindowEvent::Focused(false) => {
                // TODO
                None
            }
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                let (key, code) = key_code_from_winit(&input);
                Some(Event::Keyboard(KeyboardEvent {
                    state: match input.state {
                        winit::event::ElementState::Pressed => keyboard_types::KeyState::Down,
                        winit::event::ElementState::Released => keyboard_types::KeyState::Up,
                    },
                    key,
                    code,
                    location: keyboard_types::Location::default(),
                    modifiers: self.inputs.modifiers,
                    repeat: false,
                    is_composing: false,
                }))
            }
            WindowEvent::ModifiersChanged(mods) => {
                // TODO
                //window_info.inputs.modifiers = mods;
                None
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                let logical_position = Point::new(
                    position.x * self.scale_factor,
                    position.y * self.scale_factor,
                );
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                pointer_state.position = logical_position;
                Some(Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerMove,
                    position: logical_position,
                    window_position: logical_position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: None,
                    repeat_count: 0,
                }))
            }
            WindowEvent::CursorEntered { .. } => {
                // TODO
                None
            }
            WindowEvent::CursorLeft { .. } => {
                // TODO
                None
            }
            WindowEvent::MouseWheel { .. } => {
                // TODO
                None
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                ..
            } => {
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
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

                // determine the repeat count (double-click, triple-click, etc.) for button down event
                let repeat_count = match &mut self.last_click {
                    Some(ref mut last)
                        if last.device_id == *device_id
                            && last.button == button
                            && last.position == pointer_state.position
                            && (click_time - last.time)
                                < Platform::instance().double_click_time() =>
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
                                    position: pointer_state.position,
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

                Some(Event::Pointer(PointerEvent {
                    kind: match state {
                        winit::event::ElementState::Pressed => PointerEventKind::PointerDown,
                        winit::event::ElementState::Released => PointerEventKind::PointerUp,
                    },
                    position: pointer_state.position,
                    window_position: pointer_state.position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: Some(button),
                    repeat_count,
                }))
            }
            winit::event::WindowEvent::TouchpadPressure { .. } => None,
            winit::event::WindowEvent::AxisMotion { .. } => None,
            winit::event::WindowEvent::Touch(_) => None,
            winit::event::WindowEvent::ThemeChanged(_) => None,
            _ => None,
        };

        if let Some(event) = event {
            //------------------------------------------------
            // Follow-up 1: determine to which nodes the event should be sent
            let target_node_id = match event {
                Event::Pointer(ref pointer_event) => {
                    // Pointer events are delivered to the node that is currently grabbing the pointer.
                    // If nothing is grabbing the pointer, the pointer event is delivered to a widget
                    // that passes the hit-test
                    if let Some(id) = self.pointer_grab {
                        // deliver to pointer-grabbing widget
                        Some(id)
                    } else {
                        // hit-test children
                        children
                            .iter()
                            .find_map(|n| n.hit_test(pointer_event.position))
                    }
                }
                Event::Keyboard(ref k) => {
                    // keyboard events are delivered to the widget that has the focus.
                    // if no widget has focus, the event is dropped.
                    self.focus
                }
                _ => {
                    // TODO
                    None
                }
            };

            //------------------------------------------------
            // Follow-up 2: update 'hot' node (the node that the pointer is hovering above)
            // Post pointerout/pointerover events
            match event {
                Event::Pointer(ref pointer_event) => {
                    let old_hot = self.hot;
                    self.hot = target_node_id;

                    // send  (in that order) and update 'hot' node
                    match pointer_event.kind {
                        PointerEventKind::PointerUp
                        | PointerEventKind::PointerDown
                        | PointerEventKind::PointerMove => {
                            if old_hot != target_node_id {
                                if let Some(old_and_busted) = old_hot {
                                    trace!(node_id = ?old_and_busted, "widget going cold");
                                    let pointer_out = Event::Pointer(PointerEvent {
                                        kind: PointerEventKind::PointerOut,
                                        ..*pointer_event
                                    });
                                    // TODO why should PointerOut/Over events be tunneling?
                                    app_ctx.post_event(
                                        None,
                                        EventTarget::Tunnel(old_and_busted),
                                        pointer_out,
                                    );
                                }
                                if let Some(new_hotness) = target_node_id {
                                    trace!(node_id = ?new_hotness, "widget going hot");
                                    let pointer_over = Event::Pointer(PointerEvent {
                                        kind: PointerEventKind::PointerOver,
                                        ..*pointer_event
                                    });
                                    app_ctx.post_event(
                                        None,
                                        EventTarget::Tunnel(new_hotness),
                                        pointer_over,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            };

            //------------------------------------------------
            // Follow-up 3: force release pointer grab on pointer up
            match event {
                Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerUp,
                    ..
                }) => {
                    trace!("forcing release of pointer grab");
                    self.release_pointer_grab();
                }
                _ => {}
            }

            //------------------------------------------------
            // Finally, post the event.
            if let Some(target) = target_node_id {
                // for now, all events that originate from a window event bubble up
                app_ctx.post_event(None, EventTarget::Bubble(target), event);
            }
        }
    }

    fn logical_size(&self) -> Size {
        let scale_factor = self.window.window().scale_factor();
        let physical_window_size = self.window.window().inner_size();
        Size::new(
            physical_window_size.width as f64 * scale_factor,
            physical_window_size.height as f64 * scale_factor,
        )
    }

    fn paint(
        &mut self,
        node_id: NodeId,
        widget: &mut dyn WidgetDelegate,
        children: &mut [Widget],
        env: &Environment,
    ) {
        {
            let logical_window_size = self.logical_size();
            let bounds = Rect::new(Point::origin(), logical_window_size);
            let mut wdc = WindowDrawContext::new(&mut self.window);
            // FIXME maybe only some nodes need to be repainted: this forces a full redraw
            // TODO remove this
            wdc.clear(Color::new(0.326, 0.326, 0.326, 1.0));

            let mut ctx = PaintCtx {
                draw_ctx: &mut wdc,
                node_id,
                window_bounds: bounds,
                focus: self.focus,
                pointer_grab: self.pointer_grab,
                hot: self.hot,
                inputs: &self.inputs,
                scale_factor: self.scale_factor,
                hover: false,
                invalid: &self.invalid,
            };

            widget.paint(&mut ctx, children, bounds, env);
        }
        self.window.present();
    }

    fn post_event_processing(&mut self) {
        if !self.invalid.is_empty() || self.needs_layout {
            self.window.window().request_redraw();
        }
    }
}

/// Context passed to [`Widget::event`] during event propagation.
/// Also serves as a return value for this function.
pub struct EventCtx<'a> {
    /// Window context
    pub(crate) app_ctx: &'a mut AppCtx,
    window_state: Option<&'a mut WindowState>,
    /// The ID of the current node.
    node_id: NodeId,
    /// The bounds of the current widget, in the widget's coordinate space.
    bounds: Rect,
    /// The bounds of the current widget in its parent window coordinate space.
    window_bounds: Rect,
    /// Event handled
    handled: bool,
}

impl<'a> EventCtx<'a> {
    fn new(
        app_ctx: &'a mut AppCtx,
        window_state: Option<&'a mut WindowState>,
        node_id: NodeId,
    ) -> EventCtx<'a> {
        EventCtx {
            app_ctx,
            window_state,
            node_id,
            bounds: Default::default(),
            window_bounds: Default::default(),
            handled: false,
        }
    }

    pub fn emit_action<T: Any>(&mut self, action: T) {
        self.app_ctx.post_action(self.node_id, Box::new(action));
        self.request_recomposition();
    }

    /// Returns the bounds of the current widget.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Requests a redraw of the current node and its children.
    pub fn request_redraw(&mut self) {
        if let Some(window_state) = &mut self.window_state {
            window_state.invalid.add_rect(self.window_bounds);
            trace!(node_id=?self.node_id, "request_redraw");
        } else {
            warn!(node_id=?self.node_id, "request_redraw: node does not belong to a window");
        }
    }

    pub fn request_recomposition(&mut self) {
        self.app_ctx.request_recomposition();
    }

    /// Requests a relayout of the current node.
    pub fn request_relayout(&mut self) {
        self.app_ctx.request_relayout();
    }

    /// Requests that the current node grabs all pointer events in the parent window.
    pub fn capture_pointer(&mut self) {
        if let Some(window_state) = &mut self.window_state {
            window_state.pointer_grab = Some(self.node_id);
            trace!(
                "capture_pointer: node id {:?} will capture pointer events",
                self.node_id
            );
        } else {
            warn!(node_id=?self.node_id, "capture_pointer: node does not belong to a window and doesn't receive pointer events");
        }
    }

    /// Returns whether the current node is capturing the pointer.
    pub fn is_capturing_pointer(&self) -> bool {
        if let Some(ref window_state) = self.window_state {
            window_state.pointer_grab == Some(self.node_id)
        } else {
            warn!(node_id=?self.node_id, "is_capturing_pointer: node does not belong to a window and doesn't receive pointer events");
            false
        }
    }

    /// Releases the pointer grab, if the current node is holding it.
    pub fn release_pointer(&mut self) {
        if let Some(ref mut window_state) = self.window_state {
            if window_state.pointer_grab == Some(self.node_id) {
                window_state.pointer_grab = None;
            }
        } else {
            warn!(node_id=?self.node_id, "release_pointer: node does not belong to a window and doesn't receive pointer events");
        }
    }

    /// Acquires the focus.
    pub fn request_focus(&mut self) {
        if let Some(ref mut window_state) = self.window_state {
            if window_state.focus != Some(self.node_id) {
                // changing focus
                if let Some(old_focus) = window_state.focus {
                    self.app_ctx
                        .post_event(None, EventTarget::Direct(old_focus), Event::FocusLost);
                }
                window_state.focus = Some(self.node_id);
                self.app_ctx.post_event(
                    None,
                    EventTarget::Direct(self.node_id),
                    Event::FocusGained,
                );
            }
        } else {
            warn!(node_id=?self.node_id, "request_focus: node does not belong to a window and doesn't receive pointer events");
        }
    }

    /// Returns whether the current node has the focus.
    pub fn has_focus(&self) -> bool {
        if let Some(ref window) = self.window_state {
            window.focus == Some(self.node_id)
        } else {
            warn!(node_id=?self.node_id, "has_focus: node does not belong to a window");
            false
        }
    }

    /// Signals that the passed event was handled and should not bubble up further.
    pub fn set_handled(&mut self) {
        self.handled = true;
    }

    #[must_use]
    pub fn handled(&self) -> bool {
        self.handled
    }
}

struct FocusRequest {
    action: FocusAction,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum EventPropagationMode {
    Tunnel,
    Bubble,
    Single,
}

/// Trait that defines the behavior of a widget.
pub trait WidgetDelegate: Any {
    /// Implement to give a debug name to your widget. Used only for debugging.
    fn debug_name(&self) -> &str {
        "WidgetDelegate"
    }

    /// Handles events and pass them down to children.
    fn event(&mut self, ctx: &mut EventCtx, children: &mut [Widget], event: &Event) {}

    /// Called to measure this widget and layout the children of this widget.
    /// TODO: not having to pass the child nodes here would simplify the implementation
    /// of the composition table.
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut [Widget],
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> Measurements;

    /// Called to paint the widget
    fn paint(&mut self, ctx: &mut PaintCtx, children: &mut [Widget], bounds: Rect, env: &Environment);

    /// Called only for native window widgets.
    fn window_paint(&mut self, _ctx: &mut WindowPaintCtx, _children: &mut [Widget]) {}

    /// Returns `true` if the widget is fully opaque when drawn, `false` if it is semitransparent.
    /// This is mostly used as an optimization: if a semitransparent widget needs to be redrawn,
    /// its background (and thus the parent
    fn is_opaque(&self) -> bool {
        false
    }
}

impl<W: WidgetDelegate + ?Sized> WidgetDelegate for Box<W> {
    fn debug_name(&self) -> &str {
        WidgetDelegate::debug_name(&**self)
    }

    fn event(&mut self, ctx: &mut EventCtx, children: &mut [Widget], event: &Event) {
        WidgetDelegate::event(&mut **self, ctx, children, event)
    }

    /*fn window_event(
        &mut self,
        ctx: &mut WindowEventCtx,
        children: &mut [Node],
        event: &WindowEvent,
    ) {
        Widget::window_event(&mut **self, ctx, children, event)
    }*/

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut [Widget],
        constraints: &BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        WidgetDelegate::layout(&mut **self, ctx, children, constraints, env)
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        children: &mut [Widget],
        bounds: Rect,
        env: &Environment,
    ) {
        WidgetDelegate::paint(&mut **self, ctx, children, bounds, env)
    }

    fn window_paint(&mut self, ctx: &mut WindowPaintCtx, children: &mut [Widget]) {
        WidgetDelegate::window_paint(&mut **self, ctx, children)
    }
}

/// Widget, with associated child widgets and delegate.
pub struct Widget<W = Box<dyn WidgetDelegate>> {
    pub(crate) id: NodeId,
    pub(crate) key: CallKey,
    /// Offset of the node relative to the parent
    pub(crate) offset: Offset,
    /// Layout of the node (size and baseline).
    pub(crate) measurements: Measurements,
    /// Absolute position of the node in the parent window.
    pub(crate) window_pos: Point,
    /// Widget
    pub(crate) widget: W,
    /// ID of the parent window for this node.
    pub(crate) parent_window_id: Option<WindowId>,
    /// Child nodes.
    pub(crate) children: Vec<Widget>,
    /// A temporary value to store the index of this node during recomposition.
    pub(crate) child_index: usize,
    window_state: Option<Box<WindowState>>,
    /// This node's composition table.
    pub(crate) composition_table: Vec<CompositionSlot>,
    pub(crate) child_filter: Bloom<NodeId>,
    env: Environment,
}

impl Widget {
    pub fn dummy() -> Widget {
        unsafe {
            Widget::new(
                Box::new(Dummy),
                NodeId::next(),
                CallKey::from_caller(0),
                None,
                None,
                Environment::new(),
            )
        }
    }

    pub(crate) fn key_path_to_child(&self, target: NodeId) -> Option<Vec<CallKey>> {
        let mut path = Vec::new();
        if !self.path_to_child(target, &mut path) {
            return None;
        }

        let mut key_path = Vec::with_capacity(path.len());
        let mut node = self;
        for &i in path.iter() {
            node = &node.children[i];
            key_path.push(node.key);
        }
        Some(key_path)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum EventTarget {
    Tunnel(NodeId),
    Bubble(NodeId),
    Direct(NodeId),
    Broadcast,
}

impl<W: WidgetDelegate> Widget<W> {
    /// Creates a node with the specified ID.
    /// Safety: The ID must be unique. Use NodeId::new()
    pub unsafe fn new(
        widget: W,
        id: NodeId,
        key: CallKey,
        parent_window_id: Option<WindowId>,
        window: Option<PlatformWindow>,
        env: Environment,
    ) -> Widget<W> {
        let window_state = window.map(|w| Box::new(WindowState::new(w)));
        Widget {
            id,
            key,
            offset: Default::default(),
            measurements: Default::default(),
            window_pos: Default::default(),
            widget,
            parent_window_id,
            children: vec![],
            child_index: 0,
            window_state,
            composition_table: vec![],
            child_filter: Default::default(),
            env,
        }
    }

    pub fn debug_name(&self) -> &str {
        self.widget.debug_name()
    }

    /// Finds a child node with the specified ID.
    pub fn find_child_mut(&mut self, id: NodeId) -> Option<&mut Widget> {
        // query the bloom filter to see if this ID definitely does not belong to this node or its children
        if !self.child_filter.may_contain(&id) {
            return None;
        }

        for c in self.children.iter_mut() {
            if c.id == id {
                return Some(c);
            }
            if let Some(n) = c.find_child_mut(id) {
                return Some(n);
            }
        }

        None
    }

    pub(crate) fn window_id(&self) -> Option<WindowId> {
        self.window_state.as_ref().map(|w| w.window.id())
    }

    pub fn hit_test(&self, parent_position: Point) -> Option<NodeId> {
        if Rect::new(self.offset.to_point(), self.measurements.size).contains(parent_position) {
            //tracing::trace!("hit test pass: {:?} {:?}", self.id, parent_position);
            let local_pos = parent_position - self.offset;
            self.children
                .iter()
                .find_map(|c| c.hit_test(local_pos))
                .or(Some(self.id))
        } else {
            //tracing::trace!("hit test fail: {:?} {:?}", self.id, parent_position);
            None
        }
    }

    /// Returns the path to the a target child node, as a sequence of child node indices.
    ///
    /// # Arguments
    /// * `target` - target child node
    /// * `result` - (output) the sequence of child indices leading to the node is appended in `result` if the node has been found,
    /// otherwise its contents are left unchanged.
    ///
    /// # Return value
    /// `true` if the target has been found (and the path has been appended to `result`), `false` otherwise.
    fn path_to_child(&self, target: NodeId, result: &mut Vec<usize>) -> bool {
        if self.id == target {
            return true;
        }

        if !self.child_filter.may_contain(&target) {
            return false;
        }

        for (i, c) in self.children.iter().enumerate() {
            result.push(i);
            if c.path_to_child(target, result) {
                return true;
            }
            result.pop();
        }

        false
    }

    /// Returns whether the event was handled
    fn propagate_event_recursive(
        &mut self,
        app_ctx: &mut AppCtx,
        mut window_state: Option<&mut WindowState>,
        mode: EventPropagationMode,
        event: &Event,
        path: &[usize],
    ) -> bool {
        let bounds = self.bounds();
        let window_bounds = self.window_bounds();

        // transform pointer position to local coords
        // TODO reconsider
        let transformed_event = match event {
            Event::Pointer(p) => Event::Pointer(PointerEvent {
                position: p.position - self.window_pos.to_vector(),
                ..*p
            }),
            e => e.clone(),
        };

        let mut window_state: Option<&mut WindowState> =
            if let Some(window_state) = self.window_state.as_deref_mut() {
                Some(window_state)
            } else {
                window_state.as_deref_mut()
            };

        let mut handled = false;

        // --- bubbling phase ---
        if mode == EventPropagationMode::Bubble || mode == EventPropagationMode::Single {
            // bubbling: deliver to children first, and if the event is handled, immediately return
            if let Some((&first, rest)) = path.split_first() {
                handled = self.children[first].propagate_event_recursive(
                    app_ctx,
                    window_state.as_deref_mut(),
                    mode,
                    event,
                    rest,
                );
            }
        }

        // --- delivery phase ---
        if !handled {
            let mut event_ctx = EventCtx {
                app_ctx,
                window_state: window_state.as_deref_mut(),
                node_id: self.id,
                bounds,
                window_bounds,
                handled,
            };
            self.widget
                .event(&mut event_ctx, &mut self.children, &transformed_event);
            handled = event_ctx.handled || (mode == EventPropagationMode::Single);
        }

        // --- tunneling phase ---
        if !handled {
            if mode == EventPropagationMode::Tunnel {
                if let Some((&first, rest)) = path.split_first() {
                    handled = self.children[first].propagate_event_recursive(
                        app_ctx,
                        window_state,
                        mode,
                        event,
                        rest,
                    )
                }
            };
        }

        // --- post-processing ---
        // if the event propagated through a window, do some post-processing
        self.window_state
            .as_deref_mut()
            .map(WindowState::post_event_processing);

        handled
    }

    /// Delivers an event to the specified target, with tunneling and bubbling behaviors starting from/ending to this node.
    // (in practice, this is only called on the root node)
    pub(crate) fn propagate_event(
        &mut self,
        app_ctx: &mut AppCtx,
        event: &Event,
        target: EventTarget,
    ) {
        // determine the delivery path (as a sequence of child node indices)
        // TODO factor out common code
        let (mode, path) = match target {
            EventTarget::Tunnel(target) => {
                let mut path = vec![];
                if !self.path_to_child(target, &mut path) {
                    warn!(?target, "no path to target");
                    return;
                }
                (EventPropagationMode::Tunnel, path)
            }
            EventTarget::Bubble(target) => {
                let mut path = vec![];
                if !self.path_to_child(target, &mut path) {
                    warn!(?target, "no path to target");
                    return;
                }
                (EventPropagationMode::Bubble, path)
            }
            EventTarget::Direct(target) => {
                let mut path = vec![];
                if !self.path_to_child(target, &mut path) {
                    warn!(?target, "no path to target");
                    return;
                }
                (EventPropagationMode::Single, path)
            }
            EventTarget::Broadcast => {
                // TODO
                return;
            }
        };

        self.propagate_event_recursive(app_ctx, None, mode, event, &path[..]);
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(Point::origin(), self.measurements.size)
    }

    pub fn window_bounds(&self) -> Rect {
        Rect::new(self.window_pos, self.measurements.size)
    }

    /// Recursively compute window positions of the nodes.
    pub(crate) fn calculate_absolute_positions(&mut self, origin: Point) {
        self.window_pos = origin + self.offset;
        for c in self.children.iter_mut() {
            c.calculate_absolute_positions(self.window_pos);
        }
    }

    /// Window event processing. Calls `Widget::window_event`.
    pub(crate) fn window_event(
        &mut self,
        app_ctx: &mut AppCtx,
        window_event: &winit::event::WindowEvent,
    ) {
        self.window_state.as_mut().unwrap().process_window_event(
            app_ctx,
            &mut self.widget,
            &mut self.children,
            window_event,
        );
    }

    ///
    pub fn paint_window(&mut self, app_ctx: &AppCtx) {
        assert!(self.window_state.is_some());

        if self.window_state.as_mut().unwrap().needs_layout {
            self.do_layout(app_ctx, &BoxConstraints::new(.., ..));
            self.window_state.as_mut().unwrap().needs_layout = false;
        }

        self.window_state.as_mut().unwrap().paint(
            self.id,
            &mut self.widget,
            &mut self.children[..],
            &self.env,
        );
    }

    pub(crate) fn do_layout(
        &mut self,
        app_ctx: &AppCtx,
        constraints: &BoxConstraints,
    ) -> Measurements {
        let _span = trace_span!("do_layout", node_id = ?self.id, ty = self.widget.debug_name(), ?constraints).entered();

        let mut ctx = LayoutCtx {
            app_ctx,
            window_state: self.window_state.as_deref(),
        };
        self.measurements =
            self.widget
                .layout(&mut ctx, &mut self.children, constraints, &self.env);
        // trace!(measurements = ?self.measurements, "computed widget measurements");
        self.measurements
    }

    /// Layouts the node.
    //#[tracing::instrument(skip(self,ctx), fields(node_id=?self.id))]
    pub fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Measurements {
        self.do_layout(ctx.app_ctx, constraints)
    }

    /// Returns the measurements computed during the last call to layout.
    pub fn measurements(&self) -> Measurements {
        self.measurements
    }

    /// Sets the offset of this node relative to its parent. Call during layout.
    pub fn set_offset(&mut self, offset: Offset) {
        self.offset = offset;
    }

    /// Paints this node in the given `PaintCtx`.
    pub fn paint(&mut self, ctx: &mut PaintCtx) {
        // does this node represent a child window?
        if self.window_state.is_some() {
            // child windows shouldn't have to render anything in their parent windows. FIXME is that always the case?
            // The actual painting of child windows happens in their `window_paint` handler.
            return;
        }

        let offset = self.offset;
        let measurements = self.measurements;
        let size = measurements.size;
        let window_bounds = Rect::new(ctx.window_bounds.origin + offset, size);

        if !ctx.invalid.intersects(window_bounds) {
            // not invalidated, no need to redraw
            //return;
        }

        let _span = trace_span!(
            "paint",
            ?self.id,
            ?offset,
            ?measurements,
        )
        .entered();

        // trace!(?ctx.scale_factor, ?ctx.inputs.pointers, ?window_bounds, "paint");

        let hover = ctx.inputs.pointers.iter().any(|(_, state)| {
            window_bounds.contains(Point::new(
                state.position.x * ctx.scale_factor,
                state.position.y * ctx.scale_factor,
            ))
        });

        ctx.draw_ctx.save();
        ctx.draw_ctx.transform(&offset.to_transform());

        {
            let mut child_ctx = PaintCtx {
                draw_ctx: ctx.draw_ctx,
                window_bounds,
                focus: ctx.focus,
                pointer_grab: ctx.pointer_grab,
                hot: ctx.hot,
                inputs: ctx.inputs,
                scale_factor: ctx.scale_factor,
                node_id: self.id,
                hover,
                invalid: &ctx.invalid,
            };
            self.widget.paint(
                &mut child_ctx,
                &mut self.children,
                Rect::new(Point::origin(), size),
                &self.env,
            );
        }

        ctx.draw_ctx.restore();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dummy;

impl Dummy {
    pub fn new() -> Dummy {
        Dummy
    }
}

impl WidgetDelegate for Dummy {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&mut self, _ctx: &mut EventCtx, _children: &mut [Widget], _event: &Event) {}

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        children: &mut [Widget],
        constraints: &BoxConstraints,
        _env: &Environment,
    ) -> Measurements {
        for c in children.iter_mut() {
            c.layout(ctx, constraints);
        }
        Measurements::default()
    }

    fn paint(
        &mut self,
        ctx: &mut PaintCtx,
        _children: &mut [Widget],
        _bounds: Rect,
        _env: &Environment,
    ) {
    }
}
