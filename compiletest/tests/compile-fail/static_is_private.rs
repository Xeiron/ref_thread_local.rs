#[macro_use]
extern crate ref_thread_local_compiletest as ref_thread_local;
use ref_thread_local::RefThreadLocal;

mod outer {
    pub mod inner {
        ref_thread_local! {
            pub(in outer) static managed FOO: () = ();
        }
    }
}

fn main() {
    assert_eq!(*outer::inner::FOO.borrow(), ()); //~ ERROR static `FOO` is private
}
