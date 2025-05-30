use {
    crate::{
        define::Define,
        eval::{GlobalOut, ReadIndex, ReadInvocation, Stage},
        group::{self, Group},
        instance::{self, Instance},
        module::{ComputeKind, RenderKind},
        op::Ret,
        types::{self, MemberData, ValueType, VectorType},
        vertex::{self, Vertex},
    },
    std::ops,
};

#[derive(Clone, Copy)]
pub struct GroupInfo {
    pub def: Define<MemberData>,
    pub stages: Stages,
}

#[derive(Clone, Copy, Default)]
pub struct Stages {
    pub vs: bool,
    pub fs: bool,
    pub cs: bool,
}

impl Stages {
    pub(crate) fn with(self, stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self { vs: true, ..self },
            Stage::Fragment => Self { fs: true, ..self },
            Stage::Compute => Self { cs: true, ..self },
        }
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum InputInfo {
    Vert(VertInfo),
    Inst(InstInfo),
    Index,
    GlobalInvocationId,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct VertInfo {
    pub def: Define<VectorType>,
    pub size: usize,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct InstInfo {
    pub ty: ValueType,
}

pub(crate) struct GroupEntry {
    def: Define<MemberData>,
    out: GlobalOut,
}

impl GroupEntry {
    pub(crate) fn def(&self) -> Define<MemberData> {
        self.def
    }
}

struct Limits {
    index: u8,
    verts: u8,
    insts: u8,
    group: u8,
}

fn countdown(v: &mut u8, msg: &str) {
    match v.checked_sub(1) {
        Some(n) => *v = n,
        None => panic!("{msg}"),
    }
}

pub struct Context {
    pub(crate) inputs: Vec<InputInfo>,
    pub(crate) groups: Vec<GroupEntry>,
    limits: Limits,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            inputs: vec![],
            groups: vec![],
            limits: Limits {
                index: 1,
                verts: 1,
                insts: 2,
                group: 1,
            },
        }
    }

    fn add_index(&mut self) -> u32 {
        countdown(&mut self.limits.index, "too many indices in the shader");
        let id = self.inputs.len() as u32;
        self.inputs.push(InputInfo::Index);
        id
    }

    fn add_global_invocation_id(&mut self) -> u32 {
        countdown(
            &mut self.limits.index,
            "too many global invocation ids in the shader",
        );

        let id = self.inputs.len() as u32;
        self.inputs.push(InputInfo::GlobalInvocationId);
        id
    }

    fn add_vertex(&mut self, def: Define<VectorType>, size: usize) -> u32 {
        countdown(&mut self.limits.verts, "too many vertices in the shader");
        let id = self.inputs.len() as u32;
        let info = VertInfo { def, size };
        self.inputs.push(InputInfo::Vert(info));
        id
    }

    fn add_instance(&mut self, ty: ValueType) -> u32 {
        countdown(&mut self.limits.insts, "too many instances in the shader");
        let id = self.inputs.len() as u32;
        let info = InstInfo { ty };
        self.inputs.push(InputInfo::Inst(info));
        id
    }

    fn add_group_set(&mut self) {
        countdown(&mut self.limits.group, "too many groups in the shader");
    }

    fn add_group(&mut self, def: Define<MemberData>) -> (u32, GlobalOut) {
        let out = GlobalOut::default();
        let en = GroupEntry {
            def,
            out: out.clone(),
        };

        let id = self.groups.len() as u32;
        self.groups.push(en);
        (id, out)
    }

    #[doc(hidden)]
    pub fn count_input(&self) -> usize {
        self.inputs
            .iter()
            .filter(|info| matches!(info, InputInfo::Vert(_) | InputInfo::Inst(_)))
            .count()
    }

    #[doc(hidden)]
    pub fn input(&self) -> impl Iterator<Item = InputInfo> {
        self.inputs.iter().copied()
    }

    #[doc(hidden)]
    pub fn groups(&self) -> impl Iterator<Item = GroupInfo> {
        self.groups.iter().map(|entry| GroupInfo {
            def: entry.def,
            stages: entry.out.get(),
        })
    }
}

pub trait FromRender<K> {
    type Vertex;
    type Instance;
    fn from_render(cx: &mut Context) -> Self;
}

impl<V> FromRender<RenderKind> for V
where
    V: FromContext<RenderKind>,
{
    type Vertex = ();
    type Instance = ();

    fn from_render(cx: &mut Context) -> Self {
        V::from_context(cx)
    }
}

pub struct InVertex<V>(V::Projection)
where
    V: Vertex;

impl<V> ops::Deref for InVertex<V>
where
    V: Vertex,
{
    type Target = V::Projection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V, O> FromRender<O> for InVertex<V>
where
    V: Vertex,
{
    type Vertex = V;
    type Instance = ();

    fn from_render(cx: &mut Context) -> Self {
        let id = cx.add_vertex(V::DEF, size_of::<V>());
        Self(vertex::Projection::projection(id))
    }
}

pub struct InInstance<I>(I::Projection)
where
    I: Instance;

impl<I> ops::Deref for InInstance<I>
where
    I: Instance,
{
    type Target = I::Projection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, O> FromRender<O> for InInstance<I>
where
    I: Instance,
{
    type Vertex = ();
    type Instance = I;

    fn from_render(cx: &mut Context) -> Self {
        let mut id = None;
        for ty in I::DEF {
            id.get_or_insert(cx.add_instance(ty));
        }

        let id = id.expect("the instance must have at least one field");
        Self(instance::Projection::projection(id))
    }
}

pub struct In<V, I>(pub V::Projection, pub I::Projection)
where
    V: Vertex,
    I: Instance;

impl<V, I, O> FromRender<O> for In<V, I>
where
    V: Vertex,
    I: Instance,
{
    type Vertex = V;
    type Instance = I;

    fn from_render(cx: &mut Context) -> Self {
        let InVertex(vert) = <InVertex<V> as FromRender<O>>::from_render(cx);
        let InInstance(inst) = <InInstance<I> as FromRender<O>>::from_render(cx);
        Self(vert, inst)
    }
}

#[derive(Clone, Copy)]
pub struct Index(pub Ret<ReadIndex, u32>);

impl FromContext<RenderKind> for Index {
    type Set = ();

    fn from_context(cx: &mut Context) -> Self {
        let id = cx.add_index();
        Self(ReadIndex::new(id))
    }
}

#[derive(Clone, Copy)]
pub struct Invocation(pub Ret<ReadInvocation, types::Vec3<u32>>);

impl FromContext<ComputeKind> for Invocation {
    type Set = ();

    fn from_context(cx: &mut Context) -> Self {
        let id = cx.add_global_invocation_id();
        Self(ReadInvocation::new(id))
    }
}

pub trait ProjectionFromContext {
    type Set;
    type Projection;
    fn from_context(cx: &mut Context) -> Self::Projection;
}

impl ProjectionFromContext for () {
    type Set = Self;
    type Projection = Self;
    fn from_context(_: &mut Context) -> Self::Projection {}
}

impl<A> ProjectionFromContext for A
where
    A: Group,
{
    type Set = (A::Projection,);
    type Projection = A::Projection;

    fn from_context(cx: &mut Context) -> Self::Projection {
        cx.add_group_set();
        let (id, out) = cx.add_group(A::DEF);
        group::Projection::projection(id, out)
    }
}

macro_rules! impl_projection_from_context {
    ($($t:ident),*) => {
        impl<$($t),*> ProjectionFromContext for ($($t),*,)
        where
            $(
                $t: Group,
            )*
        {
            type Set = ($($t::Projection),*,);
            type Projection = ($($t::Projection),*,);

            fn from_context(cx: &mut Context) -> Self::Projection {
                cx.add_group_set();

                (
                    $({
                        let (id, out) = cx.add_group($t::DEF);
                        group::Projection::projection(id, out)
                    }),*,
                )
            }
        }
    };
}

impl_projection_from_context!(A);
impl_projection_from_context!(A, B);
impl_projection_from_context!(A, B, C);
impl_projection_from_context!(A, B, C, D);

pub struct Groups<G>(pub G::Projection)
where
    G: ProjectionFromContext;

impl<G, K> FromContext<K> for Groups<G>
where
    G: ProjectionFromContext,
{
    type Set = G::Set;

    fn from_context(cx: &mut Context) -> Self {
        Self(G::from_context(cx))
    }
}

pub trait FromContext<K> {
    type Set;
    fn from_context(cx: &mut Context) -> Self;
}

pub trait TakeSet {
    type Set;
}

impl TakeSet for ((), (), ()) {
    type Set = ();
}

impl<A> TakeSet for ((A,), (), ()) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((A, B), (), ()) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((A, B, C), (), ()) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((A, B, C, D), (), ()) {
    type Set = (A, B, C, D);
}

impl<A> TakeSet for ((), (A,), ()) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((), (A, B), ()) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((), (A, B, C), ()) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((), (A, B, C, D), ()) {
    type Set = (A, B, C, D);
}

impl<A> TakeSet for ((), (), (A,)) {
    type Set = (A,);
}

impl<A, B> TakeSet for ((), (), (A, B)) {
    type Set = (A, B);
}

impl<A, B, C> TakeSet for ((), (), (A, B, C)) {
    type Set = (A, B, C);
}

impl<A, B, C, D> TakeSet for ((), (), (A, B, C, D)) {
    type Set = (A, B, C, D);
}
