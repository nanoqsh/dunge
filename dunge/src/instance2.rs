//! Shader instance types and traits.

#[inline]
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

impl Rows<0> for () {
    type Inner = ();

    fn rows(&self) -> [Slice<'_>; 0] {
        []
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
