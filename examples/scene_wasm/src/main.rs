fn main() {
    let path = format!("{}/static/", env!("CARGO_MANIFEST_DIR"));
    serve::run(path);
}
