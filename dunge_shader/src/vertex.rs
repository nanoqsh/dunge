use std::{iter, slice};

#[derive(Clone, Copy)]
pub enum VectorType {
    Vec2f,
    Vec3f,
    Vec4f,
    Vec2u,
    Vec3u,
    Vec4u,
    Vec2i,
    Vec3i,
    Vec4i,
}

#[derive(Clone, Copy)]
pub struct DeclareInput(&'static [VectorType]);

impl DeclareInput {
    pub const fn new(ts: &'static [VectorType]) -> Self {
        Self(ts)
    }
}

impl IntoIterator for DeclareInput {
    type Item = VectorType;
    type IntoIter = iter::Copied<slice::Iter<'static, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait Vertex {
    type Projection: Projection + 'static;
    const DECL: DeclareInput;
}

pub trait Projection {
    fn projection(id: u32) -> Self;
}
