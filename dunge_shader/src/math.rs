use crate::{
    eval::{Eval, EvalTuple, Evaluated, Expr, Func, GetEntry},
    ret::Ret,
};

pub fn cos<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Cos,
    })
}

pub fn cosh<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Cosh,
    })
}

pub fn sin<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Sin,
    })
}

pub fn sinh<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Sinh,
    })
}

pub fn tan<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Tan,
    })
}

pub fn tanh<X, E>(x: X) -> Ret<Math<(X,)>, f32>
where
    X: Eval<E, Out = f32>,
{
    Ret::new(Math {
        args: (x,),
        func: Func::Tanh,
    })
}

pub struct Math<A> {
    args: A,
    func: Func,
}

impl<A, O, E> Eval<E> for Ret<Math<A>, O>
where
    A: EvalTuple<E>,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let mut o = Evaluated::default();
        let Math { args, func } = self.get();
        args.eval(en, &mut o);
        en.get_entry().math(func, o)
    }
}
