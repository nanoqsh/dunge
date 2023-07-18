use {
    crate::{
        color::{Color, Rgb},
        shader_data::{data::SpaceData, ModelTransform},
    },
    bytemuck::{Pod, Zeroable},
};

type Mat = [[f32; 4]; 4];

/// Parameters of the light space.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    pub data: SpaceData<'a>,
    pub model: ModelTransform,
    pub col: Rgb,
}

impl Space<'_> {
    pub(crate) fn into_uniform(self) -> SpaceUniform {
        let Color(col) = self.col;
        let size = self.data.get().size;
        SpaceUniform::new(size, self.model.into_inner(), col)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct SpaceUniform {
    model: Mat,
    col: [f32; 3],
    pad: u32,
}

impl SpaceUniform {
    pub fn new(size: (u8, u8, u8), transform: Mat, col: [f32; 3]) -> Self {
        Self {
            model: {
                use glam::{Mat4, Quat, Vec3};

                let (width, height, depth) = size;
                let texture_space = Mat4::from_scale_rotation_translation(
                    Vec3::new(1. / width as f32, 1. / depth as f32, 1. / height as f32),
                    Quat::IDENTITY,
                    Vec3::new(0.5, 0.5, 0.5),
                );

                let model = Mat4::from_cols_array_2d(&transform);
                let model = texture_space * model;
                model.to_cols_array_2d()
            },
            col,
            pad: 0,
        }
    }
}
