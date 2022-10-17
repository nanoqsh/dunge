pub(crate) use self::plain::Plain;

use wgpu::{VertexAttribute, VertexBufferLayout, VertexStepMode};

mod plain {
    /// A trait for plain structs which can be safely casted to bytes.
    ///
    /// # Safety
    /// An implementation of this trait assumes all bits of struct can be safely read.
    pub unsafe trait Plain: Sized {
        fn as_bytes(&self) -> &[u8] {
            use std::{mem, slice};

            unsafe { slice::from_raw_parts((self as *const Self).cast(), mem::size_of::<Self>()) }
        }
    }

    unsafe impl Plain for u8 {}
    unsafe impl Plain for u16 {}
    unsafe impl Plain for u32 {}
    unsafe impl Plain for u64 {}

    unsafe impl Plain for i8 {}
    unsafe impl Plain for i16 {}
    unsafe impl Plain for i32 {}
    unsafe impl Plain for i64 {}

    unsafe impl Plain for f32 {}
    unsafe impl Plain for f64 {}

    unsafe impl<T, const N: usize> Plain for [T; N] where T: Plain {}

    unsafe impl<T> Plain for &[T]
    where
        T: Plain,
    {
        fn as_bytes(&self) -> &[u8] {
            use std::{mem, slice};

            unsafe { slice::from_raw_parts(self.as_ptr().cast(), self.len() * mem::size_of::<T>()) }
        }
    }
}

/// The trait describes a layout.
pub trait Layout: Plain {
    const ATTRIBS: &'static [VertexAttribute];
    const VERTEX_STEP_MODE: VertexStepMode;
}

pub(crate) fn layout<V>() -> VertexBufferLayout<'static>
where
    V: Layout,
{
    use {std::mem, wgpu::BufferAddress};

    VertexBufferLayout {
        array_stride: mem::size_of::<V>() as BufferAddress,
        step_mode: V::VERTEX_STEP_MODE,
        attributes: V::ATTRIBS,
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct InstanceModel {
    pub(crate) mat: [[f32; 4]; 4],
}

unsafe impl Plain for InstanceModel {}

impl Layout for InstanceModel {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Instance;
}
