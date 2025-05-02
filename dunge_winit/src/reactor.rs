use {
    futures_core::Stream,
    instant::Instant,
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

pub(crate) struct Reactor {
    timers: Mutex<BTreeMap<(Instant, u64), Waker>>,
}

impl Reactor {
    pub(crate) fn get() -> &'static Self {
        static REACTOR: LazyLock<Reactor> = LazyLock::new(|| Reactor {
            timers: Mutex::new(BTreeMap::new()),
        });

        &REACTOR
    }

    fn insert_timer(&self, when: Instant, waker: Waker) -> u64 {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);

        {
            let mut timers = self.timers.lock().expect("lock timers");
            timers.insert((when, id), waker);
        }

        id
    }

    fn update_timer(&self, when: Instant, id: u64, new: &Waker) {
        let mut timers = self.timers.lock().expect("lock timers");
        if let Some(waker) = timers.get_mut(&(when, id)) {
            *waker = new.clone();
        }
    }

    fn remove_timer(&self, when: Instant, id: u64) {
        let mut timers = self.timers.lock().expect("lock timers");
        timers.remove(&(when, id));
    }

    pub(crate) fn process_timers(&self) -> Option<Instant> {
        let (ready, next) = {
            let mut timers = self.timers.lock().expect("lock timers");
            let now = Instant::now();
            let pending = timers.split_off(&(now + Duration::from_nanos(1), 0));
            let ready = mem::replace(&mut *timers, pending);

            let next = if ready.is_empty() {
                timers.keys().next().map(|(when, _)| *when)
            } else {
                Some(now)
            };

            (ready, next)
        };

        for (_, waker) in ready {
            waker.wake();
        }

        next
    }
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
    pub fn after(duration: Duration) -> Self {
        Self::at(Instant::now() + duration)
    }

    pub fn at(instant: Instant) -> Self {
        Self::interval_at(instant, Duration::MAX)
    }

    pub fn interval(period: Duration) -> Self {
        Self::interval_at(Instant::now() + period, period)
    }

    pub fn interval_at(when: Instant, period: Duration) -> Self {
        Self {
            when,
            period,
            record: None,
        }
    }

    fn register(&mut self, waker: Waker) {
        let id = Reactor::get().insert_timer(self.when, waker.clone());
        self.record = Some(Record { id, waker });
    }

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

    fn deregister(&mut self) {
        if let (when, Some(Record { id, .. })) = (self.when, self.record.take()) {
            Reactor::get().remove_timer(when, id);
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.deregister();
    }
}

impl Future for Timer {
    type Output = Instant;

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

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();
        if Instant::now() < me.when {
            me.update_waker(cx.waker());
            return Poll::Pending;
        }

        me.deregister();
        let result = me.when;
        me.when += me.period;
        me.register(cx.waker().clone());
        Poll::Ready(Some(result))
    }
}
