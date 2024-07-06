use std::{path::Path, rc::Rc};

use crate::{
    platform::load_as_binary,
    renderer::{self, shaders, textures},
};

mod obj_model;

// TODO: Add ability to precompile models to a binary format that is loadable here.

pub struct ContentManager {
    default_textures: DefaultTextures,
}

impl ContentManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            default_textures: DefaultTextures::new(device, queue),
        }
    }

    pub async fn load_obj_mesh<P>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &shaders::BindGroupLayouts,
        obj_file_path: P,
    ) -> anyhow::Result<renderer::models::Mesh>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        obj_model::load_obj_mesh(
            device,
            queue,
            layouts,
            &self.default_textures,
            obj_file_path,
        )
        .await
    }
}

#[derive(Debug)]
pub struct DefaultTextures {
    diffuse_map: Rc<wgpu::Texture>,
    specular_map: Rc<wgpu::Texture>,
    emissive_map: Rc<wgpu::Texture>,
}

impl DefaultTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            diffuse_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [255, 255, 255],
                Some("default diffuse texture"),
            )),
            specular_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [0, 0, 0],
                Some("default specular texture"),
            )),
            emissive_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [0, 0, 0],
                Some("default emissive texture"),
            )),
        }
    }
}

#[tracing::instrument(level = "info")]
pub async fn load_texture_file<P>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    file_path: P,
) -> anyhow::Result<wgpu::Texture>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let file_bytes = load_as_binary(file_path.as_ref()).await?;
    renderer::textures::from_image_bytes(
        device,
        queue,
        &file_bytes,
        Some(
            file_path
                .as_ref()
                .to_str()
                .unwrap_or("invalid utf8 chars in texture filename"),
        ),
    )
}
