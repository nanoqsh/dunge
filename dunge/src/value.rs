//! Value traits.

use crate::{color::Color, types};

/// A base value.
pub trait Value {
    type Type;
    fn value(&self) -> &[u8];
}

impl Value for u32 {
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for f32 {
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec2 {
    type Type = types::Vec2<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec3 {
    type Type = types::Vec3<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Vec4 {
    type Type = types::Vec4<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat2 {
    type Type = types::Mat2;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat3 {
    type Type = types::Mat3;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Value for glam::Mat4 {
    type Type = types::Mat4;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// A [uniform](crate::store::UniformOld) buffer value.
pub trait UniformValue {
    type Type;
    type GlobalType;
    fn uniform_value(&self) -> &[u8];
}

impl<V> UniformValue for V
where
    V: Value,
{
    type Type = V::Type;
    type GlobalType = types::Pointer<Self::Type>;

    fn uniform_value(&self) -> &[u8] {
        self.value()
    }
}

impl<V, const N: usize> UniformValue for [V; N]
where
    V: UniformValue + bytemuck::Pod,
{
    type Type = types::Array<V::Type, N, true>;
    type GlobalType = Self::Type;

    fn uniform_value(&self) -> &[u8] {
        const {
            assert!(
                size_of::<V>().is_multiple_of(16),
                "in uniforms stride must be multiple of 16",
            );
        }

        bytemuck::bytes_of(self)
    }
}

/// A [storage](crate::store::StorageOld) buffer value.
pub trait StorageValue {
    type Type;
    type GlobalType;
    fn storage_value(&self) -> &[u8];
}

impl<V> StorageValue for V
where
    V: Value,
{
    type Type = V::Type;
    type GlobalType = types::Pointer<Self::Type>;

    fn storage_value(&self) -> &[u8] {
        self.value()
    }
}

impl<V, const N: usize> StorageValue for [V; N]
where
    V: StorageValue + bytemuck::Pod,
{
    type Type = types::Array<V::Type, N, false>;
    type GlobalType = Self::Type;

    fn storage_value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<V> StorageValue for [V]
where
    V: StorageValue + bytemuck::Pod,
{
    type Type = types::DynamicArray<V::Type>;
    type GlobalType = Self::Type;

    fn storage_value(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

/// The trait to treat [colors](Color) as [values](Value).
pub trait ColorValue {
    type Type;
}

impl ColorValue for Color<1> {
    type Type = f32;
}

impl ColorValue for Color<2> {
    type Type = types::Vec2<f32>;
}

impl ColorValue for Color<3> {
    type Type = types::Vec3<f32>;
}

impl ColorValue for Color<4> {
    type Type = types::Vec4<f32>;
}

impl<const N: usize> Value for Color<N>
where
    Self: ColorValue,
{
    type Type = <Self as ColorValue>::Type;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(&self.0)
    }
}
