use std::cell::Cell;

/// A trait for WGPU objects that represent uniform buffers used by shaders with
/// CPU side storage of values that can be copied back to the GPU.
pub trait UniformBuffer {
    /// Copy data data stored in this uniform buffer to the GPU.
    ///
    /// Updating the GPU will also clear the dirty flag on this buffer.
    fn update_gpu(&self, queue: &wgpu::Queue);

    /// Get the bind group representing this uniform buffer.
    fn bind_group(&self) -> &wgpu::BindGroup;

    /// Check if the uniform buffer values are out of sync with the GPU.
    fn is_dirty(&self) -> bool;
}

/// A utility struct that simplifies mapping a Rust struct of uniform values to
/// a wgpu uniform value accessible via shader.
///
/// Once created a program can update the values stored in the buffer by calling
/// `values_mut()`, and then calling `update_gpu()` to ensure the new values are
/// copied to the GPU.
#[derive(Debug)]
pub struct GenericUniformBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    /// The values stored in this uniform buffer.
    values: T,
    /// The GPU buffer storing a copy of this uniform buffer's values.
    gpu_buffer: wgpu::Buffer,
    /// The WGPU bind group representing this uniform buffer instance.
    bind_group: wgpu::BindGroup,
    /// True if `values` is potentially out of sync with the GPU buffer and
    /// should be sent to the GPU during the next update phase.
    is_dirty: Cell<bool>,
}

impl<T> GenericUniformBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    /// Create a new generic uniform buffer.
    ///
    /// `device`: The wgpu device owning this uniform buffer.
    /// `label`: Optional name representing this uniform buffer.
    /// `values`: Initial values to store in this uniform buffer.
    /// `bind_group_layout`: The layout of this uniform buffer.
    pub fn new(
        device: &wgpu::Device,
        label: Option<&str>,
        values: T,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::bytes_of(&values),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_buffer.as_entire_binding(),
            }],
        });

        Self {
            values,
            gpu_buffer,
            bind_group,
            is_dirty: Cell::new(false),
        }
    }

    /// Access the values stored in this uniform buffer with a mutable ref.
    ///
    /// Calling this method will set the buffer's dirty flag even if no values
    /// are changed.
    pub fn values_mut(&mut self) -> &mut T {
        // Assume values are dirty any time uniforms are accessed via mutable
        // reference.
        self.is_dirty = Cell::new(true);
        &mut self.values
    }
}

impl<T> UniformBuffer for GenericUniformBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.is_dirty.swap(&Cell::new(false));
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&self.values));
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }
}
