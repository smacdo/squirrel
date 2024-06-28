//! Rust structs with memory layouts that match their same named counterparts
//! in shader code.
//!
//! To minimize the amount of data that has be transfered every frame between
//! the CPU and GPU the structs in this module are called "packed". This means
//! data is packed as tightly as possible, and any gaps between Vec1/2/3 fields
//! are exploited by adding extra data in.
//!
//! For example the packed lighting structs usually encode the light color as
//! follows:
//!
//!   light.color.x = R
//!   light.color.y = G
//!   light.colot.x = B
//!   light.color.w = ambient_modifier
//!
//! These structs must exactly match the memory layout whenever their
//! representation is changed in shader code or vice versa. In particular all
//! fields must be aligned to a 16 byte (eg `Vec4`) padding as this is a WebGPU
//! requirement.
use glam::{Vec3, Vec4};

use crate::renderer::shading::{DirectionalLight, Material, PointLight, SpotLight};

/// Rust struct with the same memory layout as the `PackedMaterialConstants`
/// used by the lighting shaders.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedMaterialConstants {
    pub ambient_color: Vec4,  // .w is unused.
    pub diffuse_color: Vec4,  // .w is unused.
    pub specular_color: Vec4, // .w is specular power.
}

impl From<Material> for PackedMaterialConstants {
    fn from(val: Material) -> Self {
        Self {
            ambient_color: vec3_w(val.ambient_color, 0.0),
            diffuse_color: vec3_w(val.diffuse_color, 0.0),
            specular_color: vec3_w(val.specular_color, val.specular_power),
        }
    }
}

/// Rust struct with the same memory layout as the `PackedDirectionLight` used
/// by the lighting shaders.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedDirectionalLight {
    pub direction: Vec4, // directional light, .xyz is normalized, .w is ambient amount.
    pub color: Vec4,     // directional light, .w is specular amount.
}

impl From<DirectionalLight> for PackedDirectionalLight {
    fn from(val: DirectionalLight) -> Self {
        Self {
            direction: vec3_w(val.direction.normalize(), val.ambient),
            color: vec3_w(val.color, val.specular),
        }
    }
}

/// Rust struct with the same memory layout as the `PackedPointLight` used
/// by the lighting shaders.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedPointLight {
    pub position: Vec4,    // .w is ambient amount.
    pub color: Vec4,       // .w is specular amount.
    pub attenuation: Vec4, // xyzw: (constant, linear, quadratic, unused).
    pub padding: Vec4,
}

impl From<PointLight> for PackedPointLight {
    fn from(val: PointLight) -> Self {
        Self {
            position: vec3_w(val.position, val.ambient),
            color: vec3_w(val.color, val.specular),
            attenuation: Vec4::new(
                val.attenuation.constant,
                val.attenuation.linear,
                val.attenuation.quadratic,
                0.0,
            ),
            padding: Vec4::ZERO,
        }
    }
}

/// Rust struct with the same memory layout as the `PackedSpotLight` used
/// by the lighting shaders.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PackedSpotLight {
    pub position: Vec4,    // .w is the precomputed cutoff angle.
    pub direction: Vec4,   // .w is ambient amount.
    pub color: Vec4,       // .w is specular amount.
    pub attenuation: Vec4, // .w is the outer precomputed cutoff angle.
}

impl From<SpotLight> for PackedSpotLight {
    fn from(val: SpotLight) -> Self {
        Self {
            position: vec3_w(val.position, f32::cos(val.cutoff_radians)),
            direction: vec3_w(val.direction.normalize(), val.ambient),
            color: vec3_w(val.color, val.specular),
            attenuation: Vec4::new(
                val.attenuation.constant,
                val.attenuation.linear,
                val.attenuation.quadratic,
                f32::cos(val.outer_cutoff_radians),
            ),
        }
    }
}

/// Returns a new `Vec4` value that is the combination of a `Vec3` x, y and z
/// and an addiitonal `w` value.
pub fn vec3_w(xyz: Vec3, w: f32) -> Vec4 {
    Vec4::new(xyz.x, xyz.y, xyz.z, w)
}
