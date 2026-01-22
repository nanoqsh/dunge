use {
    crate::{
        bytes::{self, Bytes},
        irc::{
            Composite, Fields, Fnc, GlobalVariable, GroupMember, Input, InputKind, Irc,
            MaybeSizedValue, Method, Methods, Reference, Value,
        },
        module::GroupFormat,
    },
    std::{marker::PhantomData, num::NonZeroU32, ops, slice},
};

#[allow(clippy::len_without_is_empty)]
pub trait Data {
    type Context;
    fn update(&self, cx: &Self::Context, bytes: &[u8]);
    fn byte_size(&self) -> u64;
    fn len(&self) -> NonZeroU32;
}

#[derive(Clone)]
pub struct Uniform<V, D> {
    data: D,
    ty: PhantomData<V>,
}

impl<V, D> Uniform<V, D> {
    pub fn read(&self) -> V {
        panic!()
    }
}

impl<V, D> Uniform<V, D>
where
    D: Data,
{
    /// Updates the uniform data.
    ///
    /// # Panics
    ///
    /// Panics if the buffer size is not equal to the size of the new value.
    pub fn update(&self, cx: &D::Context, new: &V)
    where
        V: Bytes,
    {
        self.data.update(cx, bytes::as_bytes(slice::from_ref(new)));
    }

    pub fn byte_size(&self) -> u64 {
        self.data.byte_size()
    }
}

impl<V, D> Uniform<V, D>
where
    D: Data,
    Self: Composite,
{
    pub fn len(&self) -> NonZeroU32 {
        self.data.len()
    }
}

impl<V, D, const N: usize> ops::Index<u32> for Uniform<[V; N], D> {
    type Output = V;

    fn index(&self, _: u32) -> &Self::Output {
        panic!()
    }
}

impl<V, D, const N: usize> Composite for Uniform<[V; N], D> {
    type Output = V;
}

impl<V, D> Input for Uniform<V, D>
where
    V: Value,
{
    const KIND: InputKind = InputKind::Group;
    type Ref = V;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<V> + use<V, D> {
        let global = fnc.irc().add_uniform(0);
        fnc.irc().new_group();
        global
    }
}

impl<V, D> GroupMember for Uniform<V, D>
where
    V: Value,
{
    const FORMAT: GroupFormat = GroupFormat::Uniform;
    type Global = V;

    fn global(irc: &mut Irc, binding: u32) -> GlobalVariable<Self::Global> {
        irc.add_uniform(binding)
    }
}

pub struct UniformFields {}

impl<V, D> Fields for Uniform<V, D> {
    type Tuple = (Self,);
    type Fields = UniformFields;

    const FIELDS: Self::Fields = UniformFields {};
}

impl<V, D> Methods for Uniform<V, D> {
    type Methods = UniformMethods<V, D>;

    const METHODS: UniformMethods<V, D> = UniformMethods { read: Method::Noop };
}

pub struct UniformMethods<V, D> {
    pub read: Method<Uniform<V, D>, V>,
}

pub trait StorageValue: MaybeSizedValue {
    fn storage_bytes(&self) -> &[u8];
}

impl<V> StorageValue for V
where
    V: Value + Bytes,
{
    fn storage_bytes(&self) -> &[u8] {
        bytes::as_bytes(slice::from_ref(self))
    }
}

impl<V> StorageValue for [V]
where
    V: Value + Bytes,
{
    fn storage_bytes(&self) -> &[u8] {
        bytes::as_bytes(self)
    }
}

#[derive(Clone)]
pub struct Storage<V, D>
where
    V: ?Sized,
{
    data: D,
    ty: PhantomData<V>,
}

impl<V, D> Storage<V, D> {
    pub fn read(&self) -> V {
        panic!()
    }
}

impl<V, D> Storage<V, D>
where
    V: ?Sized,
    D: Data,
{
    /// Updates the storage data.
    ///
    /// # Panics
    ///
    /// Panics if the buffer size is not equal to the size of the new value.
    pub fn update(&self, cx: &D::Context, new: &V)
    where
        V: StorageValue,
    {
        self.data.update(cx, new.storage_bytes());
    }

    pub fn byte_size(&self) -> u64 {
        self.data.byte_size()
    }
}

impl<V, D> Storage<V, D>
where
    D: Data,
    Self: Composite,
{
    pub fn len(&self) -> NonZeroU32 {
        self.data.len()
    }
}

impl<V, D, const N: usize> ops::Index<u32> for Storage<[V; N], D> {
    type Output = V;

    fn index(&self, _: u32) -> &Self::Output {
        panic!()
    }
}

impl<V, D, const N: usize> Composite for Storage<[V; N], D> {
    type Output = V;
}

impl<V, D> ops::Index<u32> for Storage<[V], D> {
    type Output = V;

    fn index(&self, _: u32) -> &Self::Output {
        panic!()
    }
}

impl<V, D> Composite for Storage<[V], D> {
    type Output = V;
}

impl<V, D> Input for Storage<V, D>
where
    V: StorageValue + ?Sized,
{
    const KIND: InputKind = InputKind::Group;
    type Ref = V;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<V> + use<V, D> {
        let global = fnc.irc().add_storage(0);
        fnc.irc().new_group();
        global
    }
}

impl<V, D> GroupMember for Storage<V, D>
where
    V: StorageValue + ?Sized,
{
    const FORMAT: GroupFormat = GroupFormat::Storage;
    type Global = V;

    fn global(irc: &mut Irc, binding: u32) -> GlobalVariable<Self::Global> {
        irc.add_storage(binding)
    }
}

pub struct StorageFields {}

impl<V, D> Fields for Storage<V, D>
where
    V: ?Sized,
{
    type Tuple = (Self,);
    type Fields = StorageFields;

    const FIELDS: Self::Fields = StorageFields {};
}

impl<V, D> Methods for Storage<V, D>
where
    V: ?Sized,
{
    type Methods = StorageMethods<V, D>;

    const METHODS: StorageMethods<V, D> = StorageMethods { read: Method::Noop };
}

pub struct StorageMethods<V, D>
where
    V: ?Sized,
{
    pub read: Method<Storage<V, D>, V>,
}

#[derive(Clone)]
pub struct Row<V, D> {
    data: D,
    ty: PhantomData<[V]>,
}

impl<V, D> Row<V, D>
where
    D: Data,
{
    /// Updates the row data.
    ///
    /// # Panics
    ///
    /// Panics if the buffer size is not equal to the size of the new value.
    pub fn update(&self, cx: &D::Context, new: &[V])
    where
        V: Bytes,
    {
        self.data.update(cx, bytes::as_bytes(new));
    }

    pub fn byte_size(&self) -> u64 {
        self.data.byte_size()
    }

    pub fn len(&self) -> NonZeroU32 {
        self.data.len()
    }
}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub fn uniform<V, F, D>(value: &V, f: F) -> Uniform<V, D>
    where
        V: Value + Bytes,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = bytes::as_bytes(slice::from_ref(value));

        Uniform {
            data: f(bytes),
            ty: PhantomData,
        }
    }

    pub fn storage<V, F, D>(value: &V, f: F) -> Storage<V, D>
    where
        V: StorageValue + ?Sized,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = value.storage_bytes();

        Storage {
            data: f(bytes),
            ty: PhantomData,
        }
    }

    pub fn row<V, F, D>(values: &[V], f: F) -> Row<V, D>
    where
        V: Value + Bytes,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = bytes::as_bytes(values);

        Row {
            data: f(bytes),
            ty: PhantomData,
        }
    }
}
