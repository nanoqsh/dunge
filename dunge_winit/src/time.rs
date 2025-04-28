use {instant::Instant, std::time::Duration};

pub(crate) struct Time {
    last: Instant,
    delta: Duration,
}

impl Time {
    pub fn now() -> Self {
        Self {
            last: Instant::now(),
            delta: Duration::ZERO,
        }
    }

    pub fn delta(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        self.delta += delta;
        self.delta
    }

    pub fn reset(&mut self) {
        self.delta = Duration::ZERO;
    }
}

pub(crate) struct Fps {
    timer: Duration,
    counter: u32,
}

impl Fps {
    #[expect(dead_code)]
    pub const fn new() -> Self {
        Self {
            timer: Duration::ZERO,
            counter: 0,
        }
    }

    #[expect(dead_code)]
    pub fn count(&mut self, delta_time: Duration) -> Option<u32> {
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
