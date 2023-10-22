#[no_mangle]
pub fn entry() -> i32 {
    *std::hint::black_box(Box::new(42))
}
