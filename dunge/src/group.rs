//! Shader group types and traits.

use crate::{
    buffer::Sampler,
    sl::{Global, GlobalOut, Ret},
    storage::{Storage, Uniform},
    types::{self, MemberData, MemberType},
    value::Value,
};

pub use dunge_shader::group::{Projection, Take};

/// Describes a group member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: s::Sealed {
    const MEMBER: MemberData;
    type Field;
    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field;
}

impl<M> s::Sealed for &M where M: s::Sealed {}

impl<M> MemberProjection for &M
where
    M: MemberProjection,
{
    const MEMBER: MemberData = M::MEMBER;
    type Field = M::Field;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        M::member_projection(id, binding, out)
    }
}

impl<V> s::Sealed for Uniform<V> where V: Value {}

impl<V> MemberProjection for Uniform<V>
where
    V: Value,
{
    const MEMBER: MemberData = MemberData {
        ty: MemberType::from_value(<V::Type as types::Value>::VALUE_TYPE),
        mutable: false,
    };

    type Field = Ret<Global, V::Type>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl<V, M> s::Sealed for Storage<V, M> where V: Value {}

impl<V, M> MemberProjection for Storage<V, M>
where
    V: Value,
    M: types::Mutability,
{
    const MEMBER: MemberData = MemberData {
        ty: MemberType::from_value(<V::Type as types::Value>::VALUE_TYPE),
        mutable: M::MUTABLE,
    };

    type Field = Ret<Global<M>, V::Type>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl<V, M> s::Sealed for Storage<[V], M> where V: Value {}

impl<V, M> MemberProjection for Storage<[V], M>
where
    V: Value,
    M: types::Mutability,
{
    const MEMBER: MemberData = MemberData {
        ty: MemberType::DynamicArrayType(types::DynamicArrayType {
            base: &<V::Type as types::Value>::VALUE_TYPE,
        }),
        mutable: M::MUTABLE,
    };

    type Field = Ret<Global<M>, types::DynamicArray<V::Type>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

#[derive(Clone, Copy)]
pub struct BoundTexture<'tx>(pub(crate) &'tx wgpu::TextureView);

impl s::Sealed for BoundTexture<'_> {}

impl MemberProjection for BoundTexture<'_> {
    const MEMBER: MemberData = MemberData {
        ty: MemberType::Tx2df,
        mutable: false,
    };

    type Field = Ret<Global, types::Texture2d<f32>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl s::Sealed for Sampler {}

impl MemberProjection for Sampler {
    const MEMBER: MemberData = MemberData {
        ty: MemberType::Sampl,
        mutable: false,
    };

    type Field = Ret<Global, types::Sampler>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

mod s {
    pub trait Sealed {}
}
