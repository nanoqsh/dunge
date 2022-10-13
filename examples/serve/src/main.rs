fn main() {
    use {
        std::{env, fs, net::SocketAddr, process::Command},
        tokio::runtime::Builder,
    };

    const PATH: &str = "serve/static";

    assert!(fs::read_dir(PATH).is_ok(), "path not found");

    let mut debug = false;

    let mut args = env::args().skip(1);
    let crate_name = args.next().expect("select a wasm crate");
    for arg in args {
        if arg == "--dev" {
            debug = true
        }
    }

    let out_dir = format!("../{PATH}/pkg");
    let status = Command::new("wasm-pack")
        .args([
            "build",
            "--target",
            "web",
            "--out-dir",
            &out_dir,
            "--out-name",
            "example",
            "--no-typescript",
        ])
        .args(debug.then_some("--dev"))
        .arg(crate_name)
        .status()
        .expect("build wasm");

    if !status.success() {
        eprintln!("error while compiling wasm");
        return;
    }

    let files = warp::fs::dir(PATH);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let serve = warp::serve(files).run(addr);
    println!("served at http://{addr}");

    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
        .block_on(serve)
}
