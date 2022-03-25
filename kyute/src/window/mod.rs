mod key_code;
mod skia;

use crate::{
    align_boxes, cache, composable,
    core::{FocusState, GpuResourceReferences},
    event::{InputState, KeyboardEvent, PointerButton, PointerEvent, PointerEventKind, WheelDeltaMode, WheelEvent},
    graal,
    graal::{vk::Handle, MemoryLocation},
    region::Region,
    widget::Menu,
    window::skia::SkiaWindow,
    Alignment, BoxConstraints, Data, Environment, Event, EventCtx, InternalEvent, LayoutCtx, Measurements, PaintCtx,
    Point, Rect, RoundToPixel, Size, Widget, WidgetId, WidgetPod,
};
use keyboard_types::{KeyState, Modifiers};
use kyute::GpuFrameCtx;
use kyute_common::Transform;
use kyute_shell::{
    application::Application,
    winit,
    winit::{
        event::{DeviceId, MouseScrollDelta, WindowEvent},
        platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
        window::WindowBuilder,
    },
};
use skia_safe as sk;
use std::{cell::RefCell, env, mem, sync::Arc, time::Instant};
use tracing::trace;

/// Stores information about the last click (for double-click handling)
struct LastClick {
    device_id: DeviceId,
    button: PointerButton,
    position: Point,
    time: Instant,
    repeat_count: u32,
}

/// Retained state of `Window` widgets.
///
/// This is stored in the cache and mutated in place.
pub(crate) struct WindowState {
    window: Option<SkiaWindow>,
    window_builder: Option<WindowBuilder>,
    focus_state: FocusState,
    menu: Option<Menu>,
    inputs: InputState,
    last_click: Option<LastClick>,
    scale_factor: f64,
    invalid: Region,
    recomposed: bool,
}

impl WindowState {
    /// Processes a winit `WindowEvent` sent to this window.
    ///
    /// This converts `WindowEvents` to kyute `Events` and dispatches them to target widgets.
    /// It also updates various states that are tracked across WindowEvents, such as:
    /// - the current pointer position
    /// - information about the last click, for double-click handling
    fn process_window_event(
        &mut self,
        parent_ctx: &mut EventCtx,
        content_widget: &WidgetPod,
        window_event: &winit::event::WindowEvent,
        env: &Environment,
    ) {
        //let _span = trace_span!("process_window_event", ?window_event).entered();
        let window = self
            .window
            .as_mut()
            .expect("process_window_event received but window not initialized");

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
                //content_widget.invalidate_layout();
                // TODO maybe we should relayout in this case?
                None
            }
            WindowEvent::Resized(size) => {
                window.window.resize((size.width, size.height));
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
            WindowEvent::Command(id) => {
                // send to popup menu target if any
                if let Some(target) = self.focus_state.popup_target.take() {
                    let mut content_ctx = EventCtx::new_subwindow(
                        parent_ctx,
                        self.scale_factor,
                        &mut window.window,
                        &mut self.focus_state,
                    );
                    content_widget.event(
                        &mut content_ctx,
                        &mut Event::Internal(InternalEvent::RouteEvent {
                            target,
                            event: Box::new(Event::MenuCommand(*id)),
                        }),
                        env,
                    );
                } else {
                    // command from the window menu
                    // find matching action and trigger it
                    if let Some(ref menu) = self.menu {
                        if let Some(action) = menu.find_action_by_index(*id) {
                            action.triggered.signal(());
                        }
                    }
                }
                None
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
                is_synthetic: _,
            } => {
                let (key, code) = key_code::key_code_from_winit(input);
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
                let mut modifiers = Modifiers::empty();
                if mods.ctrl() {
                    modifiers |= Modifiers::CONTROL;
                }
                if mods.shift() {
                    modifiers |= Modifiers::SHIFT;
                }
                if mods.alt() {
                    modifiers |= Modifiers::ALT;
                }
                if mods.logo() {
                    modifiers |= Modifiers::SUPER;
                }
                self.inputs.modifiers = modifiers;
                None
            }
            WindowEvent::CursorMoved {
                device_id, position, ..
            } => {
                let logical_position: (f64, f64) = position.to_logical::<f64>(self.scale_factor).into();
                let logical_position = Point::new(logical_position.0, logical_position.1);
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
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                ..
            } => {
                let pointer_state = self.inputs.pointers.entry(*device_id).or_default();
                let pointer = PointerEvent {
                    kind: PointerEventKind::PointerMove, // TODO don't care?
                    position: pointer_state.position,
                    window_position: pointer_state.position,
                    modifiers: self.inputs.modifiers,
                    buttons: pointer_state.buttons,
                    pointer_id: *device_id,
                    button: None,
                    repeat_count: 0,
                };

                let wheel_event = match *delta {
                    MouseScrollDelta::LineDelta(x, y) => Event::Wheel(WheelEvent {
                        pointer,
                        delta_x: x as f64,
                        delta_y: y as f64,
                        delta_z: 0.0,
                        delta_mode: WheelDeltaMode::Line,
                    }),
                    MouseScrollDelta::PixelDelta(pos) => {
                        let (delta_x, delta_y): (f64, f64) = pos.to_logical::<f64>(self.scale_factor).into();
                        Event::Wheel(WheelEvent {
                            pointer,
                            delta_x,
                            delta_y,
                            delta_z: 0.0,
                            delta_mode: WheelDeltaMode::Pixel,
                        })
                    }
                };
                Some(wheel_event)
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
                            && (click_time - last.time) < Application::instance().double_click_time() =>
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

        if let Some(mut event) = event {
            //trace!("window event {:?}", event);

            //------------------------------------------------
            // Send event
            let old_focus = self.focus_state.focus;

            let pointer_grab_auto_release = matches!(
                event,
                Event::Pointer(PointerEvent {
                    kind: PointerEventKind::PointerUp,
                    ..
                })
            );

            match event {
                Event::Pointer(ref pointer_event)
                | Event::Wheel(WheelEvent {
                    pointer: ref pointer_event,
                    ..
                }) => {
                    // Pointer and wheel events are delivered to the node that is currently grabbing the pointer.
                    // If nothing is grabbing the pointer, the pointer event is delivered to a widget
                    // that passes the hit-test
                    if let Some(pointer_grab) = self.focus_state.pointer_grab {
                        let mut content_ctx = EventCtx::new_subwindow(
                            parent_ctx,
                            self.scale_factor,
                            &mut window.window,
                            &mut self.focus_state,
                        );
                        trace!("routing pointer event to pointer-capturing widget {:?}", pointer_grab);

                        content_widget.event(
                            &mut content_ctx,
                            // must use RoutePointerEvent so that relative pointer positions are computed during propagation
                            &mut Event::Internal(InternalEvent::RoutePointerEvent {
                                target: pointer_grab,
                                event: *pointer_event,
                            }),
                            env,
                        );
                    } else {
                        let mut content_ctx = EventCtx::new_subwindow(
                            parent_ctx,
                            self.scale_factor,
                            &mut window.window,
                            &mut self.focus_state,
                        );
                        // just forward to content, will do a hit-test
                        content_widget.event(&mut content_ctx, &mut event, env);
                    };
                }
                Event::Keyboard(_) => {
                    // keyboard events are delivered to the widget that has the focus.
                    // if no widget has focus, the event is dropped.
                    if let Some(focus) = self.focus_state.focus {
                        // TODO: helper function to send an event to a target
                        let mut content_ctx = EventCtx::new_subwindow(
                            parent_ctx,
                            self.scale_factor,
                            &mut window.window,
                            &mut self.focus_state,
                        );
                        content_widget.event(
                            &mut content_ctx,
                            &mut Event::Internal(InternalEvent::RouteEvent {
                                target: focus,
                                event: Box::new(event),
                            }),
                            env,
                        );
                    }
                }
                _ => {
                    warn!("unhandled processed window event {:?}", event)
                }
            };

            //------------------------------------------------
            // force release pointer grab on pointer up
            if pointer_grab_auto_release {
                //trace!("forcing release of pointer grab");
                self.focus_state.pointer_grab = None;
            }

            //------------------------------------------------
            // signal focus changes
            if old_focus != self.focus_state.focus {
                let new_focus = self.focus_state.focus;
                trace!("focus changed");

                let mut content_ctx =
                    EventCtx::new_subwindow(parent_ctx, self.scale_factor, &mut window.window, &mut self.focus_state);

                if let Some(old_focus) = old_focus {
                    content_widget.event(
                        &mut content_ctx,
                        &mut Event::Internal(InternalEvent::RouteEvent {
                            target: old_focus,
                            event: Box::new(Event::FocusLost),
                        }),
                        env,
                    );
                }

                if let Some(new_focus) = new_focus {
                    content_widget.event(
                        &mut content_ctx,
                        &mut Event::Internal(InternalEvent::RouteEvent {
                            target: new_focus,
                            event: Box::new(Event::FocusGained),
                        }),
                        env,
                    );
                }
            }
        }
    }

    /// Updates the window menu if the window is created.
    fn update_menu(&mut self) {
        if let Some(ref mut skia_window) = self.window {
            if let Some(ref menu) = self.menu {
                menu.assign_menu_item_indices();
                let m = menu.to_shell_menu(false);
                skia_window.window.set_menu(Some(m));
            } else {
                skia_window.window.set_menu(None);
            }
        }
    }
}

/// A window managed by kyute.
#[derive(Clone)]
pub struct Window {
    id: WidgetId,
    window_state: Arc<RefCell<WindowState>>,
    contents: Arc<WidgetPod>,
}

impl Window {
    /// Creates a new window.
    ///
    /// TODO: explain subtleties
    #[composable]
    pub fn new(window_builder: WindowBuilder, contents: impl Widget + 'static, menu: Option<Menu>) -> Window {
        // create the initial window state
        // we don't want to recreate it every time, so it only depends on the call ID.
        let window_state = cache::once(move || {
            Arc::new(RefCell::new(WindowState {
                window: None,
                window_builder: Some(window_builder),
                focus_state: FocusState::default(),
                menu: None,
                inputs: Default::default(),
                last_click: None,
                scale_factor: 1.0, // initialized during window creation
                invalid: Default::default(),
                recomposed: true,
            }))
        });

        // update window states:
        // menu bar ...
        {
            let mut window_state = window_state.borrow_mut();
            if !window_state.menu.same(&menu) {
                //tracing::trace!("updating window menu: {:#?}", menu);
                window_state.menu = menu;
                window_state.update_menu();
            }

            // set the `recomposed` flag to indicate that we called `Window::new` and that the contents
            // might have changed
            window_state.recomposed = true;
        }
        // TODO update title, size, position, etc.

        Window {
            id: WidgetId::here(),
            window_state,
            contents: Arc::new(WidgetPod::new(contents)),
        }
    }

    /// Hot mess responsible for rendering the contents of the window with vulkan and skia.
    fn do_redraw(&self, parent_ctx: &mut EventCtx, env: &Environment) {
        //use kyute_shell::{skia, skia::gpu::vk as skia_vk};

        let mut window_state = self.window_state.borrow_mut();
        let window_state = &mut *window_state;
        if let Some(ref mut window) = window_state.window {
            // collect all child widgets
            let widgets = {
                let mut content_ctx = EventCtx::new_subwindow(
                    parent_ctx,
                    window_state.scale_factor,
                    &mut window.window,
                    &mut window_state.focus_state,
                );
                let mut widgets = Vec::new();
                self.contents.event(
                    &mut content_ctx,
                    &mut Event::Internal(InternalEvent::Traverse { widgets: &mut widgets }),
                    env,
                );
                widgets
            };

            // get and lock GPU context for frame submission
            let app = Application::instance();
            let device = app.gpu_device().clone();
            let mut context = app.lock_gpu_context();

            //---------------------------------------------------------------------------
            // acquire next image in window swap chain for painting
            // if we can't, skip the whole rendering
            let swap_chain = window.window.swap_chain();
            let swap_chain_image = unsafe { context.acquire_next_image(swap_chain) };
            let swap_chain_image = match swap_chain_image {
                Ok(image) => image,
                Err(err) => {
                    tracing::warn!("failed to acquire swapchain image: {}", err);
                    return;
                }
            };

            // start GPU context frame
            let mut frame = context.start_frame(graal::FrameCreateInfo::default());

            //---------------------------------------------------------------------------
            // propagate GpuFrame event to child widgets, allowing them to push rendering passes
            // at the same time, collect resources that will be referenced during the UI painting pass.
            // TODO move this in core
            let mut gpu_ctx = GpuFrameCtx {
                frame: &mut frame,
                resource_references: GpuResourceReferences::new(),
                measurements: Default::default(),
                scale_factor: window_state.scale_factor,
            };
            for widget in widgets.iter() {
                widget.widget().gpu_frame(&mut gpu_ctx);
            }
            let resource_references = gpu_ctx.resource_references;

            //---------------------------------------------------------------------------
            // setup skia for rendering to a GPU image
            let (swap_chain_width, swap_chain_height) = window.window.swap_chain_size();

            // skia may not support rendering directly to the swapchain image (for example, it doesn't seem to support BGRA8888_SRGB).
            // so allocate a separate image to use as a render target, then copy.
            let skia_image_usage_flags = graal::vk::ImageUsageFlags::COLOR_ATTACHMENT
                | graal::vk::ImageUsageFlags::TRANSFER_SRC
                | graal::vk::ImageUsageFlags::TRANSFER_DST;
            // TODO: allow the user to choose
            let skia_image_format = graal::vk::Format::R16G16B16A16_SFLOAT;
            let skia_image = device.create_image(
                "skia render target",
                MemoryLocation::GpuOnly,
                &graal::ImageResourceCreateInfo {
                    image_type: graal::vk::ImageType::TYPE_2D,
                    usage: skia_image_usage_flags,
                    format: skia_image_format,
                    extent: graal::vk::Extent3D {
                        width: swap_chain_width,
                        height: swap_chain_height,
                        depth: 1,
                    },
                    mip_levels: 1,
                    array_layers: 1,
                    samples: 1,
                    tiling: graal::vk::ImageTiling::OPTIMAL,
                },
            );

            //----------------------------------------------------------------------------------
            // make a copy of the stuff we want to use in the command lambda
            // because it has a 'static lifetime bound and thus we can't borrow anything inside it
            // TODO: allow temporary borrows inside passes
            let scale_factor = window.window.window().scale_factor();
            let logical_size = window.window.window().inner_size().to_logical(scale_factor);
            let window_bounds = Rect::new(Point::origin(), Size::new(logical_size.width, logical_size.height));
            let focus = window_state.focus_state.focus;
            let pointer_grab = window_state.focus_state.pointer_grab;
            let hot = window_state.focus_state.hot;
            // FIXME we must clone here because the lambda is 'static, and this might be expensive. Use Arc instead?
            let inputs = window_state.inputs.clone();
            let scale_factor = window_state.scale_factor;
            let id = parent_ctx.widget_id();
            let mut recording_context = window.skia_recording_context.clone();
            let contents = self.contents.clone();

            // create the skia render pass
            {
                let mut ui_render_pass = frame.start_graphics_pass("UI render");

                // FIXME we just assume how it's going to be used by skia
                ui_render_pass.add_image_dependency(
                    skia_image.id,
                    graal::vk::AccessFlags::MEMORY_READ | graal::vk::AccessFlags::MEMORY_WRITE,
                    graal::vk::PipelineStageFlags::ALL_COMMANDS,
                    graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    graal::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );

                // add references collected during the GpuFrame pass
                for buf in resource_references.buffers {
                    ui_render_pass.add_buffer_dependency(buf.id, buf.access_mask, buf.stage_mask)
                }
                for img in resource_references.images {
                    ui_render_pass.add_image_dependency(
                        img.id,
                        img.access_mask,
                        img.stage_mask,
                        img.initial_layout,
                        img.final_layout,
                    )
                }

                ui_render_pass.set_submit_callback(move |_cctx, _, _queue| {
                    // create skia BackendRenderTarget and Surface
                    let skia_image_info = sk::gpu::vk::ImageInfo {
                        image: skia_image.handle.as_raw() as *mut _,
                        alloc: Default::default(),
                        tiling: sk::gpu::vk::ImageTiling::OPTIMAL,
                        layout: sk::gpu::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        format: unsafe { mem::transmute(skia_image_format.as_raw()) }, // SAFETY: it's a VkFormat, and hopefully skia_vk has a definition with all the latest enumerators...
                        image_usage_flags: skia_image_usage_flags.as_raw(),
                        sample_count: 1,
                        level_count: 1,
                        current_queue_family: sk::gpu::vk::QUEUE_FAMILY_IGNORED,
                        protected: sk::gpu::Protected::No,
                        ycbcr_conversion_info: Default::default(),
                        sharing_mode: sk::gpu::vk::SharingMode::EXCLUSIVE,
                    };
                    let render_target = sk::gpu::BackendRenderTarget::new_vulkan(
                        (swap_chain_width as i32, swap_chain_height as i32),
                        1,
                        &skia_image_info,
                    );
                    let mut surface = sk::Surface::from_backend_render_target(
                        &mut recording_context,
                        &render_target,
                        sk::gpu::SurfaceOrigin::TopLeft,
                        sk::ColorType::RGBAF16Norm, // ???
                        sk::ColorSpace::new_srgb_linear(),
                        Some(&sk::SurfaceProps::new(Default::default(), sk::PixelGeometry::RGBH)),
                    )
                    .unwrap();

                    // setup PaintCtx
                    let canvas = surface.canvas();
                    let mut invalid = Region::new();
                    invalid.add_rect(window_bounds);

                    // clear to default bg color
                    canvas.scale((scale_factor as sk::scalar, scale_factor as sk::scalar));
                    canvas.clear(sk::Color4f::new(0.0, 0.0, 0.0, 1.0));

                    let mut paint_ctx = PaintCtx {
                        canvas,
                        id,
                        window_transform: Transform::identity(),
                        focus,
                        pointer_grab,
                        hot,
                        inputs: &inputs,
                        scale_factor,
                        invalid: &invalid,
                        hover: false,
                        bounds: window_bounds,
                        active: false,
                    };

                    // TODO environment
                    //tracing::trace!("window redraw");
                    contents.paint(&mut paint_ctx, env);
                    surface.flush_and_submit();
                });

                ui_render_pass.finish();
            }

            graal::utils::blit_images(
                &mut frame,
                skia_image,
                swap_chain_image.image_info,
                (swap_chain_width, swap_chain_height),
                graal::vk::ImageAspectFlags::COLOR,
            );

            device.destroy_image(skia_image.id);

            // dump frame if requested
            match env::var("KYUTE_DUMP_GPU_FRAMES") {
                Ok(v) if v.parse() == Ok(true) => {
                    frame.dump(Some("kyute_gpu_frame"));
                }
                _ => {}
            }

            // present
            frame.present("present", &swap_chain_image);
            frame.finish(&mut ());
        } else {
            tracing::warn!("WindowRedrawRequest: window has not yet been created");
        }
    }
}

impl Widget for Window {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Initialize => {
                let mut window_state = self.window_state.borrow_mut();

                // skip if the window is already created
                if window_state.window.is_some() {
                    // if the window is created, and we haven't recomposed, don't send initialize
                    if window_state.recomposed {
                        self.contents.event(ctx, event, env);
                        window_state.recomposed = false;
                    }
                } else {
                    tracing::trace!("creating window");
                    let mut window_builder = window_state.window_builder.take().unwrap();

                    // set parent window
                    if let Some(ref mut parent_window) = ctx.parent_window {
                        window_builder = window_builder.with_owner_window(parent_window.window().hwnd() as *mut _);
                    }

                    let window = kyute_shell::window::Window::new(ctx.event_loop, window_builder, None)
                        .expect("failed to create window");

                    // create skia stuff
                    let skia_window = SkiaWindow::new(window);

                    // register it to the AppCtx, necessary so that the event loop can route window events
                    // to this widget
                    ctx.register_window(skia_window.window.id());

                    let scale_factor = skia_window.window.window().scale_factor();
                    let (width, height): (f64, f64) = skia_window
                        .window
                        .window()
                        .inner_size()
                        .to_logical::<f64>(scale_factor)
                        .into();
                    // perform initial layout of contents
                    self.contents.relayout(
                        ctx.app_ctx,
                        BoxConstraints::new(0.0..width, 0.0..height),
                        scale_factor,
                        env,
                    );

                    // update window state
                    window_state.scale_factor = scale_factor;
                    window_state.window = Some(skia_window);

                    // create the window menu
                    window_state.update_menu();
                }
            }
            Event::WindowEvent(window_event) => {
                let mut window_state = self.window_state.borrow_mut();
                window_state.process_window_event(ctx, &self.contents, window_event, env);
            }
            Event::WindowRedrawRequest => self.do_redraw(ctx, env),
            _ => {
                let mut window_state = self.window_state.borrow_mut();
                let window_state = &mut *window_state; // hrmpf...

                if let Some(window) = window_state.window.as_mut() {
                    let mut content_ctx = EventCtx::new_subwindow(
                        ctx,
                        window_state.scale_factor,
                        &mut window.window,
                        &mut window_state.focus_state,
                    );
                    self.contents.event(&mut content_ctx, event, env);
                } else {
                    //tracing::warn!("received window event before initialization: {:?}", event);
                    self.contents.event(ctx, event, env);
                }
                // don't propagate, but TODO check for redraw and such
            }
        }

        let mut window_state = self.window_state.borrow_mut();
        if let Some(ref mut skia_window) = window_state.window {
            let winit_window = skia_window.window.window();
            let scale_factor = winit_window.scale_factor();
            let (width, height): (f64, f64) = winit_window.inner_size().to_logical::<f64>(scale_factor).into();
            let mut m_window = Measurements::new(Size::new(width, height).into());
            let (m_content, layout_changed) = self.contents.relayout(
                ctx.app_ctx,
                BoxConstraints::new(0.0..width, 0.0..height),
                scale_factor,
                env,
            );
            if layout_changed {
                let offset = align_boxes(Alignment::CENTER, &mut m_window, m_content).round_to_pixel(scale_factor);
                self.contents.set_child_offset(offset);
            }

            if self.contents.invalidated() {
                winit_window.request_redraw()
            }
        }
    }

    fn layout(&self, _ctx: &mut LayoutCtx, _constraints: BoxConstraints, _env: &Environment) -> Measurements {
        Measurements::default()
    }

    fn paint(&self, _ctx: &mut PaintCtx, _env: &Environment) {
        //self.contents.paint(ctx, bounds, env)
    }
}
