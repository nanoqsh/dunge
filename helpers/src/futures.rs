use std::future::IntoFuture;

pub fn block_on<F>(future: F) -> F::Output
where
    F: IntoFuture,
{
    futures_lite::future::block_on(future.into_future())
}
