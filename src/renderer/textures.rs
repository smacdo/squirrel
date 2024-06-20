use anyhow::*;
use image::{GenericImageView, Rgba, RgbaImage};

// TODO: Allow customization of texture parameters.
// TODO: Create a high level sharable texture type that can be updated at runtime
//       (`prepare(device, queue)`) to allow for reload when changed functionality.

/// Creates a new 1x1 texture with the given pixel color. `pixel` is an RGB
/// triplet with 0 being none, and 255 being maximum.
///
/// [255, 255, 255] for white.
#[allow(dead_code)]
pub fn new_1x1(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pixel: [u8; 3],
    label: Option<&str>,
) -> wgpu::Texture {
    let mut image = RgbaImage::new(1, 1);
    image.put_pixel(0, 0, Rgba([pixel[0], pixel[1], pixel[2], 255]));
    from_image(device, queue, image.into(), label)
}

/// Construct a texture represented by `image_bytes` which must be a JPEG, PNG
/// or DDS image.
#[allow(dead_code)]
pub fn from_image_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image_bytes: &[u8],
    label: Option<&str>,
) -> Result<wgpu::Texture> {
    let image = image::load_from_memory(image_bytes)?;
    Ok(from_image(device, queue, image, label))
}

/// Create a wgpu texture object from a `DynamicImage`.`
///
/// To get a texture view from the wgpu texture object use the following code:
/// `texture.create_view(&wgpu::TextureViewDescriptor::default())`
pub fn from_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    image: image::DynamicImage,
    label: Option<&str>,
) -> wgpu::Texture {
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

    texture
}

/// Create a default texture sampler with sane defaults.
pub fn create_default_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
