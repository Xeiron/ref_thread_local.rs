#[macro_use]
extern crate ref_thread_local_compiletest as ref_thread_local;
use ref_thread_local::RefThreadLocal;

ref_thread_local! {
  static managed VALUE: i32 = 233i32;
}

fn main() {
  let _ = (|| &*VALUE.borrow())(); //~ ERROR borrowed value does not live long enough
}