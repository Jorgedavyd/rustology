use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

// Sender / Receiver should be exclusive to each other
// That's why they're synchronized with Mutex

// #[derive(Clone)]
// This would be handy if the compiler knew that
// we just want to implement the clone trait given
// the features of Arc and not shared.
// If we simply derive the Clone trait, then the
// compiler would interpret that we want Shared<T>
// to be cloned, and that's something we don't want.
// We need concurrent access to the same location.
// Therefore, the Clone trait must be implemented
// manually to self-account of the fact that we just need
// to clone the atomic reference counting (Arc) object and
// not the data itself
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders += 1;
        drop(inner); // gives up the lock
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.senders -= 1;
        if inner.senders == 0 {
            self.shared.available.notify_one();
        }
    }
}

// Interesting case in communication across thread boundaries:
// If you have a lock that is currently running and the current
// thread panics, the lock is released with a flag that indicates
// that the previous thread panicked with the same data, this is
// implemented with the PoisonError<Guard> within Rust, that indicates that
// to you

// This is an implementation of an ASYNC Sender, this means that both the
// Sender and the Receiver don't have to be synchronized to work, so the
// sender can overload the queue with information the receiver can handle,
// in the sync implemention the sender is blocked on a certain quota for the
// receiver to handle the queue
impl<T> Sender<T> {
    pub fn send(&mut self, t: T) {
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        // This is because we want to make sure that when the
        // receiver thread wakes up, it can inmediately take
        // the lock
        drop(inner);
        self.shared.available.notify_one();
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
        if let Some(t) = self.buffer.pop_front() {
            return Some(t);
        }

        // figure the way to block the current thread until the data
        // can be accessed from the result of shared.pop_front() -> Option<T>
        let mut inner = self.shared.inner.lock().unwrap();
        loop {
            match inner.queue.pop_front() {
                Some(t) => {
                    if !inner.queue.is_empty() {
                        // this is a trick named batched buffering.
                        // First, we check our own buffer to receive the
                        // sequential information, if our buffer is empty, then
                        // we would want to extract from the queue the data to the
                        // buffer to keep returning it, so we literally take the queue
                        // and put it in the mem space of the buffer, while the empty space
                        // of the buffer is put in the queue, this mem swap ends and the receiver
                        // returns the Some(t) again.
                        // Basically if we find the queue to have something (Some(t)), we then
                        // check if there is more data, if there is more data we put it in the
                        // buffer and still return the first value we popped, we asume here that
                        // the best way to check for future data is by saying that it is likely
                        // that if I have something from the pop_front, then theres chances of
                        // having more than one object there, so we check, if not we just return
                        // the value from the queue.
                        // is like -> we mostly will return from buffer, but in order to fill
                        // our buffer we need to return from the queue, and if there was just
                        // one element in the queue, we won't recharge our buffer
                        std::mem::swap(&mut self.buffer, &mut inner.queue);
                    }
                    return Some(t);
                }
                None if inner.senders == 0 => return None,
                // notice that at this point we go out of scope, and hence,
                // release the lock on the mutex given that inner's lifetime
                // is tied to the recv scope
                None => inner = self.shared.available.wait(inner).unwrap(),
                // This basically puts the thread to sleep until it receives
                // a signal of waking up. It is important to note that the
                // .wait(inner) basically gives up the Mutex so the Sender
                // can Send a message, and it takes it if appropiate, within
                // the loop, the wait function is in charged of locking and
                // releasing the mutex, so we don't have to do manual labor
                // to lock, check, and release all in the same block
            }
        }
    }
}

// The reason why the Mutex is just applied to the inner and
// not the whole Shared object is that in order to notify
// other threads to wake up you need the available parameter
// that would be blocked by the Mutex on the lock, the idea
// here is to notify the receiver and let go of the lock at the
// same time and not while you have the data locked.
struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

// additional adta
struct Inner<T> {
    queue: VecDeque<T>,
    senders: usize, //maybe an AtomixUsize to have senders across thread boundaries?
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::default(),
        senders: 1,
    };
    let shared = Shared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };
    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone(),
        },
    )
}

struct SyncInner<T> {
    queue: VecDeque<T>,
    senders: usize, // tracked by mutex
    max_length: usize,
}

struct SyncShared<T> {
    inner: Mutex<SyncInner<T>>,
    available: Condvar,
}

pub struct SyncSender<T> {
    shared: Arc<SyncShared<T>>,
}

impl<T> SyncSender<T> {
    pub fn send(&mut self, t: T) {
        loop {
            let inner = self.shared.inner.lock().unwrap();
            let queue_length = inner.queue.len();
            if queue_length == inner.max_length {
                continue;
            } else {
                break;
            }
        }
        let mut inner = self.shared.inner.lock().unwrap();
        inner.queue.push_back(t);
        drop(inner);
        self.shared.available.notify_one();
    }
}

impl<T> Clone for SyncSender<T> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

pub fn sync_channel<T>() -> (SyncSender<T>, Receiver<T>) {
    let inner = SyncInner {
        queue: VecDeque::default(),
        senders: 1,
        max_length: 10,
    };
    let shared = SyncShared {
        inner: Mutex::new(inner),
        available: Condvar::new(),
    };
    let shared = Arc::new(shared);
    (
        SyncSender {
            shared: shared.clone(),
        },
        Receiver {
            shared: shared.clone() as Sender<T>, // en esta morimos
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let (mut tx, mut rx) = channel();
        tx.send(42);
        assert_eq!(rx.recv(), Some(42));
    }

    #[test]
    fn closed_tx() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
        // this creates a sleeping mode that goes forever, there should
        // be a way to drop both values on the event of no senders left
    }

    #[test]
    fn closed_rx() {
        let (mut tx, rx) = channel();
        drop(rx);
        tx.send(42); // should be that you can log someway that the channel has been
    }
}
