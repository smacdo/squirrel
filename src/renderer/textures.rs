use anyhow::*;
use image::GenericImageView;
use wgpu::naga::back::msl::sampler;

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
    pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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

    /// Create a texture representing the depth buffer.
    pub fn create_depth_texture(
        device: &wgpu::Device,
        surface: &wgpu::SurfaceConfiguration,
        label: Option<&str>,
    ) -> Self {
        // Create the GPU backing texture for the depth buffer. Including
        // `TextureUsages::RENDER_ATTACHMENT` in the usage flags ensures depth
        // information can be written to this texture.
        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: surface.width,
                height: surface.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // The sampler for this depth texture is not needed at this point but
        // future me will probably want to sample from it so lets just go ahead
        // and create it right now.
        //
        // Setting `compare` to `wgpu::CompareFunction::LessEqual` ensures that
        // pixels are drawn front to back. This is important for debugging as it
        // correctly renders the depth buffer for visualization.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture: gpu_texture,
            view,
            sampler,
        }
    }
}
