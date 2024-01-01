pub struct Image {
    pub data: Box<[u8]>,
    pub size: (u32, u32),
}

pub fn decode_png(bytes: &[u8]) -> Image {
    use png::Decoder;

    let decoder = Decoder::new(bytes);
    let mut reader = decoder.read_info().expect("png reader");
    let mut data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut data).expect("read image");
    Image {
        data: data.into(),
        size: (info.width, info.height),
    }
}

pub fn encode_png(image: &Image) -> Vec<u8> {
    use png::{BitDepth, ColorType, Encoder};

    let mut data = vec![];
    let mut encoder = {
        let (width, height) = image.size;
        Encoder::new(&mut data, width, height)
    };

    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    let mut writer = encoder.write_header().expect("write header");
    writer.write_image_data(&image.data).expect("write image");
    writer.finish().expect("write image");
    data
}
