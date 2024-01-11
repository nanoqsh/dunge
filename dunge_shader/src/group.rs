use {
    crate::{eval::GlobalOut, types::MemberType},
    std::{iter, slice},
};

/// The group type description.
pub trait Group {
    type Projection: Projection + 'static;
    const DECL: DeclareGroup;
}

pub trait Projection {
    fn projection(id: u32, out: GlobalOut) -> Self;
}

#[derive(Clone, Copy)]
pub struct DeclareGroup(&'static [MemberType]);

impl DeclareGroup {
    pub const fn new(ts: &'static [MemberType]) -> Self {
        Self(ts)
    }
}

impl IntoIterator for DeclareGroup {
    type Item = MemberType;
    type IntoIter = iter::Copied<slice::Iter<'static, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}
