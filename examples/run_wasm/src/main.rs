type Error = Box<dyn std::error::Error>;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
    }
}

fn run() -> Result<(), Error> {
    use {
        askama::Template,
        helpers::serv::{self, Page},
        std::{env, fs},
    };

    let module = env::args().nth(1).ok_or("no module specified")?;
    let index = |module| {
        #[derive(Template)]
        #[template(path = "index.html")]
        struct Index<'a> {
            module: &'a str,
        }

        Index { module }
    };

    let html = index(&module).render()?.leak();
    let prefix = "examples/run_wasm/static";
    let strip = |s: &'static str| -> &'static str { s.strip_prefix(prefix).expect("strip") };
    let js_path = format!("{prefix}/{module}.js").leak();
    let wasm_path = format!("{prefix}/{module}_bg.wasm").leak();
    let js = fs::read_to_string(&js_path)?.leak();
    let wasm = fs::read(&wasm_path)?.leak();
    let routes = &[
        ("/", Page::html(html)),
        ("/favicon.ico", Page::html("")),
        (strip(js_path), Page::js(js)),
        (strip(wasm_path), Page::wasm(wasm)),
    ];

    serv::run(routes);
    Ok(())
}
