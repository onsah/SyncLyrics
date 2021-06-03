use futures::{Future, future::{AbortHandle, Abortable}};

pub fn spawn_as_abortable<F: Future + Send + 'static>(fut: F) -> AbortHandle
where
    <F as Future>::Output: Send,
{
    let (abort_handle, abort_registration) = AbortHandle::new_pair();

    tokio::spawn(Abortable::new(fut, abort_registration));

    abort_handle
}