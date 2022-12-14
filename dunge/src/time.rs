#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use self::instant::Instant;

pub(crate) struct Time {
    last: Instant,
    delta_time: f64,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            last: Instant::now(),
            delta_time: 0.,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last);
        self.last = now;
        self.delta_time = delta.as_secs_f64();
        self.delta_time as _
    }

    pub(crate) fn reset(&mut self) {
        self.delta_time = 0.;
    }
}

#[cfg(target_arch = "wasm32")]
mod instant {
    #[derive(Clone, Copy)]
    pub(crate) struct Instant(f64);

    impl Instant {
        pub(crate) fn now() -> Self {
            use web_sys::Window;

            let performance = web_sys::window()
                .as_ref()
                .and_then(Window::performance)
                .expect("get performance");

            Self(performance.now())
        }

        pub(crate) fn duration_since(&self, Self(earlier): Self) -> Duration {
            Duration((self.0 - earlier).max(0.))
        }
    }

    pub(crate) struct Duration(f64);

    impl Duration {
        pub(crate) fn as_secs_f64(&self) -> f64 {
            self.0 / 1000.
        }
    }
}
