fn main() {
    println!("cargo:rerun-if-changed=wasm/src");
    utils_build::run_wasm_pack();
}
