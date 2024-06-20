use std::rc::Rc;

use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Material {
    pub ambient_color: Vec3,
    pub diffuse_color: Vec3,
    pub diffuse_map: Rc<wgpu::Texture>,
    pub specular_color: Vec3,
    pub specular_map: Rc<wgpu::Texture>,
    pub specular_power: f32,
}

#[derive(Clone, Debug)]
pub struct Light {
    /// The world position of the light.
    pub position: Vec3,
    /// The color of the light.
    pub color: Vec3,
    /// Modifies the amount of color that is applied to the ambient term when
    /// shading.
    pub ambient: f32,
    /// Modifies the amount of white color that is applied to the specular term
    /// when shading.
    pub specular: f32,
}

#[allow(dead_code)]
pub struct LightBuilder {
    light: Light,
}

#[allow(dead_code)]
impl LightBuilder {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        LightBuilder {
            light: Light {
                position,
                color,
                ambient: 0.1,
                specular: 1.0,
            },
        }
    }

    pub fn build(self) -> Light {
        self.light
    }

    pub fn ambient(mut self, amount: f32) -> Self {
        self.light.ambient = amount;
        self
    }

    pub fn specular(mut self, amount: f32) -> Self {
        self.light.specular = amount;
        self
    }
}
