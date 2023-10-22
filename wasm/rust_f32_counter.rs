static mut COUNTER: f32 = 0.0;

#[no_mangle]
pub fn entry() -> f32 {
    unsafe {
        COUNTER += 1.0;
        COUNTER
    }
}
