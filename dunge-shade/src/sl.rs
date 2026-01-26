use {
    crate::{
        desc::{Sampler, Texture},
        irc::Dim,
    },
    glam::{IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4},
};

pub fn discard() -> ! {
    panic!()
}

pub fn splat_vec2<S>(scalar: S) -> S::Vec
where
    S: Dim<2>,
{
    S::splat(scalar)
}

pub fn splat_vec3<S>(scalar: S) -> S::Vec
where
    S: Dim<3>,
{
    S::splat(scalar)
}

pub fn splat_vec4<S>(scalar: S) -> S::Vec
where
    S: Dim<4>,
{
    S::splat(scalar)
}

pub trait Append {
    type Scalar;
    type Output;
}

impl Append for Vec2 {
    type Scalar = f32;
    type Output = Vec3;
}

impl Append for Vec3 {
    type Scalar = f32;
    type Output = Vec4;
}

impl Append for IVec2 {
    type Scalar = i32;
    type Output = IVec3;
}

impl Append for IVec3 {
    type Scalar = i32;
    type Output = IVec4;
}

impl Append for UVec2 {
    type Scalar = u32;
    type Output = UVec3;
}

impl Append for UVec3 {
    type Scalar = u32;
    type Output = UVec4;
}

pub fn append<V>(vec: V, e: V::Scalar) -> V::Output
where
    V: Append,
{
    _ = (vec, e);
    panic!()
}

pub fn prepend<V>(e: V::Scalar, vec: V) -> V::Output
where
    V: Append,
{
    _ = (e, vec);
    panic!()
}

pub trait Concat {
    type Output;
}

impl Concat for Vec2 {
    type Output = Vec4;
}

impl Concat for IVec2 {
    type Output = IVec4;
}

impl Concat for UVec2 {
    type Output = UVec4;
}

pub fn concat<V>(a: V, b: V) -> V::Output
where
    V: Concat,
{
    _ = (a, b);
    panic!()
}

pub fn texture_dimensions<S, const D: usize>(texture: Texture<S, D>) -> <u32 as Dim<D>>::Vec
where
    u32: Dim<D>,
{
    texture.never()
}

pub fn texture_sample<const D: usize>(
    texture: Texture<f32, D>,
    sampler: Sampler,
    point: <f32 as Dim<D>>::Vec,
) -> Vec4
where
    f32: Dim<D>,
{
    _ = (sampler, point);
    texture.never()
}

pub fn texture_load<S, const D: usize>(
    texture: Texture<S, D>,
    point: <S as Dim<D>>::Vec,
) -> <S as Dim<4>>::Vec
where
    S: Dim<D> + Dim<4>,
{
    _ = point;
    texture.never()
}

pub fn abs<X, const D: usize>(x: X) -> X
where
    X: Dim<D>,
{
    _ = x;
    panic!()
}

pub fn min<X, const D: usize>(x: X) -> X
where
    X: Dim<D>,
{
    _ = x;
    panic!()
}

pub fn max<X, const D: usize>(x: X) -> X
where
    X: Dim<D>,
{
    _ = x;
    panic!()
}

pub fn clamp<X, const D: usize>(e: X, low: X, high: X) -> X
where
    X: Dim<D>,
{
    _ = (e, low, high);
    panic!()
}
