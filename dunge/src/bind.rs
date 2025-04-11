//! Shader binding types.

use {
    crate::{
        group::BoundTexture,
        shader::{Shader, ShaderData},
        state::State,
        storage::Storage,
        texture::Sampler,
        uniform::Uniform,
        Group,
    },
    std::{marker::PhantomData, mem, sync::Arc},
    wgpu::{
        BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    },
};

pub trait Visit: Group {
    const N_MEMBERS: usize;
    fn visit<'group>(&'group self, visitor: &mut Visitor<'group>);
}

impl<V> Visit for &V
where
    V: Visit,
{
    const N_MEMBERS: usize = V::N_MEMBERS;

    fn visit<'group>(&'group self, visitor: &mut Visitor<'group>) {
        (*self).visit(visitor);
    }
}

pub struct Visitor<'group>(Vec<wgpu::BindGroupEntry<'group>>);

impl<'group> Visitor<'group> {
    fn clear(&mut self) {
        self.0.clear();
    }

    fn visit<G>(&mut self, group: &'group G)
    where
        G: Visit,
    {
        let mut visitor = Visitor(mem::take(&mut self.0));
        group.visit(&mut visitor);
        self.0 = visitor.0;
    }

    fn entries(&self) -> &[wgpu::BindGroupEntry<'group>] {
        &self.0
    }
}

impl<'group> Visitor<'group> {
    fn push(&mut self, resource: BindingResource<'group>) {
        let binding = self.0.len() as u32;
        self.0.push(BindGroupEntry { binding, resource });
    }
}

pub trait VisitMember<'group> {
    fn visit_member(self, visitor: &mut Visitor<'group>);
}

impl<'group, V, M> VisitMember<'group> for &'group Storage<V, M>
where
    V: ?Sized,
{
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(BindingResource::Buffer(binding));
    }
}

impl<'group, V> VisitMember<'group> for &'group Uniform<V> {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(BindingResource::Buffer(binding));
    }
}

impl<'group> VisitMember<'group> for BoundTexture<'group> {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        visitor.push(BindingResource::TextureView(self.0.view()));
    }
}

impl<'group> VisitMember<'group> for &'group Sampler {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        visitor.push(BindingResource::Sampler(self.inner()));
    }
}

fn _visit<G>(group: &G) -> Vec<BindGroupEntry>
where
    G: Visit,
{
    let mut visitor = Visitor(Vec::with_capacity(G::N_MEMBERS));
    group.visit(&mut visitor);
    visitor.0
}

pub struct GroupHandler<S, P> {
    id: usize,
    layout: Arc<BindGroupLayout>,
    ty: PhantomData<(S, P)>,
}

pub trait Bind<S> {
    fn bind(&self) -> Bindings<'_>;
}

pub struct Bindings<'group> {
    pub(crate) bind_groups: &'group [BindGroup],
}

#[derive(Clone)]
pub struct SharedBinding {
    groups: Arc<[BindGroup]>,
}

impl SharedBinding {
    fn new(groups: Vec<BindGroup>) -> Self {
        Self {
            groups: Arc::from(groups),
        }
    }
}

impl<S> Bind<S> for SharedBinding {
    fn bind(&self) -> Bindings {
        Bindings {
            bind_groups: &self.groups,
        }
    }
}

pub(crate) fn _update<S, G>(
    state: &State,
    uni: &mut UniqueBinding,
    handler: &GroupHandler<S, G::Projection>,
    group: &G,
) where
    G: Visit,
{
    let device = state.device();
    group.set(&mut |_, visitor| {
        let entries = visitor.entries();
        let desc = BindGroupDescriptor {
            label: None,
            layout: &handler.layout,
            entries,
        };

        let new = device.create_bind_group(&desc);
        let groups = uni.groups();
        groups[handler.id] = new;
    });
}

pub(crate) fn update<S, G>(
    state: &State,
    set: &mut UniqueSet<S>,
    handler: &GroupHandler<S, G::Projection>,
    group: G,
) where
    G: Visit,
{
    let device = state.device();
    group.set(&mut |_, visitor| {
        let entries = visitor.entries();
        let desc = BindGroupDescriptor {
            label: None,
            layout: &handler.layout,
            entries,
        };

        let new = device.create_bind_group(&desc);
        let groups = set.bind_groups();
        groups[handler.id] = new;
    });
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

impl<S> Bind<S> for UniqueBinding {
    fn bind(&self) -> Bindings {
        <SharedBinding as Bind<S>>::bind(&self.0)
    }
}

/// The group binder type.
///
/// Can be created using the context's [`make_binder`](crate::Context::make_binder) function.
pub struct Binder<'state> {
    device: &'state Device,
    layout: &'state [Arc<BindGroupLayout>],
    groups: Vec<BindGroup>,
}

impl<'state> Binder<'state> {
    pub(crate) fn new(state: &'state State, shader: &'state ShaderData) -> Self {
        let layout = shader.groups();
        Self {
            device: state.device(),
            layout,
            groups: Vec::with_capacity(layout.len()),
        }
    }

    /// Adds a group to the associated shader's binding.
    ///
    /// It returns a [group handler](GroupHandler) that can be used to update
    /// the data in this binding. If you don't need to update the data, then
    /// discard this handler.
    ///
    /// # Panic
    /// It checks the group type matches to an associated shader's group at runtime.
    /// If it's violated or there are more bindings than in the shader,
    /// then this function will panic.
    pub fn add<G>(&mut self, group: &G) -> GroupHandler<(), G::Projection>
    where
        G: Visit,
    {
        let id = self.groups.len();
        let Some(layout) = self.layout.get(id) else {
            panic!("too many bindings");
        };

        let layout = Arc::clone(&layout);
        let entries = _visit(group);
        let desc = BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &entries,
        };

        let bind = self.device.create_bind_group(&desc);
        self.groups.push(bind);

        GroupHandler {
            id,
            layout,
            ty: PhantomData,
        }
    }

    /// Constructs an object that can be [used](crate::layer::SetLayer::bind)
    /// in the draw stage.
    ///
    /// # Panic
    /// It will panic if some group bindings is not set.
    pub fn into_binding(self) -> UniqueBinding {
        assert!(
            self.groups.len() == self.layout.len(),
            "some group bindings is not set",
        );

        let binding = SharedBinding::new(self.groups);
        UniqueBinding(binding)
    }
}

pub struct UniqueSet<S>(SharedSet<S>);

impl<S> UniqueSet<S> {
    pub(crate) fn new(state: &State, shader: &ShaderData, set: S) -> Self
    where
        S: Set,
    {
        let groups = shader.groups();
        let mut bind_groups = Vec::with_capacity(groups.len());

        let device = state.device();
        set.set(&mut |id, visitor| {
            let layout = &groups[id];
            let entries = visitor.entries();
            let desc = BindGroupDescriptor {
                label: None,
                layout,
                entries,
            };

            bind_groups.push(device.create_bind_group(&desc));
        });

        Self(SharedSet {
            bind_groups: Arc::from(bind_groups),
            ty: PhantomData,
        })
    }

    pub fn handler<K, const N: usize>(
        &self,
        shader: &Shader<K, S>,
    ) -> GroupHandler<S, <S::Group as Group>::Projection>
    where
        S: Take<N>,
    {
        let groups = shader.data().groups();
        let layout = Arc::clone(&groups[N]);

        GroupHandler {
            id: N,
            layout,
            ty: PhantomData,
        }
    }

    pub fn shared(self) -> SharedSet<S> {
        self.0
    }

    fn bind_groups(&mut self) -> &mut [BindGroup] {
        Arc::get_mut(&mut self.0.bind_groups).expect("uniqueness is guaranteed by the type")
    }
}

impl<S> Bind<S> for UniqueSet<S> {
    fn bind(&self) -> Bindings {
        self.0.bind()
    }
}

#[derive(Clone)]
pub struct SharedSet<S> {
    bind_groups: Arc<[BindGroup]>,
    ty: PhantomData<S>,
}

impl<S> Bind<S> for SharedSet<S> {
    fn bind(&self) -> Bindings {
        Bindings {
            bind_groups: &self.bind_groups,
        }
    }
}

pub trait Set {
    fn set(&self, f: &mut dyn FnMut(usize, &Visitor<'_>));
}

impl<G> Set for G
where
    G: Visit,
{
    fn set(&self, f: &mut dyn FnMut(usize, &Visitor<'_>)) {
        let mut visitor = Visitor(Vec::with_capacity(G::N_MEMBERS));
        visitor.visit(self);
        f(0, &visitor);
    }
}

impl<A> Set for (A,)
where
    A: Visit,
{
    fn set(&self, f: &mut dyn FnMut(usize, &Visitor<'_>)) {
        let mut visitor = Visitor(Vec::with_capacity(A::N_MEMBERS));
        visitor.visit(&self.0);
        f(0, &visitor);
    }
}

impl<A, B> Set for (A, B)
where
    A: Visit,
    B: Visit,
{
    fn set(&self, f: &mut dyn FnMut(usize, &Visitor<'_>)) {
        let cap = usize::max(A::N_MEMBERS, B::N_MEMBERS);
        let mut visitor = Visitor(Vec::with_capacity(cap));
        visitor.visit(&self.0);
        f(0, &visitor);

        visitor.clear();
        visitor.visit(&self.1);
        f(1, &visitor);
    }
}

pub trait Take<const N: usize> {
    type Group: Group;
}

impl<G> Take<0> for G
where
    G: Visit,
{
    type Group = G;
}

impl<A> Take<0> for (A,)
where
    A: Visit,
{
    type Group = A;
}

impl<A, B> Take<0> for (A, B)
where
    A: Visit,
{
    type Group = A;
}

impl<A, B> Take<1> for (A, B)
where
    B: Visit,
{
    type Group = B;
}

impl<A, B, C> Take<0> for (A, B, C)
where
    A: Visit,
{
    type Group = A;
}

impl<A, B, C> Take<1> for (A, B, C)
where
    B: Visit,
{
    type Group = B;
}

impl<A, B, C> Take<2> for (A, B, C)
where
    C: Visit,
{
    type Group = C;
}

impl<A, B, C, D> Take<0> for (A, B, C, D)
where
    A: Visit,
{
    type Group = A;
}

impl<A, B, C, D> Take<1> for (A, B, C, D)
where
    B: Visit,
{
    type Group = B;
}

impl<A, B, C, D> Take<2> for (A, B, C, D)
where
    C: Visit,
{
    type Group = C;
}

impl<A, B, C, D> Take<3> for (A, B, C, D)
where
    D: Visit,
{
    type Group = D;
}
