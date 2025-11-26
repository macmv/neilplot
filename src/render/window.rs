use vello::wgpu;

use crate::render::GpuHandle;

pub fn show(handle: GpuHandle) {
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

  let mut app = App { surface: None, handle };
  event_loop.run_app(&mut app).unwrap();

  // FIXME: Ideally, we'd drop this. But dropping it segfaults.
  std::mem::forget(app);
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
