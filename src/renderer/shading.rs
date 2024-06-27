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
    pub emissive_map: Rc<wgpu::Texture>,
}

/// Point light.
#[derive(Clone, Debug, Default)]
pub struct PointLight {
    /// The world position of the light.
    pub position: Vec3,
    /// The color of the light.
    pub color: Vec3,
    /// Attenuation terms.
    pub attenuation: LightAttenuation,
    /// Modifies the amount of color that is applied to the ambient term when
    /// shading.
    pub ambient: f32,
    /// Modifies the amount of white color that is applied to the specular term
    /// when shading.
    pub specular: f32,
}

#[derive(Clone, Debug, Default)]
pub struct LightAttenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

/// Directional light.
#[derive(Clone, Debug, Default)]
pub struct DirectionalLight {
    /// The direction of the light pointing _away_ from the light source.
    pub direction: Vec3,
    /// The color of the light.
    pub color: Vec3,
    /// Modifies the amount of color that is applied to the ambient term when
    /// shading.
    pub ambient: f32,
    /// Modifies the amount of white color that is applied to the specular term
    /// when shading.
    pub specular: f32,
}

/// A spot light.
#[derive(Clone, Debug, Default)]
pub struct SpotLight {
    /// The world position of the light.
    pub position: Vec3,
    /// The direction of the light pointing _away_ from the light source.
    pub direction: Vec3,
    /// Cut off angle in radians.
    pub cutoff_radians: f32,
    /// Outer cut off angle in radians.
    pub outer_cutoff_radians: f32,
    /// The color of the light.
    pub color: Vec3,
    /// Attenuation terms.
    pub attenuation: LightAttenuation,
    /// Modifies the amount of color that is applied to the ambient term when
    /// shading.
    pub ambient: f32,
    /// Modifies the amount of white color that is applied to the specular term
    /// when shading.
    pub specular: f32,
}
