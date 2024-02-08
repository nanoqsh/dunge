use {
    crate::{
        eval::{Eval, EvalTuple, Evaluated, Expr, GetEntry},
        ret::Ret,
    },
    naga::{Expression, MathFunction},
    std::marker::PhantomData,
};

pub const fn cos<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Cos))
}

pub const fn cosh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Cosh))
}

pub const fn sin<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Sin))
}

pub const fn sinh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Sinh))
}

pub const fn tan<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Tan))
}

pub const fn tanh<X, E>(x: X) -> Ret<Math<(X,), E>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math::new((x,), Func::Tanh))
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

pub(crate) enum Func {
    Cos,
    Cosh,
    Sin,
    Sinh,
    Tan,
    Tanh,
}

impl Func {
    pub fn expr(self, ev: Evaluated) -> Expression {
        let fun = match self {
            Self::Cos => MathFunction::Cos,
            Self::Cosh => MathFunction::Cosh,
            Self::Sin => MathFunction::Sin,
            Self::Sinh => MathFunction::Sinh,
            Self::Tan => MathFunction::Tan,
            Self::Tanh => MathFunction::Tanh,
        };

        let mut exprs = ev.into_iter().map(Expr::get);
        Expression::Math {
            fun,
            arg: exprs.next().expect("first argument"),
            arg1: exprs.next(),
            arg2: exprs.next(),
            arg3: exprs.next(),
        }
    }
}
