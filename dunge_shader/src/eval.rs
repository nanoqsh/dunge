use {
    crate::{
        glam,
        group::{self, DeclareGroup, Group},
        types::{self, Scalar, ScalarType, Vector, VectorType},
        vertex::{self, DeclareInput, Vertex},
    },
    naga::{
        AddressSpace, Arena, BinaryOperator, Binding, Block, BuiltIn, EntryPoint, Expression,
        Function, FunctionArgument, FunctionResult, GlobalVariable, Handle, Literal, Range,
        ResourceBinding, SampleLevel, ShaderStage, Span, Statement, StructMember, Type, TypeInner,
        UniqueArena,
    },
    std::{
        any::TypeId, array, cell::Cell, collections::HashMap, iter, marker::PhantomData, mem, ops,
        rc::Rc,
    },
};

#[derive(Clone, Copy)]
pub struct GroupInfo {
    pub tyid: TypeId,
    pub decl: DeclareGroup,
    pub stages: Stages,
}

#[derive(Clone, Copy)]
enum InputKind {
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

struct GroupEntry {
    tyid: TypeId,
    decl: DeclareGroup,
    out: GlobalOut,
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
    inputs: Vec<InputKind>,
    groups: Vec<GroupEntry>,
    limits: Limits,
}

impl Context {
    fn new() -> Self {
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
            stages: entry.out.stage.get(),
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

pub trait IntoModule<A> {
    type Vertex;
    fn into_module(self) -> Module;
}

impl<M, O> IntoModule<()> for M
where
    M: FnOnce() -> O,
    O: Output,
{
    type Vertex = ();

    fn into_module(self) -> Module {
        let cx = Context::new();
        Module::new(cx, self())
    }
}

impl<M, O, A> IntoModule<(A,)> for M
where
    M: FnOnce(A) -> O,
    O: Output,
    A: FromContextTyped,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        Module::new(cx, self(a))
    }
}

impl<M, O, A, B> IntoModule<(A, B)> for M
where
    M: FnOnce(A, B) -> O,
    O: Output,
    A: FromContextTyped,
    B: FromContext,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        let b = B::from_context(&mut cx);
        Module::new(cx, self(a, b))
    }
}

impl<M, O, A, B, C> IntoModule<(A, B, C)> for M
where
    M: FnOnce(A, B, C) -> O,
    O: Output,
    A: FromContextTyped,
    B: FromContext,
    C: FromContext,
{
    type Vertex = A::Vertex;

    fn into_module(self) -> Module {
        let mut cx = Context::new();
        let a = A::from_context_typed(&mut cx);
        let b = B::from_context(&mut cx);
        let c = C::from_context(&mut cx);
        Module::new(cx, self(a, b, c))
    }
}

pub struct Out<P, C> {
    pub place: P,
    pub color: C,
}

pub trait Output {
    type Place: Eval<VsEntry, Out = types::Vec4<f32>>;
    type Color: Eval<FsEntry, Out = types::Vec4<f32>>;

    fn output(self) -> Out<Self::Place, Self::Color>;
}

impl<P, C> Output for Out<P, C>
where
    P: Eval<VsEntry, Out = types::Vec4<f32>>,
    C: Eval<FsEntry, Out = types::Vec4<f32>>,
{
    type Place = P;
    type Color = C;

    fn output(self) -> Self {
        self
    }
}

pub struct Module {
    pub cx: Context,
    pub nm: naga::Module,
}

impl Module {
    fn new<O>(cx: Context, output: O) -> Self
    where
        O: Output,
    {
        let Out { place, color } = output.output();
        let mut compl = Compiler::default();
        let make_input = |kind| match kind {
            InputKind::Type(InputInfo { decl, .. }) => {
                let mut new = decl.into_iter().map(Member::from_vecty);
                Argument::from_type(compl.decl_input(&mut new))
            }
            InputKind::Index => Argument {
                ty: compl.decl_index(),
                binding: Some(Binding::BuiltIn(BuiltIn::VertexIndex)),
            },
        };

        let inputs: Vec<_> = cx.inputs.iter().copied().map(make_input).collect();
        for (id, &GroupEntry { decl, .. }) in iter::zip(0.., &cx.groups) {
            compl.decl_group(id, decl);
        }

        let (fs, required, fsty) = {
            let mut fs = FsEntry::new(compl);
            let Expr(ex) = color.eval(&mut fs);
            fs.inner.ret(ex);
            let fsty = fs.define_fragment_ty();
            let mut args = [fsty].into_iter().map(Argument::from_type);
            let built = fs.inner.build(Stage::Fragment, &mut args, Return::Color);
            (built, fs.required, fsty)
        };

        let vs = {
            let mut vs = VsEntry::new(fs.compl);
            let ex = place.eval(&mut vs);
            let eval = |req: Required| match req.evalf {
                EvalFunction::Position => ex.0,
                EvalFunction::Fn(f) => f(&mut vs).0,
            };

            let out: Vec<_> = required.into_iter().map(eval).collect();
            let res = vs.inner.compose(fsty, out);
            vs.inner.ret(res);
            let mut args = inputs.into_iter();
            vs.inner.build(Stage::Vertex, &mut args, Return::Ty(fsty))
        };

        let compl = vs.compl;
        let nm = naga::Module {
            types: compl.types,
            global_variables: compl.globs.vars,
            entry_points: vec![vs.point, fs.point],
            ..Default::default()
        };

        #[cfg(debug_assertions)]
        {
            use naga::valid::{Capabilities, ValidationFlags, Validator};

            let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
            if let Err(err) = validator.validate(&nm) {
                eprintln!("{nm:#?}");
                panic!("shader error: {err}\n{val:#?}", val = err.as_inner());
            }
        }

        Self { cx, nm }
    }
}

trait Get {
    const STAGE: Stage;
    fn get(&mut self) -> &mut Entry;
}

pub struct Expr(Handle<Expression>);

pub trait Eval<E>: Sized {
    type Out;

    fn eval(self, en: &mut E) -> Expr;
}

pub type Vec2f<A> = Ret<A, types::Vec2<f32>>;
pub type Vec3f<A> = Ret<A, types::Vec3<f32>>;
pub type Vec4f<A> = Ret<A, types::Vec4<f32>>;
pub type Vec2u<A> = Ret<A, types::Vec2<u32>>;
pub type Vec3u<A> = Ret<A, types::Vec3<u32>>;
pub type Vec4u<A> = Ret<A, types::Vec4<u32>>;
pub type Vec2i<A> = Ret<A, types::Vec2<i32>>;
pub type Vec3i<A> = Ret<A, types::Vec3<i32>>;
pub type Vec4i<A> = Ret<A, types::Vec4<i32>>;
pub type Tx2df<A> = Ret<A, types::Texture2d<f32>>;
pub type Sampl<A> = Ret<A, types::Sampler>;

pub struct Ret<A, T> {
    a: A,
    t: PhantomData<T>,
}

impl<A, T> Clone for Ret<A, T>
where
    A: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, T> Copy for Ret<A, T> where A: Copy {}

const fn ret<A, T>(a: A) -> Ret<A, T> {
    Ret { a, t: PhantomData }
}

impl<A, O> ops::Mul<f32> for Ret<A, O>
where
    O: types::Vector,
{
    type Output = Ret<Mul<Self, f32>, O>;

    fn mul(self, b: f32) -> Self::Output {
        ret(Mul { a: self, b })
    }
}

impl<A, O> ops::Mul<Ret<A, O>> for f32
where
    O: types::Vector,
{
    type Output = Ret<Mul<Self, Ret<A, O>>, O>;

    fn mul(self, b: Ret<A, O>) -> Self::Output {
        ret(Mul { a: self, b })
    }
}

pub struct Mul<A, B> {
    a: A,
    b: B,
}

impl<A, B, O, E> Eval<E> for Ret<Mul<A, B>, O>
where
    A: Eval<E>,
    B: Eval<E>,
    E: Get,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Expr(x) = self.a.a.eval(en);
        let Expr(y) = self.a.b.eval(en);
        let ex = en.get().binary([x, y]);
        Expr(ex)
    }
}

impl<E> Eval<E> for i32
where
    E: Get,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        Expr(en.get().literal(Literal::I32(self)))
    }
}

impl<E> Eval<E> for u32
where
    E: Get,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        Expr(en.get().literal(Literal::U32(self)))
    }
}

impl<E> Eval<E> for bool
where
    E: Get,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        Expr(en.get().literal(Literal::Bool(self)))
    }
}

impl<E> Eval<E> for f32
where
    E: Get,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        Expr(en.get().literal(Literal::F32(self)))
    }
}

impl<V, E> Eval<E> for V
where
    V: glam::Vector,
    V::Scalar: Eval<E>,
    E: Get,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        let mut components = Vec::with_capacity(V::TYPE.dims());
        self.visit(|scalar| {
            let Expr(v) = scalar.eval(en);
            components.push(v);
        });

        let en = en.get();
        let ty = en.new_type(V::TYPE.ty());
        Expr(en.compose(ty, components))
    }
}

#[derive(Clone, Copy)]
pub struct ReadIndex {
    id: u32,
}

impl ReadIndex {
    const fn new(id: u32) -> Ret<Self, u32> {
        ret(Self { id })
    }
}

impl Eval<VsEntry> for Ret<ReadIndex, u32> {
    type Out = u32;

    fn eval(self, en: &mut VsEntry) -> Expr {
        Expr(en.inner.argument(self.a.id))
    }
}

#[derive(Clone, Copy)]
pub struct ReadInput {
    id: u32,
    index: u32,
}

impl ReadInput {
    pub const fn new<T>(id: u32, index: u32) -> Ret<Self, T> {
        ret(Self { id, index })
    }
}

impl<T> Eval<VsEntry> for Ret<ReadInput, T> {
    type Out = T;

    fn eval(self, en: &mut VsEntry) -> Expr {
        let en = &mut en.inner;
        let arg = en.argument(self.a.id);
        Expr(en.access_index(arg, self.a.index))
    }
}

#[derive(Clone, Default)]
pub struct GlobalOut {
    stage: Rc<Cell<Stages>>,
}

impl GlobalOut {
    fn with_stage(&self, stage: Stage) {
        self.stage.set(self.stage.get().with(stage));
    }
}

pub struct ReadGlobal {
    id: u32,
    binding: u32,
    out: GlobalOut,
}

impl ReadGlobal {
    pub const fn new<T>(id: u32, binding: u32, out: GlobalOut) -> Ret<Self, T> {
        ret(Self { id, binding, out })
    }
}

impl<T, E> Eval<E> for Ret<ReadGlobal, T>
where
    E: Get,
{
    type Out = T;

    fn eval(self, en: &mut E) -> Expr {
        self.a.out.with_stage(E::STAGE);
        let en = en.get();
        let var = en.compl.globs.get(ResourceBinding {
            group: self.a.id,
            binding: self.a.binding,
        });

        Expr(en.global(var))
    }
}

pub const fn i32<A, E>(a: A) -> Ret<As<A>, i32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(As(a))
}

pub const fn u32<A, E>(a: A) -> Ret<As<A>, u32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(As(a))
}

pub const fn f32<A, E>(a: A) -> Ret<As<A>, f32>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(As(a))
}

pub const fn bool<A, E>(a: A) -> Ret<As<A>, bool>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(As(a))
}

pub struct As<A>(A);

impl<A, O, E> Eval<E> for Ret<As<A>, O>
where
    A: Eval<E>,
    A::Out: Scalar,
    E: Get,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Expr(v) = self.a.0.eval(en);
        Expr(en.get().convert(v, A::Out::TYPE))
    }
}

pub const fn texture_sample<T, S, C, E, F>(tex: T, sam: S, crd: C) -> Vec4f<Sample<T, S, C>>
where
    T: Eval<E, Out = types::Texture2d<F>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
{
    ret(Sample { tex, sam, crd })
}

pub struct Sample<T, S, C> {
    tex: T,
    sam: S,
    crd: C,
}

impl<T, S, C, E, F> Eval<E> for Vec4f<Sample<T, S, C>>
where
    T: Eval<E, Out = types::Texture2d<F>>,
    S: Eval<E, Out = types::Sampler>,
    C: Eval<E, Out = types::Vec2<f32>>,
    E: Get,
{
    type Out = types::Vec4<F>;

    fn eval(self, en: &mut E) -> Expr {
        let ex = SampleExpr {
            tex: self.a.tex.eval(en).0,
            sam: self.a.sam.eval(en).0,
            crd: self.a.crd.eval(en).0,
        };

        Expr(en.get().sample(ex))
    }
}

pub const fn fragment<A>(a: A) -> Ret<Fragment<A>, A::Out>
where
    A: Eval<VsEntry>,
    A::Out: types::Vector,
{
    ret(Fragment(a))
}

pub struct Fragment<A>(A);

impl<A> Eval<FsEntry> for Ret<Fragment<A>, A::Out>
where
    A: Eval<VsEntry> + 'static,
    A::Out: types::Vector,
{
    type Out = A::Out;

    fn eval(self, en: &mut FsEntry) -> Expr {
        let vecty = <A::Out as types::Vector>::TYPE;
        let index = en.push_evalf(vecty, |en| self.a.0.eval(en));
        let en = &mut en.inner;
        let arg = en.argument(0);
        Expr(en.access_index(arg, index))
    }
}

pub fn share<A, E, const N: usize>(a: A) -> [Ret<Share<A>, A::Out>; N]
where
    A: Eval<E>,
{
    let state = State::Eval(a);
    let inner = Rc::new(Cell::new(state));
    array::from_fn(|_| ret(Share(Rc::clone(&inner))))
}

pub struct Share<A>(Rc<Cell<State<A>>>);

impl<A, O, E> Eval<E> for Ret<Share<A>, O>
where
    A: Eval<E>,
{
    type Out = A::Out;

    fn eval(self, en: &mut E) -> Expr {
        match self.a.0.replace(State::None) {
            State::None => unreachable!(),
            State::Eval(a) => {
                let ex = a.eval(en);
                self.a.0.set(State::Expr(ex.0));
                ex
            }
            State::Expr(ex) => Expr(ex),
        }
    }
}

enum State<A> {
    None,
    Eval(A),
    Expr(Handle<Expression>),
}

pub const fn splat_vec2<A, E>(a: A) -> Ret<Splat<A>, types::Vec2<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(Splat(a))
}

pub const fn splat_vec3<A, E>(a: A) -> Ret<Splat<A>, types::Vec3<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(Splat(a))
}

pub const fn splat_vec4<A, E>(a: A) -> Ret<Splat<A>, types::Vec4<A::Out>>
where
    A: Eval<E>,
    A::Out: Scalar,
{
    ret(Splat(a))
}

pub struct Splat<A>(A);

impl<A, O, E> Eval<E> for Ret<Splat<A>, O>
where
    A: Eval<E>,
    O: Vector,
    E: Get,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Expr(v) = self.a.0.eval(en);
        let en = en.get();
        let ty = en.new_type(O::TYPE.ty());
        let components: Vec<_> = (0..O::TYPE.dims()).map(|_| v).collect();
        Expr(en.compose(ty, components))
    }
}

pub const fn vec2<A, B, E>(a: A, b: B) -> Vec2f<Compose<A, B>>
where
    A: Eval<E, Out = B::Out>,
    B: Eval<E>,
{
    ret(Compose { a, b })
}

pub const fn vec3<A, B, E>(a: A, b: B) -> Vec3f<Compose<A, B>>
where
    A: Eval<E, Out = types::Vec2<B::Out>>,
    B: Eval<E>,
{
    ret(Compose { a, b })
}

pub const fn vec4<A, B, E>(a: A, b: B) -> Vec4f<Compose<A, B>>
where
    A: Eval<E, Out = types::Vec3<B::Out>>,
    B: Eval<E>,
{
    ret(Compose { a, b })
}

pub struct Compose<A, B> {
    a: A,
    b: B,
}

impl<A, B, O, E> Eval<E> for Ret<Compose<A, B>, O>
where
    A: Eval<E>,
    B: Eval<E>,
    O: types::Vector,
    E: Get,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let Compose { a, b } = self.a;
        let Expr(x) = a.eval(en);
        let Expr(y) = b.eval(en);
        let en = en.get();
        let ty = en.new_type(O::TYPE.ty());
        Expr(en.compose(ty, [x, y]))
    }
}

#[derive(Clone, Copy)]
enum Stage {
    Vertex,
    Fragment,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Self::Vertex => "vs",
            Self::Fragment => "fs",
        }
    }

    fn shader_stage(self) -> ShaderStage {
        match self {
            Self::Vertex => ShaderStage::Vertex,
            Self::Fragment => ShaderStage::Fragment,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Stages {
    pub vs: bool,
    pub fs: bool,
}

impl Stages {
    fn with(self, stage: Stage) -> Self {
        match stage {
            Stage::Vertex => Self { vs: true, ..self },
            Stage::Fragment => Self { fs: true, ..self },
        }
    }
}

enum Return {
    Ty(Handle<Type>),
    Color,
}

pub struct VsEntry {
    inner: Entry,
}

impl VsEntry {
    fn new(compl: Compiler) -> Self {
        Self {
            inner: Entry::new(compl),
        }
    }
}

impl Get for VsEntry {
    const STAGE: Stage = Stage::Vertex;

    fn get(&mut self) -> &mut Entry {
        &mut self.inner
    }
}

struct Member {
    vecty: VectorType,
    built: Option<BuiltIn>,
}

impl Member {
    fn from_vecty(vecty: VectorType) -> Self {
        Self { vecty, built: None }
    }
}

enum EvalFunction {
    Position,
    Fn(Box<dyn FnOnce(&mut VsEntry) -> Expr>),
}

struct Required {
    vecty: VectorType,
    evalf: EvalFunction,
}

pub struct FsEntry {
    inner: Entry,
    required: Vec<Required>,
}

impl FsEntry {
    fn new(compl: Compiler) -> Self {
        Self {
            inner: Entry::new(compl),
            required: vec![Required {
                vecty: VectorType::Vec4f,
                evalf: EvalFunction::Position,
            }],
        }
    }

    fn push_evalf<F>(&mut self, vecty: VectorType, f: F) -> u32
    where
        F: FnOnce(&mut VsEntry) -> Expr + 'static,
    {
        let req = Required {
            vecty,
            evalf: EvalFunction::Fn(Box::new(f)),
        };

        let index = self.required.len();
        self.required.push(req);
        index as u32
    }

    fn define_fragment_ty(&mut self) -> Handle<Type> {
        let member = |req: &Required| match req.evalf {
            EvalFunction::Position => Member {
                vecty: req.vecty,
                built: Some(BuiltIn::Position { invariant: false }),
            },
            EvalFunction::Fn(_) => Member::from_vecty(req.vecty),
        };

        let mut members = self.required.iter().map(member);
        self.inner.compl.decl_input(&mut members)
    }
}

impl Get for FsEntry {
    const STAGE: Stage = Stage::Fragment;

    fn get(&mut self) -> &mut Entry {
        &mut self.inner
    }
}

struct Built {
    compl: Compiler,
    point: EntryPoint,
}

struct Argument {
    ty: Handle<Type>,
    binding: Option<Binding>,
}

impl Argument {
    fn from_type(ty: Handle<Type>) -> Self {
        Self { ty, binding: None }
    }

    fn into_function(self) -> FunctionArgument {
        FunctionArgument {
            name: None,
            ty: self.ty,
            binding: self.binding,
        }
    }
}

type Args<'a> = dyn Iterator<Item = Argument> + 'a;

pub struct Entry {
    compl: Compiler,
    exprs: Arena<Expression>,
    stats: Statements,
}

impl Entry {
    fn new(compl: Compiler) -> Self {
        Self {
            compl,
            exprs: Arena::default(),
            stats: Statements::default(),
        }
    }

    fn new_type(&mut self, ty: Type) -> Handle<Type> {
        self.compl.types.insert(ty, Span::UNDEFINED)
    }

    fn literal(&mut self, literal: Literal) -> Handle<Expression> {
        let ex = Expression::Literal(literal);
        self.exprs.append(ex, Span::UNDEFINED)
    }

    fn argument(&mut self, n: u32) -> Handle<Expression> {
        let ex = Expression::FunctionArgument(n);
        self.exprs.append(ex, Span::UNDEFINED)
    }

    fn global(&mut self, var: Handle<GlobalVariable>) -> Handle<Expression> {
        let ex = Expression::GlobalVariable(var);
        self.exprs.append(ex, Span::UNDEFINED)
    }

    fn access_index(&mut self, base: Handle<Expression>, index: u32) -> Handle<Expression> {
        let ex = Expression::AccessIndex { base, index };
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        handle
    }

    fn convert(&mut self, expr: Handle<Expression>, ty: ScalarType) -> Handle<Expression> {
        let (kind, width) = ty.inner();
        let ex = Expression::As {
            expr,
            kind,
            convert: Some(width),
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        handle
    }

    fn binary(&mut self, [a, b]: [Handle<Expression>; 2]) -> Handle<Expression> {
        let ex = Expression::Binary {
            op: BinaryOperator::Multiply,
            left: a,
            right: b,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        handle
    }

    fn compose<C>(&mut self, ty: Handle<Type>, components: C) -> Handle<Expression>
    where
        C: Into<Vec<Handle<Expression>>>,
    {
        let components = components.into();
        let ex = Expression::Compose { ty, components };
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        handle
    }

    fn sample(&mut self, ex: SampleExpr) -> Handle<Expression> {
        let handle = self.exprs.append(ex.expr(), Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stats.push(st, &self.exprs);
        handle
    }

    fn ret(&mut self, value: Handle<Expression>) {
        let st = Statement::Return { value: Some(value) };
        self.stats.push(st, &self.exprs);
    }

    fn build(mut self, stage: Stage, args: &mut Args, ret: Return) -> Built {
        const COLOR_TYPE: Type = VectorType::Vec4f.ty();

        let res = match ret {
            Return::Ty(ty) => FunctionResult { ty, binding: None },
            Return::Color => FunctionResult {
                ty: self.new_type(COLOR_TYPE),
                binding: Some(binding_location(0, &COLOR_TYPE.inner)),
            },
        };

        let point = EntryPoint {
            name: stage.name().to_owned(),
            stage: stage.shader_stage(),
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: Function {
                arguments: args.map(Argument::into_function).collect(),
                result: Some(res),
                expressions: self.exprs,
                body: Block::from_vec(self.stats.0),
                ..Default::default()
            },
        };

        Built {
            compl: self.compl,
            point,
        }
    }
}

#[derive(Default)]
struct Statements(Vec<Statement>);

impl Statements {
    fn push(&mut self, st: Statement, exprs: &Arena<Expression>) {
        if let Statement::Emit(new) = &st {
            if let Some(Statement::Emit(top)) = self.0.last_mut() {
                let top_range = top.zero_based_index_range();
                let new_range = new.zero_based_index_range();
                if top_range.end == new_range.start {
                    let merged = top_range.start..new_range.end;
                    *top = Range::from_zero_based_index_range(merged, exprs);
                    return;
                }
            }
        }

        self.0.push(st);
    }
}

struct SampleExpr {
    tex: Handle<Expression>,
    sam: Handle<Expression>,
    crd: Handle<Expression>,
}

impl SampleExpr {
    fn expr(self) -> Expression {
        Expression::ImageSample {
            image: self.tex,
            sampler: self.sam,
            gather: None,
            coordinate: self.crd,
            array_index: None,
            offset: None,
            level: SampleLevel::Auto,
            depth_ref: None,
        }
    }
}

type Members<'a> = dyn ExactSizeIterator<Item = Member> + 'a;

#[derive(Default)]
struct Compiler {
    types: UniqueArena<Type>,
    globs: Globals,
}

impl Compiler {
    fn decl_index(&mut self) -> Handle<Type> {
        self.types.insert(ScalarType::Uint.ty(), Span::UNDEFINED)
    }

    fn decl_input(&mut self, new: &mut Members) -> Handle<Type> {
        const VECTOR_SIZE: u32 = mem::size_of::<f32>() as u32 * 4;

        let len = new.len();
        let mut members = Vec::with_capacity(len);
        let mut location = 0;
        for (idx, Member { vecty, built }) in iter::zip(0.., new) {
            let ty = vecty.ty();
            let binding = match built {
                Some(bi @ BuiltIn::Position { .. }) => Binding::BuiltIn(bi),
                None => {
                    let curr = location;
                    location += 1;
                    binding_location(curr, &ty.inner)
                }
                _ => unimplemented!(),
            };

            members.push(StructMember {
                name: None,
                ty: self.types.insert(ty, Span::UNDEFINED),
                binding: Some(binding),
                offset: idx * VECTOR_SIZE,
            });
        }

        let ty = Type {
            name: None,
            inner: TypeInner::Struct {
                members,
                span: len as u32 * VECTOR_SIZE,
            },
        };

        self.types.insert(ty, Span::UNDEFINED)
    }

    fn decl_group(&mut self, group: u32, decl: DeclareGroup) {
        for (binding, member) in iter::zip(0.., decl) {
            let ty = self.types.insert(member.ty(), Span::UNDEFINED);
            let res = ResourceBinding { group, binding };
            self.globs.add(ty, res);
        }
    }
}

#[derive(Default)]
struct Globals {
    vars: Arena<GlobalVariable>,
    handles: HashMap<ResourceBinding, Handle<GlobalVariable>>,
}

impl Globals {
    fn add(&mut self, ty: Handle<Type>, res: ResourceBinding) {
        self.handles.entry(res.clone()).or_insert_with(|| {
            let var = GlobalVariable {
                name: None,
                space: AddressSpace::Handle,
                binding: Some(res),
                ty,
                init: None,
            };

            self.vars.append(var, Span::UNDEFINED)
        });
    }

    fn get(&self, res: ResourceBinding) -> Handle<GlobalVariable> {
        self.handles[&res]
    }
}

fn binding_location(location: u32, inner: &TypeInner) -> Binding {
    let mut binding = Binding::Location {
        location,
        second_blend_source: false,
        interpolation: None,
        sampling: None,
    };

    binding.apply_default_interpolation(inner);
    binding
}
