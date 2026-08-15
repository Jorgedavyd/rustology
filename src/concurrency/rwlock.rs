use std::sync::atomic::AtomicUsize;

pub struct RwLock<T> {
    data: *const T,
    rfc: AtomicUsize,
}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Box::into_raw(Box::new(data)),
            rfc: AtomicUsize::new(0),
        }
    }

    pub fn read(&self) -> &T {
        unimplemented!()
    }

    pub fn read_mut(&mut self) -> &mut T {
        unimplemented!()
    }
}
