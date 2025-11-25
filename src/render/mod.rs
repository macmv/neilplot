use std::path::Path;

use vello::{
  Renderer,
  kurbo::{Affine, Circle},
  peniko::{Color, color::palette},
  wgpu::{self, TextureDescriptor},
};

use crate::Plot;

struct GpuHandle {
  device: wgpu::Device,
  queue:  wgpu::Queue,
}

impl Plot<'_> {
  pub fn save(&self, path: impl AsRef<Path>) { foo(); }
}

impl GpuHandle {
  fn new() -> Self {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter =
      pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("Failed to create adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
      label:             None,
      required_features: wgpu::Features::empty(),
      required_limits:   wgpu::Limits::defaults(),
      memory_hints:      wgpu::MemoryHints::MemoryUsage,
      trace:             wgpu::Trace::Off,
    }))
    .expect("Failed to create device");

    GpuHandle { device, queue }
  }
}

fn foo() {
  let handle = GpuHandle::new();

  let width = 1024;
  let height = 1024;

  // Initialize wgpu and get handles
  let mut renderer = Renderer::new(&handle.device, vello::RendererOptions::default())
    .expect("Failed to create renderer");
  // Create scene and draw stuff in it
  let mut scene = vello::Scene::new();
  scene.fill(
    vello::peniko::Fill::NonZero,
    Affine::IDENTITY,
    Color::from_rgb8(242, 140, 168),
    None,
    &Circle::new((420.0, 200.0), 120.0),
  );

  let texture = handle.device.create_texture(&TextureDescriptor {
    label:           Some("Render Texture"),
    size:            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    mip_level_count: 1,
    sample_count:    1,
    dimension:       wgpu::TextureDimension::D2,
    format:          wgpu::TextureFormat::Rgba8Unorm,
    usage:           wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
    view_formats:    &[],
  });

  let view = &texture.create_view(&wgpu::TextureViewDescriptor::default());

  renderer
    .render_to_texture(
      &handle.device,
      &handle.queue,
      &scene,
      &view,
      &vello::RenderParams {
        base_color: palette::css::BLACK,
        width,
        height,
        antialiasing_method: vello::AaConfig::Msaa16,
      },
    )
    .expect("Failed to render to a texture");

  let buffer = handle.device.create_buffer(&wgpu::BufferDescriptor {
    label:              Some("Output Buffer"),
    size:               (4 * width * height) as u64,
    usage:              wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  let mut encoder = handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("texture_buffer_copy_encoder"),
  });

  encoder.copy_texture_to_buffer(
    wgpu::TexelCopyTextureInfo {
      texture:   &texture,
      mip_level: 0,
      origin:    wgpu::Origin3d::ZERO,
      aspect:    wgpu::TextureAspect::All,
    },
    wgpu::TexelCopyBufferInfo {
      buffer: &buffer,
      layout: wgpu::TexelCopyBufferLayout {
        offset:         0,
        bytes_per_row:  Some(4 * width),
        rows_per_image: Some(height),
      },
    },
    wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
  );

  handle.queue.submit(std::iter::once(encoder.finish()));

  let buffer_slice = buffer.slice(..);
  let buffer = buffer.clone();
  buffer_slice.map_async(wgpu::MapMode::Read, move |_| {
    let data = buffer.get_mapped_range(..);

    use image::{ImageBuffer, Rgba};
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).unwrap();
    buffer.save("foo.png").unwrap();
  });
}
