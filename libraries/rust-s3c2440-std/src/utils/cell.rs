use core::cell::{Ref, RefCell, RefMut};

static mut BORROW_ID: u32 = 0;

pub struct TracedRefCell<T> {
    inner: RefCell<T>,
    name: &'static str,
}

impl<T> TracedRefCell<T> {
    pub const fn new(value: T, name: &'static str) -> Self {
        Self {
            inner: RefCell::new(value),
            name,
        }
    }

    pub fn borrow(&self) -> TracedRef<'_, T> {
        let id = unsafe {
            BORROW_ID += 1;
            BORROW_ID
        };
        println!("[{}] borrow() -> Ref #{} (immutable)", self.name, id);
        TracedRef {
            _inner: self.inner.borrow(),
            id,
            name: self.name,
        }
    }

    pub fn borrow_mut(&self) -> TracedRefMut<'_, T> {
        let id = unsafe {
            BORROW_ID += 1;
            BORROW_ID
        };
        println!("[{}] borrow_mut() -> RefMut #{} (mutable)", self.name, id);
        TracedRefMut {
            _inner: self.inner.borrow_mut(),
            id,
            name: self.name,
        }
    }
}

pub struct TracedRef<'a, T> {
    _inner: Ref<'a, T>,
    id: u32,
    name: &'static str,
}

impl<'a, T> core::ops::Deref for TracedRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl<'a, T> Drop for TracedRef<'a, T> {
    fn drop(&mut self) {
        println!("[{}] Ref #{} dropped", self.name, self.id);
    }
}

pub struct TracedRefMut<'a, T> {
    _inner: RefMut<'a, T>,
    id: u32,
    name: &'static str,
}

impl<'a, T> core::ops::Deref for TracedRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self._inner
    }
}

impl<'a, T> core::ops::DerefMut for TracedRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._inner
    }
}

impl<'a, T> Drop for TracedRefMut<'a, T> {
    fn drop(&mut self) {
        println!("[{}] RefMut #{} dropped", self.name, self.id);
    }
}
