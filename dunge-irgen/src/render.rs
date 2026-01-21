use {
    proc_macro2::{Ident, TokenStream},
    syn::parse::{Parse, ParseStream},
};

pub(crate) fn parse(code: TokenStream) -> syn::Result<RenderTypes> {
    syn::parse2(code)
}

pub(crate) fn make(render: &RenderTypes, path: TokenStream) -> TokenStream {
    let irc = quote::quote! { #path::irc };
    let link = quote::quote! { #path::link };

    let vertex = render
        .vertex
        .as_ref()
        .map(|ty| quote::quote! { #ty })
        .unwrap_or_else(|| quote::quote! { () });

    let instance = render
        .instance
        .as_ref()
        .map(|ty| quote::quote! { #ty })
        .unwrap_or_else(|| quote::quote! { () });

    let groups = render.groups.iter().map(|ty| quote::quote! { #ty });

    let with_vertex = render
        .vertex
        .as_ref()
        .map(|ty| quote::quote! { .with_vertex::<#ty>() })
        .unwrap_or_default();

    let with_instance = render
        .instance
        .as_ref()
        .map(|ty| quote::quote! { .with_instance::<#ty>() })
        .unwrap_or_default();

    let with_groups = render
        .groups
        .iter()
        .map(|ty| quote::quote! { .with_group::<#ty>() });

    let fns = render
        .shaders
        .iter()
        .map(|s| quote::quote! { #link::func::<#s::Sign>(#s::STAGE, #s::build) });

    quote::quote! {
        {
            || -> #irc::Comp<#link::Render<#link::RenderInput<#vertex, #instance>, (#(#groups,)*)>> {
                let render = const {
                    #link::render()
                        #with_vertex
                        #with_instance
                        #(#with_groups)*
                };

                let fns = const {
                    [#(#fns,)*]
                };

                let make = #link::type_check(render, &fns);
                let module = make()?;
                ::std::result::Result::Ok(#link::Render::new(module))
            }()
        }
    }
}

pub(crate) struct RenderTypes {
    vertex: Option<syn::Type>,
    instance: Option<syn::Type>,
    groups: Vec<syn::Type>,
    shaders: Vec<syn::Path>,
}

impl Parse for RenderTypes {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut vertex = None;
        let mut instance = None;
        let mut groups = vec![];
        let mut shaders = vec![];

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            let _: syn::Token![:] = input.parse()?;

            let err = |s| Err(syn::Error::new(key.span(), s));
            match key.to_string().as_str() {
                "vertex" => {
                    if vertex.is_some() {
                        return err("duplicated vertex");
                    }

                    vertex = Some(input.parse()?);
                }
                "instance" => {
                    if instance.is_some() {
                        return err("duplicated instance");
                    }

                    instance = Some(input.parse()?);
                }
                "groups" => {
                    if !groups.is_empty() {
                        return err("duplicated groups");
                    }

                    let inner;
                    syn::bracketed!(inner in input);
                    let list = inner.parse_terminated(syn::Type::parse, syn::Token![,])?;
                    groups = list.into_iter().collect();
                }
                "shaders" => {
                    if !shaders.is_empty() {
                        return err("duplicated shaders");
                    }

                    let inner;
                    syn::bracketed!(inner in input);
                    let list = inner.parse_terminated(syn::Path::parse, syn::Token![,])?;
                    shaders = list.into_iter().collect();
                }
                _ => return err("unknown key"),
            }

            let _: syn::Token![,] = input.parse()?;
        }

        Ok(Self {
            vertex,
            instance,
            groups,
            shaders,
        })
    }
}
