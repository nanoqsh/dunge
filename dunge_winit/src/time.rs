use std::time::Duration;

#[cfg(target_family = "wasm")]
use web_time::Instant;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

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
