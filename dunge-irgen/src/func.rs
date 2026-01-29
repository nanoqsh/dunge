use {
    crate::{error::*, event::*, gener::Stage},
    proc_macro2::TokenStream,
    quote::ToTokens,
    syn::spanned::Spanned,
};

pub(crate) enum Control {
    While,
    IfElse,
}

pub(crate) fn parse_attr(stream: TokenStream) -> Parse<Stage> {
    if stream.is_empty() {
        return Ok(Stage::Regular);
    }

    match stream.to_string().as_str() {
        "vertex" => Ok(Stage::Vertex),
        "fragment" => Ok(Stage::Fragment),
        _ => Err(Error::UnknownAttribute(stream.span())),
    }
}

pub(crate) fn parse<F>(stream: TokenStream, mut send: F) -> Parse<()>
where
    F: FnMut(Event<Control>),
{
    let func: syn::ItemFn = syn::parse2(stream).map_err(Error::Syn)?;

    if let Some(unsupported) = [
        func.sig.asyncness.map(|t| t.span()),
        func.sig.unsafety.map(|t| t.span()),
        func.sig.abi.map(|t| t.span()),
        func.sig.variadic.map(|t| t.span()),
    ]
    .into_iter()
    .flatten()
    .next()
    {
        unsupported.err()?;
    }

    if !func.sig.generics.params.is_empty() {
        func.sig.generics.params.span().err()?;
    }

    let inputs = func
        .sig
        .inputs
        .into_iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(receiver) => match receiver.span().err()? {},
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                let pat_span = pat.span();
                let (ident, _, mutable) = read_pat(*pat)?;
                if mutable {
                    pat_span.err()?;
                }

                Ok(Input {
                    ident,
                    ty: ty.into_token_stream(),
                })
            }
        })
        .collect::<Result<_, _>>()?;

    let output = Output {
        ty: match func.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(ty.into_token_stream()),
        },
    };

    send(Event::Fn(Box::new(Fn {
        vis: func.vis.into_token_stream(),
        ident: func.sig.ident,
        inputs,
        output,
    })));

    let mut stack = Stack::new();
    stack.block(func.block);

    while let Some(item) = stack.pop() {
        match item {
            Item::Block(block) => {
                send(Event::BlockStart);
                stack.event(Event::BlockEnd);

                for s in block.stmts.into_iter().rev() {
                    match s {
                        syn::Stmt::Local(syn::Local { pat, init, .. }) => {
                            let (ident, ty, _) = read_pat(pat)?;
                            stack.event(Event::Semi);

                            if let Some(syn::LocalInit { expr, .. }) = init {
                                stack.event(Event::Assign);
                                stack.expr(expr);
                            }

                            stack.event(Event::Local(Box::new(Local { ident, ty })));
                        }
                        syn::Stmt::Item(item) => match item.span().err()? {},
                        syn::Stmt::Expr(expr, semi) => {
                            if semi.is_some() {
                                stack.event(Event::Semi);
                            }

                            stack.expr(expr);
                        }
                        syn::Stmt::Macro(mac) => match mac.span().err()? {},
                    }
                }
            }
            Item::Expr(expr) => match *expr {
                syn::Expr::Array(syn::ExprArray { elems, .. }) => {
                    stack.event(Event::Array { len: elems.len() });
                    for elem in elems {
                        stack.expr(elem);
                    }
                }
                syn::Expr::Assign(syn::ExprAssign { left, right, .. }) => {
                    stack.event(Event::Assign);
                    stack.expr(right);
                    stack.expr(left);
                }
                syn::Expr::Binary(syn::ExprBinary {
                    left, op, right, ..
                }) => {
                    let op_span = op.span();
                    let Some(binop) = BinOp::from_syn(op) else {
                        match op_span.err()? {}
                    };

                    stack.event(Event::BinOp(binop));
                    stack.expr(right);
                    stack.expr(left);
                }
                syn::Expr::Block(syn::ExprBlock { label, block, .. }) => {
                    if let Some(label) = label {
                        label.span().err()?;
                    }

                    stack.block(block);
                }
                syn::Expr::Call(syn::ExprCall { func, args, .. }) => {
                    let syn::Expr::Path(_) = &*func else {
                        match func.span().err()? {}
                    };

                    let arity = match args.len() {
                        0 => Arity::N0,
                        1 => Arity::N1,
                        2 => Arity::N2,
                        3 => Arity::N3,
                        4 => Arity::N4,
                        _ => match args.span().err()? {},
                    };

                    stack.event(Event::Call(arity));
                    stack.expr(func);

                    for arg in args {
                        stack.expr(arg);
                    }
                }
                syn::Expr::Cast(syn::ExprCast { expr, ty, .. }) => {
                    stack.event(Event::Cast(ty.into_token_stream()));
                    stack.expr(expr);
                }
                syn::Expr::Const(syn::ExprConst { block, .. }) => {
                    stack.event(Event::ConstExpr(block.into_token_stream()));
                }
                syn::Expr::Field(syn::ExprField { base, member, .. }) => match member {
                    syn::Member::Named(ident) => {
                        stack.event(Event::Member(ident));
                        stack.expr(base);
                    }
                    member @ syn::Member::Unnamed(_) => match member.span().err()? {},
                },
                syn::Expr::If(syn::ExprIf {
                    cond,
                    then_branch,
                    else_branch,
                    ..
                }) => {
                    let otherwise = match else_branch.map(|(_, expr)| *expr) {
                        Some(syn::Expr::Block(block)) => Ok(block.block),
                        Some(expr @ syn::Expr::If(_)) => Err(expr),
                        Some(expr) => match expr.span().err()? {},
                        None => Ok(syn::Block {
                            brace_token: syn::token::Brace::default(),
                            stmts: vec![],
                        }),
                    };

                    stack.event(Event::Control(Control::IfElse));

                    match otherwise {
                        Ok(block) => stack.block(block),
                        Err(expr) => {
                            stack.event(Event::BlockEnd);
                            stack.expr(expr);
                            stack.event(Event::BlockStart);
                        }
                    }

                    stack.block(then_branch);
                    stack.expr(cond);
                }
                syn::Expr::Index(syn::ExprIndex { expr, index, .. }) => {
                    stack.event(Event::Index);
                    stack.expr(index);
                    stack.expr(expr);
                }
                syn::Expr::Lit(syn::ExprLit { lit, .. }) => {
                    stack.event(Event::Lit(lit.into_token_stream()));
                }
                syn::Expr::MethodCall(syn::ExprMethodCall {
                    receiver,
                    method,
                    turbofish,
                    args,
                    ..
                }) => {
                    if let Some(turbofish) = turbofish {
                        turbofish.span().err()?;
                    }

                    if !args.is_empty() {
                        args.span().err()?;
                    }

                    stack.event(Event::Method(method));
                    stack.expr(receiver);
                }
                syn::Expr::Paren(syn::ExprParen { expr, .. }) => stack.expr(expr),
                syn::Expr::Path(syn::ExprPath { qself, path, .. }) => {
                    if let Some(qself) = qself {
                        qself.span().err()?;
                    }

                    stack.event(Event::Name(path.into_token_stream()));
                }
                syn::Expr::Return(syn::ExprReturn { expr, .. }) => {
                    stack.event(Event::Return);
                    if let Some(expr) = expr {
                        stack.expr(expr);
                    }
                }
                syn::Expr::Struct(syn::ExprStruct {
                    path, fields, rest, ..
                }) => {
                    if let Some(rest) = rest {
                        rest.span().err()?;
                    }

                    let members = fields
                        .iter()
                        .map(|f| match &f.member {
                            syn::Member::Named(ident) => Ok(ident.clone()),
                            syn::Member::Unnamed(_) => Err(f.span()),
                        })
                        .collect::<Result<_, _>>()
                        .map_err(Error::Unsupported)?;

                    stack.event(Event::Struct(Box::new(Struct {
                        name: path.into_token_stream(),
                        members,
                    })));

                    for field in fields {
                        stack.expr(field.expr);
                    }
                }
                syn::Expr::Unary(syn::ExprUnary { op, expr, .. }) => {
                    let op_span = op.span();
                    let Some(unop) = UnOp::from_syn(op) else {
                        match op_span.err()? {}
                    };

                    stack.event(Event::UnOp(unop));
                    stack.expr(expr);
                }
                syn::Expr::While(syn::ExprWhile {
                    label, cond, body, ..
                }) => {
                    if let Some(label) = label {
                        label.span().err()?;
                    }

                    stack.event(Event::Control(Control::While));
                    stack.block(body);
                    stack.expr(cond);
                }
                expr => match expr.span().err()? {},
            },
            Item::Send(event) => send(event),
        }
    }

    Ok(())
}

pub(crate) fn read_pat(pat: syn::Pat) -> Parse<(syn::Ident, Option<TokenStream>, bool)> {
    match pat {
        syn::Pat::Ident(syn::PatIdent {
            mutability,
            ident,
            subpat,
            ..
        }) => {
            if let Some((_, subpat)) = subpat {
                subpat.span().err()?;
            }

            Ok((ident, None, mutability.is_some()))
        }
        syn::Pat::Type(syn::PatType { pat, ty, .. }) => match *pat {
            syn::Pat::Ident(syn::PatIdent {
                mutability,
                ident,
                subpat,
                ..
            }) => {
                if let Some((_, subpat)) = subpat {
                    subpat.span().err()?;
                }

                Ok((ident, Some(ty.into_token_stream()), mutability.is_some()))
            }
            pat => match pat.span().err()? {},
        },
        pat => match pat.span().err()? {},
    }
}

enum Item {
    Block(Box<syn::Block>),
    Expr(Box<syn::Expr>),
    Send(Event<Control>),
}

struct Stack(Vec<Item>);

impl Stack {
    fn new() -> Self {
        Self(vec![])
    }

    fn block<B>(&mut self, block: B)
    where
        B: Into<Box<syn::Block>>,
    {
        self.0.push(Item::Block(block.into()));
    }

    fn expr<B>(&mut self, expr: B)
    where
        B: Into<Box<syn::Expr>>,
    {
        self.0.push(Item::Expr(expr.into()));
    }

    fn event(&mut self, event: Event<Control>) {
        self.0.push(Item::Send(event));
    }

    fn pop(&mut self) -> Option<Item> {
        self.0.pop()
    }
}
