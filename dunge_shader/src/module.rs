use {
    crate::{
        context::{Context, FromContext, FromRender},
        eval::{self, Cs, Eval, Fs, Vs},
        types,
    },
    std::marker::PhantomData,
};

pub struct Render<V, I>(PhantomData<(V, I)>);
pub struct Compute(());

pub trait IntoModule<A, K> {
    fn into_module(self) -> Module;
}

impl<M, O> IntoModule<(), Render<(), ()>> for M
where
    M: FnOnce() -> O,
    O: RenderOutput,
{
    fn into_module(self) -> Module {
        let cx = Context::new();
        eval::make_render(cx, self)
    }
}

macro_rules! impl_into_render_module {
    (A $(,)? $($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<M, O, A, $($t),*> IntoModule<(A, $($t),*), Render<A::Vertex, A::Instance>> for M
        where
            M: FnOnce(A, $($t),*) -> O,
            O: RenderOutput,
            A: FromRender,
            $(
                $t: FromContext<Render<A::Vertex, A::Instance>>,
            )*
        {
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
impl_into_render_module!(A, B);
impl_into_render_module!(A, B, C);
impl_into_render_module!(A, B, C, D);

macro_rules! impl_into_compute_module {
    ($($t:ident),*) => {
        #[allow(non_snake_case, unused_mut, unused_parens)]
        impl<M, O, $($t),*> IntoModule<($($t),*), Compute> for M
        where
            M: FnOnce($($t),*) -> O,
            O: ComputeOutput,
            $(
                $t: FromContext<Compute>,
            )*
        {
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
impl_into_compute_module!(A, B);
impl_into_compute_module!(A, B, C);
impl_into_compute_module!(A, B, C, D);

pub struct Out<P, C>
where
    P: Eval<Vs, Out = types::Vec4<f32>>,
    C: Eval<Fs, Out = types::Vec4<f32>>,
{
    pub place: P,
    pub color: C,
}

pub trait RenderOutput {
    type Place: Eval<Vs, Out = types::Vec4<f32>>;
    type Color: Eval<Fs, Out = types::Vec4<f32>>;

    fn output(self) -> Out<Self::Place, Self::Color>;
}

impl<P, C> RenderOutput for Out<P, C>
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

pub struct Unit<C>
where
    C: Eval<Cs, Out = types::Unit>,
{
    pub compute: C,
}

pub trait ComputeOutput {
    type Compute: Eval<Cs, Out = types::Unit>;

    fn output(self) -> Unit<Self::Compute>;
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
