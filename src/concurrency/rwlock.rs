pub struct RwLock<T> {
    data: *const T,
    rfc: AtomicUsize,
}

impl<T> RwLock<T> {
    pub fn new(data: T) -> Self {
        let heap_pointer = Box::new(data);
        Self {
            data: Box::into_raw(heap_pointer),
            rfc: 0,
        }
    }

    pub fn read(&self) -> &T {
        unimplemented!();
    }

    pub fn read_mut(&mut self) -> &mut T {
        unimplemented!();
    }
}
