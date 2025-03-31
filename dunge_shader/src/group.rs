use crate::{define::Define, eval::GlobalOut, types::MemberData};

/// The group type description.
pub trait Group {
    type Projection: Projection + 'static;
    const DEF: Define<MemberData>;
}

/// Group type projection in a shader.
pub trait Projection {
    fn projection(id: u32, out: GlobalOut) -> Self;
}
