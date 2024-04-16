use {
    crate::{
        context::{Context, InputInfo, InstInfo, Stages, VertInfo},
        define::Define,
        math::Func,
        module::{Module, Out, Output},
        op::{Bi, Ret, Un},
        texture::Sampled,
        types::{self, MemberType, ScalarType, ValueType, VectorType},
    },
    naga::{
        AddressSpace, Arena, Binding, BuiltIn, EntryPoint, Expression, Function, FunctionArgument,
        FunctionResult, GlobalVariable, Handle, Literal, LocalVariable, Range, ResourceBinding,
        ShaderStage, Span, Statement, StructMember, Type, TypeInner, UniqueArena,
    },
    std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        iter,
        marker::PhantomData,
        mem,
        rc::Rc,
    },
};

pub(crate) fn make<F, O>(cx: Context, f: F) -> Module
where
    F: FnOnce() -> O,
    O: Output,
{
    assert!(
        top().is_none(),
        "reentrant in a shader function isn't allowed",
    );

    let pop = push();
    let Out { place, color } = f().output();
    let mut compl = Compiler::default();
    let mut binds = Bindings::default();
    let make_input = |info: &_| match info {
        InputInfo::Vert(VertInfo { def, .. }) => {
            let mut new = def.into_iter().map(Member::from_vecty);
            Argument::from_type(compl.define_input(&mut new, &mut binds))
        }
        InputInfo::Inst(InstInfo { ty }) => Argument {
            ty: compl.define_instance(*ty, &mut binds),
            binding: match ty {
                ValueType::Scalar(_) | ValueType::Vector(_) => Some(binds.next(&ty.ty())),
                ValueType::Matrix(_) => None,
            },
        },
        InputInfo::Index => Argument {
            ty: compl.define_index(),
            binding: Some(Binding::BuiltIn(BuiltIn::VertexIndex)),
        },
    };

    let inputs: Vec<_> = cx.inputs.iter().map(make_input).collect();
    for (id, en) in iter::zip(0.., &cx.groups) {
        compl.define_group(id, en.def());
    }

    let (fs, required, fsty) = {
        let mut fs = Fs::new(compl);
        let ex = color.eval(&mut fs);
        fs.inner.ret(ex);
        let fsty = fs.define_fragment_ty();
        let mut args = [fsty].into_iter().map(Argument::from_type);
        let built = fs.inner.build(Stage::Fragment, &mut args, Return::Color);
        (built, fs.required, fsty)
    };

    let vs = {
        let mut vs = Vs::new(fs.compl);
        let ex = place.eval(&mut vs);
        let eval = |req: Required| match req.evalf {
            EvalFunction::Position => ex,
            EvalFunction::Fn(f) => f(&mut vs),
        };

        let out = required.into_iter().map(eval).collect();
        let res = vs.0.compose(fsty, out);
        vs.0.ret(res);
        let mut args = inputs.into_iter();
        vs.0.build(Stage::Vertex, &mut args, Return::Ty(fsty))
    };

    let compl = vs.compl;
    let nm = naga::Module {
        types: compl.types,
        global_variables: compl.globs.vars,
        entry_points: vec![vs.point, fs.point],
        ..Default::default()
    };

    _ = pop;
    Module::new(cx, nm)
}

#[derive(Clone, Copy)]
pub struct Expr(Handle<Expression>);

impl Expr {
    pub(crate) fn get(self) -> Handle<Expression> {
        self.0
    }
}

pub(crate) struct Exprs(pub Vec<Handle<Expression>>);

impl FromIterator<Expr> for Exprs {
    fn from_iter<T: IntoIterator<Item = Expr>>(iter: T) -> Self {
        Self(iter.into_iter().map(Expr::get).collect())
    }
}

pub trait Eval<E>: Sized {
    type Out;
    fn eval(self, en: &mut E) -> Expr;
}

impl<E> Eval<E> for f32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::F32(self))
    }
}

impl<E> Eval<E> for i32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::I32(self))
    }
}

impl<E> Eval<E> for u32
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::U32(self))
    }
}

impl<E> Eval<E> for bool
where
    E: GetEntry,
{
    type Out = Self;

    fn eval(self, en: &mut E) -> Expr {
        en.get_entry().literal(Literal::Bool(self))
    }
}

#[derive(Clone, Copy)]
pub struct ReadIndex {
    id: u32,
}

impl ReadIndex {
    pub(crate) const fn new(id: u32) -> Ret<Self, u32> {
        Ret::new(Self { id })
    }
}

impl Eval<Vs> for Ret<ReadIndex, u32> {
    type Out = u32;

    fn eval(self, en: &mut Vs) -> Expr {
        en.get_entry().argument(self.get().id)
    }
}

#[derive(Clone, Copy)]
pub struct ReadVertex {
    id: u32,
    index: u32,
}

impl ReadVertex {
    pub const fn new<O>(id: u32, index: u32) -> Ret<Self, O> {
        Ret::new(Self { id, index })
    }
}

impl<O> Eval<Vs> for Ret<ReadVertex, O> {
    type Out = O;

    fn eval(self, en: &mut Vs) -> Expr {
        let en = en.get_entry();
        let arg = en.argument(self.get().id);
        en.access_index(arg, self.get().index)
    }
}

#[derive(Clone, Copy)]
pub struct ReadInstance {
    id: u32,
}

impl ReadInstance {
    pub const fn new<O>(id: u32) -> Ret<Self, O> {
        Ret::new(Self { id })
    }
}

impl<O> Eval<Vs> for Ret<ReadInstance, O>
where
    O: types::Value,
{
    type Out = O;

    fn eval(self, en: &mut Vs) -> Expr {
        let en = en.get_entry();
        let id = self.get().id;
        match O::VALUE_TYPE {
            ValueType::Scalar(_) | ValueType::Vector(_) => en.argument(id),
            ValueType::Matrix(mat) => {
                let ty = en.new_type(O::VALUE_TYPE.ty());
                let arg = en.argument(id);
                let exprs = (0..mat.dims())
                    .map(|index| en.access_index(arg, index))
                    .collect();

                en.compose(ty, exprs)
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct GlobalOut(Rc<Cell<Stages>>);

impl GlobalOut {
    fn with_stage(&self, stage: Stage) {
        self.0.set(self.0.get().with(stage));
    }

    pub fn get(&self) -> Stages {
        self.0.get()
    }
}

#[derive(Clone)]
pub struct ReadGlobal {
    id: u32,
    binding: u32,
    is_value: bool,
    out: GlobalOut,
}

impl ReadGlobal {
    pub const fn new<T>(id: u32, binding: u32, is_value: bool, out: GlobalOut) -> Ret<Self, T> {
        Ret::new(Self {
            id,
            binding,
            is_value,
            out,
        })
    }
}

impl<O, E> Eval<E> for Ret<ReadGlobal, O>
where
    E: GetEntry,
{
    type Out = O;

    fn eval(self, en: &mut E) -> Expr {
        let ReadGlobal {
            id,
            binding,
            is_value,
            out,
        } = self.get();

        out.with_stage(E::STAGE);
        let en = en.get_entry();
        let res = ResourceBinding { group: id, binding };
        let var = en.compl.globs.get(&res);
        let global = en.global(var);
        if is_value {
            en.load(global)
        } else {
            global
        }
    }
}

pub const fn fragment<A>(a: A) -> Ret<Fragment<A>, A::Out>
where
    A: Eval<Vs>,
    A::Out: types::Vector,
{
    Ret::new(Fragment(a))
}

pub struct Fragment<A>(A);

impl<A> Eval<Fs> for Ret<Fragment<A>, A::Out>
where
    A: Eval<Vs> + 'static,
    A::Out: types::Vector,
{
    type Out = A::Out;

    fn eval(self, en: &mut Fs) -> Expr {
        let vecty = <A::Out as types::Vector>::TYPE;
        let index = en.push_evalf(vecty, |en| self.get().0.eval(en));
        let en = &mut en.inner;
        let arg = en.argument(0);
        en.access_index(arg, index)
    }
}

#[derive(Default)]
struct Frames {
    stack: Vec<u32>,
    count: u32,
}

thread_local! {
    static STACK: RefCell<Frames> = RefCell::default();
}

fn push() -> PopGuard {
    STACK.with(|s| {
        let mut s = s.borrow_mut();
        let id = s.count;
        s.count += 1;
        s.stack.push(id);
    });

    PopGuard
}

struct PopGuard;

impl Drop for PopGuard {
    fn drop(&mut self) {
        STACK.with(|s| s.borrow_mut().stack.pop());
    }
}

fn top() -> Option<u32> {
    STACK.with(|s| s.borrow().stack.last().copied())
}

fn in_stack(frame_id: u32) -> bool {
    STACK.with(|s| {
        s.borrow()
            .stack
            .iter()
            .rev()
            .any(|&frame| frame == frame_id)
    })
}

pub fn thunk<A, E>(a: A) -> Ret<Thunk<A, E>, A::Out>
where
    A: Eval<E>,
{
    let Some(frame_id) = top() else {
        panic!("thunk cannot be created outside of a shader function");
    };

    Ret::new(Thunk::new(frame_id, a))
}

pub struct Thunk<A, E> {
    frame_id: u32,
    s: Rc<Cell<State<A>>>,
    e: PhantomData<E>,
}

impl<A, E> Thunk<A, E> {
    fn new(frame_id: u32, a: A) -> Self {
        Self {
            frame_id,
            s: Rc::new(Cell::new(State::Eval(a))),
            e: PhantomData,
        }
    }
}

impl<A, E> Clone for Thunk<A, E> {
    fn clone(&self) -> Self {
        Self {
            frame_id: self.frame_id,
            s: Rc::clone(&self.s),
            e: PhantomData,
        }
    }
}

impl<A, E> Eval<E> for Ret<Thunk<A, E>, A::Out>
where
    A: Eval<E>,
{
    type Out = A::Out;

    fn eval(self, en: &mut E) -> Expr {
        let Thunk { frame_id, s, .. } = self.get();
        assert!(
            in_stack(frame_id),
            "it's impossible to eval the thunk in this scope",
        );

        let ex = match s.replace(State::None) {
            State::None => unreachable!(),
            State::Eval(a) => a.eval(en),
            State::Expr(ex) => ex,
        };

        s.set(State::Expr(ex));
        ex
    }
}

enum State<A> {
    None,
    Eval(A),
    Expr(Expr),
}

#[derive(Default)]
pub(crate) struct Evaluated([Option<Expr>; 4]);

impl Evaluated {
    fn push(&mut self, expr: Expr) {
        let slot = self
            .0
            .iter_mut()
            .find(|slot| slot.is_none())
            .expect("empty slot");

        *slot = Some(expr);
    }

    pub fn into_iter(self) -> impl Iterator<Item = Expr> {
        self.0.into_iter().flatten()
    }
}

pub(crate) trait EvalTuple<E> {
    fn eval(self, en: &mut E, o: &mut Evaluated);
}

macro_rules! impl_eval_tuple {
    ($($t:ident),*) => {
        impl<$($t),*, E> EvalTuple<E> for ($($t),*,)
        where
            $(
                $t: Eval<E>,
            )*
        {
            #[allow(non_snake_case)]
            fn eval(self, en: &mut E, o: &mut Evaluated) {
                let ($($t),*,) = self;
                $(
                    o.push($t.eval(en));
                )*
            }
        }
    };
}

impl_eval_tuple!(X);
impl_eval_tuple!(X, Y);
impl_eval_tuple!(X, Y, Z);
impl_eval_tuple!(X, Y, Z, W);

#[derive(Clone, Copy)]
pub(crate) enum Stage {
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

pub(crate) trait GetEntry {
    const STAGE: Stage;
    fn get_entry(&mut self) -> &mut Entry;
}

pub struct Vs(Entry);

impl Vs {
    fn new(compl: Compiler) -> Self {
        Self(Entry::new(compl))
    }
}

impl GetEntry for Vs {
    const STAGE: Stage = Stage::Vertex;

    fn get_entry(&mut self) -> &mut Entry {
        &mut self.0
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
    Fn(Box<dyn FnOnce(&mut Vs) -> Expr>),
}

struct Required {
    vecty: VectorType,
    evalf: EvalFunction,
}

pub struct Fs {
    inner: Entry,
    required: Vec<Required>,
}

impl Fs {
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
        F: FnOnce(&mut Vs) -> Expr + 'static,
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
        let mut binds = Bindings::default();
        self.inner.compl.define_input(&mut members, &mut binds)
    }
}

impl GetEntry for Fs {
    const STAGE: Stage = Stage::Fragment;

    fn get_entry(&mut self) -> &mut Entry {
        &mut self.inner
    }
}

struct Built {
    compl: Compiler,
    point: EntryPoint,
}

#[derive(Clone, Copy)]
enum Return {
    Ty(Handle<Type>),
    Color,
}

type Args<'a> = dyn Iterator<Item = Argument> + 'a;

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

pub struct Entry {
    compl: Compiler,
    stack: Stack,
    locls: Arena<LocalVariable>,
    exprs: Arena<Expression>,
    cached_glob: HashMap<Handle<GlobalVariable>, Expr>,
    cached_locl: HashMap<Handle<LocalVariable>, Expr>,
    cached_args: HashMap<u32, Expr>,
}

impl Entry {
    fn new(compl: Compiler) -> Self {
        Self {
            compl,
            stack: Stack(vec![Statements::default()]),
            locls: Arena::default(),
            exprs: Arena::default(),
            cached_glob: HashMap::default(),
            cached_locl: HashMap::default(),
            cached_args: HashMap::default(),
        }
    }

    fn push(&mut self) -> PopGuard {
        self.stack.push();
        push()
    }

    fn pop(&mut self, pop: PopGuard) -> Statements {
        _ = pop;
        self.stack.pop()
    }

    pub(crate) fn new_type(&mut self, ty: Type) -> Handle<Type> {
        self.compl.types.insert(ty, Span::UNDEFINED)
    }

    fn add_local(&mut self, ty: Handle<Type>) -> Handle<LocalVariable> {
        let local = LocalVariable {
            name: None,
            ty,
            init: None,
        };

        self.locls.append(local, Span::UNDEFINED)
    }

    fn literal(&mut self, literal: Literal) -> Expr {
        let ex = Expression::Literal(literal);
        Expr(self.exprs.append(ex, Span::UNDEFINED))
    }

    fn argument(&mut self, n: u32) -> Expr {
        *self.cached_args.entry(n).or_insert_with(|| {
            let ex = Expression::FunctionArgument(n);
            Expr(self.exprs.append(ex, Span::UNDEFINED))
        })
    }

    fn global(&mut self, v: Handle<GlobalVariable>) -> Expr {
        *self.cached_glob.entry(v).or_insert_with(|| {
            let ex = Expression::GlobalVariable(v);
            Expr(self.exprs.append(ex, Span::UNDEFINED))
        })
    }

    fn local(&mut self, v: Handle<LocalVariable>) -> Expr {
        *self.cached_locl.entry(v).or_insert_with(|| {
            let ex = Expression::LocalVariable(v);
            Expr(self.exprs.append(ex, Span::UNDEFINED))
        })
    }

    fn load(&mut self, ptr: Expr) -> Expr {
        let ex = Expression::Load { pointer: ptr.0 };
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn access_index(&mut self, base: Expr, index: u32) -> Expr {
        let ex = Expression::AccessIndex {
            base: base.0,
            index,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn convert(&mut self, expr: Expr, ty: ScalarType) -> Expr {
        let (kind, width) = ty.inner();
        let ex = Expression::As {
            expr: expr.0,
            kind,
            convert: Some(width),
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn unary(&mut self, op: Un, a: Expr) -> Expr {
        let ex = Expression::Unary {
            op: op.operator(),
            expr: a.0,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn binary(&mut self, op: Bi, a: Expr, b: Expr) -> Expr {
        let ex = Expression::Binary {
            op: op.operator(),
            left: a.0,
            right: b.0,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn math(&mut self, f: Func, exprs: Evaluated) -> Expr {
        let ex = f.expr(exprs);
        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn compose(&mut self, ty: Handle<Type>, exprs: Exprs) -> Expr {
        let ex = Expression::Compose {
            ty,
            components: exprs.0,
        };

        let handle = self.exprs.append(ex, Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    pub(crate) fn sample(&mut self, ex: Sampled) -> Expr {
        let handle = self.exprs.append(ex.expr(), Span::UNDEFINED);
        let st = Statement::Emit(Range::new_from_bounds(handle, handle));
        self.stack.insert(st, &self.exprs);
        Expr(handle)
    }

    fn ret(&mut self, value: Expr) {
        let st = Statement::Return {
            value: Some(value.0),
        };

        self.stack.insert(st, &self.exprs);
    }

    fn build(mut self, stage: Stage, args: &mut Args, ret: Return) -> Built {
        const COLOR_TYPE: Type = VectorType::Vec4f.ty();

        let res = match ret {
            Return::Ty(ty) => FunctionResult { ty, binding: None },
            Return::Color => {
                let mut binds = Bindings::default();
                FunctionResult {
                    ty: self.new_type(COLOR_TYPE),
                    binding: Some(binds.next(&COLOR_TYPE)),
                }
            }
        };

        let point = EntryPoint {
            name: stage.name().to_owned(),
            stage: stage.shader_stage(),
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: Function {
                arguments: args.map(Argument::into_function).collect(),
                result: Some(res),
                local_variables: self.locls,
                expressions: self.exprs,
                body: self.stack.pop().0.into(),
                ..Default::default()
            },
        };

        Built {
            compl: self.compl,
            point,
        }
    }
}

pub struct Branch<'a, E> {
    en: &'a mut E,
    expr: Expr,
}

impl<'a, E> Branch<'a, E> {
    pub(crate) fn new(en: &'a mut E, ty: Handle<Type>) -> Self
    where
        E: GetEntry,
    {
        let expr = {
            let en = en.get_entry();
            let v = en.add_local(ty);
            en.local(v)
        };

        Self { en, expr }
    }

    pub(crate) fn entry(&mut self) -> &mut E {
        self.en
    }

    pub(crate) fn load(&mut self) -> Expr
    where
        E: GetEntry,
    {
        self.en.get_entry().load(self.expr)
    }

    pub(crate) fn add<A, B>(&mut self, c: Expr, a: A, b: B)
    where
        E: GetEntry,
        A: FnOnce(&mut E) -> Expr,
        B: FnOnce(&mut Self) -> Option<Expr>,
    {
        let a_branch = {
            let pop = self.en.get_entry().push();
            let a = a(self.entry());
            let en = self.en.get_entry();
            let mut s = en.pop(pop);
            let st = Statement::Store {
                pointer: self.expr.0,
                value: a.0,
            };

            s.insert(st, &en.exprs);
            s
        };

        let b_branch = {
            let pop = self.en.get_entry().push();
            let b = b(self);
            let en = self.en.get_entry();
            let mut s = en.pop(pop);
            if let Some(b) = b {
                let st = Statement::Store {
                    pointer: self.expr.0,
                    value: b.0,
                };

                s.insert(st, &en.exprs);
            }

            s
        };

        let st = Statement::If {
            condition: c.0,
            accept: a_branch.0.into(),
            reject: b_branch.0.into(),
        };

        let en = self.en.get_entry();
        en.stack.insert(st, &en.exprs);
    }
}

struct Stack(Vec<Statements>);

impl Stack {
    fn insert(&mut self, st: Statement, exprs: &Arena<Expression>) {
        self.0
            .last_mut()
            .expect("shouldn't be empty")
            .insert(st, exprs);
    }

    fn push(&mut self) {
        self.0.push(Statements::default());
    }

    fn pop(&mut self) -> Statements {
        self.0.pop().expect("shouldn't be empty")
    }
}

#[derive(Default)]
struct Statements(Vec<Statement>);

impl Statements {
    fn insert(&mut self, st: Statement, exprs: &Arena<Expression>) {
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

type Members<'a> = dyn ExactSizeIterator<Item = Member> + 'a;

#[derive(Default)]
struct Compiler {
    types: UniqueArena<Type>,
    globs: Globals,
}

impl Compiler {
    const VECTOR_SIZE: u32 = mem::size_of::<f32>() as u32 * 4;

    fn define_index(&mut self) -> Handle<Type> {
        self.types.insert(ScalarType::Uint.ty(), Span::UNDEFINED)
    }

    fn define_input(&mut self, new: &mut Members, binds: &mut Bindings) -> Handle<Type> {
        let len = new.len();
        let mut members = Vec::with_capacity(len);
        for (idx, Member { vecty, built }) in iter::zip(0.., new) {
            let ty = vecty.ty();
            let binding = match built {
                Some(bi @ BuiltIn::Position { .. }) => Binding::BuiltIn(bi),
                None => binds.next(&ty),
                _ => unimplemented!(),
            };

            members.push(StructMember {
                name: None,
                ty: self.types.insert(ty, Span::UNDEFINED),
                binding: Some(binding),
                offset: idx * Self::VECTOR_SIZE,
            });
        }

        let ty = Type {
            name: None,
            inner: TypeInner::Struct {
                members,
                span: len as u32 * Self::VECTOR_SIZE,
            },
        };

        self.types.insert(ty, Span::UNDEFINED)
    }

    fn define_instance(&mut self, ty: ValueType, binds: &mut Bindings) -> Handle<Type> {
        let ty = match ty {
            ValueType::Scalar(_) | ValueType::Vector(_) => ty.ty(),
            ValueType::Matrix(mat) => {
                let len = mat.dims();
                let mut members = Vec::with_capacity(len as usize);
                for idx in 0..len {
                    let ty = mat.vector_type().ty();
                    let binding = binds.next(&ty);
                    members.push(StructMember {
                        name: None,
                        ty: self.types.insert(ty, Span::UNDEFINED),
                        binding: Some(binding),
                        offset: idx * Self::VECTOR_SIZE,
                    });
                }

                Type {
                    name: None,
                    inner: TypeInner::Struct {
                        members,
                        span: len * Self::VECTOR_SIZE,
                    },
                }
            }
        };

        self.types.insert(ty, Span::UNDEFINED)
    }

    fn define_group(&mut self, group: u32, def: Define<MemberType>) {
        for (binding, member) in iter::zip(0.., def) {
            let space = member.address_space();
            let ty = self.types.insert(member.ty(), Span::UNDEFINED);
            let res = ResourceBinding { group, binding };
            self.globs.add(space, ty, res);
        }
    }
}

#[derive(Default)]
struct Globals {
    vars: Arena<GlobalVariable>,
    handles: HashMap<ResourceBinding, Handle<GlobalVariable>>,
}

impl Globals {
    fn add(&mut self, space: AddressSpace, ty: Handle<Type>, res: ResourceBinding) {
        self.handles.entry(res.clone()).or_insert_with(|| {
            let var = GlobalVariable {
                name: None,
                space,
                binding: Some(res),
                ty,
                init: None,
            };

            self.vars.append(var, Span::UNDEFINED)
        });
    }

    fn get(&self, res: &ResourceBinding) -> Handle<GlobalVariable> {
        self.handles[res]
    }
}

#[derive(Default)]
struct Bindings(u32);

impl Bindings {
    fn next(&mut self, ty: &Type) -> Binding {
        let mut binding = Binding::Location {
            location: self.0,
            second_blend_source: false,
            interpolation: None,
            sampling: None,
        };

        self.0 += 1;
        binding.apply_default_interpolation(&ty.inner);
        binding
    }
}
