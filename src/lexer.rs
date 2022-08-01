use crate::helper;
use crate::helper::{DynamicImport, Import};
use crate::position;
use crate::source::{self};
use std::i32::MIN;
#[allow(dead_code, non_upper_case_globals)]
static mut curDynamicImport: DynamicImport = DynamicImport {
    se: 0,
    ss: 0,
    s: 0,
    e: 0,
    a: -1,
    d: -2,
};
#[allow(dead_code, non_upper_case_globals)]
static mut acornPos: i32 = MIN;
#[allow(dead_code, non_upper_case_globals)]
pub static mut lastTokenPos: i32 = -1;
#[allow(dead_code, non_upper_case_globals)]
pub static mut openTokenDepth: i32 = 0;
#[allow(non_upper_case_globals)]
#[allow(non_upper_case_globals)]
#[allow(non_upper_case_globals)]
pub static mut templateStackDepth: i32 = 0;
#[allow(non_upper_case_globals)]
pub static mut templateDepth: i32 = -1;
#[allow(non_upper_case_globals)]
pub static mut openTokenPosStack: Vec<i32> = Vec::new();
#[allow(non_upper_case_globals)]
pub static mut openClassPosStack: Vec<i32> = Vec::new();
#[allow(non_upper_case_globals)]
pub static mut templateStack: Vec<i32> = Vec::new();
#[allow(non_upper_case_globals)]
pub static mut nextBraceIsClass: bool = false;
#[allow(non_snake_case)]
pub fn getNextBraceIsClass() -> bool {
    unsafe { nextBraceIsClass }
}
#[allow(non_snake_case)]
pub fn setNextBraceIsClass(value: bool) -> bool {
    unsafe {
        nextBraceIsClass = value;
        nextBraceIsClass
    }
}
#[allow(non_snake_case)]
pub fn nextBraceIsClassToInt() -> i32 {
    unsafe {
        if nextBraceIsClass {
            return 1;
        } else {
            return 0;
        }
    }
}

#[allow(non_upper_case_globals)]
pub static mut lastSlashWasDivision: bool = false;
#[allow(non_snake_case)]
pub fn setLastSlashWasDivision(is: bool) {
    unsafe { lastSlashWasDivision = is }
}
#[allow(non_snake_case)]
pub fn getLastSlashWasDivision() -> bool {
    unsafe { lastSlashWasDivision }
}
#[allow(non_upper_case_globals)]
pub static mut facade: bool = false;
#[allow(non_snake_case)]
pub fn setFacade(is: bool) {
    unsafe { facade = is }
}
#[allow(non_snake_case)]
pub fn getFacade() -> bool {
    unsafe { facade }
}

#[allow(non_snake_case)]
pub fn readToWhitespaceOrPunctuator(input: usize) -> usize {
    let mut ch = input;
    while !helper::is_br_or_whitespace(ch) && !helper::is_punctuator(ch) {
        let pos = position::next();
        ch = source::charCodeAt(pos);
    }
    return ch;
}
#[allow(non_snake_case)]
pub fn stringIteral(quote: usize) {
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        let mut ch = source::charCodeAt(pos) as usize;
        if ch == quote {
            return;
        }
        if ch == 92 {
            pos = position::next();
            ch = source::charCodeAt(pos);
            if ch == 13 && source::charCodeAt(pos) == 10 {
                position::next();
            }
        } else if helper::is_br(ch) {
            break;
        }
    }
    helper::syntaxError();
}

#[allow(non_snake_case)]
pub fn setLastTokenPos(pos: i32) {
    unsafe { lastTokenPos = pos };
}
#[allow(non_snake_case)]
pub fn getLastTokenPos() -> i32 {
    return unsafe { lastTokenPos };
}
#[allow(non_snake_case)]
pub fn setTemplateDepth(depth: i32) -> i32 {
    unsafe {
        templateDepth += depth;
        templateDepth
    }
}
#[allow(non_snake_case)]
pub fn getTemplateDepth() -> i32 {
    unsafe { templateDepth }
}

#[allow(non_snake_case)]
pub fn setOpenTokenDepth(depth: i32) -> i32 {
    unsafe {
        openTokenDepth += depth;
        openTokenDepth
    }
}
#[allow(non_snake_case)]
pub fn getOpenTokenDepth() -> i32 {
    unsafe { openTokenDepth }
}
#[allow(non_snake_case)]
pub fn openTokenDepthCompare(value: i32, ctype: &str) -> bool {
    unsafe {
        return helper::compare(openTokenDepth, value, ctype);
    }
}
#[allow(non_snake_case)]
pub fn openTokenDepthEqual(value: i32) -> bool {
    unsafe { openTokenDepth == value }
}
#[allow(non_snake_case)]
pub fn openTokenDepthNotEqual(value: i32) -> bool {
    unsafe { openTokenDepth != value }
}

#[allow(non_snake_case)]
pub fn getTemplateStack() -> Vec<i32> {
    let result = unsafe { templateStack.to_vec() };
    result
}

pub type Callback = dyn FnOnce(i32, usize) -> i32;
#[allow(non_snake_case)]
pub fn updateTemplateStackByIndex<F>(index: i32, callback: F)
where
    F: Fn(i32, usize) -> i32,
{
    let mut stack = getTemplateStack();
    for i in 0..stack.len() {
        if (index as usize) == i {
            let value = callback(stack[i], i);
            stack[i] = value.clone();
        }
    }
}

#[allow(non_snake_case)]
pub fn getOpenClassPosStack() -> Vec<i32> {
    unsafe { openClassPosStack.to_vec() }
}

#[allow(non_snake_case)]
pub fn setOpenClassPosStack(updated: Vec<i32>) {
    unsafe { openClassPosStack = updated }
}

#[allow(non_snake_case)]
pub fn templateString() {
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        let ch = source::charCodeAt(pos) as usize;
        if ch == 36 && source::charCodeAt(pos + 1) == 123 {
            position::next();
            let depth = setTemplateDepth(1);
            updateTemplateStackByIndex(depth, |_value: i32, _index| {
                let open_token_depth = dryOpenTokenDepth(1);
                setTemplateDepth(open_token_depth);
                depth
            });
            return;
        }
        if ch == 96 {
            return;
        }
        if ch == 92 {
            position::next();
        }
    }
    helper::syntaxError();
}

#[allow(non_snake_case)]
pub fn dryOpenTokenDepth(depth: i32) -> i32 {
    unsafe {
        let result = openTokenDepth + depth;
        result
    }
}
#[allow(non_snake_case)]
pub fn getCurDynamicImport() -> DynamicImport {
    unsafe { curDynamicImport }
}
#[allow(non_snake_case)]
pub fn setCurDynamicImport(updated: DynamicImport) {
    unsafe { curDynamicImport = updated }
}

pub fn skip(ch: usize) -> bool {
    if ch == 32 || ch < 14 && ch > 8 {
        return true;
    }
    return false;
}
#[allow(non_snake_case)]
pub fn readName(import: &mut Import) {
    let d = import.d;
    let mut s = import.s;
    if d != -1 {
        s += 1;
    }
    let next = s - 1;
    if next == 0 {
        return;
    };
    let ch = source::charCodeAt(next);
    import.n = readString(s, ch);
}
#[allow(non_snake_case)]
pub fn readString(start: i32, qoute: usize) -> String {
    let mut out: String = String::from("");
    unsafe {
        acornPos = start;
        let mut chunk_start: i32 = acornPos;
        let count = source::len() as i32;
        loop {
            if acornPos >= count {
                helper::syntaxError();
            }
            let ch = source::charCodeAt(acornPos);

            println!(
                " ACORN_POS {}  chunk_start {1} qoute {2} ch {3}",
                acornPos, chunk_start, qoute, ch,
            );
            if ch == qoute {
                break;
            }
            if ch == 92 {
                let n = source::getChar(chunk_start, acornPos);
                out.push(n);
                chunk_start = acornPos;
            } else if ch == 8232 || ch == 8233 {
                println!("ch 8232 / 8233 {0}", ch);
                acornPos += 1;
            } else {
                if helper::is_br(ch) {
                    helper::syntaxError()
                }
                acornPos += 1
            }
        }
        acornPos += 1;

        let n = source::getChar(chunk_start, acornPos);
        println!(
            "s {:?}  ACORN_POS {1}  chunk_start {2} qoute {3}",
            n, acornPos, chunk_start, qoute
        );
        out.push(n);
    }

    return out;
}
