//! Shader instance types and traits.

use crate::store2::{Row, RowSlice};

pub(crate) fn set<R, const N: usize>(rows: R, mut slot: u32, pass: &mut wgpu::RenderPass<'_>) -> u32
where
    R: Rows<N>,
{
    let slices = rows.rows();
    let instances = slices.iter().map(|s| s.len).min().unwrap_or_default();

    for slice in slices {
        pass.set_vertex_buffer(slot, slice.buffer);
        slot += 1;
    }

    instances
}

pub struct Slice<'buffer> {
    buffer: wgpu::BufferSlice<'buffer>,
    len: u32,
}

pub trait Buffer {
    type Inner;
    fn buffer(&self) -> Slice<'_>;
}

impl<B> Buffer for &B
where
    B: Buffer,
{
    type Inner = B::Inner;

    fn buffer(&self) -> Slice<'_> {
        (**self).buffer()
    }
}

impl<V> Buffer for Row<V> {
    type Inner = V;

    fn buffer(&self) -> Slice<'_> {
        let buffer = self.data().buffer().slice(..);
        let len = self.len().get();
        Slice { buffer, len }
    }
}

impl<V> Buffer for RowSlice<'_, V> {
    type Inner = V;

    fn buffer(&self) -> Slice<'_> {
        let buffer = self.slice().slice();
        let len = self.len().get();
        Slice { buffer, len }
    }
}

pub trait Rows<const N: usize> {
    type Inner;
    fn rows(&self) -> [Slice<'_>; N];
}

impl<R, const N: usize> Rows<N> for &R
where
    R: Rows<N>,
{
    type Inner = R::Inner;

    fn rows(&self) -> [Slice<'_>; N] {
        (**self).rows()
    }
}

impl<A> Rows<1> for (A,)
where
    A: Buffer,
{
    type Inner = (A::Inner,);

    fn rows(&self) -> [Slice<'_>; 1] {
        [self.0.buffer()]
    }
}

impl<A, B> Rows<2> for (A, B)
where
    A: Buffer,
    B: Buffer,
{
    type Inner = (A::Inner, B::Inner);

    fn rows(&self) -> [Slice<'_>; 2] {
        [self.0.buffer(), self.1.buffer()]
    }
}

impl<A, B, C> Rows<3> for (A, B, C)
where
    A: Buffer,
    B: Buffer,
    C: Buffer,
{
    type Inner = (A::Inner, B::Inner, C::Inner);

    fn rows(&self) -> [Slice<'_>; 3] {
        [self.0.buffer(), self.1.buffer(), self.2.buffer()]
    }
}

impl<A, B, C, D> Rows<4> for (A, B, C, D)
where
    A: Buffer,
    B: Buffer,
    C: Buffer,
    D: Buffer,
{
    type Inner = (A::Inner, B::Inner, C::Inner, D::Inner);

    fn rows(&self) -> [Slice<'_>; 4] {
        [
            self.0.buffer(),
            self.1.buffer(),
            self.2.buffer(),
            self.3.buffer(),
        ]
    }
}
