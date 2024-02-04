use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        ret::Ret,
    },
    std::marker::PhantomData,
};

pub struct Take<A, E> {
    index: u32,
    a: A,
    e: PhantomData<E>,
}

impl<A, E> Take<A, E> {
    const fn new(index: u32, a: A) -> Self {
        Self {
            index,
            a,
            e: PhantomData,
        }
    }
}

impl<A, E> Eval<E> for Ret<Take<A, E>, <A::Out as Access>::Member>
where
    A: Eval<E>,
    A::Out: Access,
    E: GetEntry,
{
    type Out = <A::Out as Access>::Member;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();
        let v = me.a.eval(en);
        en.get_entry().access_index(v, me.index)
    }
}

impl<A, O> Ret<A, O>
where
    O: Access,
{
    pub fn x<E>(self) -> Ret<Take<Self, E>, O::Member>
    where
        O::Dimension: Has<0>,
    {
        Ret::new(Take::new(0, self))
    }

    pub fn y<E>(self) -> Ret<Take<Self, E>, O::Member>
    where
        O::Dimension: Has<1>,
    {
        Ret::new(Take::new(1, self))
    }

    pub fn z<E>(self) -> Ret<Take<Self, E>, O::Member>
    where
        O::Dimension: Has<2>,
    {
        Ret::new(Take::new(2, self))
    }

    pub fn w<E>(self) -> Ret<Take<Self, E>, O::Member>
    where
        O::Dimension: Has<3>,
    {
        Ret::new(Take::new(3, self))
    }
}

pub trait Has<const D: usize> {}

pub struct Dimension<const D: usize>;
impl Has<0> for Dimension<1> {}
impl Has<0> for Dimension<2> {}
impl Has<1> for Dimension<2> {}
impl Has<0> for Dimension<3> {}
impl Has<1> for Dimension<3> {}
impl Has<2> for Dimension<3> {}
impl Has<0> for Dimension<4> {}
impl Has<1> for Dimension<4> {}
impl Has<2> for Dimension<4> {}
impl Has<3> for Dimension<4> {}

pub trait Access {
    type Dimension;
    type Member;
}
