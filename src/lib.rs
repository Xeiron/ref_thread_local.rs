// Copyright 2018 tuxzz and lazy-static.rs Developers
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. This file may not be copied, modified,
// or distributed except according to those terms.

/*!
A macro for declaring thread-local `static`s like using both of `lazy_static!` and `RefCell`

Using this macro, you can have thread-local `static`s be referenced by `borrow()` function 
like using a `RefCell`.

You may also initialize or destroy a `static` variable at any time you like.

# Syntax

```ignore
ref_thread_local! {
    [pub] static managed NAME_1: TYPE_1 = EXPR_1;
    [pub] static managed NAME_2: TYPE_2 = EXPR_2;
    ...
    [pub] static managed NAME_N: TYPE_N = EXPR_N;
}
```

Attributes (including doc comments) are supported as well:

```rust
# #[macro_use]
# extern crate ref_thread_local;
# use ref_thread_local::RefThreadLocal;
# fn main() {
ref_thread_local! {
    /// This is an example for using doc comment attributes
    static managed EXAMPLE: u8 = 42;
}
# }
```

# Semantics

For a given `static managed NAME: TYPE = EXPR;`, the macro generates a unique type that
implements `RefThreadLocal<T>` trait and stores it in a static with name `NAME`. (Attributes end up
attaching to this type.)

When calling any method of this unique type, it generated a `RefManager<T>` internally, 
which manage the reference count of borrowing, and initialize a internal
thread-local `static` variable on calling `initialize()`, `borrow()`, `borrow_mut()`, 
`borrow_mut()`, `try_borrow_mut()` only if when uninitialized or destroyed.

Like `RefCell`, `borrow()` and `borrow_mut()` don't return reference but instead 
`Ref<'a, T>` or `RefMut<'a, T>`, which manage a borrow count internally.

Like `thread_local!`, variables in `ref_thread_local!` will be dropped normally 
when thread is exiting or `destroy()` is called.

# Example

Using the macro:

```rust
#[macro_use]
extern crate ref_thread_local;
use ref_thread_local::RefThreadLocal;

ref_thread_local! {
    static managed NUMBER: i32 = 233;
}

fn main() {
    let x = NUMBER.borrow(); // a Ref<'a, i32>
    println!("The number is {}.", x);
}
```

# Additional Runtime Resource Usage Compared to `thread_local!`
In current version:
* For each `static` variable in `ref_thread_local!`: 3 pointer variables, 1 `Cell<isize>`, 1 heap allocation.
* For each reference: 1 reference
* For each borrow: some borrow count operations, some function call (may be inlined)

*/

#![doc(html_root_url = "https://docs.rs/ref_thread_local/0.0.0")]

#[doc(hidden)]
pub use std::ops::Deref as __Deref;
#[doc(hidden)]
pub mod refmanager;
#[doc(hidden)]
pub use self::refmanager::*;

pub trait RefThreadLocal<T> {
    fn initialize(&self) -> Result<(), ()>;
    fn destroy(&self) -> Result<(), ()>;
    fn is_initialized(&self) -> bool;
    fn borrow<'a>(&self) -> Ref<'a, T>;
    fn borrow_mut<'a>(&self) -> RefMut<'a, T>;
    fn try_borrow<'a>(&self) -> Result<Ref<'a, T>, BorrowError>;
    fn try_borrow_mut<'a>(&self) -> Result<RefMut<'a, T>, BorrowMutError>;
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! _ref_thread_local_internal {
  ($(#[$attr:meta])* ($($vis:tt)*) static $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
    _ref_thread_local_internal!(@MAKE TY, $(#[$attr])*, ($($vis)*), $N);
    _ref_thread_local_internal!(@TAIL, $N : $T = $e);
    ref_thread_local!($($t)*);
  };
  (@TAIL, $N:ident : $T:ty = $e:expr) => {
    impl $N {
      fn get_refmanager(&self) -> $crate::RefManager<$T> {
        fn init_value() -> $T { $e }
        _create_refmanager_data!(GUARDED_REF_MANAGER_DATA, $T);
        $crate::RefManager::new(&GUARDED_REF_MANAGER_DATA, init_value)
      }
    }

    impl $crate::RefThreadLocal<$T> for $N {
      fn initialize(&self) -> Result<(), ()> { self.get_refmanager().initialize() }
      fn destroy(&self) -> Result<(), ()> { self.get_refmanager().destroy() }
      fn is_initialized(&self) -> bool { self.get_refmanager().is_initialized() }
      fn borrow<'_lifetime>(&self) -> $crate::Ref<'_lifetime, $T> { self.get_refmanager().borrow() }
      fn borrow_mut<'_lifetime>(&self) -> $crate::RefMut<'_lifetime, $T> { self.get_refmanager().borrow_mut() }
      fn try_borrow<'_lifetime>(&self) -> Result<$crate::Ref<'_lifetime, $T>, $crate::BorrowError> { self.get_refmanager().try_borrow() }
      fn try_borrow_mut<'_lifetime>(&self) -> Result<$crate::RefMut<'_lifetime, $T>, $crate::BorrowMutError> { self.get_refmanager().try_borrow_mut() }
    }
  };
  (@MAKE TY, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident) => {
    #[allow(missing_copy_implementations)]
    #[allow(non_camel_case_types)]
    #[allow(dead_code)]
    $(#[$attr])*
    $($vis)* struct $N { _private_field: () }
    #[doc(hidden)]
    $($vis)* static $N: $N = $N { _private_field: () };
  };
  () => ()
}

#[macro_export(local_inner_macros)]
macro_rules! ref_thread_local {
  ($(#[$attr:meta])* static managed $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
    _ref_thread_local_internal!($(#[$attr])* () static $N : $T = $e; $($t)*);
  };
  ($(#[$attr:meta])* pub static managed $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
    _ref_thread_local_internal!($(#[$attr])* (pub) static $N : $T = $e; $($t)*);
  };
  ($(#[$attr:meta])* pub ($($vis:tt)+) static managed $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
    _ref_thread_local_internal!($(#[$attr])* (pub ($($vis)+)) static $N : $T = $e; $($t)*);
  };
  () => ()
}
