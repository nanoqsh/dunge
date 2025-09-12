//! Shader instance types and traits.

use {
    crate::{
        Instance,
        context::Context,
        render::VertexSetter,
        sl::{ReadInstance, Ret},
        state::State,
        types::{self, ValueType},
        value::Value,
    },
    std::{marker::PhantomData, ops::RangeBounds},
};

pub use dunge_shade::instance::Projection;

/// Describes an instance member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: s::Sealed {
    const TYPE: ValueType;
    type Field;
    fn member_projection(id: u32) -> Self::Field;
}

pub trait RowValue: Sized {
    type Type;
    fn row_value(slice: &[Self]) -> &[u8];
}

impl<V> RowValue for V
where
    V: Value + bytemuck::Pod,
{
    type Type = V::Type;

    fn row_value(slice: &[Self]) -> &[u8] {
        bytemuck::cast_slice(slice)
    }
}

impl<V> s::Sealed for Row<V> where V: RowValue<Type: types::Value> {}

impl<V> MemberProjection for Row<V>
where
    V: RowValue<Type: types::Value>,
{
    const TYPE: ValueType = <V::Type as types::Value>::VALUE_TYPE;
    type Field = Ret<ReadInstance, V::Type>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

impl<V> s::Sealed for RowSlice<'_, V> where V: RowValue<Type: types::Value> {}

impl<V> MemberProjection for RowSlice<'_, V>
where
    V: RowValue<Type: types::Value>,
{
    const TYPE: ValueType = <V::Type as types::Value>::VALUE_TYPE;
    type Field = Ret<ReadInstance, V::Type>;

    fn member_projection(id: u32) -> Self::Field {
        ReadInstance::new(id)
    }
}

pub trait Set: Instance {
    fn set(&self, setter: &mut Setter<'_, '_>);
}

pub(crate) fn set<I>(vs: VertexSetter<'_, '_>, slot: u32, instance: &I) -> u32
where
    I: Set,
{
    let len = None;
    let mut setter = Setter { len, slot, vs };
    instance.set(&mut setter);
    setter.len()
}

pub struct Setter<'ren, 'layer> {
    len: Option<u32>,
    slot: u32,
    vs: VertexSetter<'ren, 'layer>,
}

impl Setter<'_, '_> {
    fn len(&self) -> u32 {
        self.len.unwrap_or_default()
    }

    fn next_slot(&mut self) -> u32 {
        let next = self.slot;
        self.slot += 1;
        next
    }

    fn update_len(&mut self, len: u32) {
        let current = self.len.get_or_insert(len);
        *current = u32::min(*current, len);
    }
}

pub trait SetMember {
    fn set_member(&self, setter: &mut Setter<'_, '_>);
}

impl<V> SetMember for Row<V> {
    fn set_member(&self, setter: &mut Setter<'_, '_>) {
        setter.update_len(self.len);
        let slot = setter.next_slot();
        setter.vs.set(self.buf.slice(..), slot);
    }
}

impl<V> SetMember for RowSlice<'_, V> {
    fn set_member(&self, setter: &mut Setter<'_, '_>) {
        setter.update_len(self.len);
        let slot = setter.next_slot();
        setter.vs.set(self.slice, slot);
    }
}

pub struct Row<V> {
    buf: wgpu::Buffer,
    len: u32,
    ty: PhantomData<V>,
}

impl<V> Row<V> {
    pub(crate) fn new(state: &State, data: &[V]) -> Self
    where
        V: RowValue,
    {
        use wgpu::util::{self, DeviceExt};

        let buf = {
            let desc = util::BufferInitDescriptor {
                label: None,
                contents: V::row_value(data),
                usage: wgpu::BufferUsages::VERTEX,
            };

            state.device().create_buffer_init(&desc)
        };

        let len = data.len() as u32;

        Self {
            buf,
            len,
            ty: PhantomData,
        }
    }

    /// Updates the row data.
    ///
    /// # Panics
    /// Panics if the row length is not equal to the length of the new value.
    pub fn update(&self, cx: &Context, data: &[V])
    where
        V: RowValue,
    {
        assert_eq!(
            data.len(),
            self.len(),
            "cannot update row of length {} with value of length {}",
            self.len(),
            data.len(),
        );

        let queue = cx.state().queue();
        queue.write_buffer(&self.buf, 0, V::row_value(data));
    }

    pub fn slice<S>(&self, bounds: S) -> RowSlice<'_, V>
    where
        S: RangeBounds<usize>,
    {
        let byte_start = bounds.start_bound().map(|&n| (n * size_of::<V>()) as u64);
        let byte_end = bounds.end_bound().map(|&n| (n * size_of::<V>()) as u64);

        let slice = self.buf.slice((byte_start, byte_end));
        let len = slice.size().get() as u32 / size_of::<V>() as u32;

        RowSlice {
            slice,
            byte_offset: slice.offset(),
            len,
            ty: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub struct RowSlice<'slice, V> {
    slice: wgpu::BufferSlice<'slice>,
    byte_offset: u64,
    len: u32,
    ty: PhantomData<V>,
}

impl<V> RowSlice<'_, V> {
    /// Updates the row data.
    ///
    /// # Panics
    /// Panics if the row length is not equal to the length of the new value.
    pub fn update(&self, cx: &Context, data: &[V])
    where
        V: RowValue,
    {
        assert_eq!(
            data.len(),
            self.len(),
            "cannot update row slice of length {} with value of length {}",
            self.len(),
            data.len(),
        );

        let queue = cx.state().queue();
        queue.write_buffer(self.slice.buffer(), self.byte_offset, V::row_value(data));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

mod s {
    pub trait Sealed {}
}
