use {
    crate::{
        context::{Context, FromContext, FromRender},
        eval::{self, Cs, Eval, Fs, Vs},
        types,
    },
    std::marker::PhantomData,
};

pub trait IntoModule<A, K> {
    type Input;
    fn into_module(self) -> Module;
}

pub enum RenderKind {}
pub struct RenderInput<V, I>(PhantomData<(V, I)>);

impl<M, P, C> IntoModule<(), RenderKind> for M
where
    M: FnOnce() -> Render<P, C>,
    P: VsOut,
    C: FsOut,
{
    type Input = RenderInput<(), ()>;

    fn into_module(self) -> Module {
        let cx = Context::new();
        eval::make_render(cx, self)
    }
}

macro_rules! impl_into_render_module {
    (A $(,)? $($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<M, P, C, A, $($t),*> IntoModule<(A, $($t),*), RenderKind> for M
        where
            M: FnOnce(A, $($t),*) -> Render<P, C>,
            P: VsOut,
            C: FsOut,
            A: FromRender<RenderKind>,
            $(
                $t: FromContext<RenderKind>,
            )*
        {
            type Input = RenderInput<A::Vertex, A::Instance>;

            fn into_module(self) -> Module {
                let mut cx = Context::new();
                let a = A::from_render(&mut cx);
                $(
                    let $t = $t::from_context(&mut cx);
                )*
                eval::make_render(cx, || self(a, $($t),*))
            }
        }
    };
}

impl_into_render_module!(A);
impl_into_render_module!(A, X);
impl_into_render_module!(A, X, Y);
impl_into_render_module!(A, X, Y, Z);

pub enum ComputeKind {}
pub struct ComputeInput(());

macro_rules! impl_into_compute_module {
    ($($t:ident),*) => {
        #[allow(non_snake_case, unused_mut, unused_parens)]
        impl<M, C, $($t),*> IntoModule<($($t),*), ComputeKind> for M
        where
            M: FnOnce($($t),*) -> Compute<C>,
            C: CsOut,
            $(
                $t: FromContext<ComputeKind>,
            )*
        {
            type Input = ComputeInput;

            fn into_module(self) -> Module {
                let mut cx = Context::new();
                $(
                    let $t = $t::from_context(&mut cx);
                )*
                eval::make_compute(cx, || self($($t),*))
            }
        }
    };
}

impl_into_compute_module!();
impl_into_compute_module!(A);
impl_into_compute_module!(A, X);
impl_into_compute_module!(A, X, Y);
impl_into_compute_module!(A, X, Y, Z);

pub trait VsOut: Eval<Vs, Out = types::Vec4<f32>> {}
impl<E> VsOut for E where E: Eval<Vs, Out = types::Vec4<f32>> {}

pub trait FsOut: Eval<Fs, Out = types::Vec4<f32>> {}
impl<E> FsOut for E where E: Eval<Fs, Out = types::Vec4<f32>> {}

pub struct Render<P, C>
where
    P: VsOut,
    C: FsOut,
{
    pub place: P,
    pub color: C,
}

pub trait CsOut: Eval<Cs, Out = types::Unit> {}
impl<E> CsOut for E where E: Eval<Cs, Out = types::Unit> {}

pub struct Compute<C>
where
    C: CsOut,
{
    pub compute: C,
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
