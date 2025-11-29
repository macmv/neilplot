use std::path::Path;

use kurbo::{Affine, Point, Rect, Shape, Size, Stroke};
use parley::{Alignment, FontWeight, Layout, PositionedLayoutItem, StyleProperty};
use peniko::{Brush, BrushRef, Color, Fill};
use vello::{
  Renderer,
  wgpu::{self, TextureDescriptor},
};

use crate::Plot;

mod texture;
mod window;

pub(crate) struct Render {
  scene: vello::Scene,

  font:   parley::FontContext,
  layout: parley::LayoutContext<Brush>,

  config:     RenderConfig,
  transform:  Affine,
  background: Color,
}

struct GpuHandle {
  device:  wgpu::Device,
  queue:   wgpu::Queue,
  texture: wgpu::Texture,
}

#[derive(Clone, Copy)]
struct RenderConfig {
  width:  u32,
  height: u32,
}

pub struct DrawText<'a> {
  pub text:             &'a str,
  pub size:             f32,
  pub weight:           FontWeight,
  pub brush:            Brush,
  pub position:         Point,
  pub transform:        Affine,
  pub horizontal_align: Align,
  pub vertical_align:   Align,
}

#[derive(Debug, Clone, Copy)]
pub enum Align {
  Start,
  Center,
  End,
}

impl Default for DrawText<'_> {
  fn default() -> Self {
    DrawText {
      text:             "",
      size:             12.0,
      weight:           FontWeight::NORMAL,
      brush:            Color::BLACK.into(),
      position:         Point::ORIGIN,
      transform:        Affine::IDENTITY,
      horizontal_align: Align::Start,
      vertical_align:   Align::Start,
    }
  }
}

impl Plot<'_> {
  pub fn save(&self, path: impl AsRef<Path>) {
    let config = RenderConfig { width: 2048, height: 2048 };
    let handle = GpuHandle::new(&config);
    self.render(&handle, config);
    texture::save(handle, config, path.as_ref());
  }

  pub fn show(&self) { window::show(self); }

  fn render(&self, handle: &GpuHandle, config: RenderConfig) {
    let mut render = Render::new();
    render.resize(config);
    self.draw(&mut render);

    let view = &handle.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut renderer = Renderer::new(&handle.device, vello::RendererOptions::default())
      .expect("Failed to create renderer");

    renderer
      .render_to_texture(
        &handle.device,
        &handle.queue,
        &render.scene,
        &view,
        &vello::RenderParams {
          base_color:          render.background,
          width:               config.width,
          height:              config.height,
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .expect("Failed to render to a texture");
  }
}

impl Render {
  fn new() -> Self {
    Render {
      scene:      vello::Scene::new(),
      font:       parley::FontContext::new(),
      layout:     parley::LayoutContext::new(),
      config:     RenderConfig { width: 1000, height: 1000 },
      transform:  Affine::IDENTITY,
      background: Color::WHITE,
    }
  }

  pub fn size(&self) -> Size {
    if self.config.width < self.config.height {
      Size::new(1000.0, 1000.0 * f64::from(self.config.height) / f64::from(self.config.width))
    } else {
      Size::new(1000.0 * f64::from(self.config.width) / f64::from(self.config.height), 1000.0)
    }
  }

  pub fn stroke<'a>(
    &mut self,
    shape: &impl Shape,
    transform: Affine,
    brush: impl Into<BrushRef<'a>>,
    stroke: &Stroke,
  ) {
    self.scene.stroke(stroke, self.transform * transform, brush, None, shape);
  }

  pub fn fill<'a>(
    &mut self,
    shape: &impl Shape,
    transform: Affine,
    brush: impl Into<BrushRef<'a>>,
  ) {
    self.scene.fill(Fill::NonZero, self.transform * transform, brush, None, shape);
  }

  pub fn draw_text(&mut self, text: DrawText<'_>) {
    let layout = self.layout_text(&text);
    self.draw_text_layout(layout, text);
  }

  pub fn layout_text(&mut self, text: &DrawText<'_>) -> Layout<Brush> {
    let mut builder = self.layout.ranged_builder(&mut self.font, text.text, 1.0, false);

    builder.push_default(StyleProperty::FontSize(text.size));
    builder.push_default(StyleProperty::FontWeight(text.weight));
    builder.push_default(StyleProperty::Brush(text.brush.clone()));

    let mut layout = builder.build(text.text);

    layout.break_all_lines(None);
    layout.align(None, Alignment::Start, Default::default());

    layout
  }

  pub fn draw_text_layout(&mut self, layout: Layout<Brush>, text: DrawText<'_>) {
    let size = Size::new(f64::from(layout.width()), f64::from(layout.height()));
    let mut rect = Rect::from_origin_size(
      Point {
        x: match text.horizontal_align {
          Align::Start => 0.0,
          Align::Center => -size.width / 2.0,
          Align::End => -size.width,
        },
        y: match text.vertical_align {
          Align::Start => 0.0,
          Align::Center => -size.height / 2.0,
          Align::End => -size.height,
        },
      },
      size,
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
          .transform(self.transform * text.transform.then_translate(text.position.to_vec2()))
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

  fn resize(&mut self, config: RenderConfig) {
    self.config = config;
    if config.width < config.height {
      self.transform = Affine::scale(f64::from(config.width) / 1000.0);
    } else {
      self.transform = Affine::scale(f64::from(config.height) / 1000.0);
    }
  }
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

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
      format:          FORMAT,
      usage:           wgpu::TextureUsages::STORAGE_BINDING
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats:    &[],
    });

    GpuHandle { device, queue, texture }
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
