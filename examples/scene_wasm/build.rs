fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    utils_build::run_wasm_pack();
}
