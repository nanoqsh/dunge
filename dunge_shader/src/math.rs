use {
    crate::{
        eval::{Eval, EvalTuple, Evaluated, Expr, GetEntry},
        op::Ret,
        types,
    },
    std::marker::PhantomData,
};

#[inline]
pub const fn abs<X, E>(x: X) -> Ret<Math<(X,), E>, X::Out>
where
    X: Eval<E, Out: types::Number>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Abs))
}

#[inline]
pub const fn acos<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Acos))
}

#[inline]
pub const fn acosh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Acosh))
}

#[inline]
pub const fn asin<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Asin))
}

#[inline]
pub const fn asinh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Asinh))
}

#[inline]
pub const fn atan<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Atan))
}

#[inline]
pub const fn atanh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Atanh))
}

#[inline]
pub const fn atan2<Y, X, E>(y: Y, x: X) -> Ret<Math<(Y, X), E>, f32>
where
    Y: Eval<E, Out = f32>,
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((y, x), naga::MathFunction::Atan2))
}

#[inline]
pub const fn ceil<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Ceil))
}

#[inline]
pub const fn clamp<X, L, H, E>(x: X, lo: L, hi: H) -> Ret<Math<(X, L, H), E>, f32>
where
    X: Eval<E, Out: types::Number>,
    L: Eval<E, Out = X::Out>,
    H: Eval<E, Out = X::Out>,
{
    Ret::new(Math::new((x, lo, hi), naga::MathFunction::Clamp))
}

#[inline]
pub const fn cos<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Cos))
}

#[inline]
pub const fn cosh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Cosh))
}

#[inline]
pub const fn cross<X, Y, E>(x: X, y: Y) -> Ret<Math<(X, Y), E>, types::Vec3<f32>>
where
    X: Eval<E, Out = types::Vec3<f32>>,
    Y: Eval<E, Out = types::Vec3<f32>>,
{
    Ret::new(Math::new((x, y), naga::MathFunction::Cross))
}

#[allow(clippy::type_complexity)]
#[inline]
pub const fn dot<X, Y, E>(x: X, y: Y) -> Ret<Math<(X, Y), E>, <X::Out as types::Vector>::Scalar>
where
    X: Eval<E, Out: types::Vector<Scalar: types::Number>>,
    Y: Eval<E, Out = X::Out>,
{
    Ret::new(Math::new((x, y), naga::MathFunction::Dot))
}

#[inline]
pub const fn exp<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Exp))
}

#[inline]
pub const fn exp2<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Exp2))
}

#[inline]
pub const fn floor<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Floor))
}

#[inline]
pub const fn log<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Log))
}

#[inline]
pub const fn log2<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Log2))
}

#[inline]
pub const fn pow<B, X, E>(base: B, exp: X) -> Ret<Math<(B, X), E>, f32>
where
    B: Eval<E, Out = f32>,
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((base, exp), naga::MathFunction::Pow))
}

#[inline]
pub const fn sin<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Sin))
}

#[inline]
pub const fn sinh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Sinh))
}

#[inline]
pub const fn sqrt<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Sqrt))
}

#[inline]
pub const fn tan<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Tan))
}

#[inline]
pub const fn tanh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), naga::MathFunction::Tanh))
}

pub struct Math<A, E> {
    args: A,
    func: Func,
    e: PhantomData<E>,
}

impl<A, E> Math<A, E> {
    #[inline]
    const fn new(args: A, func: naga::MathFunction) -> Self {
        Self {
            args,
            func: Func(func),
            e: PhantomData,
        }
    }
}

impl<A, O, E> Eval<E> for Ret<Math<A, E>, O>
where
    A: EvalTuple<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        let Math { args, func, .. } = self.inner();
        args.eval(en, &mut o);
        en.get_entry().math(func, o)
    }
}

pub(crate) struct Func(naga::MathFunction);

impl Func {
    #[inline]
    pub(crate) fn expr(self, ev: Evaluated) -> naga::Expression {
        let mut exprs = ev.into_iter().map(Expr::get);
        naga::Expression::Math {
            fun: self.0,
            arg: exprs.next().expect("first argument"),
            arg1: exprs.next(),
            arg2: exprs.next(),
            arg3: exprs.next(),
        }
    }
}
