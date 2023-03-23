#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use self::instant::Instant;

pub(crate) struct Time {
    last: Instant,
    delta_time: f32,
}

impl Time {
    pub fn new() -> Self {
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

#[cfg(target_arch = "wasm32")]
mod instant {
    #[derive(Clone, Copy)]
    pub(crate) struct Instant(f32);

    impl Instant {
        pub fn now() -> Self {
            use web_sys::Window;

            let performance = web_sys::window()
                .as_ref()
                .and_then(Window::performance)
                .expect("get performance");

            Self(performance.now() as f32)
        }

        pub fn duration_since(&self, Self(earlier): Self) -> Duration {
            Duration((self.0 - earlier).max(0.))
        }
    }

    pub(crate) struct Duration(f32);

    impl Duration {
        pub fn as_secs_f32(&self) -> f32 {
            self.0 / 1000.
        }
    }
}
