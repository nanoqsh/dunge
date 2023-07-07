use {
    crate::{buffer::BufferView, error::TooLargeSize, render::State},
    bytemuck::{Pod, Zeroable},
    glam::Mat4,
    std::sync::Arc,
    wgpu::{Buffer, Queue, VertexBufferLayout, VertexStepMode},
};

type Mat = [[f32; 4]; 4];

#[repr(transparent)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Model(Mat);

impl Model {
    pub(crate) const LOCATION_OFFSET: u32 = 4;
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

impl Default for Model {
    fn default() -> Self {
        Self::from(Mat4::IDENTITY)
    }
}

impl From<Mat> for Model {
    fn from(mat: Mat) -> Self {
        Self(mat)
    }
}

impl From<Mat4> for Model {
    fn from(mat: Mat4) -> Self {
        Self::from(mat.to_cols_array_2d())
    }
}

pub struct Instance {
    buf: Buffer,
    queue: Arc<Queue>,
}

impl Instance {
    pub(crate) fn new(models: &[Model], state: &State) -> Self {
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
        }
    }

    pub fn update(&self, models: &[Model]) -> Result<(), TooLargeSize> {
        use std::mem;

        if self.buf.size() != mem::size_of_val(models) as u64 {
            return Err(TooLargeSize);
        }

        self.queue
            .write_buffer(&self.buf, 0, bytemuck::cast_slice(models));

        Ok(())
    }

    pub(crate) fn buffer(&self) -> BufferView<Model> {
        BufferView::new(&self.buf)
    }
}
