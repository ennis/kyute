use crate::graal::vk;
use kyute::{
    application, graal,
    style::Shape,
    widget::{prelude::*, Button, Grid, Retained, RetainedWidget, Text},
    Window,
};
use kyute_common::{Color, SizeI};
use kyute_shell::{animation::Layer, application::Application, winit::window::WindowBuilder};
use std::path::Path;

/// Loads an image into a
fn load_image(
    device: &graal::Device,
    frame: &mut graal::Frame<()>,
    path: &Path,
    usage: graal::vk::ImageUsageFlags,
    mipmaps: bool,
) -> (graal::ImageInfo, u32, u32) {
    use openimageio::{ImageInput, TypeDesc};

    let image_input = ImageInput::open(path).expect("could not open image file");
    let spec = image_input.spec();

    let nchannels = spec.num_channels();
    let format_typedesc = spec.format();
    let width = spec.width();
    let height = spec.height();

    if nchannels > 4 {
        panic!("unsupported number of channels: {}", nchannels);
    }

    let (vk_format, bpp) = match (format_typedesc, nchannels) {
        (TypeDesc::U8, 1) => (vk::Format::R8_UNORM, 1usize),
        (TypeDesc::U8, 2) => (vk::Format::R8G8_UNORM, 2usize),
        (TypeDesc::U8, 3) => (vk::Format::R8G8B8A8_UNORM, 4usize), // RGB8 not very well supported
        (TypeDesc::U8, 4) => (vk::Format::R8G8B8A8_UNORM, 4usize),
        (TypeDesc::U16, 1) => (vk::Format::R16_UNORM, 2usize),
        (TypeDesc::U16, 2) => (vk::Format::R16G16_UNORM, 4usize),
        (TypeDesc::U16, 3) => (vk::Format::R16G16B16A16_UNORM, 8usize),
        (TypeDesc::U16, 4) => (vk::Format::R16G16B16A16_UNORM, 8usize),
        (TypeDesc::U32, 1) => (vk::Format::R32_UINT, 4usize),
        (TypeDesc::U32, 2) => (vk::Format::R32G32_UINT, 8usize),
        (TypeDesc::U32, 3) => (vk::Format::R32G32B32A32_UINT, 16usize),
        (TypeDesc::U32, 4) => (vk::Format::R32G32B32A32_UINT, 16usize),
        (TypeDesc::HALF, 1) => (vk::Format::R16_SFLOAT, 2usize),
        (TypeDesc::HALF, 2) => (vk::Format::R16G16_SFLOAT, 4usize),
        (TypeDesc::HALF, 3) => (vk::Format::R16G16B16A16_SFLOAT, 8usize),
        (TypeDesc::HALF, 4) => (vk::Format::R16G16B16A16_SFLOAT, 8usize),
        (TypeDesc::FLOAT, 1) => (vk::Format::R32_SFLOAT, 4usize),
        (TypeDesc::FLOAT, 2) => (vk::Format::R32G32_SFLOAT, 8usize),
        (TypeDesc::FLOAT, 3) => (vk::Format::R32G32B32A32_SFLOAT, 16usize),
        (TypeDesc::FLOAT, 4) => (vk::Format::R32G32B32A32_SFLOAT, 16usize),
        _ => panic!("unsupported image format"),
    };

    let mip_levels = graal::get_mip_level_count(width, height);

    // create the texture
    let graal::ImageInfo {
        handle: image_handle,
        id: image_id,
    } = device.create_image(
        path.to_str().unwrap(),
        graal::MemoryLocation::GpuOnly,
        &graal::ImageResourceCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            usage: usage | vk::ImageUsageFlags::TRANSFER_DST,
            format: vk_format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels,
            array_layers: 1,
            samples: 1,
            tiling: Default::default(),
        },
    );

    let byte_size = width as u64 * height as u64 * bpp as u64;

    // create a staging buffer
    let staging_buffer = device.create_buffer(
        "staging",
        graal::MemoryLocation::CpuToGpu,
        &graal::BufferResourceCreateInfo {
            usage: vk::BufferUsageFlags::TRANSFER_SRC,
            byte_size,
            map_on_create: true,
        },
    );

    // read image data
    unsafe {
        image_input
            .read_unchecked(
                0,
                0,
                0..nchannels,
                format_typedesc,
                staging_buffer.mapped_ptr.unwrap().as_ptr() as *mut u8,
                bpp,
            )
            .expect("failed to read image");
    }

    let staging_buffer_handle = staging_buffer.handle;

    // === upload pass ===
    let mut pass = frame.add_pass(
        graal::PassBuilder::new()
            .name("image upload")
            .image_dependency(
                image_id,
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TRANSFER,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            )
            .buffer_dependency(
                staging_buffer.id,
                vk::AccessFlags::TRANSFER_READ,
                vk::PipelineStageFlags::TRANSFER,
            )
            .record_callback(move |context, _, command_buffer| unsafe {
                let device = context.vulkan_device();
                let regions = &[vk::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: width,
                    buffer_image_height: height,
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                    image_extent: vk::Extent3D {
                        width,
                        height,
                        depth: 1,
                    },
                }];

                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    staging_buffer_handle,
                    image_handle,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    regions,
                );
            }),
    );

    frame.destroy_buffer(staging_buffer.id);
    (
        graal::ImageInfo {
            handle: image_handle,
            id: image_id,
        },
        width,
        height,
    )
}

pub struct NativeLayerWidget {
    image: graal::ImageInfo,
    image_size: SizeI,
}

impl Drop for NativeLayerWidget {
    fn drop(&mut self) {
        let gpu_device = Application::instance().gpu_device();
        gpu_device.destroy_image(self.image.id);
    }
}

impl NativeLayerWidget {
    /// Renders the current view.
    fn render(&mut self, layer: &Layer, scale_factor: f64) {
        let mut gpu_context = Application::instance().lock_gpu_context();
        let gpu_device = Application::instance().gpu_device();
        let layer_surface = layer.acquire_surface();
        let layer_image = layer_surface.image_info();

        let blit_w = self.image_size.width.min(layer.size().width);
        let blit_h = self.image_size.height.min(layer.size().height);

        let mut frame = graal::Frame::new();
        frame.add_pass(
            graal::PassBuilder::new()
                .name("blit to screen")
                .image_dependency(
                    self.image.id,
                    vk::AccessFlags::TRANSFER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                )
                .image_dependency(
                    layer_image.id,
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                )
                .record_callback(move |context, _, command_buffer| {
                    let dst_image_handle = layer_image.handle;
                    let src_image_handle = self.image.handle;

                    let regions = &[vk::ImageBlit {
                        src_subresource: vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        src_offsets: [
                            vk::Offset3D { x: 0, y: 0, z: 0 },
                            vk::Offset3D {
                                x: blit_w as i32,
                                y: blit_h as i32,
                                z: 1,
                            },
                        ],
                        dst_subresource: vk::ImageSubresourceLayers {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            mip_level: 0,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        dst_offsets: [
                            vk::Offset3D { x: 0, y: 0, z: 0 },
                            vk::Offset3D {
                                x: blit_w as i32,
                                y: blit_h as i32,
                                z: 1,
                            },
                        ],
                    }];

                    unsafe {
                        context.vulkan_device().cmd_blit_image(
                            command_buffer,
                            src_image_handle,
                            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                            dst_image_handle,
                            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                            regions,
                            vk::Filter::NEAREST,
                        );
                    }
                }),
        );

        gpu_context.submit_frame(&mut (), frame, &graal::SubmitInfo::default());
    }
}

impl RetainedWidget for NativeLayerWidget {
    type Args = ();

    fn new(args: &Self::Args) -> Self {
        // load the image
        let mut gpu_context = Application::instance().lock_gpu_context();
        let gpu_device = Application::instance().gpu_device();
        let mut frame = graal::Frame::new();

        let (file_image_info, file_image_width, file_image_height) = load_image(
            &gpu_device,
            &mut frame,
            "data/haniyasushin_keiki.jpg".as_ref(),
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED,
            false,
        );
        gpu_context.submit_frame(&mut (), frame, &graal::SubmitInfo::default());

        NativeLayerWidget {
            image: file_image_info,
            image_size: SizeI::new(file_image_width as i32, file_image_height as i32),
        }
    }

    fn update(&mut self, args: &Self::Args) {
        // nothing
    }

    fn widget_id(&self) -> Option<WidgetId> {
        None
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams, env: &Environment) -> Geometry {
        Geometry {
            x_align: Alignment::CENTER,
            y_align: Alignment::CENTER,
            padding_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            measurements: Measurements::new(params.max),
        }
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // nothing to do
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // nothing to do (all done in layer_paint)
    }

    fn layer_paint(&mut self, ctx: &mut kyute::LayerPaintCtx, layer: &Layer, scale_factor: f64) {
        self.render(layer, scale_factor)
    }
}

#[composable]
fn scaffold() -> impl Widget {
    let mut grid = Grid::row(60.dip());
    grid.insert(Text::new("Text overlay"));
    grid.insert(Button::new("Click me"));
    grid.set_column_gap(10.dip());
    let grid_background = grid.background("#00FF0022");
    let view_background = WidgetPod::with_native_layer(Retained::<NativeLayerWidget>::new(&()));
    grid_background.above(view_background, Alignment::START)
}

#[composable]
fn ui_root() -> impl Widget {
    Window::new(WindowBuilder::new().with_title("3D view"), scaffold(), None)
}

fn main() {
    /*tracing_subscriber::fmt()
    .compact()
    .with_target(false)
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();*/
    /*use tracing_subscriber::layer::SubscriberExt;
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::new().with_stackdepth(0)),
    )
    .expect("set up the subscriber");*/
    let mut env = Environment::new();
    application::run_with_env(ui_root, env);
}
