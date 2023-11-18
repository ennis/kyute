use std::collections::HashMap;

use egui::{
    vec2, Align2, CentralPanel, CollapsingHeader, Color32, FontFamily, FontId, Frame, Grid, Rect, RichText, Sense,
    SidePanel, Stroke, TextStyle, TopBottomPanel, Ui,
};
use egui_wgpu::wgpu;
use kurbo::Affine;
use winit::{
    event::WindowEvent,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use crate::{
    application::{AppState, ExtEvent},
    debug_util,
    debug_util::{
        get_debug_snapshots, DebugAffine, DebugRect, ElementDebugNode, ElementPtrId, LayoutDebugInfo, PaintDebugInfo,
        PropertyValueKind, SnapshotCause, WindowSnapshot,
    },
    window::{DebugOverlay, WindowPaintOptions},
    ElementId, Geometry,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// USEFUL CODE

#[derive(Copy, Clone)]
struct DebugElementLayout {
    parent_transform: Affine,
    transform: Affine,
    geometry: Geometry,
}

fn debug_element_layout(debug_root: &ElementDebugNode, target: ElementPtrId) -> Option<DebugElementLayout> {
    if let Some(node) = debug_root.find_by_ptr(target) {
        let mut layout = DebugElementLayout {
            parent_transform: Default::default(),
            transform: Default::default(),
            geometry: Default::default(),
        };
        layout.parent_transform = node.property("window_transform").cloned().unwrap_or_default();
        layout.transform = node.property("transform").cloned().unwrap_or_default();
        layout.geometry = node.property("geometry").cloned().unwrap_or_default();
        Some(layout)
    } else {
        None
    }
}

struct DebugState {
    selection: Option<ElementPtrId>,
    hover_selection: Option<ElementPtrId>,
    focused: Option<ElementId>,
    pointer_grab: Option<ElementId>,
    show_current_snapshot: bool,
    snapshot_index: usize,
    force_continuous_redraw: bool,
    force_relayout: bool,
    debug_overlays: HashMap<WindowId, DebugOverlay>,
}

fn element_in_propgation_path(debug_snapshot: &WindowSnapshot, element_ptr_id: ElementPtrId) -> bool {
    debug_snapshot
        .event_info
        .elements
        .iter()
        .find(|e| e.element_ptr == element_ptr_id)
        .is_some()
}

impl DebugState {
    /// Draw a row (and its children) of the element tree.
    fn draw_element_tree_node(
        &mut self,
        ui: &mut Ui,
        node: &ElementDebugNode,
        debug_snapshot: &WindowSnapshot,
        level: u32,
    ) {
        let id = ui.id();
        let mut expanded: bool = ui.data(|data| data.get_temp(id).unwrap_or(true));

        let active_color = Color32::from_rgb(51, 126, 255); // selected color
        let hover_color = Color32::from_rgb(51, 51, 51); // hovered color
        let default_bg_color = ui.style().visuals.faint_bg_color; // default background color
        let text_color = ui.style().visuals.text_color(); // text color

        let height = 24.0; // row height
        let indent = 24.0;
        let indent_offset = vec2(level as f32 * indent, 0.0); // indent offset
        let width = ui.available_width(); // row width
        let size = vec2(width, height); // size of the row
        let (rect, response_bg) = ui.allocate_exact_size(size, Sense::click_and_drag()); // allocate rect

        //let text_pos = rect.left_center() + indent_offset + vec2(18.0, 0.0); // label position
        let text_rect = Rect::from_two_pos(
            rect.left_top() + indent_offset + vec2(17.0, 1.0),
            rect.right_bottom() - vec2(16.0, 0.0),
        ); // label rect
        let delete_icon_pos = rect.right_center() - vec2(16.0, 0.0); // button position
        let expand_icon_pos = rect.left_center() + indent_offset + vec2(8.0, 0.0); // expand icon position

        let in_event_propagation_path = element_in_propgation_path(debug_snapshot, node.ptr_id);

        let event_highlight = if in_event_propagation_path {
            ui.ctx().animate_value_with_time(id, 1.0, 0.0)
        } else {
            ui.ctx().animate_value_with_time(id, 0.0, 0.25)
        };

        // determine the background color
        let bg_color = if self.selection == Some(node.ptr_id) {
            active_color
        } else if response_bg.hovered() {
            hover_color
        } else {
            default_bg_color
        };
        // Fade out event highlight
        let mut bg_color_f = egui::Rgba::from(bg_color);
        bg_color_f = (1.0 - event_highlight) * bg_color_f + event_highlight * egui::Rgba::from_rgb(1.0, 0.0, 0.0);
        let bg_color: Color32 = bg_color_f.into();

        let stroke = Stroke::new(0.0, Color32::BLACK);

        if response_bg.clicked() {
            self.selection = Some(node.ptr_id);
        }
        if response_bg.hovered() {
            self.hover_selection = Some(node.ptr_id);
        }

        // Layer row & label
        {
            let _pixel_rect = response_bg.rect.shrink(0.5);
            ui.painter().rect(response_bg.rect, 0.0, bg_color, stroke);

            ui.allocate_ui_at_rect(text_rect, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    use egui_phosphor::regular as icons;
                    let icon = match node.ty {
                        "GridElement" => icons::SQUARES_FOUR,
                        "TransformNode" => icons::ARROWS_OUT_CARDINAL,
                        "DecoratedBoxElement" => icons::PAINT_BRUSH,
                        "TextElement" => icons::TEXT_T,
                        "FrameElement" => icons::RECTANGLE,
                        "ClickableElement" => icons::CURSOR_CLICK,
                        "NullElement" => icons::PLACEHOLDER,
                        "OverlayElement" => icons::STACK_SIMPLE,
                        "AlignElement" => icons::ALIGN_LEFT,
                        "Background" => icons::PAINT_ROLLER,
                        "PaddingElement" => icons::FRAME_CORNERS,
                        "ConstrainedElement" => icons::BOUNDING_BOX,
                        _ => icons::QUESTION,
                    };

                    ui.colored_label(Color32::WHITE, RichText::new(icon).size(20.0));
                    ui.colored_label(Color32::YELLOW, format!(" {} ", node.ty));
                    ui.colored_label(text_color, node.name);

                    if !node.id.is_anonymous() {
                        ui.colored_label(Color32::GRAY, format!(" ({:08X})", node.id.to_u64() >> 32));
                    }

                    if Some(node.id) == self.pointer_grab {
                        ui.label(RichText::new(" pointer grab").font(FontId::monospace(12.0)));
                    }
                    if Some(node.id) == self.focused {
                        ui.label(RichText::new(" focus").font(FontId::monospace(12.0)));
                    }
                })
                .response
            });
            /*ui.painter().text(
                text_pos,
                Align2::LEFT_CENTER,
                &item.label,
                FontId::proportional(16.0),
                text_color,
            );*/
        }

        {
            // caret
            if !node.children.is_empty() {
                //let mut expanded: bool = ui.data(|data| data.get_temp(id).unwrap_or(false));
                let rect = Rect::from_center_size(expand_icon_pos, vec2(16.0, 16.0));
                let resp = ui.allocate_rect(rect, Sense::click());
                let caret = if expanded {
                    egui_phosphor::regular::CARET_DOWN
                } else {
                    egui_phosphor::regular::CARET_RIGHT
                };
                ui.painter().text(
                    expand_icon_pos,
                    Align2::CENTER_CENTER,
                    caret,
                    FontId::proportional(16.0),
                    text_color,
                );
                if resp.clicked() {
                    expanded = !expanded;
                }
            }
        }

        {
            // useless button
            let rect = Rect::from_center_size(delete_icon_pos, vec2(16.0, 16.0));
            let _resp = ui.allocate_rect(rect, Sense::click());
            ui.painter().text(
                delete_icon_pos,
                Align2::CENTER_CENTER,
                egui_phosphor::regular::TRASH,
                FontId::proportional(16.0),
                text_color,
            );
        }

        ui.advance_cursor_after_rect(rect);

        if expanded {
            for child in node.children {
                ui.push_id(child.ptr_id, |ui| {
                    self.draw_element_tree_node(ui, child, debug_snapshot, level + 1);
                });
            }
        }

        ui.data_mut(|data| data.insert_temp(id, expanded));
    }

    fn element_tree_panel_contents(&mut self, ui: &mut Ui, debug_snapshot: &WindowSnapshot) {
        self.hover_selection = None;

        ui.heading(format!(
            "Window {} ({:?})",
            u64::from(debug_snapshot.window),
            debug_snapshot.window_title
        ));
        ui.separator();
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.draw_element_tree_node(ui, &debug_snapshot.root, debug_snapshot, 0);
        });

        // ID tree
        CollapsingHeader::new("ID Tree").default_open(true).show(ui, |ui| {
            Grid::new("id_tree").num_columns(2).striped(true).show(ui, |ui| {
                ui.label("Child");
                ui.label("Parent");
                ui.end_row();
                for (child, parent) in debug_snapshot.element_id_tree.map.iter() {
                    ui.label(format!("{:08X}", child.to_u64() >> 32));
                    ui.label(format!("{:08X}", parent.to_u64() >> 32));
                    ui.end_row();
                }
            });
        });
    }

    fn event_panel_contents(&mut self, ui: &mut Ui, debug_snapshot: &WindowSnapshot) {
        ui.heading("Event delivery");

        Grid::new("layout").num_columns(3).striped(true).show(ui, |ui| {
            ui.label("Type");
            ui.label("ID");
            ui.label("Change Flags");
            ui.label("Event");
            ui.end_row();
            ui.end_row();
            for event in debug_snapshot.event_info.iter() {
                let Some(node) = debug_snapshot.root.find_by_ptr(event.element_ptr) else {
                    continue;
                };
                ui.label(format!("{:?}", node.ty));
                ui.label(format!("{:08X}", event.element_id.to_u64() >> 32));
                ui.label(format!("{:?}", event.change_flags));
                ui.label(format!("{:?}", event.event));
                ui.end_row();
            }
        });
    }

    fn property_panel_contents(
        &mut self,
        ui: &mut Ui,
        debug_root: &ElementDebugNode,
        layout_info: &LayoutDebugInfo,
        paint_info: &PaintDebugInfo,
    ) {
        if let Some(selection) = self.selection {
            if let Some(layout_info) = layout_info.get(selection) {
                ui.heading("Geometry");
                Grid::new("layout").num_columns(2).striped(true).show(ui, |ui| {
                    ui.label("Constraints");
                    ui.monospace(format!("{:?}", layout_info.constraints));
                    ui.end_row();
                    ui.label("Size");
                    ui.monospace(format!("{:?}", layout_info.geometry.size));
                    ui.end_row();
                    ui.label("Bounding Rect");
                    ui.monospace(format!("{:?}", DebugRect(layout_info.geometry.bounding_rect)));
                    ui.end_row();
                    ui.label("Paint Bounding Rect");
                    ui.monospace(format!("{:?}", DebugRect(layout_info.geometry.paint_bounding_rect)));
                    ui.end_row();
                    ui.label("Baseline");
                    if let Some(b) = layout_info.geometry.baseline {
                        ui.monospace(format!("{}", b));
                    } else {
                        ui.monospace("Unspecified");
                    }
                    ui.end_row();
                });
            }
            if let Some(paint_info) = paint_info.get(selection) {
                ui.separator();
                ui.heading("Paint");
                Grid::new("paint").num_columns(2).striped(true).show(ui, |ui| {
                    ui.label("Transform");
                    ui.monospace(format!("{:?}", DebugAffine(paint_info.transform)));
                    ui.end_row();
                });
            }
            if let Some(node) = debug_root.find_by_ptr(selection) {
                ui.separator();
                ui.heading("Properties");
                Grid::new("root").num_columns(2).striped(true).show(ui, |ui| {
                    for prop in node.properties {
                        ui.label(prop.name);
                        match prop.value {
                            PropertyValueKind::Erased(p) => {
                                ui.monospace(format!("{:?}", p));
                            }
                            PropertyValueKind::Str(str) => {
                                ui.monospace(format!("{}", str));
                            }
                        }
                        ui.end_row();
                    }
                });
            }
        }
    }

    fn update_debug_overlays(&mut self, snapshot: &WindowSnapshot) {
        self.debug_overlays.clear();

        // element selected for debug overlay
        let overlay_element = self.hover_selection.or(self.selection);

        // update debug overlay
        if let Some(elem) = overlay_element {
            if let Some(layout_info) = snapshot.layout_info.get(elem) {
                if let Some(paint_info) = snapshot.paint_info.get(elem) {
                    let paint_rect = paint_info
                        .transform
                        .transform_rect_bbox(layout_info.geometry.paint_bounding_rect);
                    self.debug_overlays.insert(
                        snapshot.window,
                        DebugOverlay {
                            debug_bounds: vec![paint_rect],
                        },
                    );
                    return;
                }
            }
        }
    }

    fn ui(&mut self, ctx: &egui::Context, _app_state: &mut AppState) {
        let snapshots = get_debug_snapshots();
        let num_snapshots = snapshots.len();

        if num_snapshots == 0 {
            return;
        }

        let snapshot_index = if self.show_current_snapshot {
            num_snapshots - 1
        } else {
            self.snapshot_index
        };

        let snapshot = &snapshots[snapshot_index];

        //self.focused = snapshot.focused;
        //self.pointer_grab = snapshot.pointer_grab;

        TopBottomPanel::top("event_selector_panel")
            .default_height(80.0)
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.heading(format!(
                    "Snapshot #{} ({})",
                    snapshot_index,
                    match snapshot.cause {
                        SnapshotCause::Relayout => "After Relayout",
                        SnapshotCause::Event => "After Event",
                        SnapshotCause::AfterPaint => "After Paint",
                    }
                ));

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.show_current_snapshot, "Current").clicked() {
                        self.show_current_snapshot = true;
                    }
                    if ui.selectable_label(!self.show_current_snapshot, "Snapshot").clicked() {
                        self.show_current_snapshot = false;
                    }
                    let max_snapshots = if num_snapshots == 0 { 0 } else { num_snapshots - 1 };
                    ui.add_enabled(
                        !self.show_current_snapshot,
                        egui::DragValue::new(&mut self.snapshot_index)
                            .clamp_range(0..=max_snapshots)
                            .suffix(format!(" / {}", max_snapshots)),
                    );
                });

                {
                    let mut v = debug_util::is_collection_enabled();
                    ui.checkbox(&mut v, "Enable debug collection");
                    debug_util::enable_collection(v);
                }

                ui.checkbox(&mut self.force_continuous_redraw, "Force continuous redraw");
                ui.checkbox(&mut self.force_relayout, "Force relayout");
                //ui.label(format!("focused: {:?}", snapshot.focused));
                //ui.label(format!("pointer_grab: {:?}", snapshot.pointer_grab));
            });

        TopBottomPanel::bottom("event_panel")
            .default_height(200.0)
            .min_height(200.0)
            .show(ctx, |ui| {
                self.event_panel_contents(ui, &snapshot.window_snapshots[0]);
            });

        SidePanel::left("debug_panel")
            .default_width(300.0)
            .min_width(300.0)
            .frame(Frame::none())
            .show(ctx, |ui| {
                self.element_tree_panel_contents(ui, &snapshot.window_snapshots[0]);
            });

        CentralPanel::default().show(ctx, |ui| {
            self.property_panel_contents(
                ui,
                &snapshot.window_snapshots[0].root,
                &snapshot.window_snapshots[0].layout_info,
                &snapshot.window_snapshots[0].paint_info,
            );
        });

        self.update_debug_overlays(&snapshot.window_snapshots[0]);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// BOILERPLATE CRAP

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "Inter-Medium".to_owned(),
        egui::FontData::from_static(include_bytes!("../../data/Inter-Medium.otf")),
    );

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter-Medium".to_owned());
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

fn setup_wgpu(
    window: &winit::window::Window,
) -> (
    wgpu::Instance,
    wgpu::Device,
    wgpu::Queue,
    wgpu::Surface<'static>,
    wgpu::SurfaceConfiguration,
) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::empty(), // disable default debug flags because it conflicts with our DX12 device
        dx12_shader_compiler: Default::default(),
        ..Default::default()
    });
    let surface = unsafe { instance.create_surface_from_raw(&window) }.unwrap();

    // thank the web backend for the async bullshit
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None, // Trace path
    ))
    .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    (instance, device, queue, surface, config)
}

pub(crate) struct DebugWindow {
    window: Window,
    modifiers: winit::event::Modifiers,
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    wgpu_device: wgpu::Device,
    wgpu_queue: wgpu::Queue,
    egui_wgpu: egui_wgpu::Renderer,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    state: DebugState,
}

impl DebugWindow {
    pub(crate) fn new(elwt: &EventLoopWindowTarget<ExtEvent>) -> DebugWindow {
        /*let style = egui::Style {
            visuals: egui::Visuals::light(),
            ..egui::Style::default()
        };*/
        let window = WindowBuilder::new()
            .with_title("Widget Inspector")
            .with_visible(false) // intiially invisible
            .with_inner_size(winit::dpi::LogicalSize::new(1000.0, 800.0))
            .build(elwt)
            .expect("failed to open debug window");

        // lots of bullshit

        let (_, device, queue, surface, surface_config) = setup_wgpu(&window);
        let egui_wgpu = egui_wgpu::Renderer::new(&device, surface_config.format, None, 1);
        let mut egui_winit = egui_winit::State::new(&window);
        egui_winit.set_pixels_per_point(window.scale_factor() as f32);
        let mut egui_ctx = egui::Context::default();
        setup_custom_fonts(&mut egui_ctx);

        DebugWindow {
            window,
            modifiers: Default::default(),
            egui_wgpu,
            egui_ctx,
            egui_winit,
            surface,
            surface_config,
            wgpu_queue: queue,
            wgpu_device: device,
            state: DebugState {
                selection: None,
                hover_selection: None,
                focused: None,
                pointer_grab: None,
                show_current_snapshot: true,
                snapshot_index: 0,
                force_continuous_redraw: false,
                force_relayout: false,
                debug_overlays: Default::default(),
            },
        }
    }

    /// Whether the "force continuous redraw" option is checked.
    pub(crate) fn force_continuous_redraw(&self) -> bool {
        self.state.force_continuous_redraw
    }

    /// Returns the debug paint options for the specified window.
    pub(crate) fn window_paint_options(&self, window_id: WindowId) -> WindowPaintOptions {
        let mut options = WindowPaintOptions::default();
        if let Some(overlay) = self.state.debug_overlays.get(&window_id) {
            options.debug_overlay = Some(overlay.clone());
        }
        options
    }

    fn window_event(&mut self, event: &WindowEvent, app_state: &mut AppState) {
        match event {
            WindowEvent::CloseRequested => {
                self.window.set_visible(false);
            }
            WindowEvent::Resized(size) => {
                eprintln!("resized to {:?}", size);
                self.surface_config.width = size.width;
                self.surface_config.height = size.height;
                self.surface.configure(&self.wgpu_device, &self.surface_config);
            }
            WindowEvent::RedrawRequested => {
                let raw_input = self.egui_winit.take_egui_input(&self.window);
                let egui::FullOutput {
                    platform_output,
                    repaint_after: _,
                    textures_delta,
                    shapes,
                } = self.egui_ctx.run(raw_input, |ctx| self.state.ui(ctx, app_state));
                self.egui_winit
                    .handle_platform_output(&self.window, &self.egui_ctx, platform_output);

                // bunch of rendering code that should really be in egui-wgpu already, but whatever
                let tris = self.egui_ctx.tessellate(shapes);

                for (tex_id, delta) in textures_delta.set {
                    self.egui_wgpu
                        .update_texture(&self.wgpu_device, &self.wgpu_queue, tex_id, &delta);
                }

                match self.surface.get_current_texture() {
                    Ok(frame) => {
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder = self.wgpu_device.create_command_encoder(&Default::default());

                        {
                            self.egui_wgpu.update_buffers(
                                &self.wgpu_device,
                                &self.wgpu_queue,
                                &mut encoder,
                                &tris,
                                &egui_wgpu::renderer::ScreenDescriptor {
                                    size_in_pixels: [self.surface_config.width, self.surface_config.height],
                                    pixels_per_point: 1.0, // TODO
                                },
                            );

                            let mut egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations::default(),
                                })],
                                ..Default::default()
                            });

                            self.egui_wgpu.render(
                                &mut egui_pass,
                                &tris,
                                &egui_wgpu::renderer::ScreenDescriptor {
                                    size_in_pixels: [self.surface_config.width, self.surface_config.height],
                                    pixels_per_point: 1.0,
                                },
                            );
                        }
                        self.wgpu_queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    Err(e) => {
                        println!("Failed to acquire next swap chain texture {}", e);
                    }
                }
                for tex_id in textures_delta.free {
                    self.egui_wgpu.free_texture(&tex_id);
                }
            }
            event => {
                let response = self.egui_winit.on_event(&mut self.egui_ctx, event);
                if response.repaint {
                    self.window.request_redraw();
                }
            }
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(crate) fn event(
        &mut self,
        _elwt: &EventLoopWindowTarget<ExtEvent>,
        event: &winit::event::Event<ExtEvent>,
        app_state: &mut AppState,
    ) -> bool {
        match event {
            winit::event::Event::WindowEvent { window_id, event } => {
                if *window_id == self.window.id() {
                    // handle window event
                    self.window_event(&event, app_state);
                    true
                } else {
                    match event {
                        WindowEvent::ModifiersChanged(modifiers) => {
                            // need to keep track of the modifiers to catch "Ctrl+F12"
                            // because modifier state isn't passed to KeyboardInput anymore
                            // because the winit API is hostile, or stupid, or both
                            self.modifiers = modifiers.clone();
                            false
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            // catch the "Ctrl+F12" key combination to open the debug window
                            if self.modifiers.state().control_key()
                                && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12)
                                && event.state == winit::event::ElementState::Pressed
                            {
                                self.window.set_visible(true);
                                true
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                }
            }
            _ => false,
        }
    }
}
