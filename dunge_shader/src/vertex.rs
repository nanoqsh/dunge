use {
    crate::{
        define::Define,
        op::Ret,
        sl::ReadVertex,
        types::{self, VectorType},
    },
    std::slice,
};

/// The vertex type description.
///
/// # Safety
/// The fields of [`Define`] must exactly match the actual struct fields.
/// To do this, the fields must be ordered, so the struct must have the `#[repr(C)]`
/// attribute and the fields must have the same order as specified in [`Define`].
///
/// # Deriving
/// Although the library tries to formalize the safety invariant, you still shouldn’t
/// implement the vertex yourself. The most reliable and simple way to do this is to
/// use a derive macro:
/// ```rust
/// # mod dunge {
/// #    // fake `Vertex` derive
/// #    pub use std::default::Default as Vertex;
/// # }
/// #
/// use dunge::Vertex;
///
/// #[repr(C)]
/// #[derive(Vertex)]
/// struct Vert {
///     pos: [f32; 2],
///     col: [f32; 3],
/// }
/// ```
///
pub unsafe trait Vertex {
    type Projection: Projection + 'static;
    const DEF: Define<VectorType>;
}

unsafe impl<I> Vertex for I
where
    I: InputProjection<Field: Projection + 'static>,
{
    type Projection = I::Field;
    const DEF: Define<VectorType> = Define::new(&[I::TYPE]);
}

/// Maps the slice of vertices to the slice of bytes.
pub fn verts_as_bytes<V>(verts: &[V]) -> &[u8]
where
    V: Vertex,
{
    // SAFETY:
    // * The `Vertex` invariant states converting a slice of vertices to bytes is safe
    unsafe { slice::from_raw_parts(verts.as_ptr().cast(), size_of_val(verts)) }
}

/// Vertex type projection in a shader.
pub trait Projection {
    fn projection(id: u32) -> Self;
}

impl<T> Projection for Ret<ReadVertex, T> {
    fn projection(id: u32) -> Self {
        ReadVertex::new(id, 0)
    }
}

/// Describes an input type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait InputProjection: f::F32Aligned {
    const TYPE: VectorType;
    type Field;
    fn input_projection(id: u32, index: u32) -> Self::Field;
}

impl f::F32Aligned for [f32; 2] {}

impl InputProjection for [f32; 2] {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadVertex, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for [f32; 3] {}

impl InputProjection for [f32; 3] {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadVertex, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for [f32; 4] {}

impl InputProjection for [f32; 4] {
    const TYPE: VectorType = VectorType::Vec4f;
    type Field = Ret<ReadVertex, types::Vec4<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for glam::Vec2 {}

impl InputProjection for glam::Vec2 {
    const TYPE: VectorType = VectorType::Vec2f;
    type Field = Ret<ReadVertex, types::Vec2<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

impl f::F32Aligned for glam::Vec3 {}

impl InputProjection for glam::Vec3 {
    const TYPE: VectorType = VectorType::Vec3f;
    type Field = Ret<ReadVertex, types::Vec3<f32>>;

    fn input_projection(id: u32, index: u32) -> Self::Field {
        ReadVertex::new(id, index)
    }
}

#[cfg(false)]
mod ignore {
    // glam::Vec4 is not f32 aligned
    impl !InputProjection for glam::Vec4 {}
}

mod f {
    pub trait F32Aligned {}
}
