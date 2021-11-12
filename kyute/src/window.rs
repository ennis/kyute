use crate::{
    align_boxes,
    application::AppCtx,
    composable,
    event::{InputState, KeyboardEvent, PointerButton, PointerEvent, PointerEventKind},
    region::Region,
    Alignment, BoxConstraints, Cache, Environment, Event, EventCtx, InternalEvent, LayoutCtx,
    LayoutItem, Measurements, Offset, PaintCtx, PhysicalSize, Point, Rect, Size, Widget, WidgetId,
    WidgetPod,
};
use keyboard_types::KeyState;
use kyute_shell::{
    drawing::Color,
    platform::Platform,
    window::PlatformWindow,
    winit,
    winit::{
        event::{DeviceId, VirtualKeyCode, WindowEvent},
        window::WindowBuilder,
    },
};
use std::{
    cell::RefCell,
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::Instant,
};
use tracing::trace_span;

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

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

struct WindowState {
    window: Option<PlatformWindow>,
    window_builder: Option<WindowBuilder>,
    focus: Option<WidgetId>,
    pointer_grab: Option<WidgetId>,
    hot: Option<WidgetId>,
    inputs: InputState,
    last_click: Option<LastClick>,
    scale_factor: f64,
    invalid: Region,
}

impl WindowState {
    /// Window event processing.
    ///
    /// Returns whether we should relayout and redraw the contents.
    fn process_window_event(
        &mut self,
        event_ctx: &mut EventCtx,
        content_widget: &WidgetPod,
        window_event: &winit::event::WindowEvent,
    ) -> bool {
        //let _span = trace_span!("process_window_event", ?window_event).entered();

        let mut relayout_contents = false;

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
                relayout_contents = true;
                // TODO
                None
            }
            WindowEvent::Resized(size) => {
                if let Some(window) = self.window.as_mut() {
                    window.resize((size.width, size.height));
                    relayout_contents = true;
                } else {
                    tracing::warn!("Resized event received but window has not been created");
                }
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
            // force release pointer grab on pointer up
            match event {
                Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerUp,
                    ..
                }) => {
                    tracing::trace!("forcing release of pointer grab");
                    self.pointer_grab = None;
                }
                _ => {}
            }

            //------------------------------------------------
            // Send event
            match event {
                Event::Pointer(ref pointer_event) => {
                    // Pointer events are delivered to the node that is currently grabbing the pointer.
                    // If nothing is grabbing the pointer, the pointer event is delivered to a widget
                    // that passes the hit-test
                    if let Some(id) = self.pointer_grab {
                        content_widget.event(
                            event_ctx,
                            &Event::Internal(InternalEvent::RouteEvent {
                                target: id,
                                event: Box::new(event.clone()),
                            }),
                        );
                    } else {
                        // just forward to content, will do a hit-test
                        // FIXME: we shouldn't be using the same EventCtx
                        content_widget.event(event_ctx, &event);
                    };
                }
                Event::Keyboard(ref k) => {
                    // keyboard events are delivered to the widget that has the focus.
                    // if no widget has focus, the event is dropped.
                    if let Some(focus) = self.focus {
                        content_widget.event(
                            event_ctx,
                            &Event::Internal(InternalEvent::RouteEvent {
                                target: focus,
                                event: Box::new(event.clone()),
                            }),
                        );
                    }
                }
                _ => {
                    // TODO
                }
            };
        }

        relayout_contents
    }
}

/// A window managed by kyute.
pub struct Window {
    window_state: Arc<RefCell<WindowState>>,
    contents: WidgetPod,
}

impl Window {
    /// Creates a new window.
    ///
    /// TODO: explain subtleties
    #[composable(uncached)]
    pub fn new(window_builder: WindowBuilder, contents: WidgetPod) -> WidgetPod<Window> {
        let window_state = Cache::memoize((), move || {
            Arc::new(RefCell::new(WindowState {
                window: None,
                window_builder: Some(window_builder),
                focus: None,
                pointer_grab: None,
                hot: None,
                inputs: Default::default(),
                last_click: None,
                scale_factor: 1.0, // initialized during window creation
                invalid: Default::default(),
            }))
        });

        WidgetPod::new(Window {
            window_state,
            contents,
        })
    }
}

impl Widget for Window {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::Initialize => {
                // create the window
                tracing::trace!("creating window");
                let mut window_state = self.window_state.borrow_mut();
                let window = PlatformWindow::new(
                    ctx.event_loop,
                    window_state.window_builder.take().unwrap(),
                    None,
                )
                .expect("failed to create window");
                ctx.register_window(window.id());
                window_state.scale_factor = window.window().scale_factor();
                window_state.window = Some(window);
            }
            Event::WindowEvent(window_event) => {
                let mut window_state = self.window_state.borrow_mut();
                window_state.process_window_event(ctx, &self.contents, window_event);
            }
            Event::WindowRedrawRequest => {
                let mut window_state = self.window_state.borrow_mut();
                let window_state = &mut *window_state;
                if let Some(ref mut window) = window_state.window {
                    let scale_factor = window.window().scale_factor();
                    let window_size = window.window().inner_size().to_logical(scale_factor);
                    let window_bounds = Rect::new(
                        Point::origin(),
                        Size::new(window_size.width, window_size.height),
                    );
                    let mut invalid = Region::new();
                    invalid.add_rect(window_bounds);

                    let focus = window_state.focus;
                    let pointer_grab = window_state.pointer_grab;
                    let hot = window_state.hot;
                    let inputs = &window_state.inputs;
                    let scale_factor = window_state.scale_factor;
                    let id = ctx.widget_id();

                    window.draw_skia(move |canvas| {
                        let mut paint_ctx = PaintCtx {
                            canvas,
                            id,
                            window_bounds,
                            focus,
                            pointer_grab,
                            hot,
                            inputs,
                            scale_factor,
                            invalid: &invalid,
                            hover: false,
                        };
                        // TODO environment
                        tracing::trace!("window redraw");
                        self.contents
                            .paint(&mut paint_ctx, window_bounds, &Environment::new());
                    });
                } else {
                    tracing::warn!("WindowRedrawRequest: window has not yet been created");
                }
            }
            _ => self.contents.event(ctx, event),
        }

        let mut window_state = self.window_state.borrow_mut();
        if let Some(ref mut window) = window_state.window {
            let (width, height): (f64, f64) = window.window().inner_size().into();
            let mut m_window = Measurements::new(Size::new(width, height));
            let (m_content, layout_changed) = self.contents.relayout(
                BoxConstraints::new(0.0..width, 0.0..height),
                &Environment::new(),
            );
            let offset = align_boxes(Alignment::CENTER, &mut m_window, m_content);
            self.contents.set_child_offset(offset);
            // layout_changed can be true because:
            // - the root constraints have changed (i.e. the size of the window), and the layouts had to be recalculated
            // - a widget had no layout, either because it was just created, or because it was invalidated (by `EventCtx::request_relayout`, which propagates upward).
            if layout_changed | ctx.redraw {
                window.window().request_redraw();
            }
        }
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        Measurements {
            size: Default::default(),
            baseline: None,
            is_window: true,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        //self.contents.paint(ctx, bounds, env)
    }
}
