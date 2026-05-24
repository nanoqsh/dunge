//! Render management.

use {
    crate::{
        color::Format,
        instance::{self, Rows},
        layer::Layer,
        mesh::Mesh,
        set::{Bind, Bindings},
        shader::SlotNumbers,
    },
    dunge_shade::irc::Fields,
    std::{iter, marker::PhantomData},
};

#[derive(Clone, Copy)]
pub(crate) struct TargetState {
    pub format: Format,
    pub use_depth: bool,
}

impl TargetState {
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
    #[must_use]
    pub fn layer<I>(&mut self, layer: &Layer<I>) -> On<'ren, '_, I> {
        let mut on = On::new(Runner {
            pass: &mut self.pass,
            target: self.target,
            slots: layer.slots(),
            instances: 1,
        });

        on.run.target.check_layer(layer);
        on.run.layer(layer.render());
        on
    }
}

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

struct Runner<'ren, 'layer> {
    pass: &'layer mut wgpu::RenderPass<'ren>,
    target: TargetState,
    slots: SlotNumbers,
    instances: u32,
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

    fn instance<R, const N: usize>(&mut self, rows: R)
    where
        R: Rows<N>,
    {
        self.instances = instance::set(rows, self.slots.instance, self.pass);
    }

    fn draw<V>(&mut self, mesh: &Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, self.instances);
    }

    fn draw_points(&mut self, n: u32) {
        self.pass.draw(0..n, 0..self.instances);
    }
}

pub struct On<'ren, 'layer, I> {
    run: Runner<'ren, 'layer>,
    inp: PhantomData<fn(I)>,
}

impl<'ren, 'layer, I> On<'ren, 'layer, I> {
    fn new(run: Runner<'ren, 'layer>) -> Self {
        Self {
            run,
            inp: PhantomData,
        }
    }

    #[must_use]
    pub fn layer<J>(mut self, layer: &Layer<J>) -> On<'ren, 'layer, J> {
        self.run.slots = layer.slots();
        self.run.instances = 1;

        self.run.target.check_layer(layer);
        self.run.layer(layer.render());
        On {
            run: self.run,
            inp: PhantomData,
        }
    }

    #[must_use]
    pub fn set<S>(mut self, set: S) -> Self
    where
        I: Types,
        S: Bind<I::Set>,
    {
        self.run.set(set.bind());
        self
    }

    #[must_use]
    pub fn instance<R, const N: usize>(mut self, rows: R) -> Self
    where
        R: Rows<N>,
        I: Types<Instance: Fields<Tuple = R::Inner>>,
    {
        self.run.instance(rows);
        self
    }

    pub fn draw(mut self, mesh: &Mesh<I::Vertex>) -> Self
    where
        I: Types,
    {
        self.run.draw(mesh);
        self
    }

    pub fn draw_points(mut self, n: u32) -> Self
    where
        I: Types<Vertex = ()>,
    {
        self.run.draw_points(n);
        self
    }
}
