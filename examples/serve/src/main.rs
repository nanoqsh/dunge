fn main() {
    use {
        std::{env, fs, net::SocketAddr, process::Command},
        tokio::runtime::Builder,
    };

    const PATH: &str = "serve/static";

    assert!(fs::read_dir(PATH).is_ok(), "path not found");

    let mut debug = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--dev" => debug = true,
            _ => {}
        }
    }

    let out_dir = format!("../{PATH}/pkg");
    let crate_path = "cube_wasm";
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
        .args(debug.then(|| "--dev"))
        .arg(crate_path)
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
