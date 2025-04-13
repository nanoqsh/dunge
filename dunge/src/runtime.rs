use std::{sync::mpsc, thread};

pub(crate) fn poll_in_background(instance: wgpu::Instance) -> Worker {
    let (s, r) = pair();

    thread::spawn(move || loop {
        instance.poll_all(true);
        r.recv();
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
    #[cfg(not(target_family = "wasm"))]
    {
        let (s, r) = mpsc::channel();
        (Sender(s), Receiver(r))
    }

    #[cfg(target_family = "wasm")]
    {
        (Sender(()), Receiver(()))
    }
}

#[cfg(not(target_family = "wasm"))]
struct Sender(mpsc::Sender<()>);

#[cfg(target_family = "wasm")]
struct Sender(());

impl Sender {
    fn send(&self) {
        #[cfg(not(target_family = "wasm"))]
        {
            _ = self.0.send(());
        }
    }
}

#[cfg(not(target_family = "wasm"))]
struct Receiver(mpsc::Receiver<()>);

#[cfg(target_family = "wasm")]
struct Receiver(());

impl Receiver {
    fn recv(&self) {
        #[cfg(not(target_family = "wasm"))]
        {
            _ = self.0.recv();
        }
    }
}
