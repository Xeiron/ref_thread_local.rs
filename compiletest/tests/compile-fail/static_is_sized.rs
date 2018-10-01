// error-pattern: the size for values of type `str` cannot be known at compilation time
#[macro_use]
extern crate ref_thread_local_compiletest as ref_thread_local;

ref_thread_local! {
    pub static managed FOO: str = panic!();
}

fn main() {
}
