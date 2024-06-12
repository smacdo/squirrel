use glam::{Mat4, Quat, Vec3};

/// Stores data unique to each model instance including world position and
/// rotation.
pub struct ModelInstance {
    /// Position of model in world space.
    pub position: Vec3,
    /// Rotation of model.
    pub rotation: Quat,
}

pub struct ModelInstanceBuffer {
    instances: Vec<ModelInstance>,
    cpu_buffer: Vec<ModelInstanceRawData>,
    gpu_buffer: wgpu::Buffer,
}

impl ModelInstanceBuffer {
    pub fn new(device: &wgpu::Device, instances: Vec<ModelInstance>) -> Self {
        let cpu_buffer: Vec<ModelInstanceRawData> =
            instances.iter().map(|m| m.into()).collect::<Vec<_>>();
        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&cpu_buffer),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        Self {
            instances,
            cpu_buffer,
            gpu_buffer,
        }
    }

    pub fn gpu_buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }

    pub fn instances_count(&self) -> usize {
        self.instances.len()
    }

    pub fn layout_desc() -> wgpu::VertexBufferLayout<'static> {
        // XXX(scott): transform matrix = 4x vec4 columns.
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ModelInstanceRawData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 0]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4 * 2]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4 * 3]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelInstanceRawData {
    model: [[f32; 4]; 4],
}

impl From<&ModelInstance> for ModelInstanceRawData {
    fn from(value: &ModelInstance) -> Self {
        // TODO(scott): use Mat4::from_rotation_translation(rotation, translation)
        let xform: Mat4 = Mat4::from_translation(value.position) * Mat4::from_quat(value.rotation);

        ModelInstanceRawData {
            model: xform.to_cols_array_2d(),
        }
    }
}

pub fn spawn_object_instances_as_grid(
    num_rows: usize,
    instances_per_row: usize,
    displacement: Vec3,
) -> Vec<ModelInstance> {
    (0..num_rows)
        .flat_map(|z| {
            (0..instances_per_row).map(move |x| {
                let position: Vec3 = Vec3 {
                    x: x as f32 * 2.0,
                    y: 0.0,
                    z: z as f32 * 2.0,
                } - displacement;

                let rotation = if position == Vec3::ZERO {
                    Quat::from_axis_angle(Vec3::Z, 0.0)
                } else {
                    Quat::from_axis_angle(position.normalize(), 45.0_f32.to_radians())
                };

                ModelInstance { position, rotation }
            })
        })
        .collect::<Vec<_>>()
}
