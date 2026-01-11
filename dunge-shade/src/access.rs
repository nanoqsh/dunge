use {
    crate::{
        eval::{Eval, Expr, GetEntry},
        op::Ret,
        types,
    },
    std::marker::PhantomData,
};

impl<A, O> Ret<A, O> {
    pub fn x<E>(self) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Access<Dimension: Has<0>>,
    {
        Ret::new(IndexGetU32::new(0, self))
    }

    pub fn y<E>(self) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Access<Dimension: Has<1>>,
    {
        Ret::new(IndexGetU32::new(1, self))
    }

    pub fn z<E>(self) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Access<Dimension: Has<2>>,
    {
        Ret::new(IndexGetU32::new(2, self))
    }

    pub fn w<E>(self) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Access<Dimension: Has<3>>,
    {
        Ret::new(IndexGetU32::new(3, self))
    }

    /// Loads a value from an array-like, using *computed* u32 index.
    ///
    /// If the index is known in advance, use the [`get_u32`](Ret::get_u32) method instead.
    ///
    /// # Examples
    ///
    #[cfg_attr(doctest, doc = "```ignore")]
    #[cfg_attr(not(doctest), doc = "```")]
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Invocation(v): Invocation, Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set(v.x(), i.get(v.x()).load()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    pub const fn get<I, E>(self, index: I) -> Ret<IndexGet<I, Self, E>, O::Read>
    where
        O: Access,
        I: Eval<E, Out = u32>,
    {
        Ret::new(IndexGet::new(index, self))
    }

    /// Loads a value from an array-like, using *direct* u32 index.
    ///
    /// # Examples
    ///
    #[cfg_attr(doctest, doc = "```ignore")]
    #[cfg_attr(not(doctest), doc = "```")]
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set_u32(0, i.get_u32(0).load()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    pub const fn get_u32<E>(self, index: u32) -> Ret<IndexGetU32<Self, E>, O::Read>
    where
        O: Access,
    {
        Ret::new(IndexGetU32::new(index, self))
    }
}

impl<A, O> Ret<A, types::Pointer<O>> {
    /// Loads the value by pointer.
    ///
    /// # Examples
    ///
    #[cfg_attr(doctest, doc = "```ignore")]
    #[cfg_attr(not(doctest), doc = "```")]
    /// use dunge::{
    ///     sl::{Compute, Groups, Invocation},
    ///     storage::{Storage, RwStorage},
    /// };
    ///
    /// type Io = (Storage<[f32; 4]>, RwStorage<[f32; 4]>);
    ///
    /// let code = |Groups((i, o)): Groups<Io>| Compute {
    ///     compute: o.set_u32(0, i.get_u32(0).load()),
    ///     workgroup_size: [1; 3],
    /// };
    /// ```
    pub fn load<E>(self) -> Ret<Load<Self, E>, O> {
        Ret::new(Load {
            p: self,
            e: PhantomData,
        })
    }
}

pub struct Load<P, E> {
    p: P,
    e: PhantomData<E>,
}

impl<P, E, O> Eval<E> for Ret<Load<P, E>, O>
where
    P: Eval<E>,
    E: GetEntry,
{
    type Out = O;

    #[inline]
    fn eval(self, en: &mut E) -> Expr {
        let Load { p, .. } = self.inner();
        let ptr = p.eval(en);
        en.get_entry().load(ptr)
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

/// An expression that can be indexed, like an array.
pub trait Access {
    type Dimension;
    type Read;
    type Write;
}

impl<V> Access for types::Pointer<V>
where
    V: Access,
{
    type Dimension = ();
    type Read = types::Pointer<V::Read>;
    type Write = V::Write;
}

impl<V> Access for types::Vec2<V> {
    type Dimension = Dimension<2>;
    type Read = V;
    type Write = V;
}

impl<V> Access for types::Vec3<V> {
    type Dimension = Dimension<3>;
    type Read = V;
    type Write = V;
}

impl<V> Access for types::Vec4<V> {
    type Dimension = Dimension<4>;
    type Read = V;
    type Write = V;
}

impl Access for types::Mat2 {
    type Dimension = Dimension<2>;
    type Read = types::Vec2<f32>;
    type Write = types::Vec2<f32>;
}

impl Access for types::Mat3 {
    type Dimension = Dimension<3>;
    type Read = types::Vec3<f32>;
    type Write = types::Vec3<f32>;
}

impl Access for types::Mat4 {
    type Dimension = Dimension<4>;
    type Read = types::Vec4<f32>;
    type Write = types::Vec4<f32>;
}

impl<V, const N: usize, const U: bool> Access for types::Array<V, N, U> {
    type Dimension = ();
    type Read = types::Pointer<V>;
    type Write = V;
}

impl<V> Access for types::DynamicArray<V> {
    type Dimension = ();
    type Read = types::Pointer<V>;
    type Write = V;
}

pub struct IndexGet<I, A, E> {
    index: I,
    a: A,
    e: PhantomData<E>,
}

impl<I, A, E> IndexGet<I, A, E> {
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
