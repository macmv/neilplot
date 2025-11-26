use std::path::Path;

use vello::wgpu;

use crate::render::{GpuHandle, RenderConfig};

pub fn render(handle: GpuHandle, config: RenderConfig, path: &Path) {
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
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(config.width, config.height, data).unwrap();
    buffer.save(path).unwrap();
  });
}
