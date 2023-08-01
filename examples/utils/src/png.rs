use {
    image::{io::Reader, ImageFormat, RgbaImage},
    std::io::Cursor,
};

#[must_use]
pub fn read_png(bytes: &[u8]) -> RgbaImage {
    Reader::with_format(Cursor::new(bytes), ImageFormat::Png)
        .decode()
        .expect("decode png")
        .to_rgba8()
}

#[must_use]
pub fn create_image(width: u32, height: u32, data: Vec<u8>) -> RgbaImage {
    RgbaImage::from_vec(width, height, data).expect("create an image")
}
