use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ArcInner<T> {
    data: T,
    rfc: AtomicUsize,
}

pub struct Arc<T> {
    inner: NonNull<ArcInner<T>>,
    _marker: PhantomData<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Send for ArcInner<T> {}
unsafe impl<T: Send + Sync> Sync for ArcInner<T> {}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        let inner = Box::new(ArcInner {
            data,
            rfc: AtomicUsize::new(1),
        });

        Self {
            // SAFETY: value has been created and is guaranteed to be non null,
            // dereferencing into non null is valid
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(inner)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // SAFETY: Cannot mutate:
        // ArcInner {
        //      data: T,
        //      rfc: Cell<AtomicUsize>
        // }
        // The data value is guarded by the *const ArcInner,
        // and the interior mutability of Cell is guarded by
        // thread locks for data management, so thread racing
        // is guaranteed not to happen
        let refcount = &unsafe { self.inner.as_ref() }.rfc;
        refcount.fetch_add(1, Ordering::Release);
        Arc {
            inner: self.inner,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // Giving out a inmutable reference to the
        // inner value is guaranteed to be legal
        // given that exclusive references are not
        // given out in any point
        &unsafe { self.inner.as_ref() }.data
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        let rfc = &unsafe { self.inner.as_ref() }.rfc;
        if rfc.fetch_sub(1, Ordering::Release) == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
            unsafe {
                Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}
