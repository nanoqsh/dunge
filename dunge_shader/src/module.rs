use crate::{
    context::{Context, FromContext, FromContextTyped},
    eval::{self, Eval, Fs, Vs},
    types,
};

pub trait IntoModule<A> {
    type Vertex;
    fn into_module(self) -> Module;
}

impl<M, O> IntoModule<()> for M
where
    M: FnOnce() -> O,
    O: Output,
{
    type Vertex = ();

    fn into_module(self) -> Module {
        let cx = Context::new();
        eval::make(cx, self())
    }
}

impl<M, O, A> IntoModule<(A,)> for M
where
    M: FnOnce(A) -> O,
    O: Output,
    A: FromContextTyped,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        eval::make(cx, self(a))
    }
}

impl<M, O, A, B> IntoModule<(A, B)> for M
where
    M: FnOnce(A, B) -> O,
    O: Output,
    A: FromContextTyped,
    B: FromContext,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        let b = B::from_context(&mut cx);
        eval::make(cx, self(a, b))
    }
}

impl<M, O, A, B, C> IntoModule<(A, B, C)> for M
where
    M: FnOnce(A, B, C) -> O,
    O: Output,
    A: FromContextTyped,
    B: FromContext,
    C: FromContext,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        let b = B::from_context(&mut cx);
        let c = C::from_context(&mut cx);
        eval::make(cx, self(a, b, c))
    }
}

pub struct Out<P, C> {
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
}
