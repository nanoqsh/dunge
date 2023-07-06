use {
    crate::{buffer::BufferView, error::TooLargeSize},
    bytemuck::{Pod, Zeroable},
    glam::Mat4,
    wgpu::{Buffer, Device, Queue, VertexBufferLayout, VertexStepMode},
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

pub(crate) struct Instance(Buffer);

impl Instance {
    pub fn new(models: &[Model], device: &Device) -> Self {
        use wgpu::{
            util::{BufferInitDescriptor, DeviceExt},
            BufferUsages,
        };

        Self(device.create_buffer_init(&BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(models),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        }))
    }

    pub fn update(&self, models: &[Model], queue: &Queue) -> Result<(), TooLargeSize> {
        use std::mem;

        let Self(buf) = self;
        if buf.size() != mem::size_of_val(models) as u64 {
            return Err(TooLargeSize);
        }

        queue.write_buffer(buf, 0, bytemuck::cast_slice(models));
        Ok(())
    }

    pub fn buffer(&self) -> BufferView<Model> {
        BufferView::new(&self.0)
    }
}
