use {
    crate::{
        eval::{Eval, Expr, GetEntry, Global},
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
    type Member: types::Value;
}

impl<V, const N: usize> Indexable for types::Array<V, N>
where
    V: types::Value,
{
    type Member = V;
}

impl<V> Indexable for types::DynamicArray<V>
where
    V: types::Value,
{
    type Member = V;
}

impl<A, O> Ret<A, O>
where
    O: Indexable,
{
    /// Loads a value from an array, using a computed u32 index.
    pub fn load<I, E>(self, index: I) -> Ret<IndexLoad<I, Self, E>, O::Member>
    where
        I: Eval<E, Out = u32>,
    {
        Ret::new(IndexLoad::new(index, self))
    }
}

impl<O> Ret<Global<types::Mutable>, O>
where
    O: Indexable,
{
    /// Stores a value to an array, using a computed u32 index.
    pub fn store<I, V, E>(self, index: I, value: V) -> Ret<IndexStore<I, Self, V, E>, O::Member>
    where
        I: Eval<E, Out = u32>,
        V: Eval<E, Out = O::Member>,
    {
        Ret::new(IndexStore::new(index, self, value))
    }
}

pub struct IndexLoad<I, A, E> {
    index: I,
    array: A,
    e: PhantomData<E>,
}

impl<I, A, E> IndexLoad<I, A, E> {
    const fn new(index: I, array: A) -> Self {
        Self {
            index,
            array,
            e: PhantomData,
        }
    }
}

impl<I, A, E> Eval<E> for Ret<IndexLoad<I, A, E>, <A::Out as Indexable>::Member>
where
    I: Eval<E, Out = u32>,
    A: Eval<E, Out: Indexable>,
    E: GetEntry,
{
    type Out = <A::Out as Indexable>::Member;

    fn eval(self, en: &mut E) -> Expr {
        let me = self.get();
        let array = me.array.eval(en);
        let index = me.index.eval(en);

        let en = en.get_entry();
        let access = en.access(array, index);

        if const { types::indirect_load::<Self::Out>() } {
            en.load(access)
        } else {
            access
        }
    }
}

pub struct IndexStore<I, A, V, E> {
    index: I,
    array: A,
    value: V,
    e: PhantomData<E>,
}

impl<I, A, V, E> IndexStore<I, A, V, E> {
    const fn new(index: I, array: A, value: V) -> Self {
        Self {
            index,
            array,
            value,
            e: PhantomData,
        }
    }
}

impl<I, A, V, E> Eval<E> for Ret<IndexStore<I, A, V, E>, V::Out>
where
    I: Eval<E, Out = u32>,
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
