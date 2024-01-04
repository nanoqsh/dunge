use {
    crate::{
        group::{BoundTexture, Group, Visitor},
        shader::Shader,
        state::State,
        texture::Sampler,
    },
    std::{any::TypeId, fmt, marker::PhantomData, sync::Arc},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    },
};

#[derive(Default)]
pub struct VisitGroup<'g>(Vec<BindGroupEntry<'g>>);

impl<'g> VisitGroup<'g> {
    fn visit_texture(&mut self, texture: BoundTexture<'g>) {
        self.push_resource(BindingResource::TextureView(texture.get().view()));
    }

    fn visit_sampler(&mut self, sampler: &'g Sampler) {
        self.push_resource(BindingResource::Sampler(sampler.inner()));
    }

    fn push_resource(&mut self, resource: BindingResource<'g>) {
        let binding = self.0.len() as u32;
        self.0.push(BindGroupEntry { binding, resource });
    }
}

impl<'g> Visitor for VisitGroup<'g> {
    type Texture = BoundTexture<'g>;
    type Sampler = &'g Sampler;

    fn visit_texture(&mut self, texture: Self::Texture) {
        self.visit_texture(texture);
    }

    fn visit_sampler(&mut self, sampler: Self::Sampler) {
        self.visit_sampler(sampler);
    }
}

fn visit<'g, G>(group: &'g G) -> Vec<BindGroupEntry<'g>>
where
    G: Group<Visitor = VisitGroup<'g>>,
{
    let mut visit = VisitGroup::default();
    group.group(&mut visit);
    visit.0
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
}

impl Binding for GroupBinding {
    fn binding(&self) -> Bind {
        Bind {
            shader_id: self.shader_id,
            groups: &self.groups,
        }
    }
}

pub type Update = Result<(), ForeignShader>;

pub(crate) fn update<'g, G>(
    state: &State,
    uni: &mut UniqueGroupBinding,
    handler: GroupHandler<G>,
    group: &'g G,
) -> Update
where
    G: Group<Visitor = VisitGroup<'g>>,
{
    if handler.shader_id != uni.0.shader_id {
        return Err(ForeignShader);
    }

    let entries = visit(group);
    let desc = BindGroupDescriptor {
        label: None,
        layout: &handler.layout,
        entries: &entries,
    };

    let new = state.device().create_bind_group(&desc);
    let groups = uni.groups();
    groups[handler.id] = new;
    Ok(())
}

pub struct UniqueGroupBinding(GroupBinding);

impl UniqueGroupBinding {
    pub fn into_inner(self) -> GroupBinding {
        self.0
    }

    fn groups(&mut self) -> &mut [BindGroup] {
        Arc::get_mut(&mut self.0.groups).expect("uniqueness is guaranteed by the type")
    }
}

impl Binding for UniqueGroupBinding {
    fn binding(&self) -> Bind {
        self.0.binding()
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

    pub fn bind<'g, G>(&mut self, group: &'g G) -> GroupHandler<G>
    where
        G: Group<Visitor = VisitGroup<'g>>,
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
