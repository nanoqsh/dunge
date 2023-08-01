use dunge::{Projection, View};

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
    pub fn view<P>(&self, proj: P) -> View
    where
        P: Into<Projection>,
    {
        let x = self.distance * self.angle.sin() * self.pitch.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.angle.cos() * self.pitch.cos();

        View {
            eye: [x, y, z].into(),
            proj: proj.into(),
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
