// Rc
// Is a single threaded smart pointer that gives out
// pointers to a single heap allocation, and vanishes on
// the last usage

// Reference counting in this context, unlike RefCell, must
// persist for the entire lifetime of the heap allocated object itself
// to keep track of the amount of references to the same location

use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use super::cell::Cell;

struct RcInner<T> {
    data: T,
    refcount: Cell<usize>,
}

// refcount: usize (Cannot use this implementation given that on clone, references are
// transfered, must implement an static machine that keeps track of the different references
// across the multiple clones there could be. Moreover, we also need a place to store the
// actual value that is pointed by Rc.value, so for both of these means we can create an
// RcInner that acts as a global refcount and holds the object itself at the same time
pub struct Rc<T> {
    inner: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>, // compiler doesn't know we own the RcInner, and that
                                      // deallocation happens within Rc drop. Here the drop checker will try to access each field
                                      // within the Rc to drop it finally, but if we've already dropped the RcInner<T> it would be a
                                      // dangling pointer for the short period of dropping Rc (this is problematic because the drop
                                      // checker identifies dropping as accessing each field), therefore we got to find a way to
                                      // tell the compiler that we have a type T whose drop method needs to be checked so that
                                      // deallocation of the pointed value doesn't happen before the dealocation of the interface
}

// impl<T> !Send for Rc<T> {} // reference counting can cause thread racing problems, Arc solves this
// with atomics, the only difference between Rc and Arc is that Arc uses Atomics to maintain the
// count of references to the Inner type

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        // creates the heap allocation
        let inner = Box::new(RcInner {
            data: value,
            refcount: Cell::new(1),
        });
        // SAFETY: This is a little bit convoluted, but we are trying to parse
        // a ptr to the heap allocation to the Box, but the argument must be
        // *mut T, so we have this zero cost work around in rust that creates an
        // abstraction over the raw *const T pointer. It gives us the perforamance
        // edge given that the compiler knows it is NonNull pointer and it can
        // use the null as a None, that's why the return type is Option<...>
        // and it also gives an interface that lets us invoke a *mut T required
        // to recraft the Box
        // SAFETY: unchecked means that null is not checked, but given that we
        // are parsing a known allocation, it is safe
        Rc {
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
        // into raw is necessary so that the memory is not freed at
        // the end of the scope
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        // let inner = unsafe { self.inner.as_ref() }; this was valid for create a reference to a
        // derefenced raw pointer
        let inner = unsafe { self.inner.as_ref() };
        let value = inner.refcount.get();
        inner.refcount.set(value + 1);
        Rc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The problem with this dereferencing to the heap
        // is that we are not sure if the value is still there or not,
        // so dangling pointer, and this is enforced by the runtime
        // checking rules that are being enforced through the drop
        // clause
        &unsafe { self.inner.as_ref() }.data
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        // check the amount of Rcs cloned
        let inner = unsafe { self.inner.as_ref() };
        let value = inner.refcount.get();
        if value == 1 {
            // if this is the last one then deallocate
            // the heap allocated value
            // drop(inner); // this is would be intelligent in because inner is a
            // reference to the value that holds data that is being deallocated
            // right now, but given that the lifetime of the reference is not scope
            // wise, but per section, this is superfluous, the lifetime engine is
            // guaranteed to not give problems on a control flow schema where the
            // reference is not going to be accessed any more. We are gonna let it
            // stay here to avoid someone writing code with inner at the end of the scope
            drop(inner);
            // SAFETY: Getting a mutable reference from the self.inner is valid
            // given that we know this is the last reference
            unsafe {
                Box::from_raw(self.inner.as_ptr());
            }
        } else {
            inner.refcount.set(value - 1);
        }
    }
}
