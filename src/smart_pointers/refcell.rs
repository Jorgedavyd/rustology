// RefCell
// RefCell is an intelligent pointer that gives us
// the ability to programmatically work with references
// and manage borrowing related routines without the
// borrow checker getting in the middle, but enforcing
// the rules stablished by the borrow checker

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

use super::cell::Cell;

pub struct RefCell<T> {
    data: UnsafeCell<T>,
    rfc: Cell<isize>, // gives us the possibility to mutate the reference counting through a shared
                      // reference
}

// this type cannot be thread safe given that manually incrementing a counter
// with no thread boundaries to the same location is thread racing, and
// because data (UnsafeCell) is !Sync

// impl<T> !Sync for RefCell {} is embedded given the children

// The Sync analogous is RwLock, but instead of dynamically giving references,
// which can be unconvenient for the multi-threaded case, the read (borrow analogous)
// blocks the entire thread until it receives the reference (in the case that other
// exclusive reference is alive in the case of borrowing inmutably, or if you are asking
// for an exclusive reference but there is aliasing across threads)
// It basically enforces thread safety and data safety through locking mechanisms that allow
// Send and Sync to be possible

impl<T> RefCell<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            rfc: Cell::new(0),
        }
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        // this is the crazy part about the dinamic borrowing: Option!

        let value = self.rfc.get();
        if value < 0 {
            None
        } else {
            self.rfc.set(value + 1);
            // self.rfc += 1;
            // got to implement the drop to update the reference count each time
            // a value is drop for coherence with the lifetimes

            // SAFETY: No exclusive references given out, validation of inmutable
            // reference is guaranteed
            // unsafe { Some(&*self.data.get()) }
            Some(Ref { refcell: self })
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        let value = self.rfc.get();

        if value < 0 {
            None
        } else if value == 0 {
            self.rfc.set(value + 1);
            // self.rfc -= 1;
            // got to implement the drop to update the reference count each time
            // a value is drop for coherence with the lifetimes

            // SAFETY: No inmutable references would be invalidated
            // SAFETY: There's no mutable references hanging around
            Some(RefMut { refcell: self })
        } else {
            None
        }
    }
}

// the lifetimes points to the RefCell's lifetime
// to ensure that references can only be tracked
// within the RefCell entity lifespan
pub struct Ref<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        let value = self.refcell.rfc.get();
        if value <= 0 {
            // imposible to try to deallocate a Ref value with a
            // RefCell that holds exclusive data, Ref can only be given
            // out on inmutable borrowing (aliasing)
            unreachable!()
            // imposible to try to deallocate a Ref value with a
            // RefCell that holds non-shared data, Ref can only be given
            // out on inmutable borrowing (aliasing), at least a positive
            // value must be given out
        } else {
            self.refcell.rfc.set(value - 1);
        }
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: A Ref is only created if no exlusive
        // references are alive and invalidation is imposible
        // therefore, dereferencing into an inmutable reference
        // is valid
        unsafe { &*self.refcell.data.get() }
    }
}

pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        let value = self.refcell.rfc.get();
        if value == -1 {
            self.refcell.rfc.set(0)
        } else {
            // imposible to try to deallocate a RefMut value with a
            // RefCell that holds inmutable references to the data,
            // Ref can only be given out on borrow_mut, which enforces
            // no inmutable references invalidation
            unreachable!()
            // imposible to try to deallocate a RefMut value with a
            // RefCell that holds non-shared data, RefMut can only be given
            // out on mutable borrowing, only a negative one is the right case
        }
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // see safety for deref mut
        unsafe { &*self.refcell.data.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: A RefMut can only be created
        // if no other references have been given out
        // and invalidation is not possible
        unsafe { &mut *self.refcell.data.get() }
    }
}

// The reason why DerefMut and Deref can be both referenced
// is because in these scenarios, both reference types are
// being governed by the borrow checker

#[cfg(test)]
mod test {}
