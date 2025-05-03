// Cell
// Provides interior mutability with inmutable borrowing by:
// 1. unabling inmutable referencing of the inner type
// 2. providing get, get_mut, and take methods to enforce borrowing rules.
// 3. Create a !Send implementation to not allow mutability of the same
// object with two different

// Quick notes
// You probably want to use Cell with a cheap T so that accessing the value
// is not that pricy.

use std::cell::UnsafeCell;
// a wrapper that allows interior mutability in rust
// gives us an unsafety API to create safe code via
// interface boundaries

pub struct Cell<T> {
    data: UnsafeCell<T>,
}

// Defined in data field for UnsafeCell already
// impl<T> !Sync for Cell<T>

impl<T> Cell<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn set(&self, value: T) {
        // is ok given that there's no inmutable references to the value
        // data race is not allowed given the !Sync
        unsafe { *self.data.get() = value };
    }

    pub fn get(&self) -> T
    // this is safe since dereferencing this value is guaranteed to be thread safe
    // given !Sync and it doesn't invalidate any inmutable references
    where
        T: Copy,
    {
        unsafe { *self.data.get() }
    }

    pub fn swap(&self, other: &Cell<T>)
    // this allows for the swap programmatically
    // Given that other is a cell type and is checked not to be parsed
    // across thread boundaries, this dereferencing comes with no data
    // racing nor inmutable references alive
    where
        T: Copy,
    {
        let local_value = self.get();
        let other_value = other.get();
        other.set(local_value);
        self.set(other_value);
    }
}

#[cfg(test)]
mod test {
    use super::Cell;
    #[test]
    fn thread_racing() {
        unsafe impl<T> Send for Cell<T> {}
        unsafe impl<T> Sync for Cell<T> {}
        use std::sync::Arc;
        use std::thread;
        let x = Arc::new(Cell::new(0));
        let x1 = Arc::clone(&x);
        let jb1 = thread::spawn(move || {
            for _ in 0..100000 {
                let x = x1.get(); // getting the data from Cell (Allowed mutable reference)
                x1.set(x + 1); // setting the new value for the inner Cell
            }
        });
        let x2 = Arc::clone(&x);
        let jb2 = thread::spawn(move || {
            for _ in 0..100000 {
                let x = x2.get();
                x2.set(x + 1)
            }
        });
        jb1.join().unwrap();
        jb2.join().unwrap();
        // SAFETY NOTE: Due to the nature of the accessing pattern, synchronizations may overlap
        // and both can be modifying non-serialized memory, overriding both the same value to the
        // disk on an operation that logically should add the number 2 to x per parallel cycle
        assert!(x.get() < 200000)
    }

    #[test]
    fn set() {
        let c = Cell::new(5);
        c.set(10);
        assert_eq!(c.get(), 10);
    }

    #[test]
    fn get() {
        let c = Cell::new(5);
        assert_eq!(c.get(), 5);
    }

    #[test]
    fn swap() {
        let c = Cell::new(5);
        let d = Cell::new(10);
        c.swap(&d);
        assert_eq!(c.get(), 10);
        assert_eq!(d.get(), 5);
    }
}
