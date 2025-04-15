//! Shader binding types.

use {
    crate::{
        Group,
        group::{BoundTexture, Take},
        shader::{Shader, ShaderData},
        state::State,
        storage::Storage,
        texture::Sampler,
        uniform::Uniform,
    },
    std::{marker::PhantomData, sync::Arc},
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

    fn entries(&self) -> &[wgpu::BindGroupEntry<'group>] {
        &self.0
    }
}

impl<'group> Visitor<'group> {
    fn push(&mut self, resource: wgpu::BindingResource<'group>) {
        let binding = self.0.len() as u32;
        self.0.push(wgpu::BindGroupEntry { binding, resource });
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
        visitor.push(wgpu::BindingResource::Buffer(binding));
    }
}

impl<'group, V> VisitMember<'group> for &'group Uniform<V> {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        let binding = self.buffer().as_entire_buffer_binding();
        visitor.push(wgpu::BindingResource::Buffer(binding));
    }
}

impl<'group> VisitMember<'group> for BoundTexture<'group> {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        visitor.push(wgpu::BindingResource::TextureView(self.0.view()));
    }
}

impl<'group> VisitMember<'group> for &'group Sampler {
    fn visit_member(self, visitor: &mut Visitor<'group>) {
        visitor.push(wgpu::BindingResource::Sampler(self.inner()));
    }
}

pub struct GroupHandler<S, P> {
    id: usize,
    layout: Arc<wgpu::BindGroupLayout>,
    ty: PhantomData<(S, P)>,
}

pub trait Bind<S> {
    fn bind(&self) -> Bindings<'_>;
}

pub struct Bindings<'group> {
    pub(crate) bind_groups: &'group [wgpu::BindGroup],
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
    group.set(|_, visitor| {
        let entries = visitor.entries();
        let desc = wgpu::BindGroupDescriptor {
            label: None,
            layout: &handler.layout,
            entries,
        };

        let new = device.create_bind_group(&desc);
        let groups = set.bind_groups();
        groups[handler.id] = new;
    });
}

pub struct UniqueSet<S>(SharedSet<S>);

impl<S> UniqueSet<S> {
    pub(crate) fn new<D>(state: &State, shader: &ShaderData, set: D) -> Self
    where
        D: Data<Set = S>,
    {
        let groups = shader.groups();
        let mut bind_groups = Vec::with_capacity(groups.len());

        let device = state.device();
        set.set(|id, visitor| {
            let layout = &groups[id];
            let entries = visitor.entries();
            let desc = wgpu::BindGroupDescriptor {
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

    pub fn shared(self) -> SharedSet<S> {
        self.0
    }

    pub fn handler<K>(&self, shader: &Shader<K, S>) -> GroupHandler<S, S::Projection>
    where
        S: Take<0>,
    {
        self.handler_n(shader)
    }

    fn handler_n<K, const N: usize>(&self, shader: &Shader<K, S>) -> GroupHandler<S, S::Projection>
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

    fn bind_groups(&mut self) -> &mut [wgpu::BindGroup] {
        Arc::get_mut(&mut self.0.bind_groups).expect("uniqueness is guaranteed by the type")
    }
}

impl<S> Bind<S> for UniqueSet<S> {
    fn bind(&self) -> Bindings<'_> {
        self.0.bind()
    }
}

#[derive(Clone)]
pub struct SharedSet<S> {
    bind_groups: Arc<[wgpu::BindGroup]>,
    ty: PhantomData<S>,
}

impl<S> Bind<S> for SharedSet<S> {
    fn bind(&self) -> Bindings<'_> {
        Bindings {
            bind_groups: &self.bind_groups,
        }
    }
}

pub trait Data {
    type Set;

    fn set<F>(&self, f: F)
    where
        F: FnMut(usize, &Visitor<'_>);
}

impl<G> Data for G
where
    G: Visit,
{
    type Set = (G::Projection,);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let mut visitor = Visitor(Vec::with_capacity(G::N_MEMBERS));
        self.visit(&mut visitor);
        f(0, &visitor);
    }
}

impl<A> Data for (A,)
where
    A: Visit,
{
    type Set = (A::Projection,);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let mut visitor = Visitor(Vec::with_capacity(A::N_MEMBERS));
        self.0.visit(&mut visitor);
        f(0, &visitor);
    }
}

impl<A, B> Data for (A, B)
where
    A: Visit,
    B: Visit,
{
    type Set = (A::Projection, B::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(A::N_MEMBERS, B::N_MEMBERS);
        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);
    }
}

impl<A, B, C> Data for (A, B, C)
where
    A: Visit,
    B: Visit,
    C: Visit,
{
    type Set = (A::Projection, B::Projection, C::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(A::N_MEMBERS, usize::max(B::N_MEMBERS, C::N_MEMBERS));
        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);

        visitor.clear();
        self.2.visit(&mut visitor);
        f(2, &visitor);
    }
}

impl<A, B, C, D> Data for (A, B, C, D)
where
    A: Visit,
    B: Visit,
    C: Visit,
    D: Visit,
{
    type Set = (A::Projection, B::Projection, C::Projection, D::Projection);

    fn set<F>(&self, mut f: F)
    where
        F: FnMut(usize, &Visitor<'_>),
    {
        let cap = usize::max(
            usize::max(A::N_MEMBERS, B::N_MEMBERS),
            usize::max(C::N_MEMBERS, D::N_MEMBERS),
        );

        let mut visitor = Visitor(Vec::with_capacity(cap));
        self.0.visit(&mut visitor);
        f(0, &visitor);

        visitor.clear();
        self.1.visit(&mut visitor);
        f(1, &visitor);

        visitor.clear();
        self.2.visit(&mut visitor);
        f(2, &visitor);

        visitor.clear();
        self.3.visit(&mut visitor);
        f(3, &visitor);
    }
}
