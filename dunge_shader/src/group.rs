use {
    crate::{eval::GlobalOut, types::GroupMemberType},
    std::{iter, slice},
};

/// Group description.
pub trait Group {
    type Projection: Projection + 'static;
    const DECL: DeclareGroup;

    fn group<V>(&self, visit: &mut V)
    where
        V: Visitor;
}

pub trait Projection {
    fn projection(id: u32, out: GlobalOut) -> Self;
}

pub trait Visitor {
    type Texture;
    type Sampler;

    fn visit_texture(&mut self, texture: Self::Texture);
    fn visit_sampler(&mut self, sampler: Self::Sampler);
}

#[derive(Clone, Copy)]
pub struct DeclareGroup(&'static [GroupMemberType]);

impl DeclareGroup {
    pub const fn new(ts: &'static [GroupMemberType]) -> Self {
        Self(ts)
    }
}

impl IntoIterator for DeclareGroup {
    type Item = GroupMemberType;
    type IntoIter = iter::Copied<slice::Iter<'static, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}
