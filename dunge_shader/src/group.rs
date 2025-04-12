use crate::{define::Define, eval::GlobalOut, types::MemberData};

/// The group type description.
pub trait Group {
    type Projection: Projection + 'static;
    const DEF: Define<MemberData>;
}

impl<G> Group for &G
where
    G: Group,
{
    type Projection = G::Projection;
    const DEF: Define<MemberData> = G::DEF;
}

/// Group type projection in a shader.
pub trait Projection {
    fn projection(id: u32, out: GlobalOut) -> Self;
}

pub trait Take<const N: usize> {
    type Projection;
}

impl<G> Take<0> for G
where
    G: Projection,
{
    type Projection = Self;
}

impl<A> Take<0> for (A,)
where
    A: Projection,
{
    type Projection = A;
}

impl<A, B> Take<0> for (A, B)
where
    A: Projection,
{
    type Projection = A;
}

impl<A, B> Take<1> for (A, B)
where
    B: Projection,
{
    type Projection = B;
}

impl<A, B, C> Take<0> for (A, B, C)
where
    A: Projection,
{
    type Projection = A;
}

impl<A, B, C> Take<1> for (A, B, C)
where
    B: Projection,
{
    type Projection = B;
}

impl<A, B, C> Take<2> for (A, B, C)
where
    C: Projection,
{
    type Projection = C;
}

impl<A, B, C, D> Take<0> for (A, B, C, D)
where
    A: Projection,
{
    type Projection = A;
}

impl<A, B, C, D> Take<1> for (A, B, C, D)
where
    B: Projection,
{
    type Projection = B;
}

impl<A, B, C, D> Take<2> for (A, B, C, D)
where
    C: Projection,
{
    type Projection = C;
}

impl<A, B, C, D> Take<3> for (A, B, C, D)
where
    D: Projection,
{
    type Projection = D;
}
