use {
    crate::{
        buffer::BufferView,
        color::{Color, Rgb},
        render::State,
    },
    bytemuck::{Pod, Zeroable},
    glam::Mat4,
    std::{marker::PhantomData, sync::Arc},
    wgpu::{Buffer, Queue, VertexBufferLayout, VertexStepMode},
};

type Mat = [[f32; 4]; 4];

#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ModelTransform(Mat);

impl ModelTransform {
    pub(crate) const LAYOUT_ATTRIBUTES_LEN: u32 = Self::LAYOUT.attributes.len() as u32;
    pub(crate) const LAYOUT: VertexBufferLayout<'_> = {
        use std::mem;

        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x4,
                1 => Float32x4,
                2 => Float32x4,
                3 => Float32x4,
            ],
        }
    };

    pub(crate) fn into_inner(self) -> Mat {
        self.0
    }
}

impl Default for ModelTransform {
    fn default() -> Self {
        Self::from(Mat4::IDENTITY)
    }
}

impl From<Mat> for ModelTransform {
    fn from(mat: Mat) -> Self {
        Self(mat)
    }
}

impl From<Mat4> for ModelTransform {
    fn from(mat: Mat4) -> Self {
        Self::from(mat.to_cols_array_2d())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct ModelColor([f32; 3]);

impl ModelColor {
    pub(crate) const LAYOUT_ATTRIBUTES_LEN: u32 = Self::LAYOUT.attributes.len() as u32;
    pub(crate) const LAYOUT: VertexBufferLayout<'_> = {
        use std::mem;

        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as _,
            step_mode: VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                ModelTransform::LAYOUT_ATTRIBUTES_LEN => Float32x3,
            ],
        }
    };
}

impl From<Rgb> for ModelColor {
    fn from(Color(col): Rgb) -> Self {
        Self(col)
    }
}

pub struct Instance(Inner<ModelTransform>);

impl Instance {
    pub(crate) fn new(models: &[ModelTransform], state: &State) -> Self {
        Self(Inner::new(models, state))
    }

    /// Updates the instance with new [models](`ModelTransform`).
    ///
    /// # Errors
    /// Will return [`InvalidSize`] if the size of the [models](`ModelTransform`)
    /// slice doesn't match the current instance size.
    pub fn update(&self, models: &[ModelTransform]) -> Result<(), InvalidSize> {
        self.0.update(models)
    }

    pub(crate) fn buffer(&self) -> BufferView {
        self.0.buffer()
    }
}

pub struct InstanceColor(Inner<ModelColor>);

impl InstanceColor {
    pub(crate) fn new(models: &[ModelColor], state: &State) -> Self {
        Self(Inner::new(models, state))
    }

    /// Updates the instance color with new [models](`ModelColor`).
    ///
    /// # Errors
    /// Will return [`InvalidSize`] if the size of the [models](`ModelColor`)
    /// slice doesn't match the current instance size.
    pub fn update(&self, models: &[ModelColor]) -> Result<(), InvalidSize> {
        self.0.update(models)
    }

    pub(crate) fn buffer(&self) -> BufferView {
        self.0.buffer()
    }
}

struct Inner<M> {
    buf: Buffer,
    queue: Arc<Queue>,
    ty: PhantomData<M>,
}

impl<M> Inner<M> {
    fn new(models: &[M], state: &State) -> Self
    where
        M: Pod,
    {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self {
            buf: state.device().create_buffer_init(&BufferInitDescriptor {
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(models),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
            queue: Arc::clone(state.queue()),
            ty: PhantomData,
        }
    }

    fn update(&self, models: &[M]) -> Result<(), InvalidSize>
    where
        M: Pod,
    {
        use std::mem;

        if self.buf.size() != mem::size_of_val(models) as u64 {
            return Err(InvalidSize);
        }

        self.queue
            .write_buffer(&self.buf, 0, bytemuck::cast_slice(models));

        Ok(())
    }

    fn buffer(&self) -> BufferView {
        BufferView::new::<M>(&self.buf)
    }
}

/// An error returned from the instance updation.
#[derive(Debug)]
pub struct InvalidSize;
