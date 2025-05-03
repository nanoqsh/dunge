use {std::time::Duration, web_time::Instant};

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

#[derive(Default)]
pub(crate) struct Fps {
    timer: Duration,
    counter: u32,
}

impl Fps {
    pub(crate) fn count(&mut self, delta_time: Duration) -> Option<u32> {
        self.timer += delta_time;
        self.counter += 1;
        if self.timer > const { Duration::from_secs(1) } {
            self.timer = Duration::ZERO;
            let n = self.counter;
            self.counter = 0;
            Some(n)
        } else {
            None
        }
    }
}
