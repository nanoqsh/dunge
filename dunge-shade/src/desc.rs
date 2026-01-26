use {
    crate::{
        irc::{self, Descriptor, Scalar, Type},
        module::{Format, GroupFormat, TextureDimension},
    },
    std::{convert::Infallible, marker::PhantomData},
};

pub struct Texture<S = f32, const D: usize = 2> {
    never: Infallible,
    scalar: PhantomData<S>,
}

impl<S, const D: usize> Texture<S, D> {
    pub(crate) fn never<T>(self) -> T {
        match self.never {}
    }
}

impl<S, const D: usize> Clone for Texture<S, D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, const D: usize> Copy for Texture<S, D> {}

impl<S, const D: usize> Descriptor for Texture<S, D>
where
    S: Scalar,
{
    const NAGA: Type = Type::Image {
        dim: irc::dimension(D),
        arrayed: false,
        class: naga::ImageClass::Sampled {
            kind: S::NAGA.kind,
            multi: false,
        },
    };

    const FORMAT: GroupFormat = GroupFormat::Texture {
        dim: TextureDimension::new(D).expect("texture dimension"),
        scalar: Format::from_naga(S::NAGA.kind).expect("scalar"),
    };
}

#[derive(Clone, Copy)]
pub enum Sampler {}

impl Descriptor for Sampler {
    const NAGA: Type = Type::Sampler { comparison: false };
    const FORMAT: GroupFormat = GroupFormat::Sampler;
}
