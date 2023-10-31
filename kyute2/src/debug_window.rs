use crate::{
    application::{AppState, ExtEvent},
    context::ElementTree,
    debug_util::{debug_element_tree, DebugArena, DebugNode, ElementPtrId, PropertyValue, PropertyValueKind},
    window::{DebugOverlayData, UiHostWindowHandler},
    Element, ElementId, Geometry, PaintCtx,
};
use egui::{
    collapsing_header::CollapsingState,
    epaint::text,
    text::{Fonts, LayoutJob},
    vec2, Align2, CentralPanel, CollapsingHeader, Color32, FontFamily, FontId, Frame, Grid, Rect, Response, RichText,
    Sense, SidePanel, Stroke, TextFormat, TextStyle, Ui, WidgetText,
};
use egui_json_tree::JsonTree;
use egui_wgpu::{wgpu, wgpu::TextureFormat};
use egui_winit::winit::event::Event;
use kurbo::Affine;
use kyute2::debug_util::get_debug_snapshots;
use once_cell::sync::OnceCell;
use raw_window_handle::RawWindowHandle;
use std::{collections::hash_map::DefaultHasher, hash::Hasher, mem, ptr};
use winit::{
    event::WindowEvent,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// USEFUL CODE

#[derive(Copy, Clone)]
struct DebugElementLayout {
    parent_transform: Affine,
    transform: Affine,
    geometry: Geometry,
}

fn debug_element_layout(debug_root: &DebugNode, target: ElementPtrId) -> Option<DebugElementLayout> {
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

struct DebugWindowState {
    selection: Option<ElementPtrId>,
    hover_selection: Option<ElementPtrId>,
    focused: Option<ElementId>,
    pointer_grab: Option<ElementId>,
    show_current_snapshot: bool,
    snapshot_index: usize,
}

impl DebugWindowState {
    fn draw_list_item(&mut self, ui: &mut Ui, item: &DebugNode, level: u32) {
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

        let bg_color = if self.selection == Some(item.ptr_id) {
            active_color
        } else if response_bg.hovered() {
            hover_color
        } else {
            default_bg_color
        };

        let stroke = Stroke::new(0.0, Color32::BLACK);

        if response_bg.clicked() {
            self.selection = Some(item.ptr_id);
        }
        if response_bg.hovered() {
            self.hover_selection = Some(item.ptr_id);
        }

        // Layer row & label
        {
            let pixel_rect = response_bg.rect.shrink(0.5);
            ui.painter().rect(response_bg.rect, 0.0, bg_color, stroke);

            ui.allocate_ui_at_rect(text_rect, |ui: &mut egui::Ui| {
                ui.horizontal(|ui| {
                    use egui_phosphor::regular as icons;
                    let icon = match item.ty {
                        "GridElement" => icons::GRID_FOUR,
                        "TransformNode" => icons::ARROWS_OUT_CARDINAL,
                        "TextElement" => icons::TEXT_T,
                        "FrameElement" => icons::FRAME_CORNERS,
                        "ClickableElement" => icons::CURSOR_CLICK,
                        "NullElement" => icons::PLACEHOLDER,
                        "OverlayElement" => icons::STACK_SIMPLE,
                        "Background" => icons::PAINT_ROLLER,
                        _ => icons::QUESTION,
                    };

                    ui.colored_label(Color32::WHITE, RichText::new(icon).size(20.0));
                    ui.colored_label(Color32::YELLOW, format!(" {} ", item.ty));
                    ui.colored_label(text_color, item.name);

                    if !item.id.is_anonymous() {
                        ui.colored_label(Color32::GRAY, format!(" ({:08X})", item.id.to_u64() >> 32));
                    }

                    if Some(item.id) == self.pointer_grab {
                        ui.label(RichText::new(" pointer grab").font(FontId::monospace(12.0)));
                    }
                    if Some(item.id) == self.focused {
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
            if !item.children.is_empty() {
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
            let resp = ui.allocate_rect(rect, Sense::click());
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
            for child in item.children {
                ui.push_id(child.ptr_id, |ui| {
                    self.draw_list_item(ui, child, level + 1);
                });
            }
        }

        ui.data_mut(|data| data.insert_temp(id, expanded));
    }

    fn element_tree_panel_contents(&mut self, ui: &mut Ui, debug_root: &DebugNode, elem_tree: &ElementTree) {
        self.hover_selection = None;

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            self.draw_list_item(ui, debug_root, 0);
        });

        // element tree
        CollapsingHeader::new("Element Tree").default_open(true).show(ui, |ui| {
            Grid::new("element_tree").num_columns(2).striped(true).show(ui, |ui| {
                ui.label("Child");
                ui.label("Parent");
                ui.end_row();
                for (child, parent) in elem_tree.iter() {
                    ui.label(format!("{:08X}", child.to_u64() >> 32));
                    ui.label(format!("{:08X}", parent.to_u64() >> 32));
                    ui.end_row();
                }
            });
        });
    }

    fn property_panel_contents(&mut self, ui: &mut Ui, debug_root: &DebugNode) {
        if let Some(selection) = self.selection {
            if let Some(node) = debug_root.find_by_ptr(selection) {
                Grid::new("root").num_columns(2).striped(true).show(ui, |ui| {
                    for prop in node.properties {
                        ui.label(prop.name);
                        match prop.value {
                            PropertyValueKind::Erased(p) => {
                                if let Some(geom) = p.cast::<Geometry>() {
                                    // pretty-print geometry
                                    Grid::new("geometry").num_columns(2).show(ui, |ui| {
                                        ui.label("Size");
                                        ui.label(format!("{:?}", geom.size));
                                        ui.end_row();
                                        ui.label("Bounding Rect");
                                        ui.label(format!("{:?}", geom.bounding_rect));
                                        ui.end_row();
                                        ui.label("Paint Bounding Rect");
                                        ui.label(format!("{:?}", geom.paint_bounding_rect));
                                        ui.end_row();
                                        ui.label("Baseline");
                                        if let Some(b) = geom.baseline {
                                            ui.label(format!("{}", b));
                                        } else {
                                            ui.label("Unspecified");
                                        }
                                        ui.end_row();
                                    });
                                } else {
                                    ui.label(format!("{:?}", p));
                                }
                            }
                            PropertyValueKind::Str(str) => {
                                ui.label(format!("{}", str));
                            }
                        }
                        ui.end_row();
                    }
                });
            }
        }
    }

    fn ui(&mut self, ctx: &egui::Context, app_state: &mut AppState) {
        // TODO: pick window
        let Some((id, window)) = app_state.windows.iter_mut().next() else {
            return;
        };
        let Some(handler) = window.as_any().downcast_mut::<UiHostWindowHandler>() else {
            return;
        };

        let arena = DebugArena::new();
        let root = &handler.root;
        self.focused = handler.focus;
        self.pointer_grab = handler.pointer_grab;

        let snapshots = get_debug_snapshots();
        let num_snapshots = snapshots.len();

        let elem_tree_debug_root;
        let elem_tree;
        if self.show_current_snapshot || num_snapshots == 0 {
            elem_tree_debug_root = debug_element_tree(&arena, "root", root);
            elem_tree = handler.element_tree.clone();
        } else {
            elem_tree_debug_root = snapshots[self.snapshot_index].root;
            elem_tree = handler.element_tree.clone();
        };

        SidePanel::left("debug_panel")
            .default_width(300.0)
            .min_width(300.0)
            .frame(Frame::none())
            .show(ctx, |ui| {
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
                self.element_tree_panel_contents(ui, &elem_tree_debug_root, &elem_tree);

                // update debug overlay
                if let Some(hover_selection) = self.hover_selection {
                    if let Some(layout) = debug_element_layout(&elem_tree_debug_root, hover_selection) {
                        let transform = layout.parent_transform * layout.transform;
                        let rect = transform.transform_rect_bbox(layout.geometry.bounding_rect);
                        handler.set_debug_overlay(Some(DebugOverlayData {
                            debug_bounds: vec![rect],
                        }));
                    } else {
                        handler.set_debug_overlay(None);
                    }
                } else {
                    handler.set_debug_overlay(None);
                }
                handler.window.request_redraw();
            });

        CentralPanel::default().show(ctx, |ui| {
            self.property_panel_contents(ui, &elem_tree_debug_root);
        });
    }

    /*fn snapshot(&mut self, app_state: &mut AppState) {
        // TODO: pick window
        let Some((id, window)) = app_state.windows.iter_mut().next() else {
            return;
        };
        let Some(handler) = window.as_any().downcast_mut::<UiHostWindowHandler>() else {
            return;
        };

        let arena = get_debug_arena();
        let root = debug_element_tree(arena, "root", &handler.root);
        let event_data = mem::take(&mut handler.event_debug_data);

        /*self.snapshots.push(Snapshot {
            event: event_data,
            root: &DebugNode {},
        })*/
    }*/
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
    //egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter-Medium".to_owned());
    /*fonts
    .families
    .entry(FontFamily::Monospace)
    .or_default()
    .insert(0, "roboto".to_owned());*/
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Proportional)),
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
    wgpu::Surface,
    wgpu::SurfaceConfiguration,
) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        dx12_shader_compiler: Default::default(),
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
    surface: wgpu::Surface,
    state: DebugWindowState,
}

impl DebugWindow {
    pub(crate) fn new(elwt: &EventLoopWindowTarget<ExtEvent>) -> DebugWindow {
        /*let style = egui::Style {
            visuals: egui::Visuals::light(),
            ..egui::Style::default()
        };*/
        let mut window = WindowBuilder::new()
            .with_title("Widget Inspector")
            .with_visible(true)
            .with_inner_size(winit::dpi::LogicalSize::new(1350.0, 800.0))
            .build(elwt)
            .expect("failed to open debug window");

        // lots of bullshit

        let (_, device, queue, surface, surface_config) = setup_wgpu(&window);
        let mut egui_wgpu = egui_wgpu::Renderer::new(&device, surface_config.format, None, 1);
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
            state: DebugWindowState {
                selection: None,
                hover_selection: None,
                focused: None,
                pointer_grab: None,
                show_current_snapshot: true,
                snapshot_index: 0,
            },
        }
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
                    repaint_after,
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

            _ => {}
        }
    }

    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(crate) fn event(
        &mut self,
        elwt: &EventLoopWindowTarget<ExtEvent>,
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
                            true
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            // catch the "Ctrl+F12" key combination to open the debug window
                            if self.modifiers.state().control_key()
                                && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::F12)
                                && event.state == winit::event::ElementState::Pressed
                            {
                                self.window.set_visible(true);
                            }
                            true
                        }
                        _ => false,
                    }
                }
            }
            _ => false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// DEBUG OVERLAY
