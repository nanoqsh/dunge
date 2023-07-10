use {
    crate::shader::{self, Shader},
    dunge_shader::Shader as ShaderData,
    std::marker::PhantomData,
};

/// The shader scheme.
pub struct Scheme<S> {
    data: ShaderData,
    ty: PhantomData<S>,
}

impl<S> Scheme<S> {
    pub(crate) fn new() -> Self
    where
        S: Shader,
    {
        let scheme = shader::scheme::<S>();
        let data = ShaderData::generate(scheme);
        log::debug!("generated shader:\n{src}", src = data.source);

        Self {
            data,
            ty: PhantomData,
        }
    }

    pub(crate) fn data(&self) -> &ShaderData {
        &self.data
    }
}
