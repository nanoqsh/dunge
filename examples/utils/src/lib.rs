use {
    dunge::View,
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

pub struct Camera {
    angle: f32,
    pitch: f32,
    distance: f32,
}

impl Camera {
    pub fn update(&mut self, (x, y, z): (f32, f32, f32)) {
        use std::f32::consts::TAU;

        self.angle -= x % TAU;
        self.pitch = (self.pitch + z).clamp(-1., 1.);
        self.distance = (self.distance - y).clamp(3., 10.);
    }

    pub fn view(&self) -> View {
        let x = self.distance * self.angle.sin() * self.pitch.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.angle.cos() * self.pitch.cos();

        View {
            eye: [x, y, z],
            ..Default::default()
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            angle: 0.,
            pitch: 0.,
            distance: 3.,
        }
    }
}
