type Error = Box<dyn std::error::Error>;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
    }
}

fn run() -> Result<(), Error> {
    use std::{env, path::Path, process::Command};

    let module = env::args().nth(1).ok_or("no module specified")?;
    let root = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root dir not found")?;

    let status = Command::new("wasm-pack")
        .current_dir(root)
        .args([
            "build",
            "examples/wasm",
            "--no-pack",
            "--no-typescript",
            "--target",
            "web",
        ])
        .arg("--out-dir")
        .arg(root.join("xtask/web").join(&module))
        .args(["-F", &module])
        .status()?;

    if !status.success() {
        return Err(Error::from("wasm-pack build failed"));
    }

    serv(&module)
}

fn serv(module: &str) -> Result<(), Error> {
    use {
        askama::Template,
        helpers::serv::{self, Page},
        std::fs,
    };

    let index = {
        #[derive(Template)]
        #[template(path = "index.html")]
        struct Index<'a> {
            module: &'a str,
        }

        Index { module }
    };

    let html = index.render()?.leak();
    let prefix = "xtask/web";
    let strip = |s: &'static str| -> &'static str { s.strip_prefix(prefix).expect("strip") };
    let js_path = format!("{prefix}/{module}/wasm.js").leak();
    let wasm_path = format!("{prefix}/{module}/wasm_bg.wasm").leak();
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
