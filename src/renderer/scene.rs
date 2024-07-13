use super::{
    lighting::{DirectionalLight, PointLight, SpotLight},
    models::Model,
};

/// A set of models and associated properties that can be drawn with the
/// renderer.
///
/// A `Scene` is not a scene graph!
#[derive(Default)]
pub struct Scene {
    pub point_lights: Vec<PointLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub spot_lights: Vec<SpotLight>,
    pub models: Vec<Model>,
}
