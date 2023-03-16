use {
    dunge::View,
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

pub struct Camera {
    angle: f32,
    pitch: f32,
    distance: f32,
}

impl Camera {
    pub fn update(&mut self, (x, y, z): (f32, f32, f32)) {
        use std::f32::consts::TAU;

        self.angle = (self.angle + x) % TAU;
        self.pitch = (self.pitch + z).clamp(-1.5, 1.5);
        self.distance = (self.distance - y).clamp(3., 10.);
    }

    #[must_use]
    pub fn view<P>(&self, proj: P) -> View<P> {
        let x = self.distance * self.angle.sin() * self.pitch.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.angle.cos() * self.pitch.cos();

        View {
            eye: [x, y, z],
            look: [0.; 3],
            proj,
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
