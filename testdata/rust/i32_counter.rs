static mut COUNTER: i32 = 0;

#[no_mangle]
pub fn entry() -> i32 {
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}
