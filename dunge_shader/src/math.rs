use {
    crate::{
        eval::{Eval, EvalTuple, Evaluated, Expr, GetEntry},
        op::Ret,
        types::Number,
    },
    naga::{Expression, MathFunction},
    std::marker::PhantomData,
};

pub const fn abs<X, E>(x: X) -> Ret<Math<(X,), E>, X::Out>
where
    X: Eval<E>,
    X::Out: Number,
{
    Ret::new(Math::new((x,), Func(MathFunction::Abs)))
}

pub const fn acos<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Acos)))
}

pub const fn acosh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Acosh)))
}

pub const fn asin<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Asin)))
}

pub const fn asinh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Asinh)))
}

pub const fn atan<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Atan)))
}

pub const fn atanh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Atanh)))
}

pub const fn ceil<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Ceil)))
}

pub const fn cos<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Cos)))
}

pub const fn cosh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Cosh)))
}

pub const fn floor<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Floor)))
}

pub const fn sin<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Sin)))
}

pub const fn sinh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Sinh)))
}

pub const fn tan<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Tan)))
}

pub const fn tanh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func(MathFunction::Tanh)))
}

pub struct Math<A, E> {
    args: A,
    func: Func,
    e: PhantomData<E>,
}

impl<A, E> Math<A, E> {
    const fn new(args: A, func: Func) -> Self {
        Self {
            args,
            func,
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

    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        let Math { args, func, .. } = self.get();
        args.eval(en, &mut o);
        en.get_entry().math(func, o)
    }
}

pub(crate) struct Func(MathFunction);

impl Func {
    pub fn expr(self, ev: Evaluated) -> Expression {
        let mut exprs = ev.into_iter().map(Expr::get);
        Expression::Math {
            fun: self.0,
            arg: exprs.next().expect("first argument"),
            arg1: exprs.next(),
            arg2: exprs.next(),
            arg3: exprs.next(),
        }
    }
}
