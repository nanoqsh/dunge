use {instant::Instant, std::time::Duration};

pub(crate) struct Time {
    last: Instant,
}

impl Time {
    pub(crate) fn now() -> Self {
        Self {
            last: Instant::now(),
        }
    }

    pub(crate) fn delta(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        delta
    }
}
