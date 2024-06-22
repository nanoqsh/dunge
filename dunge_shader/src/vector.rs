use {
    crate::{
        access::{Access, Dimension},
        eval::{Eval, EvalTuple, Evaluated, Expr, Exprs, GetEntry},
        op::Ret,
        types::{self, Scalar, Vector},
    },
    std::marker::PhantomData,
};

macro_rules! impl_eval_vec {
    ($g:ty => $t:ty) => {
        impl<E> Eval<E> for $g
        where
            E: GetEntry,
        {
            type Out = $t;

            fn eval(self, en: &mut E) -> Expr {
                let mut components = Vec::with_capacity(<$t>::TYPE.dims());
                self.into_vector(|scalar| {
                    let v = scalar.eval(en).get();
                    components.push(v);
                });

                let en = en.get_entry();
                let ty = en.new_type(<$t>::TYPE.ty());
                en.compose(ty, Exprs(components))
            }
        }
    };
}

impl_eval_vec!(glam::Vec2 => types::Vec2<f32>);
impl_eval_vec!(glam::Vec3 => types::Vec3<f32>);
impl_eval_vec!(glam::Vec3A => types::Vec3<f32>);
impl_eval_vec!(glam::Vec4 => types::Vec4<f32>);
impl_eval_vec!(glam::IVec2 => types::Vec2<i32>);
impl_eval_vec!(glam::IVec3 => types::Vec3<i32>);
impl_eval_vec!(glam::IVec4 => types::Vec4<i32>);
impl_eval_vec!(glam::UVec2 => types::Vec2<u32>);
impl_eval_vec!(glam::UVec3 => types::Vec3<u32>);
impl_eval_vec!(glam::UVec4 => types::Vec4<u32>);

impl<S> Access for types::Vec2<S>
where
    S: Scalar,
{
    type Dimension = Dimension<2>;
    type Member = S;
}

impl<S> Access for types::Vec3<S>
where
    S: Scalar,
{
    type Dimension = Dimension<3>;
    type Member = S;
}

impl<S> Access for types::Vec4<S>
where
    S: Scalar,
{
    type Dimension = Dimension<4>;
    type Member = S;
}

pub const fn splat_vec2<A, E>(a: A) -> Ret<Splat<A, E>, types::Vec2<A::Out>>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(Splat::new(a))
}

pub const fn splat_vec3<A, E>(a: A) -> Ret<Splat<A, E>, types::Vec3<A::Out>>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(Splat::new(a))
}

pub const fn splat_vec4<A, E>(a: A) -> Ret<Splat<A, E>, types::Vec4<A::Out>>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(Splat::new(a))
}

pub struct Splat<A, E> {
    a: A,
    e: PhantomData<E>,
}

impl<A, E> Splat<A, E> {
    const fn new(a: A) -> Self {
        Self { a, e: PhantomData }
    }
}

impl<A, O, E> Eval<E> for Ret<Splat<A, E>, O>
where
    A: Eval<E>,
    O: Vector,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let val = self.get().a.eval(en);
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        let components = (0..O::TYPE.dims()).map(|_| val).collect();
        en.compose(ty, components)
    }
}

type Vector2<X, Y, O, E> = Ret<NewVec<(X, Y), E>, types::Vec2<O>>;

pub const fn vec2<X, Y, E>(x: X, y: Y) -> Vector2<X, Y, X::Out, E>
where
    X: Eval<E, Out: Scalar>,
    Y: Eval<E, Out = X::Out>,
{
    Ret::new(NewVec::new((x, y)))
}

type Vector3<X, Y, Z, O, E> = Ret<NewVec<(X, Y, Z), E>, types::Vec3<O>>;

pub const fn vec3<X, Y, Z, E>(x: X, y: Y, z: Z) -> Vector3<X, Y, Z, X::Out, E>
where
    X: Eval<E, Out: Scalar>,
    Y: Eval<E, Out = X::Out>,
    Z: Eval<E, Out = X::Out>,
{
    Ret::new(NewVec::new((x, y, z)))
}

type Vector4<X, Y, Z, W, O, E> = Ret<NewVec<(X, Y, Z, W), E>, types::Vec4<O>>;

pub const fn vec4<X, Y, Z, W, E>(x: X, y: Y, z: Z, w: W) -> Vector4<X, Y, Z, W, X::Out, E>
where
    X: Eval<E, Out: Scalar>,
    Y: Eval<E, Out = X::Out>,
    Z: Eval<E, Out = X::Out>,
    W: Eval<E, Out = X::Out>,
{
    Ret::new(NewVec::new((x, y, z, w)))
}

pub struct NewVec<A, E> {
    a: A,
    e: PhantomData<E>,
}

impl<A, E> NewVec<A, E> {
    const fn new(a: A) -> Self {
        Self { a, e: PhantomData }
    }
}

impl<A, O, E> Eval<E> for Ret<NewVec<A, E>, O>
where
    A: EvalTuple<E>,
    O: Vector,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        self.get().a.eval(en, &mut o);
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        let components = o.into_iter().collect();
        en.compose(ty, components)
    }
}

pub const fn vec4_concat<A, B, S, E>(a: A, b: B) -> Ret<Compose<A, B>, types::Vec4<S>>
where
    A: Eval<E, Out = types::Vec2<S>>,
    B: Eval<E, Out = types::Vec2<S>>,
    S: Scalar,
{
    Ret::new(Compose { a, b })
}

pub const fn vec3_with<A, B, S, E>(a: A, b: B) -> Ret<Compose<A, B>, types::Vec3<B::Out>>
where
    A: Eval<E, Out = types::Vec2<B::Out>>,
    B: Eval<E, Out: Scalar>,
{
    Ret::new(Compose { a, b })
}

pub const fn vec4_with<A, B, E>(a: A, b: B) -> Ret<Compose<A, B>, types::Vec4<B::Out>>
where
    A: Eval<E, Out = types::Vec3<B::Out>>,
    B: Eval<E, Out: Scalar>,
{
    Ret::new(Compose { a, b })
}

pub struct Compose<A, B> {
    a: A,
    b: B,
}

impl<A, B, O, E> Eval<E> for Ret<Compose<A, B>, O>
where
    A: Eval<E>,
    B: Eval<E>,
    O: types::Vector,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Compose { a, b } = self.get();
        let x = a.eval(en).get();
        let y = b.eval(en).get();
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        en.compose(ty, Exprs(vec![x, y]))
    }
}

trait IntoVector {
    type Scalar;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar);
}

impl IntoVector for glam::Vec2 {
    type Scalar = f32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::Vec3 {
    type Scalar = f32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::Vec3A {
    type Scalar = f32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::Vec4 {
    type Scalar = f32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::IVec2 {
    type Scalar = i32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::IVec3 {
    type Scalar = i32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::IVec4 {
    type Scalar = i32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::UVec2 {
    type Scalar = u32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::UVec3 {
    type Scalar = u32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}

impl IntoVector for glam::UVec4 {
    type Scalar = u32;

    fn into_vector<F>(self, f: F)
    where
        F: FnMut(Self::Scalar),
    {
        self.to_array().map(f);
    }
}
