fn main() {
    println!("cargo:rerun-if-changed=android/src");
    utils_build::run_cargo_apk();
}
