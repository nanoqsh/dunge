use std::{future::Future, pin::Pin};

type Sender<T> = Box<dyn FnOnce(T) + Send>;
type Receiver<T> = Pin<Box<dyn Future<Output = T>>>;

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>)
where
    T: Send + 'static,
{
    let (tx, rx) = async_channel::bounded(1);
    (
        Box::new(move |r| tx.send_blocking(r).expect("send mapped result")),
        Box::pin(async move { rx.recv().await.expect("recv mapped result") }),
    )
}
