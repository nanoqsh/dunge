//! Shader group types and traits.

use crate::{
    sl::{GlobalOut, ReadGlobal, Ret},
    storage::Storage,
    texture::{BindTexture, Sampler, Texture2d},
    types::{self, MemberType},
    uniform::Uniform,
    value::Value,
};

pub use dunge_shader::group::Projection;

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
    const TYPE: MemberType;
    type Field;
    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field;
}

impl<V> private::Sealed for &Uniform<V> where V: Value {}

impl<V> MemberProjection for &Uniform<V>
where
    V: Value,
{
    const TYPE: MemberType = MemberType::from_value(<V::Type as types::Value>::VALUE_TYPE);
    type Field = Ret<ReadGlobal, V::Type>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

impl<V> private::Sealed for &Storage<V> where V: Value {}

impl<V> MemberProjection for &Storage<V>
where
    V: Value,
{
    const TYPE: MemberType = MemberType::from_value(<V::Type as types::Value>::VALUE_TYPE);
    type Field = Ret<ReadGlobal, V::Type>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

impl<V> private::Sealed for &Storage<[V]> where V: Value {}

impl<V> MemberProjection for &Storage<[V]>
where
    V: Value,
{
    const TYPE: MemberType = MemberType::DynamicArrayType(types::DynamicArrayType {
        base: &<V::Type as types::Value>::VALUE_TYPE,
    });

    type Field = Ret<ReadGlobal, types::DynamicArray<V::Type>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

impl private::Sealed for BoundTexture<'_> {}

impl MemberProjection for BoundTexture<'_> {
    const TYPE: MemberType = MemberType::Tx2df;
    type Field = Ret<ReadGlobal, types::Texture2d<f32>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

impl private::Sealed for &Sampler {}

impl MemberProjection for &Sampler {
    const TYPE: MemberType = MemberType::Sampl;
    type Field = Ret<ReadGlobal, types::Sampler>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

mod private {
    pub trait Sealed {}
}
