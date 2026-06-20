use {
    crate::event::*,
    proc_macro2::{Ident, Span, TokenStream},
    quote::ToTokens,
    std::{collections::HashSet, fmt, iter},
};

pub(crate) enum Stage {
    Regular,
    Vertex,
    Fragment,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) enum Control {
    BreakIf,
    Loop,
    IfElse,
}

type Gener<T> = Result<T, Error>;

pub(crate) fn produce<I>(events: I, stage: Stage, path: &TokenStream) -> Gener<TokenStream>
where
    I: IntoIterator<Item = Event<Control>>,
{
    let irc = quote::quote! { #path::irc };

    let stage = match stage {
        Stage::Regular => quote::quote! { #irc::Stage::Regular },
        Stage::Vertex => quote::quote! { #irc::Stage::Vertex },
        Stage::Fragment => quote::quote! { #irc::Stage::Fragment },
    };

    let mut make_block = maker(|count| Block::new(quote::format_ident!("_b{count}")));
    let mut make_texpr = maker(|count| quote::format_ident!("_e{count}"));
    let mixed = |mut ident: Ident| {
        ident.set_span(Span::mixed_site());
        ident
    };

    let mut mod_vis = TokenStream::new();
    let mut mod_ident = None;
    let mut sign = TokenStream::new();
    let mut boot = TokenStream::new();
    let mut output_ty = None;
    let mut body = make_block();
    let mut root = None;
    let mut stack = Stack::new();
    let mut locals = HashSet::new();

    for event in events {
        match event {
            Event::Fn(func) => {
                let Fn {
                    vis,
                    ident,
                    inputs,
                    output,
                } = *func;

                mod_vis = vis;
                mod_ident = Some(mixed(ident));

                let input_types = inputs.iter().map(|i| &i.ty);
                let output_type = match &output.ty {
                    Some(ty) => ty,
                    None => &quote::quote! { () },
                };

                sign = quote::quote! { fn (#(#input_types),*) -> #output_type };

                let boot_inputs = inputs
                    .iter()
                    .map(|Input { ty, .. }| quote::quote! { _init.input::<#ty>(); });

                boot = quote::quote! {
                    _irc.boot(_boot, |_init| {
                        #(#boot_inputs)*
                    });
                };

                for Input { ident, ty } in inputs {
                    body.extend(quote::quote! {
                        let #ident = _fnc.input::<#ty>();
                    });
                }

                if let Some(ty) = &output.ty {
                    body.extend(quote::quote! {
                        _fnc.output::<#ty>(#stage);
                    });
                }

                output_ty = output.ty;
            }
            Event::BlockStart => {
                let mut block = make_block();
                if root.is_none() {
                    root = Some(block.name.clone());
                }

                block.extend(quote::quote! {
                    _fnc.push();
                });

                stack.push(block);
            }
            Event::BlockEnd => {
                let Block {
                    name, code: stream, ..
                } = stack.pop()?;

                let curr = stack.top().unwrap_or(&mut body);
                curr.extend(quote::quote! {
                    let #name = {
                        #stream
                        _fnc.pop()
                    };
                });

                curr.push_block(name);
            }
            Event::Semi => {
                let curr = stack.top()?;
                curr.ops.clear();
                curr.extend(quote::quote! { ; });
            }
            Event::Local(local) => {
                let curr = stack.top()?;
                let Local { ident, ty } = *local;
                if !locals.insert(ident.clone()) {
                    return Err(Error::Shadowing);
                }

                let varty = match ty {
                    Some(ty) => quote::quote! { #irc::Variable<#ty> },
                    None => quote::quote! { #irc::Variable<_> },
                };

                curr.extend(quote::quote! {
                    let #ident: #varty = _fnc.add_local();
                });

                curr.push_name(ident.into_token_stream());
            }
            Event::Name(name) => stack.top()?.push_name(name),
            Event::Lit(lit) => stack.top()?.push_lit(lit),
            Event::Array { len } => {
                let curr = stack.top()?;

                let vars: Vec<_> = iter::repeat_with(&mut make_texpr).take(len).collect();
                let items: Vec<_> = (0..len)
                    .map(|_| Ok(curr.pop()?.read()))
                    .collect::<Gener<_>>()?;

                let res = make_texpr();

                curr.extend(quote::quote! {
                    #(
                        let #vars: #irc::Expr<_> = #items;
                    )*
                    let #res: #irc::Expr<[_; #len]> = _fnc.do_compose_array([#(#vars),*]);
                });

                curr.push_expr(res);
            }
            Event::Assign => {
                let curr = stack.top()?;
                let from = curr.pop()?.read();
                let to = curr.pop()?.point()?;

                let varfrom = make_texpr();
                let varto = make_texpr();

                curr.extend(quote::quote! {
                    let #varto: #irc::Pointer<_> = #to;
                    let #varfrom: #irc::Expr<_> = #from;
                    _fnc.do_store(#varto, #varfrom);
                });
            }
            Event::UnOp(unop) => {
                let op = Unary::new(unop).fnname;
                let curr = stack.top()?;
                let arg = curr.pop()?.read();

                let vara = make_texpr();
                let varb = make_texpr();

                curr.extend(quote::quote! {
                    let #vara: #irc::Expr<_> = #arg;
                    let #varb: #irc::Expr<_> = _fnc.do_op(#irc::#op(#vara));
                });

                curr.push_expr(varb);
            }
            Event::BinOp(binop) => {
                let bin = Binary::new(binop);
                let op = bin.fnname;
                let retty = match bin.retty {
                    Some(ty) => ty,
                    None => quote::quote! { _ },
                };

                let (aty, bty) = match bin.args {
                    Some((aty, bty)) => (aty, bty),
                    None => (quote::quote! { _ }, quote::quote! { _ }),
                };

                let curr = stack.top()?;
                let b = curr.pop()?.read();
                let aop = curr.pop()?;
                let aread = aop.read();

                let vara = make_texpr();
                let varb = make_texpr();
                let varc = make_texpr();

                if bin.assign {
                    let apoint = aop.point()?;

                    let vard = make_texpr();

                    curr.extend(quote::quote! {
                        let #vara: #irc::Expr<#aty> = #aread;
                        let #varb: #irc::Expr<#bty> = #b;
                        let #varc: #irc::Expr<#retty> = _fnc.do_op(#irc::#op(#vara, #varb));
                        let #vard: #irc::Pointer<_> = #apoint;
                        _fnc.do_store(#vard, #varc);
                    });
                } else {
                    curr.extend(quote::quote! {
                        let #vara: #irc::Expr<#aty> = #aread;
                        let #varb: #irc::Expr<#bty> = #b;
                        let #varc: #irc::Expr<#retty> = _fnc.do_op(#irc::#op(#vara, #varb));
                    });

                    curr.push_expr(varc);
                }
            }
            Event::Index => {
                let curr = stack.top()?;
                let index = curr.pop()?.read();
                let base = curr.pop()?.point()?;

                let vari = make_texpr();
                let varb = make_texpr();

                curr.extend(quote::quote! {
                    let #vari: #irc::Expr<_> = #index;
                    let #varb: #irc::Pointer<_> = #base;
                });

                curr.push_index(varb, vari);
            }
            Event::Member(name) => {
                let curr = stack.top()?;
                let a = curr.pop()?.point()?;

                let base = make_texpr();

                curr.extend(quote::quote! {
                    let #base: #irc::Pointer<_> = #a;
                });

                let name = mixed(name);
                curr.push_member(base, name);
            }
            Event::Method(name) => {
                let curr = stack.top()?;
                let recv = curr.pop()?.read();

                let var = make_texpr();
                let res = make_texpr();
                let name = mixed(name);

                curr.extend(quote::quote! {
                    let #var: #irc::Expr<_> = #recv;
                    let #res: #irc::Expr<_> = _fnc.do_method_call(#var, |f| f.#name);
                });

                curr.push_method(res);
            }
            Event::Call(arity) => {
                let curr = stack.top()?;
                let Op::Name(name) = curr.pop()? else {
                    return Err(Error::NoName);
                };

                let vars: Vec<_> = iter::repeat_with(&mut make_texpr)
                    .take(arity.get())
                    .collect();

                let reads: Vec<_> = iter::repeat_with(|| Ok(curr.pop()?.read()))
                    .take(arity.get())
                    .collect::<Gener<_>>()?;

                let export = make_texpr();
                let res = make_texpr();

                let error: String = iter::once("attempt to call function ")
                    .chain(name.to_string().split_ascii_whitespace())
                    .collect();

                curr.extend(quote::quote! {
                    #(
                        let #vars: #irc::Expr<_> = #reads;
                    )*
                    let #export: #irc::Export<_, _> = #irc::Export::new(#name);
                    let #res: #irc::Expr<_> = _fnc.do_call(#export, (#(#vars,)*)).map_err(|e| e.add_context(#error))?;
                });

                curr.push_expr(res);
            }
            Event::Cast(ty) => {
                let curr = stack.top()?;
                let a = curr.pop()?.read();

                let vara = make_texpr();
                let cast = make_texpr();

                curr.extend(quote::quote! {
                    let #vara: #irc::Expr<_> = #a;
                    let #cast: #irc::Expr<#ty> = _fnc.do_as(#vara);
                });

                curr.push_expr(cast);
            }
            Event::ConstExpr(expr) => {
                let curr = stack.top()?;

                let vare = make_texpr();

                curr.extend(quote::quote! {
                    let #vare: #irc::Expr<_> = _fnc.do_value(const #expr);
                });

                curr.push_expr(vare);
            }
            Event::Return => {
                let curr = stack.top()?;
                let a = curr.pop()?.read();

                let vara = make_texpr();
                let output_ty = output_ty.as_ref().ok_or(Error::NoFunction)?;

                curr.extend(quote::quote! {
                    let #vara: #irc::Expr<_> = #a;
                    _fnc.do_return::<#output_ty>(#vara);
                });
            }
            Event::Struct(s) => {
                let curr = stack.top()?;

                let name = &s.name;
                let members: Vec<_> = s.members.into_iter().map(mixed).collect();

                let vars: Vec<_> = iter::repeat_with(&mut make_texpr)
                    .take(members.len())
                    .collect();

                let reads: Vec<_> = iter::repeat_with(|| Ok(curr.pop()?.read()))
                    .take(members.len())
                    .collect::<Gener<_>>()?;

                let res = make_texpr();

                curr.extend(quote::quote! {
                    #(
                        let #vars: #irc::Expr<_> = #reads;
                    )*
                    let #res: #irc::Expr<#name> = {
                        let mut _construct = #irc::Constructor::<#name>::default();
                        #(
                            _construct.set_field(#vars, |f| f.#members);
                        )*
                        _construct.build(&mut _fnc)?
                    };
                });

                curr.push_expr(res);
            }
            Event::Control(Control::BreakIf) => {
                let curr = stack.top()?;
                let cond = curr.pop()?.read();

                let varc = make_texpr();

                curr.extend(quote::quote! {
                    let _accept = {
                        _fnc.push();
                        _fnc.pop()
                    };

                    let _reject = {
                        _fnc.push();
                        _fnc.do_break();
                        _fnc.pop()
                    };

                    let #varc: #irc::Expr<::std::primitive::bool> = #cond;
                    _fnc.do_if(#varc, _accept, _reject);
                });
            }
            Event::Control(Control::Loop) => {
                let curr = stack.top()?;
                let block = curr.pop_block()?;

                curr.extend(quote::quote! {
                    let _continuing = {
                        _fnc.push();
                        _fnc.pop()
                    };

                    _fnc.do_loop(#block, _continuing);
                });
            }
            Event::Control(Control::IfElse) => {
                let curr = stack.top()?;
                let else_block = curr.pop_block()?;
                let if_block = curr.pop_block()?;
                let cond = curr.pop()?.read();

                let varc = make_texpr();

                curr.extend(quote::quote! {
                    let #varc: #irc::Expr<::std::primitive::bool> = #cond;
                    _fnc.do_if(#varc, #if_block, #else_block);
                });
            }
        }
    }

    let mod_ident = mod_ident.ok_or(Error::NoFunction)?;
    let code = body.code();

    Ok(quote::quote! {
        #mod_vis mod #mod_ident {
            #![allow(unreachable_pub)]

            use super::*;

            pub type Sign = #sign;

            pub const STAGE: #irc::Stage = #stage;

            pub fn build(_irc: &mut #irc::Irc, _boot: #path::module::Boot) -> #irc::Comp<()> {
                #boot
                let mut _fnc = #irc::Fnc::new(_irc);
                #code
                _fnc.build(#root, #stage);
                ::std::result::Result::Ok(())
            }
        }
    })
}

pub(crate) enum Error {
    StackUnderflow,
    BlockUnderflow,
    NoBlock,
    NoName,
    NoFunction,
    PointToLiteral,
    PointToExpression,
    Shadowing,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StackUnderflow => f.write_str("stack underflow"),
            Self::BlockUnderflow => f.write_str("block underflow"),
            Self::NoBlock => f.write_str("no block"),
            Self::NoName => f.write_str("no name"),
            Self::NoFunction => f.write_str("no function"),
            Self::PointToLiteral => f.write_str("point to literal"),
            Self::PointToExpression => f.write_str("point to expression"),
            Self::Shadowing => f.write_str("variable shadowing is unsupported"),
        }
    }
}

#[derive(Clone, Debug)]
enum Op {
    Name(TokenStream),
    Lit(TokenStream),
    Expr(Ident),
    Index { base: Ident, index: Ident },
    Member { base: Ident, name: Ident },
    Method { name: Ident },
}

impl Op {
    fn read(&self) -> TokenStream {
        match self {
            Self::Name(ident) => quote::quote! {
                {
                    let _tmp = _fnc.do_point(#ident);
                    _fnc.do_load(_tmp)
                }
            },
            Self::Lit(lit) => quote::quote! {
                _fnc.do_value(#lit)
            },
            Self::Expr(ex) => quote::quote! { #ex },
            Self::Index { base, index } => quote::quote! {
                {
                    let _tmp = _fnc.do_access_index(#base, #index);
                    _fnc.do_load(_tmp)
                }
            },
            Self::Member { base, name } => quote::quote! {
                {
                    let _tmp = _fnc.do_access_field(#base, |f| f.#name);
                    _fnc.do_load(_tmp)
                }
            },
            Self::Method { name } => quote::quote! { #name },
        }
    }

    fn point(&self) -> Gener<TokenStream> {
        match self {
            Self::Name(ident) => Ok(quote::quote! {
                _fnc.do_point(#ident)
            }),
            Self::Lit(_) => Err(Error::PointToLiteral),
            Self::Expr(_) => Err(Error::PointToExpression),
            Self::Index { base, index } => Ok(quote::quote! {
                _fnc.do_access_index(#base, #index)
            }),
            Self::Member { base, name } => Ok(quote::quote! {
                _fnc.do_access_field(#base, |f| f.#name)
            }),
            Self::Method { name } => Ok(quote::quote! {
                _fnc.do_point_noop(#name)
            }),
        }
    }
}

struct Stack(Vec<Block>);

impl Stack {
    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, block: Block) {
        self.0.push(block);
    }

    fn pop(&mut self) -> Gener<Block> {
        self.0.pop().ok_or(Error::StackUnderflow)
    }

    fn top(&mut self) -> Gener<&mut Block> {
        self.0.last_mut().ok_or(Error::StackUnderflow)
    }
}

struct Block {
    name: Ident,
    code: TokenStream,
    ops: Vec<Op>,
    blocks: Vec<Ident>,
}

impl Block {
    fn new(name: Ident) -> Self {
        Self {
            name,
            code: TokenStream::new(),
            ops: vec![],
            blocks: vec![],
        }
    }

    fn push_name(&mut self, name: TokenStream) {
        self.ops.push(Op::Name(name));
    }

    fn push_lit(&mut self, lit: TokenStream) {
        self.ops.push(Op::Lit(lit));
    }

    fn push_expr(&mut self, ex: Ident) {
        self.ops.push(Op::Expr(ex));
    }

    fn push_index(&mut self, base: Ident, index: Ident) {
        self.ops.push(Op::Index { base, index });
    }

    fn push_member(&mut self, base: Ident, name: Ident) {
        self.ops.push(Op::Member { base, name });
    }

    fn push_method(&mut self, name: Ident) {
        self.ops.push(Op::Method { name });
    }

    fn pop(&mut self) -> Gener<Op> {
        self.ops.pop().ok_or(Error::BlockUnderflow)
    }

    fn push_block(&mut self, block: Ident) {
        self.blocks.push(block);
    }

    fn pop_block(&mut self) -> Gener<Ident> {
        self.blocks.pop().ok_or(Error::NoBlock)
    }

    fn extend<I>(&mut self, tokens: I)
    where
        I: IntoIterator,
        TokenStream: Extend<I::Item>,
    {
        self.code.extend(tokens);
    }

    fn code(self) -> TokenStream {
        self.code
    }
}

struct Unary {
    fnname: Ident,
}

impl Unary {
    fn new(unop: UnOp) -> Self {
        Self {
            fnname: match unop {
                UnOp::Neg => quote::format_ident!("neg"),
                UnOp::Not => quote::format_ident!("not"),
            },
        }
    }
}

struct Binary {
    assign: bool,
    fnname: Ident,
    args: Option<(TokenStream, TokenStream)>,
    retty: Option<TokenStream>,
}

impl Binary {
    fn new(binop: BinOp) -> Self {
        match binop {
            BinOp::Add { assign } => Self {
                assign,
                fnname: quote::format_ident!("add"),
                args: None,
                retty: None,
            },
            BinOp::Sub { assign } => Self {
                assign,
                fnname: quote::format_ident!("sub"),
                args: None,
                retty: None,
            },
            BinOp::Mul { assign } => Self {
                assign,
                fnname: quote::format_ident!("mul"),
                args: None,
                retty: None,
            },
            BinOp::Div { assign } => Self {
                assign,
                fnname: quote::format_ident!("div"),
                args: None,
                retty: None,
            },
            BinOp::Rem { assign } => Self {
                assign,
                fnname: quote::format_ident!("rem"),
                args: None,
                retty: None,
            },
            BinOp::Shl { assign } => Self {
                assign,
                fnname: quote::format_ident!("shl"),
                args: None,
                retty: None,
            },
            BinOp::Shr { assign } => Self {
                assign,
                fnname: quote::format_ident!("shr"),
                args: None,
                retty: None,
            },
            BinOp::BitAnd { assign } => Self {
                assign,
                fnname: quote::format_ident!("bitand"),
                args: None,
                retty: None,
            },
            BinOp::BitOr { assign } => Self {
                assign,
                fnname: quote::format_ident!("bitor"),
                args: None,
                retty: None,
            },
            BinOp::BitXor { assign } => Self {
                assign,
                fnname: quote::format_ident!("bitxor"),
                args: None,
                retty: None,
            },
            BinOp::And => Self {
                assign: false,
                fnname: quote::format_ident!("and"),
                args: Some((
                    quote::quote! { ::core::primitive::bool },
                    quote::quote! { ::core::primitive::bool },
                )),
                retty: None,
            },
            BinOp::Or => Self {
                assign: false,
                fnname: quote::format_ident!("or"),
                args: Some((
                    quote::quote! { ::core::primitive::bool },
                    quote::quote! { ::core::primitive::bool },
                )),
                retty: None,
            },
            BinOp::Eq => Self {
                assign: false,
                fnname: quote::format_ident!("eq"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
            BinOp::Ne => Self {
                assign: false,
                fnname: quote::format_ident!("ne"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
            BinOp::Lt => Self {
                assign: false,
                fnname: quote::format_ident!("lt"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
            BinOp::Le => Self {
                assign: false,
                fnname: quote::format_ident!("le"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
            BinOp::Ge => Self {
                assign: false,
                fnname: quote::format_ident!("ge"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
            BinOp::Gt => Self {
                assign: false,
                fnname: quote::format_ident!("gt"),
                args: None,
                retty: Some(quote::quote! { ::core::primitive::bool }),
            },
        }
    }
}

fn maker<F, T>(mut make: F) -> impl FnMut() -> T
where
    F: FnMut(u64) -> T,
{
    let mut count = 0;
    move || {
        let new = make(count);
        count += 1;
        new
    }
}
