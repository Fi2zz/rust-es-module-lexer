use crate::helper::Source;
use crate::position;

#[allow(non_upper_case_globals)]
static mut source: Source = Vec::new();
#[allow(non_upper_case_globals)]
static mut SOURCE_END: i32 = -1;

#[allow(non_snake_case)]
pub fn setSource(inputSource: &Source) {
    unsafe {
        source = inputSource.to_vec();
        SOURCE_END = source.len() as i32 - 1;
    }
}
pub fn end() -> i32 {
    unsafe { SOURCE_END }
}
pub fn len() -> i32 {
    unsafe { source.len() as i32 }
}
// JS: pos++ < end
#[allow(non_snake_case)]
pub fn posIncLtEnd() -> bool {
    let prev = position::position();
    position::next();
    prev < end()
}
// JS: source.charCodeAt(index). Out-of-range is NaN in JS; 0 plays the same
// role here (falsy, never equal to a compared char code).
#[allow(non_snake_case)]
pub fn charCodeAt(index: i32) -> i32 {
    unsafe {
        if index < 0 || index >= source.len() as i32 {
            return 0;
        }
        source[index as usize] as i32
    }
}
// JS: source.startsWith(s, from)
#[allow(non_snake_case)]
pub fn starts_with(s: &str, from: i32) -> bool {
    unsafe {
        if from < 0 {
            return false;
        }
        let from = from as usize;
        let bytes = s.as_bytes();
        if from + bytes.len() > source.len() {
            return false;
        }
        &source[from..from + bytes.len()] == bytes
    }
}
// JS: source.slice(start, end)
pub fn stringify(start: i32, end: i32) -> String {
    unsafe {
        let s = start.max(0) as usize;
        let e = (end.max(0) as usize).min(source.len());
        if s >= e {
            return String::new();
        }
        String::from_utf8_lossy(&source[s..e]).into_owned()
    }
}
