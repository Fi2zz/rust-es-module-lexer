#[allow(non_upper_case_globals)]
pub static mut _pos: i32 = -1;
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn step(steps: i32) -> i32 {
    unsafe { _pos += steps };
    position()
}
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn next() -> i32 {
    step(1)
}
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn prev() -> i32 {
    step(-1)
}
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn position() -> i32 {
    return unsafe { _pos };
}
#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn setPos(pos: i32) {
    unsafe { _pos = pos };
}
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn dry(step: i32) -> i32 {
    return unsafe { _pos + step };
}
