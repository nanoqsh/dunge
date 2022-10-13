pub fn run(path: String) {
    use {std::net::SocketAddr, tokio::runtime::Builder};

    let files = warp::fs::dir(path);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let serve = warp::serve(files).run(addr);
    println!("served at http://{addr}");

    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
        .block_on(serve)
}
