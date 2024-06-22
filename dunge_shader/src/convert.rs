use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        op::Ret,
        types::Scalar,
    },
    std::marker::PhantomData,
};

pub const fn f32<A, E>(a: A) -> Ret<As<A, E>, f32>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(As::new(a))
}

pub const fn i32<A, E>(a: A) -> Ret<As<A, E>, i32>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(As::new(a))
}

pub const fn u32<A, E>(a: A) -> Ret<As<A, E>, u32>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(As::new(a))
}

pub const fn bool<A, E>(a: A) -> Ret<As<A, E>, bool>
where
    A: Eval<E, Out: Scalar>,
{
    Ret::new(As::new(a))
}

pub struct As<A, E> {
    a: A,
    e: PhantomData<E>,
}

impl<A, E> As<A, E> {
    const fn new(a: A) -> Self {
        Self { a, e: PhantomData }
    }
}

impl<A, O, E> Eval<E> for Ret<As<A, E>, O>
where
    A: Eval<E>,
    O: Scalar,
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let v = self.get().a.eval(en);
        en.get_entry().convert(v, O::TYPE)
    }
}
