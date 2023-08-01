use instant::Instant;

pub struct Time {
    last: Instant,
    delta_time: f32,
}

impl Time {
    pub fn now() -> Self {
        Self {
            last: Instant::now(),
            delta_time: 0.,
        }
    }

    pub fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        self.delta_time += delta.as_secs_f32();
        self.delta_time
    }

    pub fn reset(&mut self) {
        self.delta_time = 0.;
    }
}
