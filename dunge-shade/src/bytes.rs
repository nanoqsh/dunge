use {
    glam::{IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4},
    std::slice,
};

/// Get bytes of a value.
///
/// You shouldn't implement this trait manually, use `derive` macro.
///
/// # Safety
///
/// It should be safe to cast a slice of `Self` to bytes.
pub unsafe trait Bytes: Sized + Copy + 'static {}

pub const fn bytes_size<B>() -> usize
where
    B: Bytes,
{
    size_of::<B>()
}

pub const fn as_bytes<B>(slice: &[B]) -> &[u8]
where
    B: Bytes,
{
    // SAFETY:
    // The invariant of `Bytes` is it's safe to get bytes of the values
    unsafe {
        let bytelen = slice.len() * bytes_size::<B>();
        let data = slice.as_ptr().cast();
        slice::from_raw_parts(data, bytelen)
    }
}

unsafe impl Bytes for bool {}
unsafe impl Bytes for u8 {}
unsafe impl Bytes for f32 {}
unsafe impl Bytes for u32 {}
unsafe impl Bytes for i32 {}
unsafe impl<B, const N: usize> Bytes for [B; N] where B: Bytes {}

unsafe impl Bytes for Vec2 {}
unsafe impl Bytes for Vec3 {}
unsafe impl Bytes for Vec4 {}
unsafe impl Bytes for UVec2 {}
unsafe impl Bytes for UVec3 {}
unsafe impl Bytes for UVec4 {}
unsafe impl Bytes for IVec2 {}
unsafe impl Bytes for IVec3 {}
unsafe impl Bytes for IVec4 {}
unsafe impl Bytes for Mat2 {}
unsafe impl Bytes for Mat3 {}
unsafe impl Bytes for Mat4 {}
