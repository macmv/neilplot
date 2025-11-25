use std::path::Path;

use vello::{
  Renderer,
  kurbo::{Affine, Circle},
  peniko::{Color, color::palette},
  wgpu::{self, TextureDescriptor},
};

use crate::Plot;

struct GpuHandle {
  device:  wgpu::Device,
  queue:   wgpu::Queue,
  texture: wgpu::Texture,
}

struct RenderConfig {
  width:  u32,
  height: u32,
}

enum Target<'a> {
  Image(&'a Path),
}

impl Plot<'_> {
  pub fn save(&self, path: impl AsRef<Path>) { self.render(Target::Image(path.as_ref())); }

  fn render(&self, target: Target<'_>) {
    let config = RenderConfig { width: 1024, height: 1024 };
    let handle = GpuHandle::new(&config);

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

    let view = &handle.texture.create_view(&wgpu::TextureViewDescriptor::default());

    renderer
      .render_to_texture(
        &handle.device,
        &handle.queue,
        &scene,
        &view,
        &vello::RenderParams {
          base_color:          palette::css::BLACK,
          width:               config.width,
          height:              config.height,
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .expect("Failed to render to a texture");

    target.render(&handle, config);
  }
}

impl GpuHandle {
  fn new(config: &RenderConfig) -> Self {
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

    let texture = device.create_texture(&TextureDescriptor {
      label:           Some("Render Texture"),
      size:            config.extent_3d(),
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Rgba8Unorm,
      usage:           wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
      view_formats:    &[],
    });

    GpuHandle { device, queue, texture }
  }
}

impl Target<'_> {
  fn render(&self, handle: &GpuHandle, config: RenderConfig) {
    match self {
      Target::Image(path) => {
        let buffer = handle.device.create_buffer(&wgpu::BufferDescriptor {
          label:              Some("Output Buffer"),
          size:               (4 * config.width * config.height) as u64,
          usage:              wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
          mapped_at_creation: false,
        });

        let mut encoder = handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("texture_buffer_copy_encoder"),
        });

        encoder.copy_texture_to_buffer(
          wgpu::TexelCopyTextureInfo {
            texture:   &handle.texture,
            mip_level: 0,
            origin:    wgpu::Origin3d::ZERO,
            aspect:    wgpu::TextureAspect::All,
          },
          wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
              offset:         0,
              bytes_per_row:  Some(4 * config.width),
              rows_per_image: Some(config.height),
            },
          },
          config.extent_3d(),
        );

        handle.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let buffer = buffer.clone();
        let path = path.to_path_buf();
        buffer_slice.map_async(wgpu::MapMode::Read, move |_| {
          let data = buffer.get_mapped_range(..);

          use image::{ImageBuffer, Rgba};
          let buffer =
            ImageBuffer::<Rgba<u8>, _>::from_raw(config.width, config.height, data).unwrap();
          buffer.save(path).unwrap();
        });
      }
    }
  }
}

impl RenderConfig {
  fn extent_3d(&self) -> wgpu::Extent3d {
    wgpu::Extent3d {
      width:                 self.width,
      height:                self.height,
      depth_or_array_layers: 1,
    }
  }
}
