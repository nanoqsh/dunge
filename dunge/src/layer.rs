use {
    crate::{
        bind::Binding,
        format::Format,
        instance::{Set, Setter},
        mesh::Mesh,
        shader::{Shader, Slots},
        state::State,
    },
    std::{iter, marker::PhantomData},
    wgpu::{BlendState, PrimitiveTopology, RenderPass, RenderPipeline},
};

pub struct SetLayer<'p, V, I> {
    shader_id: usize,
    no_bindings: bool,
    only_indexed_mesh: bool,
    slots: Slots,
    pass: RenderPass<'p>,
    ty: PhantomData<(V, I)>,
}

impl<'p, V, I> SetLayer<'p, V, I> {
    #[inline]
    pub fn bind<B>(&mut self, bind: &'p B) -> SetBinding<'_, 'p, V, I>
    where
        B: Binding,
    {
        let bind = bind.binding();
        assert!(
            self.shader_id == bind.shader_id,
            "the binding doesn't belong to this shader",
        );

        for (id, group) in iter::zip(0.., bind.groups) {
            self.pass.set_bind_group(id, group, &[]);
        }

        SetBinding::new(self.only_indexed_mesh, self.slots, &mut self.pass)
    }

    #[inline]
    pub fn bind_empty(&mut self) -> SetBinding<'_, 'p, V, I> {
        assert!(self.no_bindings, "ths shader has any bindings");
        SetBinding::new(self.only_indexed_mesh, self.slots, &mut self.pass)
    }
}

pub struct SetBinding<'s, 'p, V, I> {
    only_indexed_mesh: bool,
    slots: Slots,
    pass: &'s mut RenderPass<'p>,
    ty: PhantomData<(V, I)>,
}

impl<'s, 'p, V, I> SetBinding<'s, 'p, V, I> {
    fn new(only_indexed_mesh: bool, slots: Slots, pass: &'s mut RenderPass<'p>) -> Self {
        Self {
            only_indexed_mesh,
            slots,
            pass,
            ty: PhantomData,
        }
    }

    #[inline]
    pub fn instance(&'s mut self, instance: &'p I) -> SetInstance<'s, 'p, V>
    where
        I: Set,
    {
        let mut setter = Setter::new(self.slots.instance, self.pass);
        instance.set(&mut setter);
        SetInstance {
            only_indexed_mesh: self.only_indexed_mesh,
            len: setter.len(),
            slots: self.slots,
            pass: self.pass,
            ty: PhantomData,
        }
    }
}

impl<'p, V> SetBinding<'_, 'p, V, ()> {
    #[inline]
    pub fn draw(&mut self, mesh: &'p Mesh<V>) {
        assert!(
            !self.only_indexed_mesh || mesh.is_indexed(),
            "only an indexed mesh can be drawn on this layer",
        );

        mesh.draw(self.pass, self.slots.vertex, 1);
    }
}

impl SetBinding<'_, '_, (), ()> {
    #[inline]
    pub fn draw_points(&mut self, n: u32) {
        assert!(
            !self.only_indexed_mesh,
            "only an indexed mesh can be drawn on this layer",
        );

        self.pass.draw(0..n, 0..1);
    }
}

pub struct SetInstance<'s, 'p, V> {
    only_indexed_mesh: bool,
    len: u32,
    slots: Slots,
    pass: &'s mut RenderPass<'p>,
    ty: PhantomData<V>,
}

impl<'p, V> SetInstance<'_, 'p, V> {
    #[inline]
    pub fn draw(&mut self, mesh: &'p Mesh<V>) {
        assert!(
            !self.only_indexed_mesh || mesh.is_indexed(),
            "only an indexed mesh can be drawn on this layer",
        );

        mesh.draw(self.pass, self.slots.vertex, self.len);
    }
}

impl SetInstance<'_, '_, ()> {
    #[inline]
    pub fn draw_points(&mut self, n: u32) {
        assert!(
            !self.only_indexed_mesh,
            "only an indexed mesh can be drawn on this layer",
        );

        self.pass.draw(0..n, 0..self.len);
    }
}

#[derive(Clone, Copy, Default)]
pub enum Blend {
    #[default]
    None,
    Replace,
    Alpha,
}

impl Blend {
    fn wgpu(self) -> Option<BlendState> {
        match self {
            Self::None => None,
            Self::Replace => Some(BlendState::REPLACE),
            Self::Alpha => Some(BlendState::ALPHA_BLENDING),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

impl Topology {
    fn wgpu(self) -> PrimitiveTopology {
        match self {
            Self::PointList => PrimitiveTopology::PointList,
            Self::LineList => PrimitiveTopology::LineList,
            Self::LineStrip => PrimitiveTopology::LineStrip,
            Self::TriangleList => PrimitiveTopology::TriangleList,
            Self::TriangleStrip => PrimitiveTopology::TriangleStrip,
        }
    }
}

#[derive(Default)]
pub struct Config {
    pub format: Format,
    pub blend: Blend,
    pub topology: Topology,
    pub indexed_mesh: bool,
    pub depth: bool,
}

impl From<Format> for Config {
    fn from(format: Format) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }
}

pub struct Layer<V, I> {
    shader_id: usize,
    no_bindings: bool,
    only_indexed_mesh: bool,
    slots: Slots,
    depth: bool,
    format: Format,
    render: RenderPipeline,
    ty: PhantomData<(V, I)>,
}

impl<V, I> Layer<V, I> {
    pub(crate) fn new(state: &State, shader: &Shader<V, I>, conf: &Config) -> Self {
        use wgpu::*;

        let Config {
            format,
            blend,
            topology,
            indexed_mesh,
            depth,
        } = conf;

        let targets = [Some(ColorTargetState {
            format: format.wgpu(),
            blend: blend.wgpu(),
            write_mask: ColorWrites::ALL,
        })];

        let module = shader.module();
        let buffers = shader.buffers();
        let topology = topology.wgpu();
        let only_indexed_mesh = *indexed_mesh && topology.is_strip();
        let desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            vertex: VertexState {
                module,
                entry_point: "vs",
                buffers: &buffers,
            },
            primitive: PrimitiveState {
                topology,
                strip_index_format: only_indexed_mesh.then_some(IndexFormat::Uint16),
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: depth.then_some(DepthStencilState {
                format: Format::Depth.wgpu(),
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module,
                entry_point: "fs",
                targets: &targets,
            }),
            multiview: None,
        };

        let render = state.device().create_render_pipeline(&desc);
        Self {
            shader_id: shader.id(),
            no_bindings: shader.groups().is_empty(),
            only_indexed_mesh,
            slots: shader.slots(),
            depth: *depth,
            format: *format,
            render,
            ty: PhantomData,
        }
    }

    pub fn depth(&self) -> bool {
        self.depth
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn set<'p>(&'p self, mut pass: RenderPass<'p>) -> SetLayer<'p, V, I> {
        pass.set_pipeline(&self.render);
        SetLayer {
            shader_id: self.shader_id,
            no_bindings: self.no_bindings,
            only_indexed_mesh: self.only_indexed_mesh,
            slots: self.slots,
            pass,
            ty: PhantomData,
        }
    }
}
