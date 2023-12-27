use {
    crate::{
        shader::Shader,
        state::State,
        texture::{self, Sampler, Texture},
    },
    std::{any::TypeId, fmt, marker::PhantomData, sync::Arc},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    },
};

pub use dunge_shader::group::*;

#[derive(Clone, Copy)]
pub struct BindTexture<'a>(&'a Texture);

impl<'a> BindTexture<'a> {
    pub fn new<T>(texture: &'a T) -> Self
    where
        T: texture::BindTexture,
    {
        Self(texture.bind_texture())
    }
}

fn visit<'a, G>(group: &'a G) -> Vec<BindGroupEntry<'a>>
where
    G: Group,
{
    let mut visit = VisitGroup::default();
    group.group(&mut visit);
    visit.0
}

#[derive(Default)]
struct VisitGroup<'a>(Vec<BindGroupEntry<'a>>);

impl<'a> VisitGroup<'a> {
    fn visit_texture(&mut self, texture: BindTexture<'a>) {
        self.push_resource(BindingResource::TextureView(texture.0.view()));
    }

    fn visit_sampler(&mut self, sampler: &'a Sampler) {
        self.push_resource(BindingResource::Sampler(sampler.inner()));
    }

    fn push_resource(&mut self, resource: BindingResource<'a>) {
        let binding = self.0.len() as u32;
        self.0.push(BindGroupEntry { binding, resource });
    }
}

impl<'a> Visitor for VisitGroup<'a> {
    type Texture = BindTexture<'a>;
    type Sampler = &'a Sampler;

    fn visit_texture(&mut self, texture: Self::Texture) {
        self.visit_texture(texture);
    }

    fn visit_sampler(&mut self, sampler: Self::Sampler) {
        self.visit_sampler(sampler);
    }
}

pub struct GroupHandler<G> {
    shader_id: usize,
    id: usize,
    layout: Arc<BindGroupLayout>,
    ty: PhantomData<G>,
}

pub struct ForeignShader;

impl fmt::Display for ForeignShader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the handler doesn't belong to this shader")
    }
}

pub trait Binding {
    fn binding(&self) -> Bind;
}

pub struct Bind<'a> {
    pub(crate) shader_id: usize,
    pub(crate) groups: &'a [BindGroup],
}

#[derive(Clone)]
pub struct GroupBinding {
    shader_id: usize,
    groups: Arc<[BindGroup]>,
}

impl GroupBinding {
    fn new(shader_id: usize, groups: Vec<BindGroup>) -> Self {
        Self {
            shader_id,
            groups: Arc::from(groups),
        }
    }

    fn bind(&self) -> Bind {
        Bind {
            shader_id: self.shader_id,
            groups: &self.groups,
        }
    }
}

pub type Update = Result<(), ForeignShader>;

pub struct UniqueGroupBinding(GroupBinding);

impl UniqueGroupBinding {
    pub(crate) fn update<G>(&mut self, handler: GroupHandler<G>, state: &State, group: &G) -> Update
    where
        G: Group,
    {
        if handler.shader_id != self.0.shader_id {
            return Err(ForeignShader);
        }

        let entries = visit(group);
        let desc = BindGroupDescriptor {
            label: None,
            layout: &handler.layout,
            entries: &entries,
        };

        let new = state.device().create_bind_group(&desc);
        let groups = self.groups();
        groups[handler.id] = new;
        Ok(())
    }

    pub fn into_inner(self) -> GroupBinding {
        self.0
    }

    fn groups(&mut self) -> &mut [BindGroup] {
        Arc::get_mut(&mut self.0.groups).expect("uniqueness is guaranteed by the type")
    }

    fn bind(&self) -> Bind {
        self.0.bind()
    }
}

pub(crate) struct TypedGroup {
    tyid: TypeId,
    bind: Arc<BindGroupLayout>,
}

impl TypedGroup {
    pub fn new(tyid: TypeId, bind: BindGroupLayout) -> Self {
        Self {
            tyid,
            bind: Arc::new(bind),
        }
    }

    pub fn bind(&self) -> &BindGroupLayout {
        &self.bind
    }
}

pub struct Binder<'a> {
    shader_id: usize,
    device: &'a Device,
    layout: &'a [TypedGroup],
    groups: Vec<BindGroup>,
}

impl<'a> Binder<'a> {
    pub(crate) fn new<V>(state: &'a State, shader: &'a Shader<V>) -> Self {
        let layout = shader.groups();
        Self {
            shader_id: shader.id(),
            device: state.device(),
            layout,
            groups: Vec::with_capacity(layout.len()),
        }
    }

    pub fn bind<G>(&mut self, group: &G) -> GroupHandler<G>
    where
        G: Group,
    {
        let id = self.groups.len();
        let Some(layout) = self.layout.get(id) else {
            panic!("too many bindings");
        };

        if layout.tyid != TypeId::of::<G::Projection>() {
            panic!("group type doesn't match");
        }

        let layout = Arc::clone(&layout.bind);
        let entries = visit(group);
        let desc = BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &entries,
        };

        let bind = self.device.create_bind_group(&desc);
        self.groups.push(bind);

        GroupHandler {
            shader_id: self.shader_id,
            id,
            layout,
            ty: PhantomData,
        }
    }

    pub fn into_binding(self) -> UniqueGroupBinding {
        if self.groups.len() != self.layout.len() {
            panic!("some group bindings is not set");
        }

        let binding = GroupBinding::new(self.shader_id, self.groups);
        UniqueGroupBinding(binding)
    }
}
