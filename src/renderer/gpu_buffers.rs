use std::cell::Cell;

/// Trait for objects that represent a GPU buffer that can be updated from the
/// CPU.
pub trait DynamicGpuBuffer {
    /// Copy data stored in this buffer to the GPU.
    ///
    /// Updating the GPU will also clear the dirty flag on this buffer.
    fn update_gpu(&self, queue: &wgpu::Queue);

    /// Check if this buffer has values that have not yet been copied to the GPU.
    fn is_dirty(&self) -> bool;
}

/// A trait for bind groups that contain uniforms.
pub trait UniformBindGroup {
    /// Get the bind group representing this uniform buffer.
    fn bind_group(&self) -> &wgpu::BindGroup;
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

    /// Access the values stored in this uniform buffer.
    pub fn values(&self) -> &T {
        &self.values
    }

    /// Access the values stored in this uniform buffer with a mutable ref.
    ///
    /// Calling this method will set the buffer's dirty flag even if no values
    /// are changed.
    pub fn values_mut(&mut self) -> &mut T {
        self.is_dirty = Cell::new(true);
        &mut self.values
    }
}

impl<T> DynamicGpuBuffer for GenericUniformBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.is_dirty.swap(&Cell::new(false));
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&self.values));
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }
}

impl<T> UniformBindGroup for GenericUniformBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// A utility struct to abstract an array of uniform values when used for
/// instancing.
///
/// Once created a program can update the values stored in the buffer by calling
/// `values_mut()`, and then calling `update_gpu()` to ensure the new values are
/// copied to the GPU.
#[derive(Debug)]
pub struct InstanceBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    /// A copy of all the instances in the buffer.
    instances: Vec<T>,
    /// The GPU buffer storing a copy of this uniform buffer's values.
    gpu_buffer: wgpu::Buffer,
    /// True if `values` has new data that needs to be copied to the GPU.
    is_dirty: Cell<bool>,
}

impl<T> InstanceBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    /// Create a new instance buffer.
    ///
    /// `device`: The wgpu device owning this uniform buffer.
    /// `label`: Optional name representing this uniform buffer.
    /// `instances`: The initial instance values to place in this buffer.
    pub fn new(device: &wgpu::Device, label: Option<&str>, instances: Vec<T>) -> Self {
        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            },
        );

        Self {
            instances,
            gpu_buffer,
            is_dirty: Cell::new(false),
        }
    }

    /// Access an instance stored in this instance buffer via const reef.
    #[allow(dead_code)]
    pub fn values(&self, index: usize) -> &T {
        &self.instances[index]
    }

    /// Access an instance stored in this instance buffer via mutable ref.
    ///
    /// Calling this method will set the buffer's dirty flag even if no values
    /// are changed.
    pub fn values_mut(&mut self, index: usize) -> &mut T {
        self.is_dirty = Cell::new(true);
        &mut self.instances[index]
    }

    /// Get the GPU buffer object used by this instance buffer.
    pub fn gpu_buffer_slice<S>(&self, bounds: S) -> wgpu::BufferSlice
    where
        S: std::ops::RangeBounds<wgpu::BufferAddress>,
    {
        self.gpu_buffer.slice(bounds)
    }
}

impl<T> DynamicGpuBuffer for InstanceBuffer<T>
where
    T: Clone + Copy + std::fmt::Debug + bytemuck::Pod + bytemuck::Zeroable,
{
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.is_dirty.swap(&Cell::new(false));
        queue.write_buffer(
            &self.gpu_buffer,
            0,
            bytemuck::cast_slice(self.instances.as_slice()),
        );
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }
}
