use {
    crate::{
        bytes::{self, Bytes},
        irc::{
            self, Array, Composite, Fields, Fnc, GlobalVariable, GroupMember, Input, InputKind,
            Irc, MaybeSizedValue, Method, Methods, Reference, Value,
        },
        module::GroupFormat,
    },
    std::{marker::PhantomData, num::NonZeroU32, ops, slice},
};

pub trait Store {
    type Context;
    fn update(&self, cx: &Self::Context, bytes: &[u8]);
    fn byte_size(&self) -> u64;
    fn len_non_zero(&self) -> NonZeroU32;
}

pub trait Data: Store {
    type Slice<'slice>: Store<Context = Self::Context> + Copy
    where
        Self: 'slice;

    fn slice(&self, bounds: ops::Range<u64>, len: NonZeroU32) -> Self::Slice<'_>;
    fn byte_offset(slice: &Self::Slice<'_>) -> u64;
}

pub struct Uniform<V, D> {
    data: D,
    ty: PhantomData<V>,
}

impl<V, D> Uniform<V, D> {
    pub fn read(&self) -> V {
        panic!()
    }

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32
    where
        V: Array,
    {
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

    #[doc(hidden)]
    pub fn data(&self) -> &D {
        &self.data
    }
}

impl<V, D> Clone for Uniform<V, D>
where
    D: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            ty: PhantomData,
        }
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
    type Ref = Self;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<Self> + use<V, D> {
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
    type Global = Self;

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

impl<V, D> Methods for Uniform<V, D>
where
    V: Value,
{
    type Methods = UniformMethods<V, D>;

    const METHODS: UniformMethods<V, D> = UniformMethods {
        read: Method::Load,
        len: irc::array_len::<V, Self>(),
    };
}

pub struct UniformMethods<V, D> {
    pub read: Method<Uniform<V, D>, V>,
    pub len: Method<Uniform<V, D>, u32>,
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

    #[doc(hidden)]
    pub fn data(&self) -> &D {
        &self.data
    }

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32
    where
        V: Array,
    {
        panic!()
    }
}

impl<V, D> Clone for Storage<V, D>
where
    V: ?Sized,
    D: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            ty: PhantomData,
        }
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
    type Ref = Self;

    fn input(fnc: &mut Fnc<'_>) -> impl Reference<Self> + use<V, D> {
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
    type Global = Self;

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
    V: MaybeSizedValue + ?Sized,
{
    type Methods = StorageMethods<V, D>;

    const METHODS: StorageMethods<V, D> = StorageMethods {
        read: Method::Load,
        len: irc::array_len::<V, Self>(),
    };
}

pub struct StorageMethods<V, D>
where
    V: ?Sized,
{
    pub read: Method<Storage<V, D>, V>,
    pub len: Method<Storage<V, D>, u32>,
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

    pub fn slice<S>(&self, bounds: S) -> Option<RowSlice<'_, V, D>>
    where
        S: ops::RangeBounds<u32>,
    {
        let slice_bounds = non_zero_bounds(bounds, self.data.len_non_zero().get())?;
        let len = slice_bounds
            .len()
            .try_into()
            .ok()
            .and_then(NonZeroU32::new)?;

        let item_size = size_of::<V>() as u64;
        let bytes_bounds =
            u64::from(slice_bounds.start) * item_size..u64::from(slice_bounds.end) * item_size;

        let slice = self.data.slice(bytes_bounds, len);

        Some(RowSlice {
            slice,
            ty: PhantomData,
        })
    }

    pub fn byte_size(&self) -> u64 {
        self.data.byte_size()
    }

    pub fn len(&self) -> NonZeroU32 {
        self.data.len_non_zero()
    }

    #[doc(hidden)]
    pub fn data(&self) -> &D {
        &self.data
    }
}

fn non_zero_bounds<S>(bounds: S, upper: u32) -> Option<ops::Range<u32>>
where
    S: ops::RangeBounds<u32>,
{
    let (start, end) = match (bounds.start_bound(), bounds.end_bound()) {
        (ops::Bound::Included(&start), ops::Bound::Included(&end)) => {
            (start, u32::checked_sub(end, 1)?)
        }
        (ops::Bound::Included(&start), ops::Bound::Excluded(&end)) => (start, end),
        (ops::Bound::Included(&start), ops::Bound::Unbounded) => (start, upper),
        (ops::Bound::Excluded(&start), ops::Bound::Included(&end)) => {
            (u32::checked_add(start, 1)?, u32::checked_sub(end, 1)?)
        }
        (ops::Bound::Excluded(&start), ops::Bound::Excluded(&end)) => {
            (u32::checked_add(start, 1)?, end)
        }
        (ops::Bound::Excluded(&start), ops::Bound::Unbounded) => {
            (u32::checked_add(start, 1)?, upper)
        }
        (ops::Bound::Unbounded, ops::Bound::Included(&end)) => (0, u32::checked_sub(end, 1)?),
        (ops::Bound::Unbounded, ops::Bound::Excluded(&end)) => (0, end),
        (ops::Bound::Unbounded, ops::Bound::Unbounded) => (0, upper),
    };

    if (start..end).is_empty() {
        None
    } else {
        Some(start..end)
    }
}

pub struct RowSlice<'slice, V, D>
where
    D: Data + 'slice,
{
    slice: D::Slice<'slice>,
    ty: PhantomData<[V]>,
}

impl<'slice, V, D> RowSlice<'slice, V, D>
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
        self.slice.update(cx, bytes::as_bytes(new));
    }

    pub fn len(&self) -> NonZeroU32 {
        self.slice.len_non_zero()
    }

    pub fn offset(&self) -> u32 {
        (D::byte_offset(&self.slice) / size_of::<V>() as u64) as u32
    }

    #[doc(hidden)]
    pub fn slice(&self) -> D::Slice<'slice> {
        self.slice
    }
}

impl<V, D> Clone for RowSlice<'_, V, D>
where
    D: Data,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, D> Copy for RowSlice<'_, V, D> where D: Data {}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub fn uniform<V, F, D>(value: &V, f: F) -> Option<Uniform<V, D>>
    where
        V: Value + Bytes,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = bytes::as_bytes(slice::from_ref(value));
        if bytes.is_empty() {
            return None;
        }

        Some(Uniform {
            data: f(bytes),
            ty: PhantomData,
        })
    }

    pub fn storage<V, F, D>(value: &V, f: F) -> Option<Storage<V, D>>
    where
        V: StorageValue + ?Sized,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = value.storage_bytes();
        if bytes.is_empty() {
            return None;
        }

        Some(Storage {
            data: f(bytes),
            ty: PhantomData,
        })
    }

    pub fn row<V, F, D>(values: &[V], f: F) -> Option<Row<V, D>>
    where
        V: Value + Bytes,
        F: FnOnce(&[u8]) -> D,
    {
        let bytes = bytes::as_bytes(values);
        if bytes.is_empty() {
            return None;
        }

        Some(Row {
            data: f(bytes),
            ty: PhantomData,
        })
    }
}
