use crate::position;

// The upstream C lexer operates on char16_t units, and all positions in the
// baseline data are UTF-16 code unit offsets, so the source is stored as
// UTF-16 units rather than UTF-8 bytes.
// 缓冲区尾部带 4 个 0 哨兵（对应 C 的 null 终止符），让热循环里 pos+1/pos+2
// 的预读可以不做边界检查（见 charCodeAtUnchecked）。
#[allow(non_upper_case_globals)]
static mut source: Vec<u16> = Vec::new();
#[allow(non_upper_case_globals)]
static mut SOURCE_LEN: i32 = 0; // 不含哨兵的真实长度

#[allow(non_snake_case)]
pub fn setSource(inputSource: &str) {
    unsafe {
        let mut units: Vec<u16> = inputSource.encode_utf16().collect();
        SOURCE_LEN = units.len() as i32;
        units.extend_from_slice(&[0, 0, 0, 0]);
        source = units;
    }
}
// 快速通道：直接接管一块已填充的 UTF-16 缓冲（wasm lex_alloc/lex_parse_at），
// 零拷贝零转码。buf 长度 = units + 4，尾部 4 个哨兵位在此清零。
#[allow(non_snake_case)]
#[allow(dead_code)]
pub fn setSourceUtf16(mut buf: Vec<u16>, units: usize) {
    unsafe {
        for i in units..units + 4 {
            buf[i] = 0;
        }
        SOURCE_LEN = units as i32;
        source = buf;
    }
}
// C: end = source + sourceLen - 1（最后一个真实字符）
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn end() -> i32 {
    unsafe { SOURCE_LEN - 1 }
}
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn len() -> i32 {
    unsafe { SOURCE_LEN }
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
#[inline(always)]
pub fn charCodeAt(index: i32) -> i32 {
    unsafe {
        if index < 0 || index >= SOURCE_LEN {
            return 0;
        }
        *source.as_ptr().add(index as usize) as i32
    }
}
// 无边界检查版本：缓冲区尾部有 4 个 0 哨兵，调用方需保证 0 <= index <=
// SOURCE_LEN + 3（热循环里 pos <= end 时的 pos+1/pos+2 预读均满足）
#[allow(non_snake_case)]
#[inline(always)]
pub fn charCodeAtUnchecked(index: i32) -> i32 {
    unsafe { *source.as_ptr().add(index as usize) as i32 }
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
        let e = (end.max(0) as usize).min(SOURCE_LEN as usize);
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
        let e = (end.max(0) as usize).min(SOURCE_LEN as usize);
        if s >= e {
            return Vec::new();
        }
        source[s..e].to_vec()
    }
}
