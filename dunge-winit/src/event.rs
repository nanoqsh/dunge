use std::{
    cell::{Cell, RefCell},
    fmt, ops,
    task::{self, Poll, Waker},
};

pub(crate) struct Event<T> {
    inner: Cell<T>,
    waker: RefCell<Waker>,
}

impl<T> Event<T> {
    pub(crate) fn new() -> Self
    where
        T: Default,
    {
        Self {
            inner: Cell::new(T::default()),
            waker: RefCell::new(Waker::noop().clone()),
        }
    }
}

impl Event<bool> {
    pub(crate) fn set_flag(&self) {
        self.inner.set(true);
        self.waker.borrow().wake_by_ref();
    }

    pub(crate) fn poll_flag(&self, cx: &mut task::Context<'_>) -> Poll<()> {
        if self.inner.take() {
            Poll::Ready(())
        } else {
            self.waker.borrow_mut().clone_from(cx.waker());
            Poll::Pending
        }
    }
}

impl<T> Event<Option<T>> {
    fn set_value(&self, value: T) {
        self.inner.set(Some(value));
        self.waker.borrow().wake_by_ref();
    }

    pub(crate) fn add_value(&self, value: T)
    where
        T: Copy + ops::AddAssign,
    {
        match self.inner.get() {
            Some(mut curr) => {
                curr += value;
                self.set_value(curr);
            }
            None => self.set_value(value),
        }
    }

    pub(crate) fn poll_value(&self, cx: &mut task::Context<'_>) -> Poll<T> {
        if let Some(value) = self.inner.take() {
            Poll::Ready(value)
        } else {
            self.waker.borrow_mut().clone_from(cx.waker());
            Poll::Pending
        }
    }
}

impl<T> fmt::Debug for Event<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event").finish()
    }
}
