use crate::{
    context::{Context, FromContext, FromContextInput},
    eval::{self, Cs, Eval},
    sl::Module,
};

pub trait IntoCsModule<A> {
    type Instance;
    fn into_cs_module(self) -> Module;
}

impl<M, P, O> IntoCsModule<()> for M
where
    M: FnOnce() -> CsOut<P, O>,
    P: Eval<Cs, Out = O>,
{
    type Instance = ();

    fn into_cs_module(self) -> Module {
        let cx = Context::new();
        eval::make_cs(cx, self)
    }
}

macro_rules! impl_into_module {
    (A $(,)? $($t:ident),*) => {
        impl<M, P, O, A, $($t),*> IntoCsModule<(A, $($t),*)> for M
        where
            M: FnOnce(A, $($t),*) -> CsOut<P, O>,
            A: FromContextInput,
            P: Eval<Cs, Out = O>,
            $(
                $t: FromContext,
            )*
        {
            type Instance = A::Instance;

            #[allow(non_snake_case)]
            fn into_cs_module(self) -> Module {
                let mut cx = Context::new();
                let a = A::from_context_input(&mut cx);
                $(
                    let $t = $t::from_context(&mut cx);
                )*
                eval::make_cs(cx, || self(a, $($t),*))
            }
        }
    };
}

impl_into_module!(A);
impl_into_module!(A, B);
impl_into_module!(A, B, C);
impl_into_module!(A, B, C, D);

pub struct CsOut<P, O>
where
    P: Eval<Cs, Out = O>,
{
    pub compute: P,
}
