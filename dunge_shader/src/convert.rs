use crate::{
    eval::{Eval, Expr, GetEntry},
    ret::Ret,
    types::Scalar,
};

pub const fn f32<A, E>(a: A) -> Ret<As<A>, f32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(As(a))
}

pub const fn i32<A, E>(a: A) -> Ret<As<A>, i32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(As(a))
}

pub const fn u32<A, E>(a: A) -> Ret<As<A>, u32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(As(a))
}

pub const fn bool<A, E>(a: A) -> Ret<As<A>, bool>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    Ret::new(As(a))
}

pub struct As<A>(A);

impl<A, O, E> Eval<E> for Ret<As<A>, O>
where
    A: Eval<E>,
    O: Scalar,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let v = self.get().0.eval(en);
        en.get_entry().convert(v, O::TYPE)
    }
}
