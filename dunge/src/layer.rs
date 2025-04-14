//! Layer types.

use {
    crate::{
        format::Format,
        instance::Set,
        mesh::Mesh,
        render::Input,
        set::Bind,
        shader::{ShaderData, SlotNumbers},
        state::State,
    },
    std::{iter, marker::PhantomData},
};

pub struct SetLayer<'ren, D, S> {
    slots: SlotNumbers,
    pass: wgpu::RenderPass<'ren>,
    ty: PhantomData<(D, S)>,
}

impl<'ren, V, I, S> SetLayer<'ren, (V, I), S> {
    #[inline]
    pub fn with<B>(&mut self, bind: &'ren B) -> SetBinding<'_, 'ren, (V, I)>
    where
        B: Bind<S>,
    {
        let bind = bind.bind();
        for (id, group) in iter::zip(0.., bind.bind_groups) {
            self.pass.set_bind_group(id, group, &[]);
        }

        SetBinding::new(self.slots, &mut self.pass)
    }
}

impl<'ren, V, I> SetLayer<'ren, (V, I), ()> {
    #[inline]
    pub fn bind_empty(&mut self) -> SetBinding<'_, 'ren, (V, I)> {
        SetBinding::new(self.slots, &mut self.pass)
    }
}

pub struct SetBinding<'bind, 'ren, D> {
    slots: SlotNumbers,
    pass: &'bind mut wgpu::RenderPass<'ren>,
    ty: PhantomData<D>,
}

impl<'bind, 'ren, V, I> SetBinding<'bind, 'ren, (V, I)> {
    fn new(slots: SlotNumbers, pass: &'bind mut wgpu::RenderPass<'ren>) -> Self {
        Self {
            slots,
            pass,
            ty: PhantomData,
        }
    }

    #[inline]
    pub fn instance(&'bind mut self, instance: &'ren I) -> SetInstance<'bind, 'ren, V>
    where
        I: Set,
    {
        let len = crate::instance::set(
            crate::render::VertexSetter::_new(self.pass),
            self.slots.instance,
            instance,
        );

        SetInstance {
            len,
            slots: self.slots,
            pass: self.pass,
            ty: PhantomData,
        }
    }
}

impl<'ren, V> SetBinding<'_, 'ren, (V, ())> {
    #[inline]
    pub fn draw(&mut self, mesh: &'ren Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, 1);
    }
}

impl SetBinding<'_, '_, ((), ())> {
    #[inline]
    pub fn draw_points(&mut self, n: u32) {
        self.pass.draw(0..n, 0..1);
    }
}

pub struct SetInstance<'bind, 'ren, V> {
    len: u32,
    slots: SlotNumbers,
    pass: &'bind mut wgpu::RenderPass<'ren>,
    ty: PhantomData<V>,
}

impl<'ren, V> SetInstance<'_, 'ren, V> {
    #[inline]
    pub fn draw(&mut self, mesh: &'ren Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, self.len);
    }
}

impl SetInstance<'_, '_, ()> {
    #[inline]
    pub fn draw_points(&mut self, n: u32) {
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
    fn wgpu(self) -> Option<wgpu::BlendState> {
        match self {
            Self::None => None,
            Self::Replace => Some(wgpu::BlendState::REPLACE),
            Self::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
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
    fn wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::PointList => wgpu::PrimitiveTopology::PointList,
            Self::LineList => wgpu::PrimitiveTopology::LineList,
            Self::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Self::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Self::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        }
    }
}

#[derive(Default)]
pub struct Config {
    pub format: Format,
    pub blend: Blend,
    pub topology: Topology,
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

pub struct Layer<I> {
    slots: SlotNumbers,
    depth: bool,
    format: Format,
    render: wgpu::RenderPipeline,
    inp: PhantomData<I>,
}

impl<I> Layer<I> {
    pub(crate) fn new(state: &State, shader: &ShaderData, conf: &Config) -> Self {
        let Config {
            format,
            blend,
            topology,
            depth,
        } = conf;

        let targets = [Some(wgpu::ColorTargetState {
            format: format.wgpu(),
            blend: blend.wgpu(),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let module = shader.module();
        let buffers = shader.vertex_buffers();
        let topology = topology.wgpu();
        let desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            vertex: wgpu::VertexState {
                module,
                entry_point: Some("vs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &buffers,
            },
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: topology.is_strip().then_some(wgpu::IndexFormat::Uint16),
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: depth.then_some(wgpu::DepthStencilState {
                format: Format::Depth.wgpu(),
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: Some("fs"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &targets,
            }),
            multiview: None,
            cache: None,
        };

        let render = state.device().create_render_pipeline(&desc);

        Self {
            slots: shader.slots(),
            depth: *depth,
            format: *format,
            render,
            inp: PhantomData,
        }
    }

    pub(crate) fn slots(&self) -> SlotNumbers {
        self.slots
    }

    pub fn depth(&self) -> bool {
        self.depth
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn render(&self) -> &wgpu::RenderPipeline {
        &self.render
    }
}

impl<V, I, S> Layer<Input<V, I, S>> {
    pub(crate) fn _set<'ren>(
        &'ren self,
        mut pass: wgpu::RenderPass<'ren>,
    ) -> SetLayer<'ren, (V, I), S> {
        pass.set_pipeline(&self.render);
        SetLayer {
            slots: self.slots,
            pass,
            ty: PhantomData,
        }
    }
}
