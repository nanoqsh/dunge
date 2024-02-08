use crate::{
    context::{Context, FromContext, FromContextInput},
    eval::{self, Eval, Fs, Vs},
    types,
};

pub trait IntoModule<A> {
    type Vertex;
    type Instance;
    fn into_module(self) -> Module;
}

impl<M, O> IntoModule<()> for M
where
    M: FnOnce() -> O,
    O: Output,
{
    type Vertex = ();
    type Instance = ();

    fn into_module(self) -> Module {
        let cx = Context::new();
        eval::make(cx, self)
    }
}

macro_rules! impl_into_module {
    (A $(,)? $($t:ident),*) => {
        impl<M, O, A, $($t),*> IntoModule<(A, $($t),*)> for M
        where
            M: FnOnce(A, $($t),*) -> O,
            O: Output,
            A: FromContextInput,
            $(
                $t: FromContext,
            )*
        {
            type Vertex = A::Vertex;
            type Instance = A::Instance;

            #[allow(non_snake_case)]
            fn into_module(self) -> Module {
                let mut cx = Context::new();
                let a = A::from_context_input(&mut cx);
                $(
                    let $t = $t::from_context(&mut cx);
                )*
                eval::make(cx, || self(a, $($t),*))
            }
        }
    };
}

impl_into_module!(A);
impl_into_module!(A, B);
impl_into_module!(A, B, C);
impl_into_module!(A, B, C, D);

pub struct Out<P, C>
where
    P: Eval<Vs, Out = types::Vec4<f32>>,
    C: Eval<Fs, Out = types::Vec4<f32>>,
{
    pub place: P,
    pub color: C,
}

pub trait Output {
    type Place: Eval<Vs, Out = types::Vec4<f32>>;
    type Color: Eval<Fs, Out = types::Vec4<f32>>;

    fn output(self) -> Out<Self::Place, Self::Color>;
}

impl<P, C> Output for Out<P, C>
where
    P: Eval<Vs, Out = types::Vec4<f32>>,
    C: Eval<Fs, Out = types::Vec4<f32>>,
{
    type Place = P;
    type Color = C;

    fn output(self) -> Self {
        self
    }
}

pub struct Module {
    pub cx: Context,
    pub nm: naga::Module,
    pub wgsl: String,
}

impl Module {
    pub(crate) fn new(cx: Context, nm: naga::Module) -> Self {
        let wgsl;

        #[cfg(any(debug_assertions, feature = "wgsl"))]
        {
            use naga::valid::{Capabilities, ValidationFlags, Validator};

            let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
            let info = match validator.validate(&nm) {
                Ok(info) => info,
                Err(err) => {
                    log::error!("{nm:#?}");
                    panic!("shader error: {err}\n{val:#?}", val = err.as_inner());
                }
            };

            #[cfg(feature = "wgsl")]
            {
                use naga::back::wgsl::{self, WriterFlags};

                wgsl = match wgsl::write_string(&nm, &info, WriterFlags::all()) {
                    Ok(wgsl) => wgsl,
                    Err(err) => panic!("wgsl writer error: {err}"),
                };
            }

            #[cfg(not(feature = "wgsl"))]
            {
                _ = info;
            }
        }

        #[cfg(not(feature = "wgsl"))]
        {
            wgsl = String::new();
        }

        Self { cx, nm, wgsl }
    }
}
