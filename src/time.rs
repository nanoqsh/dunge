use std::time::Instant;

pub(crate) struct Time {
    last: Instant,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    pub(crate) fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        delta.as_secs_f32()
    }
}
