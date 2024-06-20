use glam::Vec2;

/// Calculates the (x, y) position that results from orbiting around `pivot` at
/// a distance of `radius`.
pub fn rotate_around_pivot(pivot: Vec2, radius: f32, angle_radian: f32) -> Vec2 {
    Vec2 {
        x: pivot.x + radius * f32::cos(angle_radian),
        y: pivot.y + radius * f32::sin(angle_radian),
    }
}
