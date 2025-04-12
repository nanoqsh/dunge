//! Shader group types and traits.

use crate::{
    sl::{Global, GlobalOut, Ret},
    storage::Storage,
    texture::{BindTexture, Sampler, Texture2d},
    types::{self, MemberData, MemberType},
    uniform::Uniform,
    value::Value,
};

pub use dunge_shader::group::{Projection, Take};

#[derive(Clone, Copy)]
pub struct BoundTexture<'a>(pub(crate) &'a Texture2d);

impl<'a> BoundTexture<'a> {
    pub fn new<T>(texture: &'a T) -> Self
    where
        T: BindTexture,
    {
        Self(texture.bind_texture())
    }
}

/// Describes a group member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: private::Sealed {
    const MEMBER: MemberData;
    type Field;
    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field;
}

impl<V> private::Sealed for &Uniform<V> where V: Value {}

impl<V> MemberProjection for &Uniform<V>
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

impl<V, M> private::Sealed for &Storage<V, M> where V: Value {}

impl<V, M> MemberProjection for &Storage<V, M>
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

impl<V, M> private::Sealed for &Storage<[V], M> where V: Value {}

impl<V, M> MemberProjection for &Storage<[V], M>
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

impl private::Sealed for BoundTexture<'_> {}

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

impl private::Sealed for &Sampler {}

impl MemberProjection for &Sampler {
    const MEMBER: MemberData = MemberData {
        ty: MemberType::Sampl,
        mutable: false,
    };

    type Field = Ret<Global, types::Sampler>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

mod private {
    pub trait Sealed {}
}
