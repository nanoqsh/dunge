use image::{codecs::png::PngDecoder, DynamicImage, GrayImage, RgbaImage};

/// Decodes the rgba png image from bytes.
pub fn decode_rgba_png(data: &[u8]) -> RgbaImage {
    let decoder = PngDecoder::new(data).expect("create decoder");
    DynamicImage::from_decoder(decoder)
        .expect("decode png image")
        .into_rgba8()
}

/// Decodes the gray png image from bytes.
pub fn decode_gray_png(data: &[u8]) -> GrayImage {
    let decoder = PngDecoder::new(data).expect("create decoder");
    DynamicImage::from_decoder(decoder)
        .expect("decode png image")
        .into_luma8()
}

pub fn create_image(width: u32, height: u32, data: Vec<u8>) -> RgbaImage {
    RgbaImage::from_vec(width, height, data).expect("create an image")
}
