// Copyright 2018 tuxzz and lazy-static.rs Developers
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified,
// or distributed except according to those terms.

extern crate std;
use super::RefThreadLocal;
use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use std::thread::LocalKey;

struct RefManagerInnerData<T> {
    borrow_count: Cell<isize>,
    value: T,
}

pub struct RefManagerPeekData<T> {
    ptr_inner_data: *mut RefManagerInnerData<T>,
    ptr_borrow_count: *const Cell<isize>,
    ptr_value: *mut T,
}

impl<T> Clone for RefManagerPeekData<T> {
    fn clone(&self) -> Self {
        return Self {
            ptr_inner_data: self.ptr_inner_data,
            ptr_borrow_count: self.ptr_borrow_count,
            ptr_value: self.ptr_value,
        };
    }
}

impl<T> Copy for RefManagerPeekData<T> {}

pub struct RefManagerDataGuard<T> {
    peek_data: Cell<RefManagerPeekData<T>>,
}

pub struct Ref<'a, T: 'a> {
    borrow_count: &'a Cell<isize>,
    value: &'a T,
}

#[derive(Debug)]
pub struct RefMut<'a, T: 'a> {
    borrow_count: &'a Cell<isize>,
    value: &'a mut T,
}

#[derive(Debug)]
pub struct RefManager<T: 'static> {
    local_key: &'static LocalKey<RefManagerDataGuard<T>>,
    init_func: fn() -> T,
}

#[derive(Debug)]
pub struct BorrowError {
    _private: (),
}

#[derive(Debug)]
pub struct BorrowMutError {
    _private: (),
}

#[macro_export]
#[doc(hidden)]
macro_rules! _create_refmanager_data {
    ($NAME:ident, $T:ty) => {
        thread_local! {
          static $NAME: $crate::RefManagerDataGuard<$T> = $crate::RefManagerDataGuard::INIT_SELF;
        }
    };
}

impl<T> RefManager<T> {
    pub fn new(local_key: &'static LocalKey<RefManagerDataGuard<T>>, init_func: fn() -> T) -> Self {
        RefManager {
            local_key,
            init_func,
        }
    }

    fn get_initialized_peek(&self) -> RefManagerPeekData<T> {
        self.local_key.with(|guard| {
            if guard.peek_data.get().ptr_inner_data.is_null() {
                self.initialize().expect("failed to initialize");
            }
            guard.peek_data.get()
        })
    }
}

impl<T> RefThreadLocal<T> for RefManager<T> {
    fn initialize(&self) -> Result<(), ()> {
        self.local_key.with(|guard| {
            if guard.peek_data.get().ptr_inner_data.is_null() {
                let mut box_inner_data = Box::new(RefManagerInnerData {
                    borrow_count: Cell::new(0),
                    value: (self.init_func)(),
                });
                let ptr_borrow_count = &box_inner_data.borrow_count as *const Cell<isize>;
                let ptr_value = &mut box_inner_data.value as *mut T;
                let ptr_inner_data = Box::into_raw(box_inner_data);
                guard.peek_data.set(RefManagerPeekData {
                    ptr_inner_data,
                    ptr_borrow_count,
                    ptr_value,
                });
                Ok(())
            } else {
                Err(())
            }
        })
    }

    fn destroy(&self) -> Result<(), ()> {
        self.local_key.with(|guard| guard.destroy())
    }

    fn is_initialized(&self) -> bool {
        self.local_key
            .with(|guard| !guard.peek_data.get().ptr_inner_data.is_null())
    }

    fn borrow<'a>(&self) -> Ref<'a, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    fn borrow_mut<'a>(&self) -> RefMut<'a, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    fn try_borrow<'a>(&self) -> Result<Ref<'a, T>, BorrowError> {
        let peek_data = self.get_initialized_peek();
        let (ptr_borrow_count, ptr_value) = (peek_data.ptr_borrow_count, peek_data.ptr_value);

        let cell_borrow_count = unsafe { ptr_borrow_count.as_ref() }.unwrap();
        let borrow_count = cell_borrow_count.get();
        if borrow_count < 0 {
            return Err(BorrowError { _private: () });
        }
        cell_borrow_count.set(borrow_count + 1);
        Ok(Ref {
            borrow_count: cell_borrow_count,
            value: unsafe { ptr_value.as_ref() }.unwrap(),
        })
    }

    fn try_borrow_mut<'a>(&self) -> Result<RefMut<'a, T>, BorrowMutError> {
        let peek_data = self.get_initialized_peek();
        let (ptr_borrow_count, ptr_value) = (peek_data.ptr_borrow_count, peek_data.ptr_value);

        let cell_borrow_count = unsafe { ptr_borrow_count.as_ref() }.unwrap();
        let borrow_count = cell_borrow_count.get();
        if borrow_count != 0 {
            return Err(BorrowMutError { _private: () });
        }
        cell_borrow_count.set(-1);
        Ok(RefMut {
            borrow_count: cell_borrow_count,
            value: unsafe { ptr_value.as_mut() }.unwrap(),
        })
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.borrow_count.set(self.borrow_count.get() - 1);
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T: Debug> Debug for Ref<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<'a, T: Display> Display for Ref<'a, T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.borrow_count.set(0);
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.value
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<T> RefManagerDataGuard<T> {
    pub const INIT_PEEK_DATA: RefManagerPeekData<T> = RefManagerPeekData {
        ptr_inner_data: null_mut(),
        ptr_borrow_count: null(),
        ptr_value: null_mut(),
    };
    pub const INIT_SELF: Self = RefManagerDataGuard {
        peek_data: Cell::new(Self::INIT_PEEK_DATA),
    };

    pub fn destroy(&self) -> Result<(), ()> {
        let peek_data = self.peek_data.get();
        let (ptr_inner_data, ptr_borrow_count) =
            (peek_data.ptr_inner_data, peek_data.ptr_borrow_count);
        if ptr_inner_data.is_null() {
            Err(())
        } else {
            let borrow_count = unsafe { ptr_borrow_count.as_ref() }.unwrap().get();
            if borrow_count != 0 {
                panic!("cannot destroy before all references are dropped");
            }
            unsafe { Box::from_raw(ptr_inner_data) };
            self.peek_data.set(Self::INIT_PEEK_DATA);
            Ok(())
        }
    }
}

impl<T> Drop for RefManagerDataGuard<T> {
    fn drop(&mut self) {
        let _ = self.destroy();
    }
}
