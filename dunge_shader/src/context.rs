use {
    crate::{
        eval::{GlobalOut, ReadIndex, Stage},
        group::{self, DeclareGroup, Group},
        ret::Ret,
        vertex::{self, DeclareInput, Vertex},
    },
    std::{any::TypeId, mem, ops},
};

#[derive(Clone, Copy)]
pub struct GroupInfo {
    pub tyid: TypeId,
    pub decl: DeclareGroup,
    pub stages: Stages,
}

#[derive(Clone, Copy, Default)]
pub struct Stages {
    pub vs: bool,
    pub fs: bool,
}

impl Stages {
    pub(crate) fn with(self, stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self { vs: true, ..self },
            Stage::Fragment => Self { fs: true, ..self },
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum InputKind {
    Type(InputInfo),
    Index,
}

impl InputKind {
    fn into_input_info(self) -> Option<InputInfo> {
        match self {
            Self::Type(info) => Some(info),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct InputInfo {
    pub decl: DeclareInput,
    pub size: usize,
}

pub(crate) struct GroupEntry {
    tyid: TypeId,
    decl: DeclareGroup,
    out: GlobalOut,
}

impl GroupEntry {
    pub fn decl(&self) -> DeclareGroup {
        self.decl
    }
}

struct Limits {
    index: u8,
    input: u8,
    group: u8,
}

fn countdown(v: &mut u8, msg: &str) {
    match v.checked_sub(1) {
        Some(n) => *v = n,
        None => panic!("{msg}"),
    }
}

pub struct Context {
    pub(crate) inputs: Vec<InputKind>,
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
                input: 1,
                group: 4,
            },
        }
    }

    fn declare_index(&mut self) -> u32 {
        countdown(&mut self.limits.index, "too many indices in the shader");
        let id = self.inputs.len() as u32;
        self.inputs.push(InputKind::Index);
        id
    }

    fn declare_input(&mut self, decl: DeclareInput, size: usize) -> u32 {
        countdown(&mut self.limits.input, "too many inputs in the shader");
        let id = self.inputs.len() as u32;
        let info = InputInfo { decl, size };
        self.inputs.push(InputKind::Type(info));
        id
    }

    fn declare_group(&mut self, tyid: TypeId, decl: DeclareGroup) -> (u32, GlobalOut) {
        countdown(&mut self.limits.group, "too many groups in the shader");
        let out = GlobalOut::default();
        let en = GroupEntry {
            tyid,
            decl,
            out: out.clone(),
        };

        let id = self.groups.len() as u32;
        self.groups.push(en);
        (id, out)
    }

    pub fn groups(&self) -> impl Iterator<Item = GroupInfo> + '_ {
        self.groups.iter().map(|entry| GroupInfo {
            tyid: entry.tyid,
            decl: entry.decl,
            stages: entry.out.get(),
        })
    }

    pub fn inputs(&self) -> impl Iterator<Item = InputInfo> + '_ {
        self.inputs
            .iter()
            .copied()
            .flat_map(InputKind::into_input_info)
    }
}

pub trait FromContextTyped {
    type Vertex;
    fn from_context_typed(cx: &mut Context) -> Self;
}

impl<V> FromContextTyped for V
where
    V: FromContext,
{
    type Vertex = ();

    fn from_context_typed(cx: &mut Context) -> Self {
        V::from_context(cx)
    }
}

pub struct Input<V>(V::Projection)
where
    V: Vertex;

impl<V> ops::Deref for Input<V>
where
    V: Vertex,
{
    type Target = V::Projection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> FromContextTyped for Input<V>
where
    V: Vertex,
{
    type Vertex = V;

    fn from_context_typed(cx: &mut Context) -> Self {
        let id = cx.declare_input(V::DECL, mem::size_of::<V>());
        Self(vertex::Projection::projection(id))
    }
}

pub trait FromContext {
    fn from_context(cx: &mut Context) -> Self;
}

#[derive(Clone, Copy)]
pub struct Index(pub Ret<ReadIndex, u32>);

impl FromContext for Index {
    fn from_context(cx: &mut Context) -> Self {
        let id = cx.declare_index();
        Self(ReadIndex::new(id))
    }
}

pub trait ProjectionFromContext {
    type Projection;
    fn from_context(cx: &mut Context) -> Self::Projection;
}

impl ProjectionFromContext for () {
    type Projection = ();
    fn from_context(_: &mut Context) -> Self::Projection {}
}

impl<A> ProjectionFromContext for A
where
    A: Group,
{
    type Projection = A::Projection;

    fn from_context(cx: &mut Context) -> Self::Projection {
        let (id, out) = cx.declare_group(TypeId::of::<A::Projection>(), A::DECL);
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
            type Projection = ($($t::Projection),*,);

            fn from_context(cx: &mut Context) -> Self::Projection {
                (
                    $({
                        let (id, out) = cx.declare_group(TypeId::of::<$t::Projection>(), $t::DECL);
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

impl<G> FromContext for Groups<G>
where
    G: ProjectionFromContext,
{
    fn from_context(cx: &mut Context) -> Self {
        Self(G::from_context(cx))
    }
}
