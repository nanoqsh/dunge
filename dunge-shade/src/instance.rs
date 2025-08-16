use crate::{define::Define, types::ValueType};

/// The instance type description.
pub trait Instance {
    type Projection: Projection + 'static;
    const DEF: Define<ValueType>;
}

/// Instance type projection in a shader.
pub trait Projection {
    fn projection(id: u32) -> Self;
}
