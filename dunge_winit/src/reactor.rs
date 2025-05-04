use {
    futures_core::Stream,
    std::{
        collections::BTreeMap,
        mem,
        pin::Pin,
        sync::{
            LazyLock, Mutex,
            atomic::{AtomicU64, Ordering},
        },
        task::{Context, Poll, Waker},
        time::Duration,
    },
};

#[cfg(target_family = "wasm")]
use web_time::Instant;

#[cfg(not(target_family = "wasm"))]
use std::time::Instant;

pub(crate) struct Reactor {
    timers: Mutex<BTreeMap<(Instant, u64), Waker>>,
}

impl Reactor {
    #[inline]
    pub(crate) fn get() -> &'static Self {
        static REACTOR: LazyLock<Reactor> = LazyLock::new(|| Reactor {
            timers: Mutex::new(BTreeMap::new()),
        });

        &REACTOR
    }

    #[inline]
    fn insert_timer(&self, when: Instant, waker: Waker) -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        {
            let mut timers = self.timers.lock().expect("lock timers");
            timers.insert((when, id), waker);
        }

        id
    }

    #[inline]
    fn update_timer(&self, when: Instant, id: u64, new: &Waker) {
        let mut timers = self.timers.lock().expect("lock timers");
        if let Some(waker) = timers.get_mut(&(when, id)) {
            *waker = new.clone();
        }
    }

    #[inline]
    fn remove_timer(&self, when: Instant, id: u64) {
        let mut timers = self.timers.lock().expect("lock timers");
        timers.remove(&(when, id));
    }

    pub(crate) fn process_timers(&self) -> Process {
        let (ready, out) = {
            let mut timers = self.timers.lock().expect("lock timers");
            let now = Instant::now();
            let pending = timers.split_off(&(now + Duration::from_nanos(1), 0));
            let ready = mem::replace(&mut *timers, pending);

            let out = if ready.is_empty() {
                timers
                    .keys()
                    .next()
                    .map_or(Process::Sleep, |(when, _)| Process::Wait(*when))
            } else {
                Process::Ready
            };

            (ready, out)
        };

        for (_, waker) in ready {
            waker.wake();
        }

        out
    }
}

pub(crate) enum Process {
    Ready,
    Wait(Instant),
    Sleep,
}

struct Record {
    id: u64,
    waker: Waker,
}

pub struct Timer {
    when: Instant,
    period: Duration,
    record: Option<Record>,
}

impl Timer {
    #[inline]
    pub fn after(duration: Duration) -> Self {
        Self::at(Instant::now() + duration)
    }

    #[inline]
    pub fn at(instant: Instant) -> Self {
        Self::interval_at(instant, Duration::MAX)
    }

    #[inline]
    pub fn interval(period: Duration) -> Self {
        Self::interval_at(Instant::now() + period, period)
    }

    #[inline]
    pub fn interval_at(when: Instant, period: Duration) -> Self {
        Self {
            when,
            period,
            record: None,
        }
    }

    #[inline]
    fn register(&mut self, waker: Waker) {
        let id = Reactor::get().insert_timer(self.when, waker.clone());
        self.record = Some(Record { id, waker });
    }

    #[inline]
    fn update_waker(&mut self, new: &Waker) {
        match &mut self.record {
            Some(Record { waker, .. }) if waker.will_wake(new) => {}
            Some(Record { id, waker }) => {
                Reactor::get().update_timer(self.when, *id, new);
                *waker = new.clone();
            }
            None => self.register(new.clone()),
        }
    }

    #[inline]
    fn deregister(&mut self) {
        if let (when, Some(Record { id, .. })) = (self.when, self.record.take()) {
            Reactor::get().remove_timer(when, id);
        }
    }
}

impl Drop for Timer {
    #[inline]
    fn drop(&mut self) {
        self.deregister();
    }
}

impl Future for Timer {
    type Output = Instant;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_next(cx) {
            Poll::Ready(Some(when)) => Poll::Ready(when),
            Poll::Ready(None) => unreachable!(),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Stream for Timer {
    type Item = Instant;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();
        if Instant::now() < me.when {
            me.update_waker(cx.waker());
            return Poll::Pending;
        }

        me.deregister();
        let result = me.when;
        if me.period != Duration::MAX {
            me.when += me.period;
            me.register(cx.waker().clone());
        }

        Poll::Ready(Some(result))
    }
}

pub trait DurationTimerExt {
    fn after(self) -> Timer;
    fn interval(self) -> Timer;
}

impl DurationTimerExt for Duration {
    #[inline]
    fn after(self) -> Timer {
        Timer::after(self)
    }

    #[inline]
    fn interval(self) -> Timer {
        Timer::interval(self)
    }
}

pub trait InstantTimerExt {
    fn at(self) -> Timer;
    fn interval_at(self, period: Duration) -> Timer;
}

impl InstantTimerExt for Instant {
    #[inline]
    fn at(self) -> Timer {
        Timer::at(self)
    }

    #[inline]
    fn interval_at(self, period: Duration) -> Timer {
        Timer::interval_at(self, period)
    }
}
