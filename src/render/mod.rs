use std::path::Path;

use parley::{Alignment, FontWeight, PositionedLayoutItem, StyleProperty};
use vello::{
  Renderer,
  kurbo::{Affine, Circle, Point, Rect},
  peniko::{Brush, Color, Fill, color::palette},
  wgpu::{self, TextureDescriptor},
};

use crate::Plot;

struct Render {
  scene:  vello::Scene,
  font:   parley::FontContext,
  layout: parley::LayoutContext<Brush>,
}

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

    let mut render = Render::new();
    render.foo();

    render.scene.fill(
      vello::peniko::Fill::NonZero,
      Affine::IDENTITY,
      Color::from_rgb8(242, 140, 168),
      None,
      &Circle::new((420.0, 200.0), 120.0),
    );

    let view = &handle.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Initialize wgpu and get handles
    let mut renderer = Renderer::new(&handle.device, vello::RendererOptions::default())
      .expect("Failed to create renderer");

    renderer
      .render_to_texture(
        &handle.device,
        &handle.queue,
        &render.scene,
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

impl Render {
  fn new() -> Self {
    Render {
      scene:  vello::Scene::new(),
      font:   parley::FontContext::new(),
      layout: parley::LayoutContext::new(),
    }
  }

  fn foo(&mut self) {
    const DISPLAY_SCALE: f32 = 5.0;
    const TEXT: &str = "Lorem Ipsum...";
    let mut builder = self.layout.ranged_builder(&mut self.font, &TEXT, DISPLAY_SCALE, true);

    builder.push_default(StyleProperty::FontSize(32.0));
    builder.push_default(StyleProperty::Brush(Brush::Solid(Color::WHITE)));
    builder.push(StyleProperty::FontWeight(FontWeight::new(600.0)), 0..4);

    let mut layout = builder.build(&TEXT);

    const MAX_WIDTH: Option<f32> = Some(100.0);
    layout.break_all_lines(MAX_WIDTH);
    layout.align(MAX_WIDTH, Alignment::Start, Default::default());

    let origin = Point::new(50.0, 50.0);

    let mut rect = Rect::new(
      origin.x,
      origin.y,
      origin.x + f64::from(layout.width()),
      origin.y + f64::from(layout.height()),
    );

    for line in layout.lines() {
      for item in line.items() {
        let PositionedLayoutItem::GlyphRun(glyph_run) = item else { continue };

        let run = glyph_run.run();
        rect.y0 = rect.y1.round() - rect.height();
        let mut x = rect.x0 as f32 + glyph_run.offset();
        let baseline = (rect.y0 as f32 + glyph_run.baseline()).round();

        self
          .scene
          .draw_glyphs(run.font())
          .brush(&glyph_run.style().brush)
          .hint(true)
          .transform(Affine::IDENTITY)
          .glyph_transform(
            run.synthesis().skew().map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0)),
          )
          .font_size(run.font_size())
          .normalized_coords(run.normalized_coords())
          .draw(
            Fill::NonZero,
            glyph_run.glyphs().map(|glyph| {
              let gx = x + glyph.x;
              let gy = baseline + glyph.y;
              x += glyph.advance;
              vello::Glyph { id: glyph.id.into(), x: gx, y: gy }
            }),
          );
      }
    }
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
