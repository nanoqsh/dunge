fn main() {
    let path = format!("{dir}/static/", dir = env!("CARGO_MANIFEST_DIR"));
    serve::run(path);
}
