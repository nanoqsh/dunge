use {
    crate::{
        buffer::Format,
        instance::{self, Set},
        layer::Layer,
        mesh::Mesh,
        set::{Bind, Bindings},
        shader::SlotNumbers,
    },
    std::{iter, marker::PhantomData},
};

pub struct Input<V, I, S>(V, I, S);

pub trait Types {
    type Vertex;
    type Instance;
    type Set;
}

impl<V, I, S> Types for Input<V, I, S> {
    type Vertex = V;
    type Instance = I;
    type Set = S;
}

#[derive(Clone, Copy)]
pub(crate) struct TargetState {
    pub format: Format,
    pub use_depth: bool,
}

impl TargetState {
    #[inline]
    fn check_layer<I>(self, layer: &Layer<I>) {
        assert_eq!(
            self.format,
            layer.format(),
            "layer format doesn't match frame format",
        );

        assert!(
            !layer.depth() || self.use_depth,
            "the target for a layer with depth must contain a depth buffer",
        );
    }
}

pub struct Render<'ren> {
    pub(crate) pass: wgpu::RenderPass<'ren>,
    pub(crate) target: TargetState,
}

impl<'ren> Render<'ren> {
    #[inline]
    pub fn layer<I>(&mut self, layer: &Layer<I>) -> On<'ren, '_, I, state::Layer> {
        let mut on = On::new(Runner {
            pass: &mut self.pass,
            target: self.target,
            slots: layer.slots(),
            count: 1,
        });

        on.run.layer(layer.render());
        on
    }
}

pub mod state {
    pub enum Layer {}
    pub enum Set {}
    pub enum Inst {}
    pub enum Draw {}
    pub enum DrawPoints {}
}

#[diagnostic::on_unimplemented(
    message = "Render cannot transition into `{B}` state",
    label = "This render function cannot be called"
)]
pub trait To<A, B> {}

impl<V, I, S> To<state::Layer, state::Set> for Input<V, I, S> {}
impl<V, I> To<state::Layer, state::Inst> for Input<V, I, ()> {}
impl<V> To<state::Layer, state::Draw> for Input<V, (), ()> {}
impl To<state::Layer, state::DrawPoints> for Input<(), (), ()> {}

impl<V, I, S> To<state::Set, state::Inst> for Input<V, I, S> {}
impl<V, S> To<state::Set, state::Draw> for Input<V, (), S> {}
impl<S> To<state::Set, state::DrawPoints> for Input<(), (), S> {}

impl<V, I, S> To<state::Inst, state::Draw> for Input<V, I, S> {}
impl<I, S> To<state::Inst, state::DrawPoints> for Input<(), I, S> {}

impl<V, I, S> To<state::Draw, state::Layer> for Input<V, I, S> {}
impl<V, I, S> To<state::Draw, state::Set> for Input<V, I, S> {}
impl<V, I, S> To<state::Draw, state::Inst> for Input<V, I, S> {}
impl<V, I, S> To<state::Draw, state::Draw> for Input<V, I, S> {}

impl<I, S> To<state::DrawPoints, state::Layer> for Input<(), I, S> {}
impl<I, S> To<state::DrawPoints, state::Set> for Input<(), I, S> {}
impl<I, S> To<state::DrawPoints, state::Inst> for Input<(), I, S> {}
impl<I, S> To<state::DrawPoints, state::DrawPoints> for Input<(), I, S> {}

struct Runner<'ren, 'layer> {
    pass: &'layer mut wgpu::RenderPass<'ren>,
    target: TargetState,
    slots: SlotNumbers,
    count: u32,
}

impl Runner<'_, '_> {
    #[inline]
    fn layer(&mut self, render: &wgpu::RenderPipeline) {
        self.pass.set_pipeline(render);
    }

    #[inline]
    fn set(&mut self, bindings: Bindings<'_>) {
        for (id, group) in iter::zip(0.., bindings.bind_groups) {
            self.pass.set_bind_group(id, group, &[]);
        }
    }

    #[inline]
    fn instance<S>(&mut self, instance: &S)
    where
        S: Set,
    {
        let vs = VertexSetter(self.pass);
        self.count = instance::set(vs, self.slots.instance, instance);
    }

    #[inline]
    fn draw<V>(&mut self, mesh: &Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, self.count);
    }

    #[inline]
    fn draw_points(&mut self, n: u32) {
        self.pass.draw(0..n, 0..self.count);
    }
}

pub struct On<'ren, 'layer, I, A> {
    run: Runner<'ren, 'layer>,
    inp: PhantomData<(I, A)>,
}

impl<'ren, 'layer, I, A> On<'ren, 'layer, I, A> {
    #[inline]
    fn new(run: Runner<'ren, 'layer>) -> Self {
        Self {
            run,
            inp: PhantomData,
        }
    }

    #[inline]
    fn to<B>(self) -> On<'ren, 'layer, I, B>
    where
        I: To<A, B>,
    {
        On {
            run: self.run,
            inp: PhantomData,
        }
    }

    #[inline]
    pub fn layer(mut self, layer: &Layer<I>) -> On<'ren, 'layer, I, state::Layer>
    where
        I: To<A, state::Layer>,
    {
        self.run.target.check_layer(layer);
        self.run.layer(layer.render());
        self.to()
    }

    #[inline]
    pub fn set<S>(mut self, set: &S) -> On<'ren, 'layer, I, state::Set>
    where
        I: To<A, state::Set> + Types,
        S: Bind<I::Set>,
    {
        self.run.set(set.bind());
        self.to()
    }

    #[inline]
    pub fn instance(mut self, instance: &I::Instance) -> On<'ren, 'layer, I, state::Inst>
    where
        I: To<A, state::Inst> + Types<Instance: Set>,
    {
        self.run.instance(instance);
        self.to()
    }

    #[inline]
    pub fn draw(mut self, mesh: &Mesh<I::Vertex>) -> On<'ren, 'layer, I, state::Draw>
    where
        I: To<A, state::Draw> + Types,
    {
        self.run.draw(mesh);
        self.to()
    }

    #[inline]
    pub fn draw_points(mut self, n: u32) -> On<'ren, 'layer, I, state::DrawPoints>
    where
        I: To<A, state::DrawPoints>,
    {
        self.run.draw_points(n);
        self.to()
    }
}

pub(crate) struct VertexSetter<'ren, 'layer>(&'layer mut wgpu::RenderPass<'ren>);

impl VertexSetter<'_, '_> {
    #[inline]
    pub(crate) fn set(&mut self, buf: &wgpu::Buffer, slot: u32) {
        let slice = buf.slice(..);
        self.0.set_vertex_buffer(slot, slice);
    }
}
