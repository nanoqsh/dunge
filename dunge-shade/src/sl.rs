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
    _ = texture;
    panic!()
}

pub fn texture_sample<const D: usize>(
    texture: Texture<f32, D>,
    sampler: Sampler,
    point: <f32 as Dim<D>>::Vec,
) -> Vec4
where
    f32: Dim<D>,
{
    _ = (texture, sampler, point);
    panic!()
}

pub fn texture_load<S, const D: usize>(
    texture: Texture<S, D>,
    point: <S as Dim<D>>::Vec,
) -> <S as Dim<4>>::Vec
where
    S: Dim<D> + Dim<4>,
{
    _ = (texture, point);
    panic!()
}

pub trait Vector {
    const DIM: usize;
    type Scalar;
}

impl Vector for Vec2 {
    const DIM: usize = 2;
    type Scalar = f32;
}

impl Vector for Vec3 {
    const DIM: usize = 3;
    type Scalar = f32;
}

impl Vector for Vec4 {
    const DIM: usize = 4;
    type Scalar = f32;
}

impl Vector for IVec2 {
    const DIM: usize = 2;
    type Scalar = i32;
}

impl Vector for IVec3 {
    const DIM: usize = 3;
    type Scalar = i32;
}

impl Vector for IVec4 {
    const DIM: usize = 4;
    type Scalar = i32;
}

impl Vector for UVec2 {
    const DIM: usize = 2;
    type Scalar = u32;
}

impl Vector for UVec3 {
    const DIM: usize = 3;
    type Scalar = u32;
}

impl Vector for UVec4 {
    const DIM: usize = 4;
    type Scalar = u32;
}

pub trait FloatParameter {}
impl FloatParameter for f32 {}
impl FloatParameter for Vec2 {}
impl FloatParameter for Vec3 {}
impl FloatParameter for Vec4 {}

pub trait Parameter {}
impl<F> Parameter for F where F: FloatParameter {}
impl Parameter for i32 {}
impl Parameter for u32 {}
impl Parameter for IVec2 {}
impl Parameter for IVec3 {}
impl Parameter for IVec4 {}
impl Parameter for UVec2 {}
impl Parameter for UVec3 {}
impl Parameter for UVec4 {}

pub fn abs<X>(x: X) -> X
where
    X: Parameter,
{
    _ = x;
    panic!()
}

pub fn min<X>(x: X) -> X
where
    X: Parameter,
{
    _ = x;
    panic!()
}

pub fn max<X>(x: X) -> X
where
    X: Parameter,
{
    _ = x;
    panic!()
}

pub fn clamp<X>(e: X, low: X, high: X) -> X
where
    X: Parameter,
{
    _ = (e, low, high);
    panic!()
}

pub fn sin<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn cos<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn tan<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn saturate<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn ceil<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn floor<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn round<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn fract<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn trunc<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn exp<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn exp2<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn log<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn log2<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn pow<X>(x: X, p: X) -> X
where
    X: FloatParameter,
{
    _ = (x, p);
    panic!()
}

pub fn dot<V>(a: V, b: V) -> V::Scalar
where
    V: Vector,
{
    _ = (a, b);
    panic!()
}

pub fn cross<V>(a: V, b: V) -> V
where
    V: Vector,
{
    const {
        assert!(V::DIM == 3, "vector type must have 3 dimensions");
    }

    _ = (a, b);
    panic!()
}

pub fn distance<X>(a: X, b: X) -> f32
where
    X: FloatParameter,
{
    _ = (a, b);
    panic!()
}

pub fn length<X>(x: X) -> f32
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}

pub fn normalize<V>(v: V) -> V
where
    V: Vector<Scalar = f32>,
{
    _ = v;
    panic!()
}

pub fn reflect<V>(a: V, b: V) -> V
where
    V: Vector<Scalar = f32>,
{
    _ = (a, b);
    panic!()
}

pub fn refract<V>(a: V, b: V, i: f32) -> V
where
    V: Vector<Scalar = f32>,
{
    _ = (a, b, i);
    panic!()
}

pub fn sign<X>(x: X) -> X
where
    X: Parameter,
{
    _ = x;
    panic!()
}

pub fn mul_add<X>(x: X, a: X, b: X) -> X
where
    X: FloatParameter,
{
    _ = (x, a, b);
    panic!()
}

pub fn mix<X>(a: X, b: X, t: f32) -> X
where
    X: FloatParameter,
{
    _ = (a, b, t);
    panic!()
}

pub fn mix_vec<X, V>(a: X, b: X, t: V) -> X
where
    X: FloatParameter,
    V: Vector<Scalar = f32>,
{
    _ = (a, b, t);
    panic!()
}

pub fn sqrt<X>(x: X) -> X
where
    X: FloatParameter,
{
    _ = x;
    panic!()
}
