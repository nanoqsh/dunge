use {
    crate::{group::Bind, mesh::Mesh, shader::Shader, state::State},
    std::{iter, marker::PhantomData},
    wgpu::{RenderPass, RenderPipeline, TextureFormat},
};

pub struct SetLayer<'p, V> {
    shader_id: usize,
    pass: RenderPass<'p>,
    vert: PhantomData<V>,
}

impl<'p, V> SetLayer<'p, V> {
    pub(crate) fn bind<B>(&mut self, binding: &'p B) -> BoundLayer<'_, 'p, V>
    where
        B: Bind,
    {
        if self.shader_id != binding.shader_id() {
            panic!("the binding doesn't belong to this shader");
        }

        for (id, group) in iter::zip(0.., binding.binds()) {
            self.pass.set_bind_group(id, group, &[]);
        }

        BoundLayer {
            pass: &mut self.pass,
            vert: PhantomData,
        }
    }
}

pub struct BoundLayer<'s, 'p, V> {
    pass: &'s mut RenderPass<'p>,
    vert: PhantomData<V>,
}

impl<'p, V> BoundLayer<'_, 'p, V> {
    pub(crate) fn draw(&mut self, mesh: &'p Mesh<V>) {
        mesh.draw(self.pass);
    }
}

pub struct Layer<V> {
    inner: Inner,
    ty: PhantomData<V>,
}

impl<V> Layer<V> {
    pub(crate) fn new(state: &State, format: TextureFormat, shader: &Shader<V>) -> Self {
        Self {
            inner: Inner::new(state, format, shader),
            ty: PhantomData,
        }
    }

    pub(crate) fn set<'pass>(&'pass self, mut pass: RenderPass<'pass>) -> SetLayer<'pass, V> {
        pass.set_pipeline(&self.inner.pipeline);
        SetLayer {
            shader_id: self.inner.shader_id,
            pass,
            vert: PhantomData,
        }
    }
}

struct Inner {
    shader_id: usize,
    pipeline: RenderPipeline,
}

impl Inner {
    fn new<V>(state: &State, format: TextureFormat, shader: &Shader<V>) -> Self {
        use wgpu::*;

        let targets = [Some(ColorTargetState {
            format,
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

        let pipeline = state.device().create_render_pipeline(&desc);
        Self {
            shader_id: 0,
            pipeline,
        }
    }
}
