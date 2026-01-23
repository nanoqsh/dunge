use {
    crate::gener,
    proc_macro2::{Ident, TokenStream},
};

pub(crate) enum Event<C = gener::Control> {
    Fn(Box<Fn>),
    BlockStart,
    BlockEnd,
    Semi,
    Local(Box<Local>),
    Name(TokenStream),
    Lit(TokenStream),
    Array { len: usize },
    Assign,
    BinOp(BinOp),
    Index,
    Member(Ident),
    Method(Ident),
    Call(Arity),
    Cast(TokenStream),
    ConstExpr(TokenStream),
    Return,
    Struct(Box<Struct>),
    Control(C),
}

impl Event {
    pub(crate) fn arity(&self) -> usize {
        match self {
            Self::Fn(_) => 0,
            Self::BlockStart => 0,
            Self::BlockEnd => 0,
            Self::Semi => 0,
            Self::Local(_) => 0,
            Self::Name(_) => 0,
            Self::Lit(_) => 0,
            Self::Array { len } => *len,
            Self::Assign => 2,
            Self::BinOp(_) => 2,
            Self::Index => 2,
            Self::Member(_) => 1,
            Self::Method(_) => 1,
            Self::Call(arity) => 1 + arity.get(),
            Self::Cast(_) => 1,
            Self::ConstExpr(_) => 0,
            Self::Return => 1,
            Self::Struct(s) => s.members.len(),
            Self::Control(gener::Control::BreakIf) => 1,
            Self::Control(gener::Control::Loop) => 1,
            Self::Control(gener::Control::IfElse) => 3,
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn debug(&self) -> impl std::fmt::Display {
        std::fmt::from_fn(move |f| match self {
            Self::Fn(func) => {
                write!(f, "FN\t\t{}", func.ident)?;
                if !func.vis.is_empty() {
                    write!(f, " ({})", func.vis)?;
                }

                for Input { ident, ty } in &func.inputs {
                    write!(f, " {ident}: {ty}")?;
                }

                if let Some(ty) = &func.output.ty {
                    write!(f, " -> {ty}")?;
                }

                Ok(())
            }
            Self::BlockStart => write!(f, "BLOCKSTART\t\t"),
            Self::BlockEnd => write!(f, "BLOCKEND\t\t"),
            Self::Semi => write!(f, "SEMI\t\t"),
            Self::Local(local) => {
                write!(f, "LOCAL\t\t{}", local.ident)?;
                if let Some(ty) = &local.ty {
                    write!(f, ": {ty}")?;
                }

                Ok(())
            }
            Self::Name(name) => write!(f, "NAME\t\t{name}"),
            Self::Lit(lit) => write!(f, "LIT\t\t{lit}"),
            Self::Array { len } => write!(f, "ARRAY\t\t{len}"),
            Self::Assign => write!(f, "ASSIGN\t\t"),
            Self::BinOp(binop) => {
                write!(f, "BINOP\t\t")?;
                f.write_str(binop.debug())
            }
            Self::Index => write!(f, "INDEX\t\t"),
            Self::Member(name) => write!(f, "MEMBER\t\t{name}"),
            Self::Method(name) => write!(f, "METHOD\t\t{name}"),
            Self::Call(arity) => write!(f, "CALL\t\t{}", arity.get()),
            Self::Cast(ty) => write!(f, "CAST\t\t{ty}"),
            Self::ConstExpr(_) => write!(f, "CONSTEXPR\t\t.."),
            Self::Return => write!(f, "RETURN\t\t"),
            Self::Struct(s) => {
                write!(f, "STRUCT\t\t{}", s.name)?;
                for member in &s.members {
                    write!(f, "{member}, ")?;
                }

                Ok(())
            }
            Self::Control(gener::Control::BreakIf) => write!(f, "BREAKIF\t\t"),
            Self::Control(gener::Control::Loop) => write!(f, "LOOP\t\t"),
            Self::Control(gener::Control::IfElse) => write!(f, "IFELSE\t\t"),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Arity {
    N0,
    N1,
    N2,
    N3,
    N4,
}

impl Arity {
    pub(crate) fn get(self) -> usize {
        match self {
            Self::N0 => 0,
            Self::N1 => 1,
            Self::N2 => 2,
            Self::N3 => 3,
            Self::N4 => 4,
        }
    }
}

pub(crate) struct Fn {
    pub vis: TokenStream,
    pub ident: Ident,
    pub inputs: Vec<Input>,
    pub output: Output,
}

pub(crate) struct Input {
    pub ident: Ident,
    pub ty: TokenStream,
}

pub(crate) struct Output {
    pub ty: Option<TokenStream>,
}

pub(crate) struct Local {
    pub ident: Ident,
    pub ty: Option<TokenStream>,
}

#[derive(Clone, Copy)]
pub(crate) enum BinOp {
    Add { assign: bool },
    Sub { assign: bool },
    Mul { assign: bool },
    Div { assign: bool },
    Rem { assign: bool },
    Shl { assign: bool },
    Shr { assign: bool },
    BitAnd { assign: bool },
    BitOr { assign: bool },
    BitXor { assign: bool },
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Ge,
    Gt,
}

impl BinOp {
    pub(crate) fn from_syn(op: syn::BinOp) -> Option<Self> {
        match op {
            syn::BinOp::Add(_) => Some(Self::Add { assign: false }),
            syn::BinOp::Sub(_) => Some(Self::Sub { assign: false }),
            syn::BinOp::Mul(_) => Some(Self::Mul { assign: false }),
            syn::BinOp::Div(_) => Some(Self::Div { assign: false }),
            syn::BinOp::Rem(_) => Some(Self::Rem { assign: false }),
            syn::BinOp::Shl(_) => Some(Self::Shl { assign: false }),
            syn::BinOp::Shr(_) => Some(Self::Shr { assign: false }),
            syn::BinOp::BitAnd(_) => Some(Self::BitAnd { assign: false }),
            syn::BinOp::BitOr(_) => Some(Self::BitOr { assign: false }),
            syn::BinOp::BitXor(_) => Some(Self::BitXor { assign: false }),
            syn::BinOp::And(_) => Some(Self::And),
            syn::BinOp::Or(_) => Some(Self::Or),
            syn::BinOp::AddAssign(_) => Some(Self::Add { assign: true }),
            syn::BinOp::SubAssign(_) => Some(Self::Sub { assign: true }),
            syn::BinOp::MulAssign(_) => Some(Self::Mul { assign: true }),
            syn::BinOp::DivAssign(_) => Some(Self::Div { assign: true }),
            syn::BinOp::RemAssign(_) => Some(Self::Rem { assign: true }),
            syn::BinOp::ShlAssign(_) => Some(Self::Shl { assign: true }),
            syn::BinOp::ShrAssign(_) => Some(Self::Shr { assign: true }),
            syn::BinOp::BitAndAssign(_) => Some(Self::BitAnd { assign: true }),
            syn::BinOp::BitOrAssign(_) => Some(Self::BitOr { assign: true }),
            syn::BinOp::BitXorAssign(_) => Some(Self::BitXor { assign: true }),
            syn::BinOp::Eq(_) => Some(Self::Eq),
            syn::BinOp::Lt(_) => Some(Self::Lt),
            syn::BinOp::Le(_) => Some(Self::Le),
            syn::BinOp::Ne(_) => Some(Self::Ne),
            syn::BinOp::Ge(_) => Some(Self::Ge),
            syn::BinOp::Gt(_) => Some(Self::Gt),
            _ => None,
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn debug(self) -> &'static str {
        match self {
            Self::Add { assign: true } => "ADD (assign)",
            Self::Add { assign: false } => "ADD",
            Self::Sub { assign: true } => "SUB (assign)",
            Self::Sub { assign: false } => "SUB",
            Self::Mul { assign: true } => "MUL (assign)",
            Self::Mul { assign: false } => "MUL",
            Self::Div { assign: true } => "DIV (assign)",
            Self::Div { assign: false } => "DIV",
            Self::Rem { assign: true } => "REM (assign)",
            Self::Rem { assign: false } => "REM",
            Self::Shl { assign: true } => "SHL (assign)",
            Self::Shl { assign: false } => "SHL",
            Self::Shr { assign: true } => "SHR (assign)",
            Self::Shr { assign: false } => "SHR",
            Self::BitAnd { assign: true } => "BITAND (assign)",
            Self::BitAnd { assign: false } => "BITAND",
            Self::BitOr { assign: true } => "BITOR (assign)",
            Self::BitOr { assign: false } => "BITOR",
            Self::BitXor { assign: true } => "BITXOR (assign)",
            Self::BitXor { assign: false } => "BITXOR",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Eq => "EQ",
            Self::Ne => "NE",
            Self::Lt => "LT",
            Self::Le => "LE",
            Self::Ge => "GE",
            Self::Gt => "GT",
        }
    }
}

pub(crate) struct Struct {
    pub name: TokenStream,
    pub members: Box<[Ident]>,
}
