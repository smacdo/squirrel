use glam::{Mat4, Vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

// TODO: Switch to LH to match DirectX/Metal popular convention.

impl Camera {
    pub fn view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let projection = Mat4::perspective_rh(self.fov_y, self.aspect, self.z_near, self.z_far);
        projection * view
    }
}
