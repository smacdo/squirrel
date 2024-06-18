use anyhow::*;
use image::GenericImageView;

// TODO: Allow customization of texture parameters either with custom types
//       or allow passing in of `wgpu::Texture` / `wgpu::Sampler`.
// TODO: Add ability to change the texture, eg for "reload when changed" functionality.
// TODO: Add ability to recreate the texture view or sampler on demand, eg for
//       "reload when changed" functionality.

/// Stores a WGPU texture along with its associated view and sampler.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Construct a texture represented by `image_bytes` which must be a JPEG,
    /// PNG or DDS image.
    pub fn from_image_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image_bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self> {
        let image = image::load_from_memory(image_bytes)?;
        Self::from_image(device, queue, &image, label)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        image: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = image.to_rgba8();
        let dims = image.dimensions();

        let size = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dims.0),
                rows_per_image: Some(dims.1),
            },
            size,
        );

        // Generate a texture view and sampler for the texture that was just
        // loaded.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
