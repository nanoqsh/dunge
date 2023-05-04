/// Runs `wasm-pack` and builds the crate.
///
/// # Panics
/// If the build fails, the function panics with error messages.
pub fn run_wasm_pack() {
    use std::{env, process::Command};

    let debug = env::var("DEBUG")
        .map(|var| var == "true")
        .unwrap_or_default();

    let output = Command::new("wasm-pack")
        .args([
            "--log-level",
            "warn",
            "build",
            "--mode",
            "no-install",
            "--out-dir",
            "../static/pkg",
            "--out-name",
            "example",
            "--target",
            "web",
            "--no-typescript",
        ])
        .args(debug.then_some("--dev"))
        .arg("wasm")
        .output()
        .expect("build wasm");

    let code = output.status.code().unwrap_or_default();

    if code != 0 {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        panic!("error while compiling wasm:\nerr: {err}\nout: {out}\ncode: {code}\n");
    }
}
