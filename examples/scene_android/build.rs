fn main() {
    println!("cargo:rerun-if-changed=../../dunge");
    println!("cargo:rerun-if-changed=android");

    utils_build::run_cargo_apk();
}
