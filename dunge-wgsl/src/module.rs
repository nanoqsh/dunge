use {
    dunge::sl::{self, Dynamic, RenderInput, RenderKind, Stages},
    naga::front::wgsl,
    std::{error, fmt, marker::PhantomData},
};

#[derive(Debug)]
pub enum Error {
    Parse(wgsl::ParseError),
    TooManyStages { actual: usize, passed: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => e.fmt(f),
            Self::TooManyStages { actual, passed } => {
                write!(
                    f,
                    "too many stages passed {passed} while the shader only has {actual}",
                )
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Parse(e) => Some(e),
            Self::TooManyStages { .. } => None,
        }
    }
}

pub fn make<A>() -> Signature<(A,)> {
    Signature {
        stages: vec![],
        args: PhantomData,
    }
}

pub fn make2<A, B>() -> Signature<(A, B)> {
    Signature {
        stages: vec![],
        args: PhantomData,
    }
}

pub fn make3<A, B, C>() -> Signature<(A, B, C)> {
    Signature {
        stages: vec![],
        args: PhantomData,
    }
}

pub fn make4<A, B, C, D>() -> Signature<(A, B, C, D)> {
    Signature {
        stages: vec![],
        args: PhantomData,
    }
}

pub struct Signature<A> {
    stages: Vec<Stages>,
    args: PhantomData<A>,
}

impl<A> Signature<A> {
    pub fn add_stages<S>(mut self, stages: S) -> Self
    where
        S: Into<Stages>,
    {
        self.stages.push(stages.into());
        self
    }

    pub fn render<V, I, S>(
        self,
        src: &str,
    ) -> Result<
        impl sl::IntoModule<A, RenderKind, Input = RenderInput<V, I>, Set = S> + use<A, V, I, S>,
        Error,
    >
    where
        Dynamic: sl::IntoModule<A, RenderKind, Input = RenderInput<V, I>, Set = S>,
    {
        make_shader(src, self.stages)
    }
}

fn make_shader<A, K, I, S>(
    src: &str,
    stages: Vec<Stages>,
) -> Result<impl sl::IntoModule<A, K, Input = I, Set = S> + use<A, K, I, S>, Error>
where
    Dynamic: sl::IntoModule<A, K, Input = I, Set = S>,
{
    let nm = wgsl::parse_str(src).map_err(Error::Parse)?;

    let n_globals = nm.global_variables.len();
    if stages.len() > n_globals {
        return Err(Error::TooManyStages {
            actual: n_globals,
            passed: stages.len(),
        });
    }

    Ok(Dynamic { stages, nm })
}
