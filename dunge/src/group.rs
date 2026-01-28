//! Shader group types and traits.

use {
    crate::{
        buffer::TextureSampler,
        sl_old::{Define, Global, GlobalOut, Ret},
        store::{StorageOld, UniformOld},
        types::{self, MemberData, MemberType, Space},
        value::{StorageValue, UniformValue},
    },
    dunge_shade_old::group::GroupLegacy,
};

pub use dunge_shade_old::group::{Projection, Take};

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

impl<V> s::Sealed for UniformOld<V> where V: UniformValue<Type: types::Member> {}

impl<V> MemberProjection for UniformOld<V>
where
    V: UniformValue<Type: types::Member>,
{
    const MEMBER: MemberData = MemberData {
        ty: <V::Type as types::Member>::MEMBER_TYPE,
        space: Space::Uniform,
    };

    type Field = Ret<Global, V::GlobalType>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl<V> GroupLegacy for UniformOld<V>
where
    Self: MemberProjection<Field: Projection>,
{
    type Projection = <Self as MemberProjection>::Field;
    const DEF: Define<MemberData> = Define::new(&[Self::MEMBER]);
}

impl<V> s::Sealed for StorageOld<V> where V: StorageValue + ?Sized {}

impl<V> MemberProjection for StorageOld<V>
where
    V: StorageValue<Type: types::Member> + ?Sized,
{
    const MEMBER: MemberData = MemberData {
        ty: <V::Type as types::Member>::MEMBER_TYPE,
        space: Space::Storage,
    };

    type Field = Ret<Global, V::GlobalType>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl<V> GroupLegacy for StorageOld<V>
where
    V: ?Sized,
    Self: MemberProjection<Field: Projection>,
{
    type Projection = <Self as MemberProjection>::Field;
    const DEF: Define<MemberData> = Define::new(&[Self::MEMBER]);
}

#[derive(Clone)]
pub struct BoundTexture(pub(crate) wgpu::TextureView);

impl s::Sealed for BoundTexture {}

impl MemberProjection for BoundTexture {
    const MEMBER: MemberData = MemberData {
        ty: MemberType::Tx2df,
        space: Space::Handle,
    };

    type Field = Ret<Global, types::Texture2d<f32>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl GroupLegacy for BoundTexture {
    type Projection = <Self as MemberProjection>::Field;
    const DEF: Define<MemberData> = Define::new(&[Self::MEMBER]);
}

impl s::Sealed for TextureSampler {}

impl MemberProjection for TextureSampler {
    const MEMBER: MemberData = MemberData {
        ty: MemberType::Sampl,
        space: Space::Handle,
    };

    type Field = Ret<Global, types::Sampler>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        Global::new(id, binding, out)
    }
}

impl GroupLegacy for TextureSampler {
    type Projection = <Self as MemberProjection>::Field;
    const DEF: Define<MemberData> = Define::new(&[Self::MEMBER]);
}

mod s {
    pub trait Sealed {}
}
