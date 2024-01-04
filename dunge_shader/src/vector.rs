use crate::{
    eval::{Eval, EvalTuple, Evaluated, Expr, Exprs, GetEntry},
    ret::Ret,
    types::{self, Scalar, Vector},
};

pub const fn splat_vec2<A, E>(a: A) -> Ret<Splat<A>, types::Vec2<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(Splat(a))
}

pub const fn splat_vec3<A, E>(a: A) -> Ret<Splat<A>, types::Vec3<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(Splat(a))
}

pub const fn splat_vec4<A, E>(a: A) -> Ret<Splat<A>, types::Vec4<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(Splat(a))
}

pub struct Splat<A>(A);

impl<A, O, E> Eval<E> for Ret<Splat<A>, O>
where
    A: Eval<E>,
    O: Vector,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let val = self.get().0.eval(en);
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        let components = (0..O::TYPE.dims()).map(|_| val).collect();
        en.compose(ty, components)
    }
}

type Vector2<X, Y, O> = Ret<New<(X, Y)>, types::Vec2<O>>;

pub const fn vec2<X, Y, E>(x: X, y: Y) -> Vector2<X, Y, X::Out>
where
    X: Eval<E>,
    X::Out: Scalar,
    Y: Eval<E, Out = X::Out>,
{
    Ret::new(New((x, y)))
}

type Vector3<X, Y, Z, O> = Ret<New<(X, Y, Z)>, types::Vec3<O>>;

pub const fn vec3<X, Y, Z, E>(x: X, y: Y, z: Z) -> Vector3<X, Y, Z, X::Out>
where
    X: Eval<E>,
    X::Out: Scalar,
    Y: Eval<E, Out = X::Out>,
    Z: Eval<E, Out = X::Out>,
{
    Ret::new(New((x, y, z)))
}

type Vector4<X, Y, Z, W, O> = Ret<New<(X, Y, Z, W)>, types::Vec4<O>>;

pub const fn vec4<X, Y, Z, W, E>(x: X, y: Y, z: Z, w: W) -> Vector4<X, Y, Z, W, X::Out>
where
    X: Eval<E>,
    X::Out: Scalar,
    Y: Eval<E, Out = X::Out>,
    Z: Eval<E, Out = X::Out>,
    W: Eval<E, Out = X::Out>,
{
    Ret::new(New((x, y, z, w)))
}

pub struct New<A>(A);

impl<A, O, E> Eval<E> for Ret<New<A>, O>
where
    A: EvalTuple<E>,
    O: Vector,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        self.get().0.eval(en, &mut o);
        let en = en.get_entry();
        let ty = en.new_type(O::TYPE.ty());
        let components = o.into_iter().collect();
        en.compose(ty, components)
    }
}

pub const fn concat<A, B, S, E>(a: A, b: B) -> Ret<Compose<A, B>, types::Vec4<S>>
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
    B: Eval<E>,
    B::Out: Scalar,
{
    Ret::new(Compose { a, b })
}

pub const fn vec4_with<A, B, E>(a: A, b: B) -> Ret<Compose<A, B>, types::Vec4<B::Out>>
where
    A: Eval<E, Out = types::Vec3<B::Out>>,
    B: Eval<E>,
    B::Out: Scalar,
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
