use {
    crate::{
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

pub struct Render<'ren>(pub(crate) wgpu::RenderPass<'ren>);

impl<'ren> Render<'ren> {
    pub fn layer<I>(&mut self, layer: &Layer<I>) -> On<'ren, '_, I, state::Layer> {
        let mut on = On::new(Runner {
            pass: &mut self.0,
            slots: layer.slots(),
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
    slots: SlotNumbers,
}

impl Runner<'_, '_> {
    fn layer(&mut self, render: &wgpu::RenderPipeline) {
        self.pass.set_pipeline(render);
    }

    fn set(&mut self, bindings: Bindings<'_>) {
        for (id, group) in iter::zip(0.., bindings.bind_groups) {
            self.pass.set_bind_group(id, group, &[]);
        }
    }

    fn instance<S>(&mut self, instance: &S)
    where
        S: Set,
    {
        let vs = VertexSetter(self.pass);
        instance::set(vs, self.slots.instance, instance);
    }

    fn draw<V>(&mut self, mesh: &Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, 1);
    }

    fn draw_points(&mut self, n: u32) {
        self.pass.draw(0..n, 0..1);
    }
}

pub struct On<'ren, 'layer, I, A> {
    run: Runner<'ren, 'layer>,
    inp: PhantomData<(I, A)>,
}

impl<'ren, 'layer, I, A> On<'ren, 'layer, I, A> {
    fn new(run: Runner<'ren, 'layer>) -> Self {
        Self {
            run,
            inp: PhantomData,
        }
    }

    fn to<B>(self) -> On<'ren, 'layer, I, B>
    where
        I: To<A, B>,
    {
        On {
            run: self.run,
            inp: PhantomData,
        }
    }

    pub fn layer(mut self, layer: &Layer<I>) -> On<'ren, 'layer, I, state::Layer>
    where
        I: To<A, state::Layer>,
    {
        self.run.layer(layer.render());
        self.to()
    }

    pub fn set<S>(mut self, set: &S) -> On<'ren, 'layer, I, state::Set>
    where
        I: To<A, state::Set> + Types,
        S: Bind<I::Set>,
    {
        self.run.set(set.bind());
        self.to()
    }

    pub fn instance(mut self, instance: &I::Instance) -> On<'ren, 'layer, I, state::Inst>
    where
        I: To<A, state::Inst> + Types<Instance: Set>,
    {
        self.run.instance(instance);
        self.to()
    }

    pub fn draw(mut self, mesh: &Mesh<I::Vertex>) -> On<'ren, 'layer, I, state::Draw>
    where
        I: To<A, state::Draw> + Types,
    {
        self.run.draw(mesh);
        self.to()
    }

    pub fn draw_points(mut self, n: u32) -> On<'ren, 'layer, I, state::DrawPoints>
    where
        I: To<A, state::DrawPoints>,
    {
        self.run.draw_points(n);
        self.to()
    }
}

pub(crate) struct VertexSetter<'ren, 'layer>(&'layer mut wgpu::RenderPass<'ren>);

impl<'ren, 'layer> VertexSetter<'ren, 'layer> {
    pub fn _new(pass: &'layer mut wgpu::RenderPass<'ren>) -> Self {
        Self(pass)
    }

    pub fn set(&mut self, buf: &wgpu::Buffer, slot: u32) {
        let slice = buf.slice(..);
        self.0.set_vertex_buffer(slot, slice);
    }
}
