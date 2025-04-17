#[cfg(not(target_family = "wasm"))]
use {
    parking::Parker,
    std::{
        cell::RefCell,
        pin,
        sync::mpsc,
        task::{Context, Poll, Waker},
        thread,
    },
};

/// Blocks on a future until it's completed.
///
/// # Examples
///
/// ```
/// # #[cfg(not(target_family = "wasm"))]
/// let out = dunge::block_on(async {
///     let x = async { 1 }.await;
///     let y = async { 2 }.await;
///     x + y
/// });
///
/// assert_eq!(out, 3);
/// ```
///
#[cfg(not(target_family = "wasm"))]
pub fn block_on<F>(f: F) -> F::Output
where
    F: IntoFuture,
{
    let mut fu = pin::pin!(f.into_future());

    fn make() -> (Parker, Waker) {
        let (p, u) = parking::pair();
        (p, Waker::from(u))
    }

    thread_local! {
        static CACHE: RefCell<(Parker, Waker)> = RefCell::new(make());
    }

    CACHE.with(|cache| {
        let borrow = cache.try_borrow_mut();
        let (p, waker) = if let Ok(cache) = &borrow {
            cache
        } else {
            &make()
        };

        let cx = &mut Context::from_waker(waker);

        loop {
            match fu.as_mut().poll(cx) {
                Poll::Ready(out) => break out,
                Poll::Pending => p.park(),
            }
        }
    })
}

pub(crate) fn poll_in_background(instance: wgpu::Instance) -> Worker {
    let (s, r) = pair();

    #[cfg(target_family = "wasm")]
    {
        _ = instance;
        _ = r;
    }

    #[cfg(not(target_family = "wasm"))]
    thread::spawn(move || {
        while r.recv() {
            instance.poll_all(true);
        }
    });

    Worker(s)
}

pub(crate) struct Worker(Sender);

impl Worker {
    pub fn work(&self) {
        self.0.send();
    }
}

fn pair() -> (Sender, Receiver) {
    #[cfg(target_family = "wasm")]
    {
        (Sender(()), Receiver(()))
    }

    #[cfg(not(target_family = "wasm"))]
    {
        let (s, r) = mpsc::channel();
        (Sender(s), Receiver(r))
    }
}

struct Sender(
    #[cfg(target_family = "wasm")] (),
    #[cfg(not(target_family = "wasm"))] mpsc::Sender<()>,
);

impl Sender {
    fn send(&self) {
        #[cfg(not(target_family = "wasm"))]
        {
            _ = self.0.send(());
        }
    }
}

struct Receiver(
    #[cfg(target_family = "wasm")] (),
    #[cfg(not(target_family = "wasm"))] mpsc::Receiver<()>,
);

impl Receiver {
    #[cfg(not(target_family = "wasm"))]
    fn recv(&self) -> bool {
        self.0.recv().is_ok()
    }
}
