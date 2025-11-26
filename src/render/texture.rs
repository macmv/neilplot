use std::path::Path;

use vello::wgpu::{self, TextureDescriptor};

use crate::render::{GpuHandle, RenderConfig};

pub fn save(handle: GpuHandle, config: RenderConfig, path: &Path) {
  const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

  // Copy the texture to a new texture in SRGB.
  let output_texture = handle.device.create_texture(&TextureDescriptor {
    label:           Some("Render Texture"),
    size:            config.extent_3d(),
    mip_level_count: 1,
    sample_count:    1,
    dimension:       wgpu::TextureDimension::D2,
    format:          OUTPUT_FORMAT,
    usage:           wgpu::TextureUsages::COPY_SRC
      | wgpu::TextureUsages::TEXTURE_BINDING
      | wgpu::TextureUsages::RENDER_ATTACHMENT,
    view_formats:    &[],
  });
  let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

  let buffer = handle.device.create_buffer(&wgpu::BufferDescriptor {
    label:              Some("Output Buffer"),
    size:               (4 * config.width * config.height) as u64,
    usage:              wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  let blit = wgpu::util::TextureBlitter::new(&handle.device, OUTPUT_FORMAT);
  let mut encoder = handle
    .device
    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Output Encoder") });

  blit.copy(&handle.device, &mut encoder, &handle.view, &output_view);
  encoder.copy_texture_to_buffer(
    wgpu::TexelCopyTextureInfo {
      texture:   &output_texture,
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
