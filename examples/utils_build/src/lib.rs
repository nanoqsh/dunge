use std::process::{Command, Output};

/// Runs `wasm-pack` and builds the crate.
///
/// # Panics
/// If the build fails, the function panics with error messages.
pub fn run_wasm_pack() {
    use std::env;

    let debug = env::var("DEBUG").is_ok_and(|var| var == "true");
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

    report(&output);
}

/// Runs `cargo apk` to build the crate.
///
/// # Panics
/// If the build fails, the function panics with error messages.
pub fn run_cargo_apk() {
    use std::env;

    let debug = env::var("DEBUG").is_ok_and(|var| var == "true");
    let output = Command::new("cargo")
        .args(["apk", "build", "--manifest-path", "./android/Cargo.toml"])
        .args((!debug).then_some("--release"))
        .output()
        .expect("build apk");

    report(&output);
}

fn report(output: &Output) {
    let code = output.status.code().unwrap_or_default();
    if code != 0 {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        panic!("error while compiling:\nerr: {err}\nout: {out}\ncode: {code}\n");
    }
}
