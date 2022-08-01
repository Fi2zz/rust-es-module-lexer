#[allow(non_upper_case_globals)]
pub static mut _pos: i32 = -1;
pub fn step(steps: i32) -> i32 {
    unsafe { _pos += steps };
    position()
}
pub fn next() -> i32 {
    step(1)
}
pub fn prev() -> i32 {
    step(-1)
}
pub fn position() -> i32 {
    return unsafe { _pos };
}
pub fn cursor(steps: i32) -> i32 {
    step(steps)
}
pub fn dry(step: i32) -> i32 {
    return unsafe { _pos + step };
}
