use vello::{
  util::{DeviceHandle, RenderContext, RenderSurface},
  wgpu,
};

use crate::{
  Plot,
  render::{Render, RenderConfig},
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
  cx:      RenderContext,
  surface: RenderSurface<'static>,
  dev_id:  usize,

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

    let mut cx = RenderContext::new();
    let surface = pollster::block_on(cx.create_surface(
      window,
      size.width,
      size.height,
      wgpu::PresentMode::AutoNoVsync,
    ))
    .unwrap();

    let dev_id = pollster::block_on(cx.device(Some(&surface.surface))).unwrap();

    let vello = vello::Renderer::new(&cx.devices[dev_id].device, vello::RendererOptions::default())
      .expect("Failed to create renderer");

    self.init = Some(Init { cx, surface, dev_id, vello });
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
            init.cx.resize_surface(&mut init.surface, new_size.width, new_size.height);

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
            self.render.as_mut().unwrap().resize(RenderConfig {
              width:  init.surface.config.width,
              height: init.surface.config.height,
            });
            self.plot.draw(self.render.as_mut().unwrap());

            let handle = &init.cx.devices[init.dev_id];
            init
              .vello
              .render_to_texture(
                &handle.device,
                &handle.queue,
                &self.render.as_ref().unwrap().scene,
                &init.surface.target_view,
                &vello::RenderParams {
                  base_color:          self.render.as_ref().unwrap().background,
                  width:               init.surface.config.width,
                  height:              init.surface.config.height,
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
  fn device_handle(&self) -> &DeviceHandle { &self.cx.devices[self.dev_id] }
  fn device(&self) -> &wgpu::Device { &self.device_handle().device }

  fn redraw(&mut self) {
    let frame = match self.surface.surface.get_current_texture() {
      Ok(frame) => frame,
      Err(wgpu::SurfaceError::Lost) => {
        // self.surface.configure(&self.handle.device, &self.config);
        return;
      }
      Err(e) => {
        eprintln!("Dropped frame: {e:?}");
        return;
      }
    };

    let surface_view = &frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .device()
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

    self.surface.blitter.copy(
      self.device(),
      &mut encoder,
      &self.surface.target_view,
      &surface_view,
    );

    self.device_handle().queue.submit(std::iter::once(encoder.finish()));

    frame.present();
  }
}
