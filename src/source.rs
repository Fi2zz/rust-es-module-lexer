use crate::helper;
use crate::helper::Source;

#[allow(non_upper_case_globals, dead_code)]
static mut source: helper::Source = Vec::new();
static mut SOURCE_END: i32 = 0;
#[allow(non_snake_case)]
pub fn setSource(inputSource: &Source) {
    unsafe {
        source = inputSource.to_vec();
        SOURCE_END = (source.len() - 1) as i32;
        if SOURCE_END <= 0 {
            helper::print_cannot_empty_file();
        }
    }
}
#[allow(non_snake_case)]
pub fn ifEnd(pos: i32) -> bool {
    unsafe { pos >= SOURCE_END }
}
#[allow(non_snake_case)]
pub fn ifNotEnd(pos: i32) -> bool {
    unsafe { pos < SOURCE_END }
}
pub fn stringify(start: i32, end: i32) -> String {
    let s = start as usize;
    let e = end as usize;
    let bytes: Vec<u8>;
    unsafe {
        bytes = source.as_slice().get(s..e).unwrap_or_default().to_vec();
    }
    return String::from_utf8(bytes).expect("no value");
}
#[allow(non_snake_case)]
pub fn getChar(start: i32, end: i32) -> char {
    let s = stringify(start, end);
    let cs: Vec<char> = s.chars().collect();
    return cs[0];
}
#[allow(non_snake_case)]
pub fn charCodeAt(index: i32) -> usize {
    let index = index as usize;
    let code: usize;
    unsafe {
        code = source[index] as usize;
    }
    code
}
pub fn len() -> usize {
    unsafe { source.len() }
}
#[allow(non_snake_case)]
pub fn getSource() -> Source {
    unsafe { source.to_vec() }
}
