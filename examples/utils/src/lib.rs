use {
    image::{io::Reader, ImageFormat, RgbaImage},
    std::io::Cursor,
};

#[allow(dead_code)]
fn main() {}

pub fn read_png(bytes: &[u8]) -> RgbaImage {
    Reader::with_format(Cursor::new(bytes), ImageFormat::Png)
        .decode()
        .expect("decode png")
        .to_rgba8()
}
