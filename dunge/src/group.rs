use crate::{
    sl::{GlobalOut, ReadGlobal, Ret},
    texture::{BindTexture, Sampler, Texture},
    types::{self, GroupMemberType},
};

pub use dunge_shader::group::{DeclareGroup, Projection};

#[derive(Clone, Copy)]
pub struct BoundTexture<'a>(&'a Texture);

impl<'a> BoundTexture<'a> {
    pub fn new<T>(texture: &'a T) -> Self
    where
        T: BindTexture,
    {
        Self(texture.bind_texture())
    }

    pub(crate) fn get(&self) -> &'a Texture {
        self.0
    }
}

/// Describes a group member type projection.
///
/// The trait is sealed because the derive macro relies on no new types being used.
pub trait MemberProjection: private::Sealed {
    const TYPE: GroupMemberType;
    type Field;
    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field;
}

impl private::Sealed for BoundTexture<'_> {}

impl MemberProjection for BoundTexture<'_> {
    const TYPE: GroupMemberType = GroupMemberType::Tx2df;
    type Field = Ret<ReadGlobal, types::Texture2d<f32>>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

impl private::Sealed for &Sampler {}

impl MemberProjection for &Sampler {
    const TYPE: GroupMemberType = GroupMemberType::Sampl;
    type Field = Ret<ReadGlobal, types::Sampler>;

    fn member_projection(id: u32, binding: u32, out: GlobalOut) -> Self::Field {
        ReadGlobal::new(id, binding, out)
    }
}

mod private {
    pub trait Sealed {}
}
