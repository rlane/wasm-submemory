static mut X: i32 = 42;

#[no_mangle]
pub fn entry() -> i32 {
    unsafe { X }
}

#[no_mangle]
pub fn inc() {
    unsafe {
        X += 1;
    }
}
