#![allow(dead_code)]

use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

use crate::{
    platform::load_as_binary,
    renderer::{
        self, shaders,
        textures::{self, ColorSpace},
    },
};

mod obj_model;

// TODO: Implement basic content loader with caching support.
// TODO: Add ability to precompile models to a binary format that is loadable here.

pub struct ContentManager {
    default_textures: DefaultTextures,
    _loaded_textures: RefCell<HashMap<String, Rc<wgpu::Texture>>>,
}

impl ContentManager {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            default_textures: DefaultTextures::new(device, queue),
            _loaded_textures: RefCell::new(HashMap::new()),
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

    // TODO: Implement cached texture loading.
    /*
    pub async fn load_texture<P>(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        file_path: P,
    ) -> anyhow::Result<Rc<wgpu::Texture>>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        // Resolve the texture file path to an unambiguous absolute file path
        // and use this value as the shared key.
        let file_path = std::fs::canonicalize(file_path.as_ref())?;
        let cache_key = file_path.to_string_lossy();

        // Return a copy of the already loaded texture if it exists in the
        // texture cache.
        if let Some(texture) = self.loaded_textures.borrow().get(cache_key.as_ref()) {
            return Ok(texture.clone());
        }

        // The texture was not already in the cache. Load it from disk and add
        // it to the cache before returning the texture to the caller.
        Ok({
            let cache_key = cache_key.into_owned();
            let texture = Rc::new(load_texture_file(device, queue, file_path).await?);

            self.loaded_textures
                .borrow_mut()
                .insert(cache_key, texture.clone());

            texture
        })
    }
    */
}

#[derive(Debug)]
pub struct DefaultTextures {
    pub diffuse_map: Rc<wgpu::Texture>,
    pub specular_map: Rc<wgpu::Texture>,
    pub emissive_map: Rc<wgpu::Texture>,
}

impl DefaultTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        Self {
            diffuse_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [255, 255, 255],
                textures::ColorSpace::Linear,
                Some("default diffuse texture"),
            )),
            specular_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [0, 0, 0],
                textures::ColorSpace::Linear,
                Some("default specular texture"),
            )),
            emissive_map: Rc::new(textures::new_1x1(
                device,
                queue,
                [0, 0, 0],
                textures::ColorSpace::Linear,
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
    color_space: ColorSpace,
) -> anyhow::Result<wgpu::Texture>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let file_bytes = load_as_binary(file_path.as_ref()).await?;
    renderer::textures::from_image_bytes(
        device,
        queue,
        &file_bytes,
        color_space,
        Some(
            file_path
                .as_ref()
                .to_str()
                .unwrap_or("invalid utf8 chars in texture filename"),
        ),
    )
}
