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
#[allow(non_upper_case_globals)]
pub static mut openTokenStack: Vec<OpenToken> = Vec::new();
#[allow(non_upper_case_globals)]
pub static mut nextBraceIsClass: bool = false;
#[allow(non_upper_case_globals)]
pub static mut lastSlashWasDivision: bool = false;
#[allow(non_upper_case_globals)]
pub static mut facade: bool = true;
#[allow(non_upper_case_globals)]
pub static mut lastTokenPos: i32 = -1;
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
        openTokenStack = vec![OpenToken { token: OpenTokenState::AnyParen, pos: 0 }; 1024];
    }
}

#[allow(non_snake_case)]
pub fn getFacade() -> bool {
    unsafe { facade }
}

#[allow(non_snake_case)]
pub fn stringIteral(quote: i32) {
    while source::posIncLtEnd() {
        let mut ch = source::charCodeAt(position::position());
        if ch == quote {
            return;
        }
        /*\*/
        if ch == 92 {
            position::next();
            ch = source::charCodeAt(position::position());
            /*\r*/ /*\n*/
            if ch == 13 && source::charCodeAt(position::position() + 1) == 10 {
                position::next();
            }
        } else if helper::isBr(ch) {
            break;
        }
    }
    helper::syntaxError();
}

#[allow(non_snake_case)]
pub fn templateString() {
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        /*$*/ /*{*/
        if ch == 36 && source::charCodeAt(position::position() + 1) == 123 {
            position::next();
            unsafe {
                openTokenStack[openTokenDepth as usize] =
                    OpenToken { token: OpenTokenState::TemplateBrace, pos: position::position() };
                openTokenDepth += 1;
            }
            return;
        }
        /*`*/
        if ch == 96 {
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
            position::next();
        }
    }
    helper::syntaxError();
}

// pos AT the opening backtick. A no-substitution template literal (no ${...})
// is a constant string, so a dynamic import can record it as a safe specifier.
// On success consumes it, leaves pos AT the closing backtick and returns true.
// On a substitution or EOF restores pos and returns false, leaving the literal
// to the main loop's template handling.
#[allow(non_snake_case)]
pub fn noSubstitutionTemplate() -> bool {
    let startPos = position::position();
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        /*`*/
        if ch == 96 {
            return true;
        }
        /*\*/
        if ch == 92 {
            position::next();
            continue;
        }
        /*$*/ /*{*/
        if ch == 36 && source::charCodeAt(position::position() + 1) == 123 {
            break;
        }
    }
    position::setPos(startPos);
    return false;
}

#[allow(non_snake_case)]
pub fn regexCharacterClass() {
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        /*]*/
        if ch == 93 {
            return;
        }
        /*\*/
        if ch == 92 {
            position::next();
        } else if ch == 10 || ch == 13 {
            break;
        }
    }
    helper::syntaxError();
}

#[allow(non_snake_case)]
pub fn regularExpression() {
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        // 47 is /
        if ch == 47 {
            return;
        }
        /*[*/
        if ch == 91 {
            regexCharacterClass();
        } else if ch == 92 {
            position::next();
        } else if ch == 10 || ch == 13 {
            break;
        }
    }
    helper::syntaxError();
}

#[allow(non_snake_case)]
pub fn readToWsOrPunctuator(ch: i32) -> i32 {
    let mut ch = ch;
    loop {
        if helper::isBrOrWs(ch) || helper::isPunctuator(ch) {
            return ch;
        }
        position::next();
        ch = source::charCodeAt(position::position());
        // C: while (ch = *(++pos)) stops on the null terminator
        if ch == 0 {
            return ch;
        }
    }
}

/*
 * Ported from Acorn (MIT License, Copyright (C) 2012-2020 by various
 * contributors) — same as the reference lexer.js. Used to emulate the
 * wrapper's eval() decoding of string / no-substitution template literals.
 */
// start AT the opening quote ('"'), end one past the closing quote;
// returns None where JS eval would throw
#[allow(non_snake_case)]
pub fn evalLiteral(start: i32, _end: i32) -> Option<String> {
    let quote = source::charCodeAt(start);
    std::panic::catch_unwind(|| readString(start + 1, quote)).ok()
}

#[allow(non_snake_case)]
pub fn readString(start: i32, quote: i32) -> String {
    unsafe {
        acornPos = start;
    }
    let mut out: Vec<u16> = Vec::new();
    let mut chunkStart = start;
    loop {
        let acorn_pos = unsafe { acornPos };
        if acorn_pos >= source::len() {
            helper::syntaxError();
        }
        let ch = source::charCodeAt(acorn_pos);
        if ch == quote {
            break;
        }
        // '\'
        if ch == 92 {
            pushChunk(&mut out, chunkStart, acorn_pos);
            out.extend(readEscapedChar());
            chunkStart = unsafe { acornPos };
        } else if ch == 0x2028 || ch == 0x2029 {
            unsafe { acornPos += 1 };
        } else {
            // raw line breaks are only permitted inside template literals
            if helper::isBr(ch) && quote != 96 {
                helper::syntaxError();
            }
            unsafe { acornPos += 1 };
        }
    }
    let acorn_pos = unsafe { acornPos };
    pushChunk(&mut out, chunkStart, acorn_pos);
    unsafe { acornPos += 1 };
    return String::from_utf16_lossy(&out);
}

#[allow(non_snake_case)]
fn pushChunk(out: &mut Vec<u16>, start: i32, end: i32) {
    out.extend(source::units(start, end));
}

// Used to read escaped characters
#[allow(non_snake_case)]
fn readEscapedChar() -> Vec<u16> {
    unsafe { acornPos += 1 };
    let ch = source::charCodeAt(unsafe { acornPos });
    unsafe { acornPos += 1 };
    match ch {
        110 => return vec![10], // 'n' -> '\n'
        114 => return vec![13], // 'r' -> '\r'
        120 => return vec![readHexChar(2) as u16], // 'x'
        117 => return readCodePointToString(), // 'u'
        116 => return vec![9],  // 't' -> '\t'
        98 => return vec![8],   // 'b' -> '\b'
        118 => return vec![11], // 'v' -> '\u000b'
        102 => return vec![12], // 'f' -> '\f'
        13 => {
            if source::charCodeAt(unsafe { acornPos }) == 10 {
                unsafe { acornPos += 1 }; // '\r\n'
            }
            return vec![];
        }
        10 => return vec![], // ' \n'
        56 | 57 => helper::syntaxError(),
        _ => {
            if ch >= 48 && ch <= 55 {
                return readOctalChar();
            }
            if helper::isBr(ch) {
                // Unicode new line characters after \ get removed from output in both
                // template literals and strings
                return vec![];
            }
            return vec![ch as u16];
        }
    }
}

// octal escape: up to 3 chars [0-7] starting at acornPos - 1
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
    let ch = source::charCodeAt(unsafe { acornPos });
    let onlyZero = octalStrLen == 1 && source::charCodeAt(first) == 48;
    if !onlyZero || ch == 56 || ch == 57 {
        helper::syntaxError();
    }
    return vec![octal as u16];
}

// Used to read character escape sequences ('\x', '\u', '\U').
#[allow(non_snake_case)]
fn readHexChar(len: i32) -> u32 {
    let start = unsafe { acornPos };
    let mut total: u32 = 0;
    let mut lastCode: i32 = 0;
    let mut i = 0;
    while i < len {
        let code = source::charCodeAt(unsafe { acornPos });
        if code == 95 {
            if lastCode == 95 || i == 0 {
                helper::syntaxError();
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
        helper::syntaxError();
    }
    return total;
}

// Read a string value, interpreting backslash-escapes.
#[allow(non_snake_case)]
fn readCodePointToString() -> Vec<u16> {
    let ch = source::charCodeAt(unsafe { acornPos });
    let code: u32;
    // '{'
    if ch == 123 {
        unsafe { acornPos += 1 };
        let close = indexOfCloseBrace(unsafe { acornPos });
        match close {
            Some(close) => {
                code = readHexChar(close - unsafe { acornPos });
            }
            // JS: indexOf returns -1, giving a negative len that fails readHexChar
            None => helper::syntaxError(),
        }
        unsafe { acornPos += 1 };
        if code > 0x10ffff {
            helper::syntaxError();
        }
    } else {
        code = readHexChar(4);
    }
    // UTF-16 Decoding
    if code <= 0xffff {
        return vec![code as u16];
    }
    let code = code - 0x10000;
    return vec![((code >> 10) + 0xd800) as u16, ((code & 1023) + 0xdc00) as u16];
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
