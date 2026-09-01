use crate::helper;
use crate::position;
use crate::source;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum OpenTokenState {
    AnyParen = 1,     // (
    AnyBrace = 2,     // {
    Template = 3,     // `
    TemplateBrace = 4,// ${
    ImportParen = 5,  // import()
    ClassBrace = 6,
    AsyncParen = 7,   // async()
    AnyBracket = 8,   // [
}
#[derive(Debug, Clone, Copy)]
pub struct OpenToken {
    pub token: OpenTokenState,
    pub pos: i32,
}

#[allow(non_upper_case_globals)]
pub static mut openTokenDepth: i32 = 0;
// 定长数组（对应 C 的栈上 OpenToken openTokenStack_[1024]）：槽位先写后读，
// reset 时无需清栈
#[allow(non_upper_case_globals)]
pub static mut openTokenStack: [OpenToken; 1024] =
    [OpenToken { token: OpenTokenState::AnyParen, pos: 0 }; 1024];
#[allow(non_upper_case_globals)]
pub static mut nextBraceIsClass: bool = false;
#[allow(non_upper_case_globals)]
pub static mut lastSlashWasDivision: bool = false;
#[allow(non_upper_case_globals)]
pub static mut facade: bool = true;
#[allow(non_upper_case_globals)]
pub static mut lastTokenPos: i32 = -1;
// JSX 区内容错退出（EOF 或扫描异常）时置位，parse 末尾的栈平衡检查放行
#[allow(non_upper_case_globals)]
pub static mut jsxTolerantEof: bool = false;
#[allow(non_upper_case_globals)]
static mut acornPos: i32 = 0;

// mirrors the state initialization at the top of parse() in lexer.c
pub fn reset() {
    unsafe {
        facade = true;
        openTokenDepth = 0;
        lastTokenPos = -1;
        lastSlashWasDivision = false;
        nextBraceIsClass = false;
        jsxTolerantEof = false;
    }
}

#[allow(non_snake_case)]
pub fn getFacade() -> bool {
    unsafe { facade }
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn stringIteral(quote: i32) {
    // 局部游标扫描；pos 循环内最多走到 end + 1，预读 pos + 1 最多 end + 2，
    // 都在哨兵范围内（charCodeAtUnchecked 安全）
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            helper::syntaxError();
        }
        let mut ch = source::charCodeAtUnchecked(pos);
        if ch == quote {
            break;
        }
        /*\*/
        if ch == 92 {
            pos += 1;
            ch = source::charCodeAtUnchecked(pos);
            /*\r*/ /*\n*/
            if ch == 13 && source::charCodeAtUnchecked(pos + 1) == 10 {
                pos += 1;
            }
        } else if helper::isBr(ch) {
            position::setPos(pos);
            helper::syntaxError();
        }
    }
    position::setPos(pos);
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn templateString() {
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            helper::syntaxError();
        }
        let ch = source::charCodeAtUnchecked(pos);
        /*$*/ /*{*/
        if ch == 36 && source::charCodeAtUnchecked(pos + 1) == 123 {
            pos += 1;
            position::setPos(pos);
            unsafe {
                openTokenStack[openTokenDepth as usize] =
                    OpenToken { token: OpenTokenState::TemplateBrace, pos };
                openTokenDepth += 1;
            }
            return;
        }
        /*`*/
        if ch == 96 {
            position::setPos(pos);
            unsafe {
                openTokenDepth -= 1;
                if openTokenStack[openTokenDepth as usize].token != OpenTokenState::Template {
                    helper::syntaxError();
                }
            }
            return;
        }
        /*\*/
        if ch == 92 {
            pos += 1;
        }
    }
}

// pos AT the opening backtick. A no-substitution template literal (no ${...})
// is a constant string, so a dynamic import can record it as a safe specifier.
// On success consumes it, leaves pos AT the closing backtick and returns true.
// On a substitution or EOF restores pos and returns false, leaving the literal
// to the main loop's template handling.
#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn noSubstitutionTemplate() -> bool {
    let startPos = position::position();
    let end = source::end();
    let mut pos = startPos;
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let ch = source::charCodeAtUnchecked(pos);
        /*`*/
        if ch == 96 {
            position::setPos(pos);
            return true;
        }
        /*\*/
        if ch == 92 {
            pos += 1;
            continue;
        }
        /*$*/ /*{*/
        if ch == 36 && source::charCodeAtUnchecked(pos + 1) == 123 {
            break;
        }
    }
    position::setPos(startPos);
    return false;
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn regexCharacterClass() {
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            helper::syntaxError();
        }
        let ch = source::charCodeAtUnchecked(pos);
        /*]*/
        if ch == 93 {
            break;
        }
        /*\*/
        if ch == 92 {
            pos += 1;
        } else if ch == 10 || ch == 13 {
            position::setPos(pos);
            helper::syntaxError();
        }
    }
    position::setPos(pos);
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn regularExpression() {
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            helper::syntaxError();
        }
        let ch = source::charCodeAtUnchecked(pos);
        // 47 is /
        if ch == 47 {
            break;
        }
        /*[*/
        if ch == 91 {
            position::setPos(pos);
            regexCharacterClass();
            pos = position::position();
        } else if ch == 92 {
            pos += 1;
        } else if ch == 10 || ch == 13 {
            position::setPos(pos);
            helper::syntaxError();
        }
    }
    position::setPos(pos);
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn readToWsOrPunctuator(ch: i32) -> i32 {
    // C: do { ... } while (ch = *(++pos)) —— 读到 null 终止符（哨兵 0）停止
    let mut ch = ch;
    let mut pos = position::position();
    loop {
        if helper::isBrOrWs(ch) || helper::isPunctuator(ch) {
            position::setPos(pos);
            return ch;
        }
        pos += 1;
        ch = source::charCodeAtUnchecked(pos);
        if ch == 0 {
            position::setPos(pos);
            return ch;
        }
    }
}

/*
 * Based on the Acorn string reader (MIT License, Copyright (C) 2012-2020 by
 * various contributors), adapted to emulate the wrapper's sloppy-mode eval()
 * decoding of string / no-substitution template literals: quoted strings allow
 * octal escapes, template literals normalize line endings and reject octal.
 */
// start AT the opening quote ('"'), end one past the closing quote;
// returns None where JS eval would throw
#[allow(non_snake_case)]
pub fn evalLiteral(start: i32, _end: i32) -> Option<String> {
    let quote = source::charCodeAt(start);
    // Result 而非 panic/catch_unwind：wasm32 默认 panic=abort，catch 不住
    readString(start + 1, quote).ok()
}

#[allow(non_snake_case)]
pub fn readString(start: i32, quote: i32) -> Result<String, ()> {
    // quote == 96: template literal; the escape/line-break rules then follow
    // eval() semantics for templates (no octal, line endings normalized)
    let template = quote == 96;
    unsafe {
        acornPos = start;
    }
    let mut out: Vec<u16> = Vec::new();
    let mut chunkStart = start;
    loop {
        let acorn_pos = unsafe { acornPos };
        if acorn_pos >= source::len() {
            return Err(());
        }
        let ch = source::charCodeAt(acorn_pos);
        if ch == quote {
            break;
        }
        // '\'
        if ch == 92 {
            pushChunk(&mut out, chunkStart, acorn_pos);
            out.extend(readEscapedChar(template)?);
            chunkStart = unsafe { acornPos };
        } else if ch == 0x2028 || ch == 0x2029 {
            unsafe { acornPos += 1 };
        } else if template && ch == 13 {
            // template literals normalize \r\n and lone \r to \n
            pushChunk(&mut out, chunkStart, acorn_pos);
            out.push(10);
            unsafe { acornPos += 1 };
            if source::charCodeAt(unsafe { acornPos }) == 10 {
                unsafe { acornPos += 1 };
            }
            chunkStart = unsafe { acornPos };
        } else {
            // raw line breaks are only permitted inside template literals
            if helper::isBr(ch) && !template {
                return Err(());
            }
            unsafe { acornPos += 1 };
        }
    }
    let acorn_pos = unsafe { acornPos };
    pushChunk(&mut out, chunkStart, acorn_pos);
    unsafe { acornPos += 1 };
    return Ok(String::from_utf16_lossy(&out));
}

#[allow(non_snake_case)]
fn pushChunk(out: &mut Vec<u16>, start: i32, end: i32) {
    out.extend(source::units(start, end));
}

// Used to read escaped characters. Sloppy-mode eval semantics for quoted
// strings (octal escapes allowed); template literals reject \8, \9 and octal
// escapes (eval throws, the caller maps that to None).
#[allow(non_snake_case)]
fn readEscapedChar(template: bool) -> Result<Vec<u16>, ()> {
    unsafe { acornPos += 1 };
    let ch = source::charCodeAt(unsafe { acornPos });
    unsafe { acornPos += 1 };
    match ch {
        110 => return Ok(vec![10]), // 'n' -> '\n'
        114 => return Ok(vec![13]), // 'r' -> '\r'
        120 => return Ok(vec![readHexChar(2)? as u16]), // 'x'
        117 => return readCodePointToString(), // 'u'
        116 => return Ok(vec![9]),  // 't' -> '\t'
        98 => return Ok(vec![8]),   // 'b' -> '\b'
        118 => return Ok(vec![11]), // 'v' -> '\u000b'
        102 => return Ok(vec![12]), // 'f' -> '\f'
        13 => {
            if source::charCodeAt(unsafe { acornPos }) == 10 {
                unsafe { acornPos += 1 }; // '\r\n'
            }
            return Ok(vec![]);
        }
        10 => return Ok(vec![]), // ' \n'
        // '\8' / '\9': literal chars in sloppy strings, errors in templates
        56 | 57 => {
            if template {
                return Err(());
            }
            return Ok(vec![ch as u16]);
        }
        _ => {
            if ch >= 48 && ch <= 55 {
                if template {
                    return Err(());
                }
                return Ok(readOctalChar());
            }
            if helper::isBr(ch) {
                // Unicode new line characters after \ get removed from output in both
                // template literals and strings
                return Ok(vec![]);
            }
            return Ok(vec![ch as u16]);
        }
    }
}

// sloppy-mode octal escape: up to 3 chars [0-7] starting at acornPos - 1; when
// the 3-digit value exceeds 255 only the first 2 digits form the escape (the
// last digit stays a normal char)
#[allow(non_snake_case)]
fn readOctalChar() -> Vec<u16> {
    let first = unsafe { acornPos } - 1;
    let mut octalStrLen: i32 = 0;
    while octalStrLen < 3 {
        let c = source::charCodeAt(first + octalStrLen);
        if c < 48 || c > 55 {
            break;
        }
        octalStrLen += 1;
    }
    let mut octal: u32 = 0;
    let mut i = 0;
    while i < octalStrLen {
        octal = octal * 8 + (source::charCodeAt(first + i) - 48) as u32;
        i += 1;
    }
    if octal > 255 {
        octalStrLen -= 1;
        octal >>= 3;
    }
    unsafe { acornPos += octalStrLen - 1 };
    return vec![octal as u16];
}

// Used to read character escape sequences ('\x', '\u', '\U').
#[allow(non_snake_case)]
fn readHexChar(len: i32) -> Result<u32, ()> {
    let start = unsafe { acornPos };
    let mut total: u32 = 0;
    let mut lastCode: i32 = 0;
    let mut i = 0;
    while i < len {
        let code = source::charCodeAt(unsafe { acornPos });
        if code == 95 {
            if lastCode == 95 || i == 0 {
                return Err(());
            }
            lastCode = code;
            unsafe { acornPos += 1 };
            i += 1;
            continue;
        }
        let val: i32;
        if code >= 97 {
            val = code - 97 + 10; // a
        } else if code >= 65 {
            val = code - 65 + 10; // A
        } else if code >= 48 && code <= 57 {
            val = code - 48; // 0-9
        } else {
            break;
        }
        if val >= 16 {
            break;
        }
        lastCode = code;
        total = total * 16 + val as u32;
        unsafe { acornPos += 1 };
        i += 1;
    }
    if lastCode == 95 || unsafe { acornPos } - start != len {
        return Err(());
    }
    return Ok(total);
}

// Read a string value, interpreting backslash-escapes.
#[allow(non_snake_case)]
fn readCodePointToString() -> Result<Vec<u16>, ()> {
    let ch = source::charCodeAt(unsafe { acornPos });
    let code: u32;
    // '{'
    if ch == 123 {
        unsafe { acornPos += 1 };
        let close = indexOfCloseBrace(unsafe { acornPos });
        match close {
            // 空码点 \u{}：eval 抛 Invalid Unicode escape sequence，不能解成 \0
            Some(close) if close == unsafe { acornPos } => return Err(()),
            Some(close) => {
                code = readHexChar(close - unsafe { acornPos })?;
            }
            // JS: indexOf returns -1, giving a negative len that fails readHexChar
            None => return Err(()),
        }
        unsafe { acornPos += 1 };
        if code > 0x10ffff {
            return Err(());
        }
    } else {
        code = readHexChar(4)?;
    }
    // UTF-16 Decoding
    if code <= 0xffff {
        return Ok(vec![code as u16]);
    }
    let code = code - 0x10000;
    return Ok(vec![((code >> 10) + 0xd800) as u16, ((code & 1023) + 0xdc00) as u16]);
}

#[allow(non_snake_case)]
fn indexOfCloseBrace(from: i32) -> Option<i32> {
    let mut i = from;
    while i < source::len() {
        if source::charCodeAt(i) == 125 {
            return Some(i);
        }
        i += 1;
    }
    return None;
}
/*
 * </ Acorn Port>
 */
