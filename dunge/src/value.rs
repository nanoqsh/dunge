//! Value traits.

use crate::{color::Color, types};

/// A buffer value.
pub trait Value<const U: bool> {
    type Type: types::Value;
    fn value(&self) -> &[u8];
}

impl<const U: bool> Value<U> for u32 {
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for f32 {
    type Type = Self;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Vec2 {
    type Type = types::Vec2<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Vec3 {
    type Type = types::Vec3<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Vec4 {
    type Type = types::Vec4<f32>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Mat2 {
    type Type = types::Mat2;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Mat3 {
    type Type = types::Mat3;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<const U: bool> Value<U> for glam::Mat4 {
    type Type = types::Mat4;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl<V, const N: usize, const U: bool> Value<U> for [V; N]
where
    V: Value<U> + bytemuck::Pod,
{
    type Type = types::Array<V::Type, N, U>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// A [uniform](crate::storage::Uniform) buffer value.
pub trait UniformValue {
    fn uniform_value(&self) -> &[u8];
}

impl<V> UniformValue for V
where
    V: Value<true>,
{
    fn uniform_value(&self) -> &[u8] {
        self.value()
    }
}

/// A [storage](crate::storage::Storage) buffer value.
pub trait StorageValue {
    fn storage_value(&self) -> &[u8];
}

impl<V> StorageValue for V
where
    V: Value<false>,
{
    fn storage_value(&self) -> &[u8] {
        self.value()
    }
}

impl<V> StorageValue for [V]
where
    V: Value<false> + bytemuck::Pod,
{
    fn storage_value(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

/// The trait to treat [colors](Color) as [values](Value).
pub trait ColorValue {
    type Type: types::Value;
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

impl<const N: usize, const U: bool> Value<U> for Color<N>
where
    Self: ColorValue,
{
    type Type = <Self as ColorValue>::Type;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(&self.0)
    }
}
