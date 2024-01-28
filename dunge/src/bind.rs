use {
    crate::{
        group::BoundTexture, shader::Shader, state::State, texture::Sampler, uniform::Uniform,
        Group,
    },
    std::{any::TypeId, error, fmt, marker::PhantomData, sync::Arc},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    },
};

pub trait Visit: Group {
    const N_MEMBERS: usize;
    fn visit<'a>(&'a self, visitor: &mut Visitor<'a>);
}

pub struct Visitor<'a>(Vec<BindGroupEntry<'a>>);

impl<'a> Visitor<'a> {
    fn push(&mut self, resource: BindingResource<'a>) {
        let binding = self.0.len() as u32;
        self.0.push(BindGroupEntry { binding, resource });
    }
}

pub trait VisitMember<'a> {
    fn visit_member(self, visitor: &mut Visitor<'a>);
}

impl<'a, V> VisitMember<'a> for &'a Uniform<V> {
    fn visit_member(self, visitor: &mut Visitor<'a>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(BindingResource::Buffer(binding));
    }
}

impl<'a> VisitMember<'a> for BoundTexture<'a> {
    fn visit_member(self, visitor: &mut Visitor<'a>) {
        visitor.push(BindingResource::TextureView(self.get().view()));
    }
}

impl<'a> VisitMember<'a> for &'a Sampler {
    fn visit_member(self, visitor: &mut Visitor<'a>) {
        visitor.push(BindingResource::Sampler(self.inner()));
    }
}

fn visit<G>(group: &G) -> Vec<BindGroupEntry>
where
    G: Visit,
{
    let mut visitor = Visitor(Vec::with_capacity(G::N_MEMBERS));
    group.visit(&mut visitor);
    visitor.0
}

pub struct GroupHandler<P> {
    shader_id: usize,
    id: usize,
    layout: Arc<BindGroupLayout>,
    ty: PhantomData<P>,
}

#[derive(Debug)]
pub struct ForeignShader;

impl fmt::Display for ForeignShader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "the handler doesn't belong to this shader")
    }
}

impl error::Error for ForeignShader {}

pub trait Binding {
    fn binding(&self) -> Bind;
}

pub struct Bind<'a> {
    pub(crate) shader_id: usize,
    pub(crate) groups: &'a [BindGroup],
}

#[derive(Clone)]
pub struct SharedBinding {
    shader_id: usize,
    groups: Arc<[BindGroup]>,
}

impl SharedBinding {
    fn new(shader_id: usize, groups: Vec<BindGroup>) -> Self {
        Self {
            shader_id,
            groups: Arc::from(groups),
        }
    }
}

impl Binding for SharedBinding {
    fn binding(&self) -> Bind {
        Bind {
            shader_id: self.shader_id,
            groups: &self.groups,
        }
    }
}

pub(crate) fn update<G>(
    state: &State,
    uni: &mut UniqueBinding,
    handler: &GroupHandler<G::Projection>,
    group: &G,
) -> Result<(), ForeignShader>
where
    G: Visit,
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

pub struct UniqueBinding(SharedBinding);

impl UniqueBinding {
    pub fn shared(self) -> SharedBinding {
        self.0
    }

    fn groups(&mut self) -> &mut [BindGroup] {
        Arc::get_mut(&mut self.0.groups).expect("uniqueness is guaranteed by the type")
    }
}

impl Binding for UniqueBinding {
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
    pub(crate) fn new<V, I>(state: &'a State, shader: &'a Shader<V, I>) -> Self {
        let layout = shader.groups();
        Self {
            shader_id: shader.id(),
            device: state.device(),
            layout,
            groups: Vec::with_capacity(layout.len()),
        }
    }

    pub fn bind<G>(&mut self, group: &G) -> GroupHandler<G::Projection>
    where
        G: Visit,
    {
        let id = self.groups.len();
        let Some(layout) = self.layout.get(id) else {
            panic!("too many bindings");
        };

        assert!(
            layout.tyid == TypeId::of::<G::Projection>(),
            "group type doesn't match",
        );

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

    pub fn into_binding(self) -> UniqueBinding {
        assert!(
            self.groups.len() == self.layout.len(),
            "some group bindings is not set",
        );

        let binding = SharedBinding::new(self.shader_id, self.groups);
        UniqueBinding(binding)
    }
}
