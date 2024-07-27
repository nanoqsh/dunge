use std::{
    env, error,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

type Error = Box<dyn error::Error>;

fn main() -> ExitCode {
    let (opts, mode) = match parse() {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("args error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match start(opts, mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn parse() -> Result<(Opts, Mode), &'static str> {
    let modeerr = "undefined mode";
    let modlerr = "undefined module";

    let mut args = env::args().skip(1);
    let mut no_install = false;
    let mode = loop {
        match args.next().ok_or(modeerr)?.as_str() {
            "build" => break Mode::Build,
            "serve" => break Mode::Serve,
            opt => match opt.strip_prefix("--") {
                Some("no-install") => no_install = true,
                _ => return Err(modeerr),
            },
        }
    };

    let opts = Opts {
        module: args.next().ok_or(modlerr)?,
        no_install,
    };

    Ok((opts, mode))
}

struct Opts {
    module: String,
    no_install: bool,
}

enum Mode {
    Build,
    Serve,
}

fn start(opts: Opts, mode: Mode) -> Result<(), Error> {
    match mode {
        Mode::Build => build(opts),
        Mode::Serve => serve(opts),
    }
}

fn build(opts: Opts) -> Result<(), Error> {
    let root = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root dir not found")?;

    let mut cmd = Command::new("wasm-pack");
    cmd.args([
        "build",
        "examples/wasm",
        "--no-pack",
        "--no-typescript",
        "--target",
        "web",
    ])
    .arg("--out-dir")
    .arg(root.join("xtask/web").join(&opts.module))
    .args(["-F", &opts.module]);

    install_and_run(&mut cmd, opts.no_install)
}

fn install_and_run(cmd: &mut Command, no_install: bool) -> Result<(), Error> {
    add_bin_path(cmd);
    let name = cmd.get_program().to_string_lossy().into_owned();
    match run(cmd, &name) {
        Run::Ok => return Ok(()),
        Run::NotFound if no_install => {}
        Run::NotFound => {
            eprintln!("{name} not found, installing..");
            install(&name)?;
        }
        Run::Failed(s) => return Err(Error::from(s)),
    }

    match run(cmd, &name) {
        Run::Ok => Ok(()),
        Run::NotFound if no_install => Err(Error::from(format!("{name} not found"))),
        Run::NotFound => Err(Error::from(format!("failed to install {name}"))),
        Run::Failed(s) => Err(Error::from(s)),
    }
}

fn add_bin_path(cmd: &mut Command) {
    let bin_path = PathBuf::from("./bin");
    let mut paths: Vec<_> = env::var_os("PATH")
        .map(|path| env::split_paths(&path).collect())
        .unwrap_or_default();

    if !paths.contains(&bin_path) {
        paths.push(bin_path);
    }

    let paths = env::join_paths(paths).expect("join paths");
    cmd.env("PATH", paths);
}

enum Run {
    Ok,
    NotFound,
    Failed(String),
}

fn run(cmd: &mut Command, name: &str) -> Run {
    match cmd.status() {
        Ok(status) if status.success() => Run::Ok,
        Ok(_) => Run::Failed(format!("execution of {name} failed")),
        Err(err) if err.kind() == ErrorKind::NotFound => Run::NotFound,
        Err(err) => Run::Failed(format!("failed to run {name}: {err}")),
    }
}

fn install(name: &str) -> Result<(), Error> {
    let status = Command::new("cargo")
        .args([
            "install",
            "--root",
            ".",
            "--target-dir",
            "target",
            "--locked",
        ])
        .arg(name)
        .status()
        .map_err(|err| format!("failed to run cargo: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::from(format!("failed to install {name}")))
    }
}

fn serve(Opts { module, .. }: Opts) -> Result<(), Error> {
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

        Index { module: &module }
    };

    let html = index.render()?.leak();
    let prefix = "xtask/web";
    let strip = |s: &'static str| -> &'static str { s.strip_prefix(prefix).expect("strip") };
    let js_path: &str = format!("{prefix}/{module}/wasm.js").leak();
    let wasm_path: &str = format!("{prefix}/{module}/wasm_bg.wasm").leak();
    let js = fs::read_to_string(js_path)?.leak();
    let wasm = fs::read(wasm_path)?.leak();
    let routes = &[
        ("/", Page::html(html)),
        ("/favicon.ico", Page::html("")),
        (strip(js_path), Page::js(js)),
        (strip(wasm_path), Page::wasm(wasm)),
    ];

    serv::run(routes);
    Ok(())
}
