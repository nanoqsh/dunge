use {
    crate::{
        color::{Color, Rgb},
        postproc::{Parameters, PostProcessor},
        render::State,
    },
    dunge_shader::Vignette,
    std::sync::{Mutex, MutexGuard},
};

/// The frame post-effect.
///
/// Can be created from the [context](crate::Context) by calling
/// the [`posteffect_builder`](crate::Context::posteffect_builder) function.
pub struct PostEffect(Box<Inner>);

impl PostEffect {
    pub(crate) fn vignette(state: &State, Color([r, g, b]): Rgb, force: f32) -> Self {
        let inner = Inner {
            postproc: Mutex::new(PostProcessor::new(state)),
            vignette: [r, g, b, force],
        };

        Self(Box::new(inner))
    }

    pub(crate) fn with_parameters(
        &self,
        state: &State,
        params: Parameters,
    ) -> MutexGuard<PostProcessor> {
        let Self(inner) = self;
        let [r, g, b, f] = inner.vignette;
        let params = Parameters {
            vignette: Vignette::Color { r, g, b, f },
            ..params
        };

        let mut postproc = inner.postproc.lock().expect("lock");
        postproc.set_parameters(state, params);
        postproc
    }
}

struct Inner {
    postproc: Mutex<PostProcessor>,
    vignette: [f32; 4],
}

/// The [post-effect](PostEffect) builder.
pub struct Builder<'a> {
    state: &'a State,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(state: &'a State) -> Self {
        Self { state }
    }

    /// Builds a post-effect with the vignette effect.
    pub fn vignette(self, col: Rgb, force: f32) -> PostEffect {
        PostEffect::vignette(self.state, col, force)
    }
}
