// incorrect visibility restriction
#[macro_use]
extern crate ref_thread_local_compiletest as ref_thread_local;

ref_thread_local! {
    pub(nonsense) static managed WRONG: () = ();
    //~^ ERROR incorrect visibility restriction
}

fn main() { }
