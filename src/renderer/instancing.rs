use std::cell::RefCell;

use glam::{Mat4, Quat, Vec3};

/// Stores data unique to each model instance including local->world translation
/// and rotation values.
pub struct ModelInstance {
    /// Model space to world space translation vector.
    pub position: Vec3,
    /// Model space rotation amount.
    pub rotation: Quat,
}

/// Represents a GPU instance buffer holding an arbitrary number of `ModelInstance`
/// values.
///
/// `ModelInstanceBuffer` currently transforms each instance value into a 4x4
/// matrix transform when submitting updates to the GPU buffer. This means that
/// values other than translate/rotation/scale cannot be accomodated without
/// rewriting this implementation.
pub struct ModelInstanceBuffer {
    /// A friendly representation of the per-model instance data.
    instances: Vec<ModelInstance>,
    /// A raw buffer of floats representing the transformation of each per-model
    /// instance into a 4x4 transform matrix.
    cpu_buffer: RefCell<Vec<ModelInstanceRawData>>,
    gpu_buffer: wgpu::Buffer,
}

impl ModelInstanceBuffer {
    /// Create a new `ModelInstanceBuffer` from the vector of model instances.
    pub fn new(device: &wgpu::Device, instances: Vec<ModelInstance>) -> Self {
        let cpu_buffer: Vec<ModelInstanceRawData> =
            instances.iter().map(|m| m.into()).collect::<Vec<_>>();
        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&cpu_buffer),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        );

        Self {
            instances,
            cpu_buffer: RefCell::new(cpu_buffer),
            gpu_buffer,
        }
    }

    /// Get a reference to the wgpu Buffer object representing this model
    /// instance buffer.
    pub fn gpu_buffer(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }

    /// Get a reference to the vector of instances stored in this model instance
    /// buffer.
    pub fn instances(&self) -> &[ModelInstance] {
        &self.instances
    }

    /// Get a mutable reference to the vector of instances stored in this model
    /// instance buffer.
    pub fn instances_mut(&mut self) -> &mut [ModelInstance] {
        &mut self.instances
    }

    /// Copy the values in this model instance buffer to the GPU.
    pub fn write_to_gpu(&self, queue: &wgpu::Queue) {
        // Copy instance data to CPU data buffer of floats prior to writing it
        // to the GPU.
        {
            let mut cpu_buffer = self.cpu_buffer.borrow_mut();

            (0..self.instances.len()).for_each(|i| {
                cpu_buffer[i] = (&self.instances[i]).into();
            });
        }

        // Write updated instance data (in the form of raw floats) to the GPU.
        queue.write_buffer(
            &self.gpu_buffer,
            0,
            bytemuck::cast_slice(&self.cpu_buffer.borrow()),
        );
    }

    /// Get a vertex buffer layout which is used when creating `VertexState`
    /// descriptons for `RenderPipeline`.
    pub fn layout_desc() -> wgpu::VertexBufferLayout<'static> {
        // NOTE: The transform matrix is represented in the GPU buffer as 4 vec4
        // column vectors.
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
        let xform = Mat4::from_rotation_translation(value.rotation, value.position);

        ModelInstanceRawData {
            model: xform.to_cols_array_2d(),
        }
    }
}

/// A helper method that creates an NxM grid of model instances suitable for use
/// in `ModelInstanceBuffer`.
pub fn spawn_object_instances_as_grid(
    num_rows: usize,
    instances_per_row: usize,
    displacement: Vec3,
    angle_radians: f32,
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
                    Quat::from_axis_angle(position.normalize(), angle_radians)
                };

                ModelInstance { position, rotation }
            })
        })
        .collect::<Vec<_>>()
}
