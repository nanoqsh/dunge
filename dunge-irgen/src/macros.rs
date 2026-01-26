use {
    crate::{
        derive::{self, Reorder},
        error, func, gener, render, translate,
    },
    proc_macro2::TokenStream,
};

fn root() -> TokenStream {
    quote::quote! { dunge }
}

fn sh() -> TokenStream {
    quote::quote! { dunge::sh }
}

pub fn make_render(item: TokenStream) -> TokenStream {
    let rt = match render::parse(item) {
        Ok(rt) => rt,
        Err(e) => return e.into_compile_error(),
    };

    render::make(&rt, sh())
}

pub fn derive_bytes(item: TokenStream) -> TokenStream {
    let s = match derive::parse(item) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error(),
    };

    match derive::derive_bytes(&s, &sh()) {
        Ok(res) => res,
        Err(e) => error::into_compile_error(e),
    }
}

pub fn derive_value(item: TokenStream) -> TokenStream {
    let s = match derive::parse(item) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error(),
    };

    let sh = sh();
    let value = match derive::derive_value(&s, &sh) {
        Ok(value) => value,
        Err(e) => return error::into_compile_error(e),
    };

    let fields = match derive::derive_fields(&s, Reorder::Yes, &sh) {
        Ok(fields) => fields,
        Err(e) => return error::into_compile_error(e),
    };

    quote::quote! {
        #value
        #fields
    }
}

pub fn derive_input(item: TokenStream) -> TokenStream {
    let s = match derive::parse(item) {
        Ok(s) => s,
        Err(e) => return e.into_compile_error(),
    };

    let sh = sh();
    let input = match derive::derive_input(&s, &sh) {
        Ok(value) => value,
        Err(e) => return error::into_compile_error(e),
    };

    let fields = match derive::derive_fields(&s, Reorder::No, &sh) {
        Ok(fields) => fields,
        Err(e) => return error::into_compile_error(e),
    };

    let group = match derive::derive_group(&s, &root()) {
        Ok(group) => group,
        Err(e) => return error::into_compile_error(e),
    };

    quote::quote! {
        #input
        #fields
        #group
    }
}

pub fn shader(attr: TokenStream, code: TokenStream) -> TokenStream {
    let stage = match func::parse_attr(attr) {
        Ok(stage) => stage,
        Err(e) => return e.into_compile_error(),
    };

    let mut events = vec![];
    if let Err(e) = func::parse(code.clone(), |event| events.push(event)) {
        return e.into_compile_error();
    }

    let events = translate::translate(events);
    match gener::produce(events, stage, &sh()) {
        Ok(res) => quote::quote! {
            #res

            #[allow(unused)]
            #code
        },
        Err(e) => error::into_compile_error(e),
    }
}

#[cfg(debug_assertions)]
pub fn debug(code: TokenStream) -> Result<impl Iterator<Item = String>, TokenStream> {
    let mut events = vec![];
    func::parse(code, |event| events.push(event)).map_err(|e| e.into_compile_error())?;

    Ok(translate::translate(events)
        .into_iter()
        .map(|event| event.debug().to_string()))
}
