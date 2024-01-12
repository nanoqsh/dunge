use crate::{define::Define, types::VectorType};

/// The instance type description.
pub trait Instance {
    type Projection: Projection + 'static;
    const DEF: Define<VectorType>;
}

pub trait Projection {
    fn projection(id: u32) -> Self;
}
