use std::rc::Rc;

use glam::Vec3;

use crate::content::DefaultTextures;

/// A render material that is compatible with the standard lighting shader
/// with phong lighting properties.
///
/// A material can set both a constant color and a texture map for the ambient,
/// diffuse and specular values. When both a constant and a texture map are set
/// the values are multiplied together. The ambient color is ambient color
/// multiplied by the diffuse texture.
#[derive(Clone, Debug)]
pub struct Material {
    pub ambient_color: Vec3,
    pub diffuse_color: Vec3,
    pub diffuse_map: Rc<wgpu::Texture>,
    pub specular_color: Vec3,
    pub specular_map: Rc<wgpu::Texture>,
    pub specular_power: f32,
    pub emissive_map: Rc<wgpu::Texture>,
}

/// A fluent builder for creating Materials without having to specify every
/// optional property.
///
/// Callers can set a constant value for a property type (ambient, diffuse or
/// specular) and or a texture map. If both a constant color and texture map are
/// specified than the shader will multiply the two values together.
#[derive(Debug)]
pub struct MaterialBuilder {
    ambient_color: Option<Vec3>,
    diffuse_color: Option<Vec3>,
    specular_color: Option<Vec3>,
    specular_power: Option<f32>,
    diffuse_map: Option<Rc<wgpu::Texture>>,
    specular_map: Option<Rc<wgpu::Texture>>,
    emissive_map: Option<Rc<wgpu::Texture>>,
}

impl MaterialBuilder {
    pub const DEFAULT_AMBIENT_COLOR: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    pub const DEFAULT_DIFFUSE_COLOR: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    pub const DEFAULT_SPECULAR_COLOR: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const DEFAULT_SPECULAR_POWER: f32 = 0.0;

    /// Create a new material builder.
    pub fn new() -> Self {
        Self {
            ambient_color: None,
            diffuse_color: None,
            specular_color: None,
            specular_power: None,
            diffuse_map: None,
            specular_map: None,
            emissive_map: None,
        }
    }

    /// Set the material's ambient color of the material to a constant value.
    #[allow(dead_code)]
    pub fn ambient_color(mut self, color: Vec3) -> Self {
        self.ambient_color = Some(color);
        self
    }

    /// Set the material's diffuse color to a constant value.
    #[allow(dead_code)]
    pub fn diffuse_color(mut self, color: Vec3) -> Self {
        self.diffuse_color = Some(color);
        self
    }

    /// Set the material's specular color to a constant value.
    pub fn specular_color(mut self, color: Vec3) -> Self {
        self.specular_color = Some(color);
        self
    }

    /// Set the material's specular power.
    pub fn specular_power(mut self, power: f32) -> Self {
        self.specular_power = Some(power);
        self
    }

    /// Set the material's diffuse texture map.
    pub fn diffuse_map(mut self, texture: Rc<wgpu::Texture>) -> Self {
        self.diffuse_map = Some(texture);
        self
    }

    /// Set the material's specular texture map.
    pub fn specular_map(mut self, texture: Rc<wgpu::Texture>) -> Self {
        self.specular_map = Some(texture);
        self
    }

    /// Set the material's emissive texture map.
    #[allow(dead_code)]
    pub fn emissive_map(mut self, texture: Rc<wgpu::Texture>) -> Self {
        self.emissive_map = Some(texture);
        self
    }

    /// Use the properties of this material builder to construct a new material.
    ///
    /// An appropriate default texture from `default_textures` is used when a
    /// texture map is not specified.
    pub fn build(self, default_textures: &DefaultTextures) -> Material {
        Material {
            ambient_color: self.ambient_color.unwrap_or(Self::DEFAULT_AMBIENT_COLOR),
            diffuse_color: self.diffuse_color.unwrap_or(Self::DEFAULT_DIFFUSE_COLOR),
            specular_color: self.specular_color.unwrap_or(Self::DEFAULT_SPECULAR_COLOR),
            specular_power: self.specular_power.unwrap_or(Self::DEFAULT_SPECULAR_POWER),
            diffuse_map: self
                .diffuse_map
                .unwrap_or(default_textures.diffuse_map.clone()),
            specular_map: self
                .specular_map
                .unwrap_or(default_textures.specular_map.clone()),
            emissive_map: self
                .emissive_map
                .unwrap_or(default_textures.emissive_map.clone()),
        }
    }
}
