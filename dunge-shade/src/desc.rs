use {
    crate::{
        irc::{self, Descriptor, Method, Methods, Scalar, Type},
        module::{Format, GroupFormat, TextureDimension},
    },
    std::{any::Any, marker::PhantomData, sync::Arc},
};

pub struct Texture<S = f32, const D: usize = 2> {
    inner: Arc<dyn Any>,
    scalar: PhantomData<S>,
}

impl<S, const D: usize> Texture<S, D> {
    #[doc(hidden)]
    pub fn inner(&self) -> &dyn Any {
        self.inner.as_ref()
    }

    #[doc(hidden)]
    pub fn with_scalar<N>(&self) -> Texture<N, D> {
        Texture {
            inner: self.inner.clone(),
            scalar: PhantomData,
        }
    }
}

impl<S, const D: usize> Clone for Texture<S, D> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            scalar: PhantomData,
        }
    }
}

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

impl<S, const D: usize> Methods for Texture<S, D> {
    type Methods = DescriptorMethods<Self>;

    const METHODS: Self::Methods = DescriptorMethods {
        clone: Method::Noop,
    };
}

#[derive(Clone)]
pub struct Sampler {
    inner: Arc<dyn Any>,
}

impl Sampler {
    #[doc(hidden)]
    pub fn inner(&self) -> &dyn Any {
        self.inner.as_ref()
    }
}

impl Methods for Sampler {
    type Methods = DescriptorMethods<Self>;

    const METHODS: Self::Methods = DescriptorMethods {
        clone: Method::Noop,
    };
}

impl Descriptor for Sampler {
    const NAGA: Type = Type::Sampler { comparison: false };
    const FORMAT: GroupFormat = GroupFormat::Sampler;
}

pub struct DescriptorMethods<D> {
    pub clone: Method<D, D>,
}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub fn texture<I, const D: usize>(inner: I) -> Texture<(), D>
    where
        I: 'static,
    {
        Texture {
            inner: Arc::new(inner),
            scalar: PhantomData,
        }
    }

    pub fn sampler<I>(inner: I) -> Sampler
    where
        I: 'static,
    {
        Sampler {
            inner: Arc::new(inner),
        }
    }
}
