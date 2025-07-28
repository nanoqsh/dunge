use {
    crate::{
        eval::{Eval, Expr, GetEntry, Global},
        op::Ret,
        types,
    },
    std::marker::PhantomData,
};

impl<A, O> Ret<A, O>
where
    O: Access,
{
    #[inline]
    pub fn x<E>(self) -> Ret<IndexGetU32<Self, E>, O::Member>
    where
        O::Dimension: Has<0>,
    {
        Ret::new(IndexGetU32::new(0, self))
    }

    #[inline]
    pub fn y<E>(self) -> Ret<IndexGetU32<Self, E>, O::Member>
    where
        O::Dimension: Has<1>,
    {
        Ret::new(IndexGetU32::new(1, self))
    }

    #[inline]
    pub fn z<E>(self) -> Ret<IndexGetU32<Self, E>, O::Member>
    where
        O::Dimension: Has<2>,
    {
        Ret::new(IndexGetU32::new(2, self))
    }

    #[inline]
    pub fn w<E>(self) -> Ret<IndexGetU32<Self, E>, O::Member>
    where
        O::Dimension: Has<3>,
    {
        Ret::new(IndexGetU32::new(3, self))
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

impl<A, O> Ret<A, types::Pointer<O>> {
    #[inline]
    pub fn deref<E>(self) -> Ret<DerefPointer<Self, E>, O> {
        Ret::new(DerefPointer {
            p: self,
            e: PhantomData,
        })
    }
}

pub struct DerefPointer<P, E> {
    p: P,
    e: PhantomData<E>,
}

impl<P, E, O> Eval<E> for Ret<DerefPointer<P, E>, O>
where
    P: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let DerefPointer { p, .. } = self.inner();
        let ptr = p.eval(en);
        en.get_entry().load(ptr)
    }
}

/// An expression that can be indexed, like an array.
pub trait Indexable {
    type Read;
    type Write;
}

impl<V> Indexable for types::Pointer<V> {
    type Read = Self;
    type Write = V;
}

impl<V> Indexable for types::Vec2<V> {
    type Read = V;
    type Write = V;
}

impl<V> Indexable for types::Vec3<V> {
    type Read = V;
    type Write = V;
}

impl<V> Indexable for types::Vec4<V> {
    type Read = V;
    type Write = V;
}

impl<V, const N: usize, const U: bool> Indexable for types::Array<V, N, U> {
    type Read = types::Pointer<V>;
    type Write = V;
}

impl<V> Indexable for types::DynamicArray<V> {
    type Read = types::Pointer<V>;
    type Write = V;
}

impl<A, O> Ret<A, O> {
    /// Loads a value from an array-like, using *computed* u32 index.
    ///
    /// If the index is known in advance, use the [`get_with_u32`](Ret::get_with_u32) method instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Invocation(v): Invocation, Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set(v.x(), i.get(v.x()).deref()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    #[inline]
    pub fn get<I, E>(self, index: I) -> Ret<IndexGet<I, Self, E>, O::Read>
    where
        O: Indexable,
        I: Eval<E, Out = u32>,
    {
        Ret::new(IndexGet::new(index, self))
    }

    /// Loads a value from an array-like, using *direct* u32 index.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set_with_u32(0, i.get_with_u32(0).deref()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    #[inline]
    pub fn get_with_u32<E>(self, index: u32) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Indexable,
    {
        Ret::new(IndexGetU32::new(index, self))
    }
}

pub struct IndexGet<I, A, E> {
    index: I,
    a: A,
    e: PhantomData<E>,
}

impl<I, A, E> IndexGet<I, A, E> {
    #[inline]
    const fn new(index: I, a: A) -> Self {
        Self {
            index,
            a,
            e: PhantomData,
        }
    }
}

impl<I, A, E, O> Eval<E> for Ret<IndexGet<I, A, E>, O>
where
    I: Eval<E>,
    A: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let me = self.inner();

        let expr = me.a.eval(en);
        let index = me.index.eval(en);
        let en = en.get_entry();
        en.access(expr, index)
    }
}

pub struct IndexGetU32<A, E> {
    index: u32,
    a: A,
    e: PhantomData<E>,
}

impl<A, E> IndexGetU32<A, E> {
    #[inline]
    const fn new(index: u32, a: A) -> Self {
        Self {
            index,
            a,
            e: PhantomData,
        }
    }
}

impl<A, E, O> Eval<E> for Ret<IndexGetU32<A, E>, O>
where
    A: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let me = self.inner();

        let expr = me.a.eval(en);
        let en = en.get_entry();
        en.access_index(expr, me.index)
    }
}

impl<O> Ret<Global<types::Mutable>, O> {
    /// Stores a value to an array-like, using *computed* u32 index.
    ///
    /// If the index is known in advance, use the [`set_with_u32`](Ret::set_with_u32) method instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Invocation(v): Invocation, Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set(v.x(), i.get(v.x()).deref()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    #[inline]
    pub fn set<I, V, E>(self, index: I, value: V) -> Ret<IndexSet<I, Self, V, E>, O::Write>
    where
        O: Indexable,
        I: Eval<E, Out = u32>,
        V: Eval<E, Out = O::Write>,
    {
        Ret::new(IndexSet::new(index, self, value))
    }

    /// Stores a value to an array-like, using *direct* u32 index.
    ///
    /// # Examples
    ///
    /// ```
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set_with_u32(0, i.get_with_u32(0).deref()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    #[inline]
    pub fn set_with_u32<V, E>(self, index: u32, value: V) -> Ret<IndexSetU32<Self, V, E>, O::Write>
    where
        O: Indexable,
        V: Eval<E, Out = O::Write>,
    {
        Ret::new(IndexSetU32::new(index, self, value))
    }
}

pub struct IndexSet<I, A, V, E> {
    index: I,
    a: A,
    v: V,
    e: PhantomData<E>,
}

impl<I, A, V, E> IndexSet<I, A, V, E> {
    #[inline]
    const fn new(index: I, a: A, v: V) -> Self {
        Self {
            index,
            a,
            v,
            e: PhantomData,
        }
    }
}

impl<I, A, V, E, O> Eval<E> for Ret<IndexSet<I, A, V, E>, O>
where
    I: Eval<E>,
    A: Eval<E>,
    V: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let me = self.inner();

        let expr = me.a.eval(en);
        let index = me.index.eval(en);
        let access = en.get_entry().access(expr, index);

        let value = me.v.eval(en);
        let en = en.get_entry();
        en.store(access, value);
        value
    }
}

pub struct IndexSetU32<A, V, E> {
    index: u32,
    a: A,
    v: V,
    e: PhantomData<E>,
}

impl<A, V, E> IndexSetU32<A, V, E> {
    #[inline]
    const fn new(index: u32, a: A, v: V) -> Self {
        Self {
            index,
            a,
            v,
            e: PhantomData,
        }
    }
}

impl<A, V, E, O> Eval<E> for Ret<IndexSetU32<A, V, E>, O>
where
    A: Eval<E>,
    V: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let me = self.inner();

        let expr = me.a.eval(en);
        let access = en.get_entry().access_index(expr, me.index);

        let value = me.v.eval(en);
        let en = en.get_entry();
        en.store(access, value);
        value
    }
}
