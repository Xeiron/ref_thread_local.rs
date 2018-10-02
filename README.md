ref_thread_local.rs
==============

A macro for declaring thread-local `static`s like using both of `lazy_static!` and `RefCell`

Using this macro, you can have thread-local `static`s be referenced by `borrow()` function 
like using a `RefCell`.

You may also initialize or destroy a `static` variable at any time you like.

[![Travis-CI Status](https://travis-ci.org/Xeiron/ref_thread_local.rs.svg?branch=master)](https://travis-ci.org/Xeiron/ref_thread_local.rs)
[![Latest version](https://img.shields.io/crates/v/ref_thread_local.svg)](https://crates.io/crates/ref_thread_local)
[![Documentation](https://docs.rs/ref_thread_local/badge.svg)](https://docs.rs/ref_thread_local)
[![License](https://img.shields.io/crates/l/ref_thread_local.svg)](https://github.com/Xeiron/ref_thread_local.rs#license)

## Minimum supported `rustc`

`1.30.0+`

Test failed on `1.29.1` due to ICE, maybe backport after ICE is solved.

# Getting Started

[ref_thread_local.rs is available on crates.io](https://crates.io/crates/ref_thread_local).
It is recommended to look there for the newest released version, as well as links to the newest builds of the docs.

At the point of the last update of this README, the latest published version could be used like this:

Add the following dependency to your Cargo manifest...

```toml
[dependencies]
ref_thread_local = "0.0"
```

...and see the [docs](https://docs.rs/ref_thread_local) for how to use it.

# Example

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

## License

Licensed under of
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)