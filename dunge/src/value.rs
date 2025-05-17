use crate::types;

/// A buffer value.
pub trait Value {
    type Type: types::Value;
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

impl<V, const N: usize> Value for [V; N]
where
    V: Value + bytemuck::Pod,
{
    type Type = types::Array<V::Type, N>;

    fn value(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// An unsized buffer value.
pub trait UnsizedValue {
    fn unsized_value(&self) -> &[u8];
}

impl<V> UnsizedValue for V
where
    V: Value,
{
    fn unsized_value(&self) -> &[u8] {
        self.value()
    }
}

impl<V> UnsizedValue for [V]
where
    V: Value + bytemuck::Pod,
{
    fn unsized_value(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}
