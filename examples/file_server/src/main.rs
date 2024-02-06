fn main() {
    use helpers::serv::{self, Page};

    serv::run(&[
        ("/", Page::html(include_str!("../static/index.html"))),
        ("/favicon.ico", Page::html("")),
        // (
        //     "/triangle.js",
        //     Page::js(include_str!("../static/triangle.js")),
        // ),
        // (
        //     "/triangle_bg.wasm",
        //     Page::wasm(include_bytes!("../static/triangle_bg.wasm")),
        // ),
    ]);
}
