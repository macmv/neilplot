use std::path::Path;

use kurbo::{Affine, Point, Rect, Shape, Size, Stroke};
use parley::{Alignment, FontWeight, PositionedLayoutItem, StyleProperty};
use peniko::{Brush, BrushRef, Color, Fill};
use vello::{
  Renderer,
  wgpu::{self, TextureDescriptor},
};

use crate::Plot;

pub(crate) struct Render {
  scene: vello::Scene,

  font:   parley::FontContext,
  layout: parley::LayoutContext<Brush>,

  transform:  Affine,
  background: Color,
}

struct GpuHandle {
  instance: wgpu::Instance,
  adapter:  wgpu::Adapter,
  device:   wgpu::Device,
  queue:    wgpu::Queue,
  texture:  wgpu::Texture,
}

struct RenderConfig {
  width:  u32,
  height: u32,
}

enum Target<'a> {
  Image(&'a Path),
  Screen,
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
  pub fn save(&self, path: impl AsRef<Path>) { self.render(Target::Image(path.as_ref())); }
  pub fn show(&self) { self.render(Target::Screen); }

  fn render(&self, target: Target<'_>) {
    let config = RenderConfig { width: 2048, height: 2048 };
    let handle = GpuHandle::new(&config, None);

    let mut render = Render::new();
    // Everything uses a 1000x1000 coordinate system.
    render.transform = Affine::scale_non_uniform(
      f64::from(config.width) / 1000.0,
      f64::from(config.height) / 1000.0,
    );
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

    target.render(handle, config);
  }
}

impl Render {
  fn new() -> Self {
    Render {
      scene:      vello::Scene::new(),
      font:       parley::FontContext::new(),
      layout:     parley::LayoutContext::new(),
      transform:  Affine::IDENTITY,
      background: Color::WHITE,
    }
  }

  pub fn stroke<'a>(
    &mut self,
    shape: &impl Shape,
    brush: impl Into<BrushRef<'a>>,
    stroke: &Stroke,
  ) {
    self.scene.stroke(stroke, self.transform, brush, None, shape);
  }

  pub fn fill<'a>(&mut self, shape: &impl Shape, brush: impl Into<BrushRef<'a>>) {
    self.scene.fill(Fill::NonZero, self.transform, brush, None, shape);
  }

  pub fn draw_text(&mut self, text: DrawText<'_>) {
    let mut builder = self.layout.ranged_builder(&mut self.font, text.text, 1.0, false);

    builder.push_default(StyleProperty::FontSize(text.size));
    builder.push_default(StyleProperty::FontWeight(text.weight));
    builder.push_default(StyleProperty::Brush(text.brush));

    let mut layout = builder.build(text.text);

    layout.break_all_lines(None);
    layout.align(None, Alignment::Start, Default::default());

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
}

impl GpuHandle {
  fn new(config: &RenderConfig, surface: Option<&wgpu::Surface>) -> Self {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      compatible_surface: surface,
      ..Default::default()
    }))
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

    GpuHandle { instance, adapter, device, queue, texture }
  }
}

impl Target<'_> {
  fn render(&self, handle: GpuHandle, config: RenderConfig) {
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

      Target::Screen => {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

        let mut app = App { surface: None, handle };
        event_loop.run_app(&mut app).unwrap();

        // FIXME: Ideally, we'd drop this. But dropping it segfaults.
        std::mem::forget(app);
      }
    }
  }
}

struct App {
  surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
  handle:  GpuHandle,
}

impl winit::application::ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    let window = event_loop
      .create_window(
        winit::window::Window::default_attributes()
          .with_min_inner_size(winit::dpi::LogicalSize::new(100, 100)),
      )
      .unwrap();
    let size = window.inner_size();
    let surface = self.handle.instance.create_surface(window).unwrap();

    let surface_caps = surface.get_capabilities(&self.handle.adapter);
    let surface_format =
      surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT,
      format:                        surface_format,
      width:                         size.width.max(1),
      height:                        size.height.max(1),
      present_mode:                  wgpu::PresentMode::Fifo,
      alpha_mode:                    surface_caps.alpha_modes[0],
      view_formats:                  vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&self.handle.device, &config);

    self.surface = Some((surface, config));
  }

  fn window_event(
    &mut self,
    event_loop: &winit::event_loop::ActiveEventLoop,
    _: winit::window::WindowId,
    event: winit::event::WindowEvent,
  ) {
    match event {
      winit::event::WindowEvent::CloseRequested => {
        event_loop.exit();
      }

      winit::event::WindowEvent::KeyboardInput {
        event: winit::event::KeyEvent { logical_key: winit::keyboard::Key::Character(c), .. },
        ..
      } if c == "q" => {
        event_loop.exit();
      }

      winit::event::WindowEvent::Resized(new_size) => {
        if let Some((surface, config)) = &mut self.surface {
          if new_size.width > 0 && new_size.height > 0 {
            config.width = new_size.width;
            config.height = new_size.height;
            surface.configure(&self.handle.device, &config);
          }
        }
      }

      winit::event::WindowEvent::RedrawRequested => {
        if let Some((surface, config)) = &self.surface {
          let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
              surface.configure(&self.handle.device, &config);
              return;
            }
            Err(e) => {
              eprintln!("Dropped frame: {e:?}");
              return;
            }
          };

          let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

          let mut encoder =
            self.handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
              label: Some("Render Encoder"),
            });

          {
            // Clear to a dark gray
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
              label:                    Some("Render Pass"),
              color_attachments:        &[Some(wgpu::RenderPassColorAttachment {
                view:           &view,
                resolve_target: None,
                ops:            wgpu::Operations {
                  load:  wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }),
                  store: wgpu::StoreOp::Discard,
                },
                depth_slice:    None,
              })],
              depth_stencil_attachment: None,
              occlusion_query_set:      None,
              timestamp_writes:         None,
            });
          }

          self.handle.queue.submit(std::iter::once(encoder.finish()));

          frame.present();
        }
      }

      _ => (),
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
