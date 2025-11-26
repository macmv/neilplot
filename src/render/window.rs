use vello::wgpu;

use crate::{
  Plot,
  render::{GpuHandle, Render, RenderConfig},
};

pub fn show(plot: &Plot) {
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

  let mut app = App { plot, stale: true, render: None, init: None };
  event_loop.run_app(&mut app).unwrap();

  // FIXME: Ideally, we'd drop this. But dropping it segfaults.
  std::mem::forget(app);
}

struct App<'a> {
  plot:   &'a Plot<'a>,
  stale:  bool,
  render: Option<Render>,

  init: Option<Init>,
}

struct Init {
  surface: wgpu::Surface<'static>,
  config:  wgpu::SurfaceConfiguration,
  handle:  GpuHandle,

  blit:  wgpu::util::TextureBlitter,
  vello: vello::Renderer,
}

impl winit::application::ApplicationHandler for App<'_> {
  fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
    if self.init.is_some() {
      return;
    }

    let window = event_loop
      .create_window(
        winit::window::Window::default_attributes()
          .with_min_inner_size(winit::dpi::LogicalSize::new(100, 100)),
      )
      .unwrap();
    let size = window.inner_size();
    let config = RenderConfig { width: 2048, height: 2048 };
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let surface = instance.create_surface(window).unwrap();

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      compatible_surface: Some(&surface),
      ..Default::default()
    }))
    .expect("Failed to create adapter");

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format =
      surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

    let handle = GpuHandle::new(&config, Some(adapter));

    let config = wgpu::SurfaceConfiguration {
      usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_DST,
      format:                        surface_format,
      width:                         size.width.max(1),
      height:                        size.height.max(1),
      present_mode:                  wgpu::PresentMode::AutoNoVsync,
      alpha_mode:                    surface_caps.alpha_modes[0],
      view_formats:                  vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&handle.device, &config);

    let vello = vello::Renderer::new(&handle.device, vello::RendererOptions::default())
      .expect("Failed to create renderer");

    let blit = wgpu::util::TextureBlitter::new(&handle.device, config.format);

    self.init = Some(Init { surface, config, handle, blit, vello });
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
        if let Some(init) = &mut self.init {
          if new_size.width > 0 && new_size.height > 0 {
            init.config.width = new_size.width;
            init.config.height = new_size.height;
            init
              .handle
              .resize(&RenderConfig { width: init.config.width, height: init.config.height });
            init.surface.configure(&init.handle.device, &init.config);

            self.stale = true;
          }
        }
      }

      winit::event::WindowEvent::RedrawRequested => {
        if let Some(init) = &mut self.init {
          if self.render.is_none() || self.stale {
            self.stale = false;
            if self.render.is_none() {
              self.render = Some(Render::new());
            }
            self.render.as_mut().unwrap().scene.reset();
            self.plot.draw(self.render.as_mut().unwrap());

            init
              .vello
              .render_to_texture(
                &init.handle.device,
                &init.handle.queue,
                &self.render.as_ref().unwrap().scene,
                &init.handle.view,
                &vello::RenderParams {
                  base_color:          self.render.as_ref().unwrap().background,
                  width:               init.config.width,
                  height:              init.config.height,
                  antialiasing_method: vello::AaConfig::Msaa16,
                },
              )
              .expect("Failed to render to a texture");
          }
          init.redraw();
        }
      }

      _ => (),
    }
  }
}

impl Init {
  fn redraw(&mut self) {
    let frame = match self.surface.get_current_texture() {
      Ok(frame) => frame,
      Err(wgpu::SurfaceError::Lost) => {
        self.surface.configure(&self.handle.device, &self.config);
        return;
      }
      Err(e) => {
        eprintln!("Dropped frame: {e:?}");
        return;
      }
    };

    let surface_view = &frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .handle
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

    self.blit.copy(&self.handle.device, &mut encoder, &self.handle.view, &surface_view);

    self.handle.queue.submit(std::iter::once(encoder.finish()));

    frame.present();
  }
}
