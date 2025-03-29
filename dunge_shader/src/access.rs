use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        op::Ret,
        types,
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

impl<T> Indexable for types::Array<T>
where
    T: types::Value,
{
    type Member = T;
}

impl<A, O> Ret<A, O>
where
    O: Indexable,
{
    /// Dynamically loads a value from an array, using a computed u32 index.
    pub fn load<Q, E>(self, idx: Q) -> Ret<IndexLoad<Q, Self, E>, O::Member>
    where
        Q: Eval<E, Out = u32>,
    {
        Ret::new(IndexLoad::new(idx, self))
    }
}

pub struct IndexLoad<Q, A, E> {
    index: Q,
    array: A,
    e: PhantomData<E>,
}

impl<Q, A, E> IndexLoad<Q, A, E> {
    const fn new(index: Q, array: A) -> Self {
        Self {
            index,
            array,
            e: PhantomData,
        }
    }
}

impl<Q, A, E> Eval<E> for Ret<IndexLoad<Q, A, E>, <A::Out as Indexable>::Member>
where
    Q: Eval<E, Out = u32>,
    A: Eval<E, Out: Indexable>,
    E: GetEntry,
{
    type Out = <A::Out as Indexable>::Member;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();
        let array = me.array.eval(en);
        let index = me.index.eval(en);
        let access = en.get_entry().access(array, index);
        en.get_entry().load(access)
    }
}

pub struct IndexStore<Q, A, V, E> {
    index: Q,
    array: A,
    value: V,
    e: PhantomData<E>,
}

impl<Q, A, V, E> IndexStore<Q, A, V, E> {
    #[expect(dead_code)]
    const fn new(index: Q, array: A, value: V) -> Self {
        Self {
            index,
            array,
            value,
            e: PhantomData,
        }
    }
}

impl<Q, A, V, E> Eval<E> for Ret<IndexStore<Q, A, V, E>, V::Out>
where
    Q: Eval<E, Out = u32>,
    A: Eval<E, Out: Indexable<Member = V::Out>>,
    V: Eval<E>,
    E: GetEntry,
{
    type Out = V::Out;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();

        let array = me.array.eval(en);
        let index = me.index.eval(en);
        let access = en.get_entry().access(array, index);

        let value = me.value.eval(en);
        let en = en.get_entry();
        en.store(access, value);
        value
    }
}
