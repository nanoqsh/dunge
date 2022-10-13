fn main() {
    use {
        std::{env, fs, net::SocketAddr, process::Command},
        tokio::runtime::Builder,
    };

    const PATH: &str = "serve/static";

    assert!(fs::read_dir(PATH).is_ok(), "path not found");

    let mut args = env::args().skip(1);
    let mut crate_name = args.next().expect("select a wasm crate");

    if !crate_name.ends_with("_wasm") {
        crate_name.push_str("_wasm");
    }

    let out_dir = format!("../{PATH}/pkg");
    let status = Command::new("wasm-pack")
        .args([
            "--log-level",
            "warn",
            "build",
            "--mode",
            "no-install",
            "--out-dir",
            &out_dir,
            "--out-name",
            "example",
            "--target",
            "web",
            "--no-typescript",
            "--dev",
        ])
        .arg(crate_name)
        .args(["--target-dir", "target"])
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
