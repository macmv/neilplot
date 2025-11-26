use vello::wgpu;

use crate::{
  Plot,
  render::{GpuHandle, RenderConfig},
};

pub fn show(plot: &Plot) {
  let event_loop = winit::event_loop::EventLoop::new().unwrap();
  event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

  let mut app = App {
    plot:    unsafe { std::mem::transmute::<&Plot, &Plot>(plot) },
    surface: None,
    handle:  None,
  };
  event_loop.run_app(&mut app).unwrap();

  // FIXME: Ideally, we'd drop this. But dropping it segfaults.
  std::mem::forget(app);
}

struct App {
  // SAFETY: We keep the plot alive for the lifetime of the application.
  plot: &'static Plot<'static>,

  surface: Option<(wgpu::Surface<'static>, wgpu::SurfaceConfiguration)>,
  handle:  Option<GpuHandle>,
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
    let config = RenderConfig { width: 2048, height: 2048 };
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let surface = instance.create_surface(window).unwrap();
    let handle = GpuHandle::new(&config, Some((instance, &surface)));

    let surface_caps = surface.get_capabilities(&handle.adapter);
    let surface_format =
      surface_caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_DST,
      format:                        surface_format,
      width:                         size.width.max(1),
      height:                        size.height.max(1),
      present_mode:                  wgpu::PresentMode::Fifo,
      alpha_mode:                    surface_caps.alpha_modes[0],
      view_formats:                  vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(&handle.device, &config);

    self.surface = Some((surface, config));
    self.handle = Some(handle);
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
            surface.configure(&self.handle.as_ref().unwrap().device, &config);
          }
        }
      }

      winit::event::WindowEvent::RedrawRequested => {
        if let Some((surface, config)) = &self.surface {
          let handle = self.handle.as_ref().unwrap();

          let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
              surface.configure(&handle.device, &config);
              return;
            }
            Err(e) => {
              eprintln!("Dropped frame: {e:?}");
              return;
            }
          };

          let config = RenderConfig { width: config.width, height: config.height };
          self.plot.render(handle, &config);

          let mut encoder = handle.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
          });

          encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
              texture:   &handle.texture,
              mip_level: 0,
              origin:    wgpu::Origin3d::ZERO,
              aspect:    wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
              texture:   &frame.texture,
              mip_level: 0,
              origin:    wgpu::Origin3d::ZERO,
              aspect:    wgpu::TextureAspect::All,
            },
            config.extent_3d(),
          );

          self.handle.as_ref().unwrap().queue.submit(std::iter::once(encoder.finish()));

          frame.present();
        }
      }

      _ => (),
    }
  }
}
