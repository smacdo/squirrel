use glam::{Mat4, Vec3};
use thiserror::Error;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// The ratio of the viewport width to its height. An example is if the view
    /// is one unit high and two units wide then the aspect ratio is 2/1.
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

// TODO: Switch to LH to match DirectX/Metal popular convention.

impl Camera {
    /// Create a new camera centered at `eye` with the center of the view
    /// aiming at `target` with `up` as the camera's upward direction.
    ///
    /// The aspect ratio is set to zero if either the viewport width or height
    /// is zero.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        fov_y: f32,
        z_near: f32,
        z_far: f32,
        view_width: u32,
        view_height: u32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect: if view_width > 0 && view_height > 0 {
                view_width as f32 / view_height as f32
            } else {
                0.0
            },
            fov_y,
            z_near,
            z_far,
        }
    }

    /// Get the camera's 4x4 view projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let projection = Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far);
        projection * view
    }

    /// Resize the camera's viewport size.
    pub fn set_viewport_size(
        &mut self,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), InvalidCameraSize> {
        if new_width > 0 && new_height > 0 {
            self.aspect = new_width as f32 / new_height as f32;
            Ok(())
        } else {
            Err(InvalidCameraSize(new_width, new_height))
        }
    }
}

#[derive(Debug, Error)]
#[error("camera viewport width and height must be larger than zero but width was {} and height was {}", .0, .1)]
pub struct InvalidCameraSize(u32, u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_valid_viewport_size() {
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(45.0),
            0.1,
            100.0,
            100,
            200,
        );

        assert_eq!(0.5, camera.aspect);

        assert!(camera.set_viewport_size(600, 300).is_ok());
        assert_eq!(2.0, camera.aspect);
    }

    #[test]
    fn set_invalid_viewport_size() {
        let mut camera = Camera::new(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(45.0),
            0.1,
            100.0,
            100,
            200,
        );

        assert!(camera.set_viewport_size(0, 100).is_err());

        let err = camera.set_viewport_size(0, 100).unwrap_err();
        assert_eq!(0, err.0);
        assert_eq!(100, err.1);

        assert!(camera.set_viewport_size(600, 0).is_err());

        let err = camera.set_viewport_size(600, 0).unwrap_err();
        assert_eq!(600, err.0);
        assert_eq!(0, err.1);

        assert!(camera.set_viewport_size(0, 0).is_err());

        let err = camera.set_viewport_size(0, 0).unwrap_err();
        assert_eq!(0, err.0);
        assert_eq!(0, err.1);
    }
}
