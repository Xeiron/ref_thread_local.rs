#[macro_use]
extern crate ref_thread_local;
use ref_thread_local::RefThreadLocal;
use std::collections::HashMap;

ref_thread_local! {
    /// Documentation!
    pub static managed NUMBER: u32 = times_two(3);

    static managed ARRAY_BOXES: [Box<u32>; 3] = [Box::new(1), Box::new(2), Box::new(3)];

    /// More documentation!
    #[allow(unused_variables)]
    #[derive(Copy, Clone, Debug)]
    pub static managed STRING: String = "hello".to_string();

    static managed HASHMAP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "abc");
        m.insert(1, "def");
        m.insert(2, "ghi");
        m
    };

    // This should not compile if the unsafe is removed.
    static managed UNSAFE: u32 = unsafe {
        std::mem::transmute::<i32, u32>(-1)
    };
}

ref_thread_local! {
    static managed S1: &'static str = "a";
    static managed S2: &'static str = "b";
}
ref_thread_local! {
    static managed S3: String = [*S1.borrow(), *S2.borrow()].join("");
}

#[test]
fn s3() {
    assert_eq!(&*S3.borrow(), "ab");
}

fn times_two(n: u32) -> u32 {
    n * 2
}

#[test]
fn test_basic() {
    assert_eq!(&**STRING.borrow(), "hello");
    assert_eq!(*NUMBER.borrow(), 6);
    assert!(HASHMAP.borrow().get(&1).is_some());
    assert!(HASHMAP.borrow().get(&3).is_none());
    assert_eq!(
        &*ARRAY_BOXES.borrow(),
        &[Box::new(1), Box::new(2), Box::new(3)]
    );
    assert_eq!(*UNSAFE.borrow(), std::u32::MAX);
}

#[test]
fn test_borrow_after_borrow_mut() {
    let _a = NUMBER.try_borrow_mut();
    let _b = NUMBER.try_borrow();
    _a.expect("failed");
    _b.expect_err("failed");
}

#[test]
fn test_borrow_mut_after_borrow() {
    let _a = NUMBER.try_borrow();
    let _b = NUMBER.try_borrow_mut();
    _a.expect("failed");
    _b.expect_err("failed");
}

#[test]
fn test_repeat() {
    assert_eq!(*NUMBER.borrow(), 6);
    assert_eq!(*NUMBER.borrow(), 6);
    assert_eq!(*NUMBER.borrow(), 6);
}

#[test]
fn test_meta() {
    // this would not compile if STRING were not marked #[derive(Copy, Clone)]
    let copy_of_string = STRING;
    // just to make sure it was copied
    assert!(&STRING as *const _ != &copy_of_string as *const _);
    // this would not compile if STRING were not marked #[derive(Debug)]
    assert_eq!(
        format!("{:?}", STRING),
        "STRING { _private_field: () }".to_string()
    );
}

mod visibility {
    use ref_thread_local::RefThreadLocal;
    ref_thread_local! {
        pub static managed FOO: Box<u32> = Box::new(0);
        static managed BAR: Box<u32> = Box::new(98);
    }

    pub mod inner {
        ref_thread_local! {
            pub(in super) static managed BAZ: Box<u32> = Box::new(42);
            pub(crate) static managed BAG: Box<u32> = Box::new(37);
        }
    }

    #[test]
    fn sub_test() {
        assert_eq!(**FOO.borrow(), 0);
        assert_eq!(**BAR.borrow(), 98);
        assert_eq!(**inner::BAZ.borrow(), 42);
        assert_eq!(**inner::BAG.borrow(), 37);
    }
}

#[test]
fn test_visibility() {
    assert_eq!(*visibility::FOO.borrow(), Box::new(0));
    assert_eq!(*visibility::inner::BAG.borrow(), Box::new(37));
}

// This should not cause a warning about a missing Copy implementation
ref_thread_local! {
    pub static managed VAR: i32 = { 0 };
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct X;
struct Once(X);
const ONCE_INIT: Once = Once(X);
static DATA: X = X;
static ONCE: X = X;
fn require_sync() -> X {
    X
}
fn transmute() -> X {
    X
}
fn __static_ref_initialize() -> X {
    X
}
fn test(_: Vec<X>) -> X {
    X
}

// All these names should not be shadowed
ref_thread_local! {
    static managed ITEM_NAME_TEST: X = {
        test(vec![X, Once(X).0, ONCE_INIT.0, DATA, ONCE,
                  require_sync(), transmute(),
                  // Except this, which will sadly be shadowed by internals:
                  // __static_ref_initialize()
                  ])
    };
}

#[test]
fn item_name_shadowing() {
    assert_eq!(*ITEM_NAME_TEST.borrow(), X);
}

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::ATOMIC_BOOL_INIT;

static PRE_INIT_FLAG: AtomicBool = ATOMIC_BOOL_INIT;

ref_thread_local! {
    static managed PRE_INIT: () = {
        PRE_INIT_FLAG.store(true, SeqCst);
        ()
    };
}

#[test]
fn pre_init() {
    assert_eq!(PRE_INIT_FLAG.load(SeqCst), false);
    let _ = PRE_INIT.initialize();
    assert_eq!(PRE_INIT_FLAG.load(SeqCst), true);
}

ref_thread_local! {
    static managed LIFETIME_NAME: for<'a> fn(&'a u8) = { fn f(_: &u8) {} f };
}

#[test]
fn lifetime_name() {
    let _ = LIFETIME_NAME.borrow();
}
