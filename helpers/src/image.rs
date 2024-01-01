pub struct Image {
    pub data: Box<[u8]>,
    pub size: (u32, u32),
}

impl Image {
    const PIXEL_SIZE: usize = 4;

    pub fn from_fn<F>(size: (u32, u32), f: F) -> Self
    where
        F: Fn(u32, u32) -> [u8; Self::PIXEL_SIZE],
    {
        let (width, height) = size;
        let mut data = Box::from(vec![0; width as usize * height as usize * Self::PIXEL_SIZE]);
        let slice = bytemuck::cast_slice_mut(&mut data);
        for y in 0..height {
            for x in 0..width {
                let idx = x + y * width;
                slice[idx as usize] = f(x, y);
            }
        }

        Self { data, size }
    }

    pub fn decode(bytes: &[u8]) -> Self {
        use png::Decoder;

        let decoder = Decoder::new(bytes);
        let mut reader = decoder.read_info().expect("png reader");
        let mut data = Box::from(vec![0; reader.output_buffer_size()]);
        let info = reader.next_frame(&mut data).expect("read image");
        Self {
            data,
            size: (info.width, info.height),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        use png::{BitDepth, ColorType, Encoder};

        let mut data = vec![];
        let mut encoder = {
            let (width, height) = self.size;
            Encoder::new(&mut data, width, height)
        };

        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder.write_header().expect("write header");
        writer.write_image_data(&self.data).expect("write image");
        writer.finish().expect("write image");
        data
    }
}
