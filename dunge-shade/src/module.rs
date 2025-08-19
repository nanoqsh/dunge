use {
    crate::{
        context::{Context, FromContext, FromRender, Info, TakeSet},
        eval::{self, Cs, Eval, Fs, Vs},
        types,
    },
    std::marker::PhantomData,
};

macro_rules! tuple {
    () => {
        ((), (), ())
    };

    ($a:path) => {
        ($a, (), ())
    };

    ($a:path, $b:path) => {
        ($a, $b, ())
    };

    ($a:path, $b:path, $c:path) => {
        ($a, $b, $c)
    };
}

pub trait IntoModule<A, K> {
    type Input;
    type Set;
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
    type Set = ();

    #[inline]
    fn into_module(self) -> Module {
        let cx = Context::new();
        eval::make_render(cx, self)
    }
}

macro_rules! impl_into_render_module {
    (A $($t:ident)*) => {
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
            tuple!($($t::Set),*): TakeSet,
        {
            type Input = RenderInput<A::Vertex, A::Instance>;
            type Set = <tuple!($($t::Set),*) as TakeSet>::Set;

            #[inline]
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
impl_into_render_module!(A X);
impl_into_render_module!(A X Y);
impl_into_render_module!(A X Y Z);

pub enum ComputeKind {}
pub struct ComputeInput(());

macro_rules! impl_into_compute_module {
    ($($t:ident)*) => {
        #[allow(non_snake_case, unused_mut, unused_parens)]
        impl<M, C, $($t),*> IntoModule<($($t),*), ComputeKind> for M
        where
            M: FnOnce($($t),*) -> Compute<C>,
            C: CsOut,
            $(
                $t: FromContext<ComputeKind>,
            )*
            tuple!($($t::Set),*): TakeSet,
        {
            type Input = ComputeInput;
            type Set = <tuple!($($t::Set),*) as TakeSet>::Set;

            #[inline]
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
impl_into_compute_module!(X);
impl_into_compute_module!(X Y);
impl_into_compute_module!(X Y Z);

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

pub trait CsOut: Eval<Cs> {}
impl<E> CsOut for E where E: Eval<Cs> {}

pub struct Compute<C>
where
    C: CsOut,
{
    pub compute: C,
    pub workgroup_size: [u32; 3],
}

pub struct Module {
    pub info: Info,
    pub nm: naga::Module,
    pub wgsl: String,
}

impl Module {
    pub(crate) fn new(info: Info, nm: naga::Module) -> Self {
        let wgsl;

        #[cfg(any(debug_assertions, feature = "wgsl"))]
        {
            use {
                naga::valid,
                std::{error::Error, fmt::Write},
            };

            let mut validator =
                valid::Validator::new(valid::ValidationFlags::all(), valid::Capabilities::empty());

            let info = match validator.validate(&nm) {
                Ok(info) => info,
                Err(e) => {
                    log::error!("{nm:#?}");

                    let mut inner = e.as_inner() as &dyn Error;
                    let mut s = format!("{inner}\n");
                    while let Some(source) = inner.source() {
                        _ = writeln!(&mut s, "{source}");
                        inner = source;
                    }

                    panic!("shader error: {s}");
                }
            };

            #[cfg(feature = "wgsl")]
            {
                use naga::back::wgsl;

                wgsl = match wgsl::write_string(&nm, &info, wgsl::WriterFlags::all()) {
                    Ok(wgsl) => wgsl,
                    Err(e) => panic!("wgsl writer error: {e}"),
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

        Self { info, nm, wgsl }
    }
}

#[doc(hidden)]
pub struct Dynamic<I> {
    pub info: Info,
    pub nm: naga::Module,
    input: PhantomData<I>,
}

impl<V, I> Dynamic<RenderInput<V, I>> {
    pub fn render(info: Info, nm: naga::Module) -> Self {
        Self {
            info,
            nm,
            input: PhantomData,
        }
    }
}

impl Dynamic<ComputeInput> {
    pub fn compute(info: Info, nm: naga::Module) -> Self {
        Self {
            info,
            nm,
            input: PhantomData,
        }
    }
}

#[doc(hidden)]
impl<V, I> IntoModule<(), RenderKind> for Dynamic<RenderInput<V, I>> {
    type Input = RenderInput<V, I>;
    type Set = ();

    #[inline]
    fn into_module(self) -> Module {
        Module::new(self.info, self.nm)
    }
}

#[doc(hidden)]
impl IntoModule<(), ComputeKind> for Dynamic<ComputeInput> {
    type Input = ComputeInput;
    type Set = ();

    fn into_module(self) -> Module {
        Module::new(self.info, self.nm)
    }
}
