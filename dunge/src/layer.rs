use crate::table::Table;

use {
    crate::{
        bind::Binding,
        format::Format,
        mesh::Mesh,
        shader::{Shader, Slots},
        state::State,
    },
    std::{iter, marker::PhantomData},
    wgpu::{RenderPass, RenderPipeline},
};

pub struct SetLayer<'p, V, I> {
    shader_id: usize,
    no_bindings: bool,
    slots: Slots,
    pass: RenderPass<'p>,
    ty: PhantomData<(V, I)>,
}

impl<'p, V, I> SetLayer<'p, V, I> {
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

        SetBinding::new(self.slots, &mut self.pass)
    }

    pub fn bind_empty(&mut self) -> SetBinding<'_, 'p, V, I> {
        assert!(self.no_bindings, "ths shader has any bindings");
        SetBinding::new(self.slots, &mut self.pass)
    }
}

pub struct SetBinding<'s, 'p, V, I> {
    slots: Slots,
    pass: &'s mut RenderPass<'p>,
    ty: PhantomData<(V, I)>,
}

impl<'s, 'p, V, I> SetBinding<'s, 'p, V, I> {
    fn new(slots: Slots, pass: &'s mut RenderPass<'p>) -> Self {
        Self {
            slots,
            pass,
            ty: PhantomData,
        }
    }

    pub fn instance(&'s mut self, table: &'p Table<I>) -> SetInstance<'s, 'p, V> {
        table.set(self.pass, self.slots.instance);
        SetInstance {
            count: table.count(),
            slots: self.slots,
            pass: self.pass,
            ty: PhantomData,
        }
    }
}

impl<'p, V> SetBinding<'_, 'p, V, ()> {
    pub fn draw(&mut self, mesh: &'p Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, 1);
    }
}

impl SetBinding<'_, '_, (), ()> {
    pub fn draw_triangles(&mut self, count: u32) {
        self.pass.draw(0..count * 3, 0..1);
    }
}

pub struct SetInstance<'s, 'p, V> {
    count: u32,
    slots: Slots,
    pass: &'s mut RenderPass<'p>,
    ty: PhantomData<V>,
}

impl<'p, V> SetInstance<'_, 'p, V> {
    pub fn draw(&mut self, mesh: &'p Mesh<V>) {
        mesh.draw(self.pass, self.slots.vertex, self.count);
    }
}

impl SetInstance<'_, '_, ()> {
    pub fn draw_triangles(&mut self, count: u32) {
        self.pass.draw(0..count * 3, 0..self.count);
    }
}

pub struct Layer<V, I> {
    shader_id: usize,
    no_bindings: bool,
    slots: Slots,
    format: Format,
    render: RenderPipeline,
    ty: PhantomData<(V, I)>,
}

impl<V, I> Layer<V, I> {
    pub(crate) fn new(state: &State, format: Format, shader: &Shader<V, I>) -> Self {
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
            slots: shader.slots(),
            format,
            render,
            ty: PhantomData,
        }
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub(crate) fn set<'p>(&'p self, mut pass: RenderPass<'p>) -> SetLayer<'p, V, I> {
        pass.set_pipeline(&self.render);
        SetLayer {
            shader_id: self.shader_id,
            no_bindings: self.no_bindings,
            slots: self.slots,
            pass,
            ty: PhantomData,
        }
    }
}
