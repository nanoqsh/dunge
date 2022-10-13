fn main() {
    use std::{env, process::Command};

    println!("cargo:rerun-if-changed=wasm");

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

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        panic!("error while compiling wasm:\n{err}");
    }
}
