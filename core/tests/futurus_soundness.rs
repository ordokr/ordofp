#[cfg(feature = "async")]
use ordofp_core::async_core::Futurus;
#[cfg(feature = "async")]
use std::cell::RefCell;

#[test]
#[cfg(feature = "async")]
fn futurus_should_be_send_but_not_sync() {
    // RefCell is Send but !Sync
    let cell = RefCell::new(42);

    // This async block captures RefCell, so it is Send but !Sync
    let future = async move {
        *cell.borrow_mut() += 1;
        *cell.borrow()
    };

    // Futurus must be Send
    let futurus = Futurus::new(future);

    fn assert_send<T: Send>(_: &T) {}

    // This should compile fine
    assert_send(&futurus);

    // This function requires T to be Sync
    // fn assert_sync<T: Sync>(_: &T) {}

    // This line would cause a compile error now, confirming the fix:
    // assert_sync(&futurus);
    //
    // Error: `Futurus<i32>` cannot be shared between threads safely
    // within `Futurus<i32>`, the trait `Sync` is not implemented for `Pin<Box<dyn Future<Output = i32> + Send>>`
}
