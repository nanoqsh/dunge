fn main() {
    use helpers::serv::{self, Page};

    serv::run(&[("/", Page::html(include_str!("../static/index.html")))]);
}
