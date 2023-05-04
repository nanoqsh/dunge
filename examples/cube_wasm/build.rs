fn main() {
    println!("cargo:rerun-if-changed=../../dunge");
    println!("cargo:rerun-if-changed=wasm");

    utils_build::run_wasm_pack();
}
