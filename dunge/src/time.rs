use instant::Instant;

pub(crate) struct Time {
    last: Instant,
    delta: f32,
}

impl Time {
    pub fn now() -> Self {
        Self {
            last: Instant::now(),
            delta: 0.,
        }
    }

    pub fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        self.delta += delta.as_secs_f32();
        self.delta
    }

    pub fn reset(&mut self) {
        self.delta = 0.;
    }
}

#[derive(Default)]
pub(crate) struct Fps {
    timer: f32,
    counter: u32,
}

impl Fps {
    pub fn count(&mut self, delta_time: f32) -> Option<u32> {
        self.timer += delta_time;
        self.counter += 1;
        (self.timer > 1.).then(|| {
            self.timer = 0.;
            let n = self.counter;
            self.counter = 0;
            n
        })
    }
}
