use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        op::Ret,
        types::{StorageReadWrite, Value},
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
    A: Eval<E, Out: Access>,
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

/// An expression that can be dynamically indexed into, like an array.
pub trait Indexable {
    type Member;
}

pub trait SetIndexable: Indexable {}

impl<A, O> Ret<A, O>
where
    O: Indexable,
{
    /// Dynamically index into the target, using a computed u32 index.
    pub fn index<E, Q>(self, idx: Ret<Q, u32>) -> Ret<Lookup<Self, Ret<Q, u32>, E>, O::Member> {
        Ret::new(Lookup::new(idx, self))
    }
}

impl<T, S> Indexable for crate::types::Array<T, S>
where
    T: Value,
{
    type Member = T;
}

impl<T> SetIndexable for crate::types::Array<T, StorageReadWrite> where T: Value {}

pub struct Lookup<A, Q, E> {
    index: Q,
    a: A,
    e: PhantomData<E>,
}

impl<A, Q, E> Lookup<A, Q, E> {
    const fn new(index: Q, a: A) -> Self {
        Self {
            index,
            a,
            e: PhantomData,
        }
    }
}

impl<A, Q, E> Eval<E> for Ret<Lookup<A, Q, E>, <A::Out as Indexable>::Member>
where
    A: Eval<E, Out: Indexable>,
    Q: Eval<E, Out = u32>,
    E: GetEntry,
{
    type Out = <A::Out as Indexable>::Member;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();
        let arr = me.a.eval(en);
        let idx = me.index.eval(en);
        let access = en.get_entry().access(arr, idx);
        en.get_entry().load(access)
    }
}

pub struct SetIndex<A, Q, B, E> {
    index: Q,
    a: A,
    value: B,
    e: PhantomData<E>,
}

impl<A, Q, B, E> SetIndex<A, Q, B, E> {
    const fn new(index: Q, a: A, value: B) -> Self {
        Self {
            index,
            a,
            value,
            e: PhantomData,
        }
    }
}

impl<A, Q, B, E> Eval<E> for Ret<SetIndex<A, Q, B, E>, B::Out>
where
    A: Eval<E, Out: SetIndexable>,
    Q: Eval<E, Out = u32>,
    B: Eval<E, Out = <A::Out as Indexable>::Member>,
    E: GetEntry,
{
    type Out = <A::Out as Indexable>::Member;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();
        let arr = me.a.eval(en);
        let idx = me.index.eval(en);
        let access = en.get_entry().access(arr, idx);
        let val = me.value.eval(en);
        let en = en.get_entry();
        en.store(access, val);
        val
    }
}

impl<A, O> Ret<A, O>
where
    O: SetIndexable,
{
    /// Dynamically index into the target, using a computed u32 index.
    pub fn set_index<E, Q, B>(
        self,
        idx: Ret<Q, u32>,
        val: Ret<B, O::Member>,
    ) -> Ret<SetIndex<Self, Ret<Q, u32>, Ret<B, O::Member>, E>, O::Member> {
        Ret::new(SetIndex::new(idx, self, val))
    }
}
