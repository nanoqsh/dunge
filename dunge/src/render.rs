use {
    crate::{
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

pub struct Render<'shed>(pub(crate) wgpu::RenderPass<'shed>);

impl<'shed> Render<'shed> {
    pub fn layer<'ren, I>(&'ren mut self, layer: &Layer<I>) -> On<'shed, 'ren, I, state::Layer> {
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

struct Runner<'shed, 'ren> {
    pass: &'ren mut wgpu::RenderPass<'shed>,
    slots: SlotNumbers,
}

impl<'shed, 'ren> Runner<'shed, 'ren> {
    fn layer(&mut self, render: &wgpu::RenderPipeline) {
        self.pass.set_pipeline(render);
    }

    fn set(&mut self, bindings: Bindings<'_>) {
        for (id, group) in iter::zip(0.., bindings.bind_groups) {
            self.pass.set_bind_group(id, group, &[]);
        }
    }

    #[expect(dead_code)]
    fn instance<I>(&mut self, _instance: &I) {
        todo!()
    }

    fn draw<V>(&mut self, mesh: &'shed Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, 1);
    }
}

pub struct On<'shed, 'ren, I, A> {
    run: Runner<'shed, 'ren>,
    inp: PhantomData<(I, A)>,
}

impl<'shed, 'ren, I, A> On<'shed, 'ren, I, A> {
    fn new(run: Runner<'shed, 'ren>) -> Self {
        Self {
            run,
            inp: PhantomData,
        }
    }

    fn to<B>(self) -> On<'shed, 'ren, I, B>
    where
        Self: To<A, B>,
    {
        On {
            run: self.run,
            inp: PhantomData,
        }
    }

    pub fn layer(mut self, layer: &Layer<I>) -> On<'shed, 'ren, I, state::Layer>
    where
        Self: To<A, state::Layer>,
    {
        self.run.layer(layer.render());
        self.to()
    }

    pub fn set<S>(mut self, set: &S) -> On<'shed, 'ren, I, state::Set>
    where
        Self: To<A, state::Set>,
        I: Types,
        S: Bind<I::Set>,
    {
        self.run.set(set.bind());
        self.to()
    }

    pub fn draw(mut self, mesh: &'shed Mesh<I::Vertex>) -> On<'shed, 'ren, I, state::Draw>
    where
        Self: To<A, state::Draw>,
        I: Types,
    {
        self.run.draw(mesh);
        self.to()
    }
}

pub(crate) struct VertexSetter<'shed, 'set>(&'set mut wgpu::RenderPass<'shed>);

impl<'shed, 'set> VertexSetter<'shed, 'set> {
    pub fn _new(pass: &'set mut wgpu::RenderPass<'shed>) -> Self {
        Self(pass)
    }

    pub fn set(&mut self, buf: &wgpu::Buffer, slot: u32) {
        let slice = buf.slice(..);
        self.0.set_vertex_buffer(slot, slice);
    }
}
