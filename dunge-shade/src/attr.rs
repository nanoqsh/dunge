use glam::Vec4;

#[diagnostic::on_unimplemented(
    message = "type `{Self}` cannot be used as a vertex index",
    label = "not a vertex index",
    note = "use `u32` instead"
)]
pub trait Index {}

impl Index for u32 {}

pub fn is_index<I>()
where
    I: Index,
{
}

#[diagnostic::on_unimplemented(
    message = "type `{Self}` cannot be used as a position",
    label = "not a position",
    note = "use `glam::Vec4` instead"
)]
pub trait Position {}

impl Position for Vec4 {}

pub fn is_position<P>()
where
    P: Position,
{
}
