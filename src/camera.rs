use glam::{Mat4, Vec3};
use thiserror::Error;

/// Camera assumes a right-handed system with the +Z axis going _out_ of the
/// screen rather than in. This is an arbitrary choice and I decided to use RH
/// because the abundance of OpenGL tutorials which typically assume RH over LH.
///
/// Positive rotations in a right handed system are counterclockwise around the
/// axis of rotation.
///
/// The following transforms points from local space to clip space:
///  `V_clip = M_projection * M_view * M_model * M_local`
///
/// WebGPU defines clip space to be a unit cube with values with the front bottom
/// left corner as (-1, -1, -1) and the back top right corner (1, 1, 1).
/// +X faces right, +Y is up and +Z is into the screen.
pub struct Camera {
    /// The position of the camera in world space.
    eye: Vec3,
    /// The target position the camera should look at.
    target: Vec3,
    /// The camera's up direction.
    up: Vec3,
    /// A world space direction vector indicating which direction is considered
    /// straight up.
    world_up: Vec3,
    /// The ratio of the viewport width to its height. An example is if the view
    /// is one unit high and two units wide then the aspect ratio is 2/1.
    aspect: f32,
    /// The vertical field of view for the camera.
    fov_y: f32,
    /// The minimum camera view distance. Fragments closer than `z_near` will not
    /// be rendered.
    z_near: f32,
    /// The maximum camera view distance. Fragments further than `z_far` will not
    /// be rendered.
    z_far: f32,
    viewport_width: f32,
    viewport_height: f32,
}

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
        viewport_width: u32,
        viewport_height: u32,
    ) -> Self {
        assert!(fov_y > 0.0);
        assert!(z_near >= 0.0);
        assert!(z_far > z_near);
        assert!(eye != target);

        let up = up.normalize();

        Self {
            eye,
            target,
            up,
            world_up: up,
            aspect: if viewport_width > 0 && viewport_height > 0 {
                viewport_width as f32 / viewport_height as f32
            } else {
                0.0
            },
            fov_y,
            z_near,
            z_far,
            viewport_width: viewport_width as f32,
            viewport_height: viewport_height as f32,
        }
    }

    /// Reorient the camera to be located at `eye` and look at `target`. Both
    /// points are should be in world space.
    ///
    /// Calling `reorient` will rebuild the camera's local coordinate system
    /// using the Gram-Schimdt process.
    pub fn reorient(&mut self, new_eye: Vec3, new_target: Vec3) {
        self.eye = new_eye;
        self.target = new_target;

        // NOTE: This direction is technically the _opposite_ of the camera's
        // facing direction (it goes from target to eye rather than eye to target).
        //
        // This is because the view matrix's coordinate system Z axis is positive
        // but by OpenGL convention the camera points towards the negative Z
        // axis.
        let new_direction = (self.eye - self.target).normalize();
        let new_right = Vec3::cross(self.world_up, new_direction).normalize();
        let new_up = Vec3::cross(new_direction, new_right);

        self.up = new_up;
        // TODO: store the right vector?
    }

    /// Get the camera's view matrix.
    ///
    /// A view matrix transforms coordinates from world space to view space.
    /// View space is a coordinate space that can be imagined as the user's view
    /// into the scene, with the user's eye located at (0, 0, 0) and looking
    /// down the -Z axis.
    ///
    /// View matrices can also be thought of the inverse of the camera's world
    /// space transform. For instance if a camera is moved backwards 3 units
    /// then it is the same as moving the scene forward 3 units!
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    /// Get the camera's projection matrix.
    ///
    /// A projection matrix transforms coordinates from view space to clip space.
    /// This camera applies a perspective projection to make objects farther from
    /// the camera appear smaller. Any fragment outside of the viewing frustum
    /// will not be rendered to the screen.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far)
    }

    /// Get the camera's view projection matrix. The view projection matrix will
    /// transform points from world space to clip space.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Resize the camera's viewport.
    pub fn set_viewport_size(
        &mut self,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), InvalidCameraSize> {
        if new_width > 0 && new_height > 0 {
            self.aspect = new_width as f32 / new_height as f32;
            self.viewport_width = new_width as f32;
            self.viewport_height = new_height as f32;
            Ok(())
        } else {
            Err(InvalidCameraSize(new_width, new_height))
        }
    }

    /// Get the position of the camera in world space.
    pub fn eye(&self) -> Vec3 {
        self.eye
    }

    /// Get the point at which the camera is focused on.
    pub fn target(&self) -> Vec3 {
        self.target
    }

    /// Get the camera's up axis.
    pub fn up(&self) -> Vec3 {
        self.up
    }

    /// Get the camera viewport width in pixels.
    pub fn viewport_width(&self) -> f32 {
        self.viewport_width
    }

    /// Get the camera viewport height in pixels.
    pub fn viewport_height(&self) -> f32 {
        self.viewport_height
    }

    /// Get the world up axis (not the camera's up axis).
    pub fn world_up(&self) -> Vec3 {
        self.world_up
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
