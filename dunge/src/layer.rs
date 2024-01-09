use {
    crate::{bind::Binding, format::Format, mesh::Mesh, shader::Shader, state::State},
    std::{iter, marker::PhantomData},
    wgpu::{RenderPass, RenderPipeline},
};

pub struct SetLayer<'p, V> {
    shader_id: usize,
    no_bindings: bool,
    pass: RenderPass<'p>,
    vert: PhantomData<V>,
}

impl<'p, V> SetLayer<'p, V> {
    pub fn bind<B>(&mut self, bind: &'p B) -> BoundLayer<'_, 'p, V>
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

        BoundLayer::new(&mut self.pass)
    }

    pub fn bind_empty(&mut self) -> BoundLayer<'_, 'p, V> {
        assert!(self.no_bindings, "ths shader has any bindings");
        BoundLayer::new(&mut self.pass)
    }
}

pub struct BoundLayer<'s, 'p, V> {
    pass: &'s mut RenderPass<'p>,
    vert: PhantomData<V>,
}

impl<'s, 'p, V> BoundLayer<'s, 'p, V> {
    fn new(pass: &'s mut RenderPass<'p>) -> Self {
        Self {
            pass,
            vert: PhantomData,
        }
    }

    pub fn draw(&mut self, mesh: &'p Mesh<V>) {
        mesh.draw(self.pass);
    }
}

impl BoundLayer<'_, '_, ()> {
    pub fn draw_triangles(&mut self, n: u32) {
        self.pass.draw(0..n * 3, 0..1);
    }
}

pub struct Layer<V> {
    shader_id: usize,
    no_bindings: bool,
    format: Format,
    render: RenderPipeline,
    vertex: PhantomData<V>,
}

impl<V> Layer<V> {
    pub(crate) fn new(state: &State, format: Format, shader: &Shader<V>) -> Self {
        use wgpu::*;

        let targets = [Some(ColorTargetState {
            format: format.wgpu(),
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        let module = shader.module();
        let buffers = shader.buffers();
        let desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(shader.layout()),
            vertex: VertexState {
                module,
                entry_point: "vs",
                buffers: &buffers,
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
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
            format,
            render,
            vertex: PhantomData,
        }
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn set<'p>(&'p self, mut pass: RenderPass<'p>) -> SetLayer<'p, V> {
        pass.set_pipeline(&self.render);
        SetLayer {
            shader_id: self.shader_id,
            no_bindings: self.no_bindings,
            pass,
            vert: PhantomData,
        }
    }
}
