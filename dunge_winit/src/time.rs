use {instant::Instant, std::time::Duration};

pub(crate) struct Time {
    last: Instant,
    delta: Duration,
}

impl Time {
    pub(crate) fn now() -> Self {
        Self {
            last: Instant::now(),
            delta: Duration::ZERO,
        }
    }

    pub(crate) fn delta(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        self.delta += delta;
        self.delta
    }

    pub(crate) fn reset(&mut self) {
        self.delta = Duration::ZERO;
    }
}
