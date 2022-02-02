use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use zaplib_cef_sys::{_cef_base_ref_counted_t, cef_base_ref_counted_t};

pub(crate) unsafe trait WrapperFor<T> {}

unsafe impl<T> Send for RefCounterGuard<T> {}
unsafe impl<T> Sync for RefCounterGuard<T> {}
pub(crate) struct RefCounterGuard<T> {
    // TODO - test this type
    base: *mut cef_base_ref_counted_t,
    val: *mut T,
    track_ref: bool,
}
impl<T> Deref for RefCounterGuard<T> {
    type Target = *mut T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
impl<T> DerefMut for RefCounterGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}
impl<T> Drop for RefCounterGuard<T> {
    fn drop(&mut self) {
        if self.track_ref && !self.base.is_null() {
            unsafe {
                let base = &mut *self.base;
                if let Some(release) = base.release {
                    release(base);
                }
            }
        }
    }
}
impl<T> Clone for RefCounterGuard<T> {
    fn clone(&self) -> Self {
        let res = Self { base: self.base, val: self.val, track_ref: true };
        unsafe { res.add_ref() };
        res
    }
}
impl<T> RefCounterGuard<T> {
    pub(crate) fn from(base: *mut cef_base_ref_counted_t, val: *mut T, track_ref: bool) -> RefCounterGuard<T> {
        if track_ref && !base.is_null() {
            unsafe {
                let base = &mut *base;
                // Let CEF know we are passing it around
                if let Some(add_ref) = base.add_ref {
                    add_ref(base);
                }
            }
        }

        RefCounterGuard { base, val, track_ref }
    }

    pub fn get(&self) -> *mut T {
        self.val
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_ref(&self) -> &mut T {
        unsafe { &mut *self.val }
    }

    pub unsafe fn add_ref(&self) {
        if !self.base.is_null() {
            let base = &mut *self.base;
            if let Some(add_ref) = base.add_ref {
                add_ref(base);
            }
        }
    }

    // Implementation when passing a cef struct as argument into CEF API
    // See https://bitbucket.org/chromiumembedded/cef/wiki/UsingTheCAPI for more context
    pub fn to_cef_as_arg(&self) -> *mut T {
        unsafe {
            self.add_ref();
        }
        self.get()
    }
}

// This relies on the c storage structure to allow casting a *_cef_base_ref_counted_t into a
// pointer of this type which starts with _cef_base_ref_counted_t
#[repr(C)]
pub(crate) struct BaseRefCountedExt<TCef, TWrapper> {
    v: TWrapper,
    count: AtomicUsize,
    phantom: PhantomData<TCef>,
}
impl<TCef, TWrapper> Deref for BaseRefCountedExt<TCef, TWrapper> {
    type Target = TWrapper;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}
impl<TCef, TWrapper> DerefMut for BaseRefCountedExt<TCef, TWrapper> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}
impl<TCef, TWrapper: WrapperFor<TCef>> BaseRefCountedExt<TCef, TWrapper> {
    fn wrap_ptr<F>(wrapper: F) -> *mut TCef
    where
        F: FnOnce(cef_base_ref_counted_t) -> TWrapper,
    {
        let base = BaseRefCountedExt::<TCef, TWrapper> {
            v: wrapper(_cef_base_ref_counted_t {
                size: std::mem::size_of::<Self>() as u64,
                add_ref: Some(Self::add_ref),
                release: Some(Self::release),
                has_one_ref: Some(Self::has_one_ref),
                has_at_least_one_ref: Some(Self::has_at_least_one_ref),
            }),
            count: AtomicUsize::new(1),
            phantom: PhantomData,
        };
        Box::into_raw(Box::new(base)) as *mut TCef
    }

    fn from_ptr<'a>(ptr: *mut cef_base_ref_counted_t) -> &'a mut BaseRefCountedExt<TCef, TWrapper> {
        unsafe { &mut *(ptr as *mut _) }
    }
    extern "C" fn add_ref(ptr: *mut cef_base_ref_counted_t) {
        let base = Self::from_ptr(ptr);
        base.count.fetch_add(1, Ordering::Relaxed);
    }
    extern "C" fn release(ptr: *mut cef_base_ref_counted_t) -> i32 {
        let base = Self::from_ptr(ptr);
        let old_count = base.count.fetch_sub(1, Ordering::Release);
        if old_count == 1 {
            // reclaim and release
            unsafe { Box::from_raw(base) };

            1 // true
        } else {
            0 // false
        }
    }
    extern "C" fn has_one_ref(ptr: *mut cef_base_ref_counted_t) -> i32 {
        let base = Self::from_ptr(ptr);
        if base.count.load(Ordering::SeqCst) == 1 {
            1 // true
        } else {
            0 // false
        }
    }
    extern "C" fn has_at_least_one_ref(ptr: *mut cef_base_ref_counted_t) -> i32 {
        let base = Self::from_ptr(ptr);
        if base.count.load(Ordering::SeqCst) >= 1 {
            1 // true
        } else {
            0 // false
        }
    }
}

pub(crate) fn wrap_ptr<TCef, TWrapper, F>(wrapper: F) -> *mut TCef
where
    F: FnOnce(cef_base_ref_counted_t) -> TWrapper,
    TWrapper: WrapperFor<TCef>,
{
    BaseRefCountedExt::wrap_ptr(wrapper)
}
