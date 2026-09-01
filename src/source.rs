use crate::position;

// The upstream C lexer operates on char16_t units, and all positions in the
// baseline data are UTF-16 code unit offsets, so the source is stored as
// UTF-16 units rather than UTF-8 bytes.
#[allow(non_upper_case_globals)]
static mut source: Vec<u16> = Vec::new();
#[allow(non_upper_case_globals)]
static mut SOURCE_END: i32 = -1;

#[allow(non_snake_case)]
pub fn setSource(inputSource: &str) {
    unsafe {
        source = inputSource.encode_utf16().collect();
        SOURCE_END = source.len() as i32 - 1;
    }
}
pub fn end() -> i32 {
    unsafe { SOURCE_END }
}
pub fn len() -> i32 {
    unsafe { source.len() as i32 }
}
// C: pos++ < end
#[allow(non_snake_case)]
pub fn posIncLtEnd() -> bool {
    let prev = position::position();
    position::next();
    prev < end()
}
// C: *pos style reads. The C source buffer is null-terminated, so out-of-range
// reads see '\0'; mirror that with 0 for any invalid index.
#[allow(non_snake_case)]
pub fn charCodeAt(index: i32) -> i32 {
    unsafe {
        if index < 0 || index >= source.len() as i32 {
            return 0;
        }
        source[index as usize] as i32
    }
}
// C: memcmp(pos + from, s, len) == 0 style prefix checks
#[allow(non_snake_case)]
pub fn starts_with(s: &str, from: i32) -> bool {
    if from < 0 {
        return false;
    }
    let mut i = from;
    for unit in s.encode_utf16() {
        if charCodeAt(i) != unit as i32 {
            return false;
        }
        i += 1;
    }
    return true;
}
// JS: source.slice(start, end) on UTF-16 units
pub fn stringify(start: i32, end: i32) -> String {
    unsafe {
        let s = start.max(0) as usize;
        let e = (end.max(0) as usize).min(source.len());
        if s >= e {
            return String::new();
        }
        String::from_utf16_lossy(&source[s..e])
    }
}
// raw UTF-16 units in [start, end), for the acorn string reader
pub fn units(start: i32, end: i32) -> Vec<u16> {
    unsafe {
        let s = start.max(0) as usize;
        let e = (end.max(0) as usize).min(source.len());
        if s >= e {
            return Vec::new();
        }
        source[s..e].to_vec()
    }
}
