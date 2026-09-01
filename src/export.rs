use crate::comment::commentWhitespace;
use crate::helper;
use crate::helper::Exports;
use crate::import::readImportString;
use crate::lexer;
use crate::position;
use crate::source;

#[allow(non_upper_case_globals)]
static mut exports: Exports = Vec::new();

pub fn reset() {
    unsafe { exports = Vec::new() };
}
#[allow(non_snake_case)]
pub fn getExports() -> Exports {
    unsafe { exports.to_vec() }
}
// JS: exports is a Set, so duplicate names are only stored once
#[allow(non_snake_case)]
pub fn addExport(start: i32, end: i32) {
    let name = source::stringify(start, end);
    unsafe {
        if !exports.contains(&name) {
            exports.push(name);
        }
    }
}

#[allow(non_snake_case)]
pub fn tryParseExportStatement() {
    let sStartPos = position::position();
    position::step(6);
    let curPos = position::position();
    let mut ch = commentWhitespace(true);
    if position::position() == curPos && !helper::isPunctuator(ch) {
        return;
    }
    match ch {
        // export default ...
        /*d*/
        100 => {
            addExport(position::position(), position::position() + 7);
            return;
        }
        // export async? function*? name () {
        /*a*/ /*f*/
        97 | 102 => {
            if ch == 97 {
                position::step(5);
                commentWhitespace(true);
            }
            position::step(8);
            ch = commentWhitespace(true);
            /***/
            if ch == 42 {
                position::next();
                ch = commentWhitespace(true);
            }
            let startPos = position::position();
            lexer::readToWsOrPunctuator(ch);
            addExport(startPos, position::position());
            position::prev();
            return;
        }
        /*c*/
        99 => {
            if source::starts_with("lass", position::position() + 1)
                && helper::isBrOrWsOrPunctuatorNotDot(source::charCodeAt(position::position() + 5))
            {
                position::step(5);
                ch = commentWhitespace(true);
                let startPos = position::position();
                lexer::readToWsOrPunctuator(ch);
                addExport(startPos, position::position());
                position::prev();
                return;
            }
            position::step(2);
            exportVarNames();
            return;
        }
        // export var/let/const name = ...(, name = ...)+
        /*v*/ /*l*/
        118 | 108 => {
            exportVarNames();
            return;
        }
        // export {...}
        /*{*/
        123 => {
            position::next();
            ch = commentWhitespace(true);
            loop {
                let startPos = position::position();
                lexer::readToWsOrPunctuator(ch);
                let endPos = position::position();
                commentWhitespace(true);
                ch = readExportAs(startPos, endPos);
                // ,
                if ch == 44 {
                    position::next();
                    ch = commentWhitespace(true);
                }
                /*}*/
                if ch == 125 {
                    break;
                }
                if position::position() == startPos {
                    helper::syntaxError();
                }
                if position::position() > source::end() {
                    helper::syntaxError();
                }
            }
            position::next();
            ch = commentWhitespace(true);
        }
        // export *
        // export * as X
        /***/
        42 => {
            position::next();
            commentWhitespace(true);
            readExportAs(position::position(), position::position());
            ch = commentWhitespace(true);
        }
        _ => {}
    }

    // from ...
    if ch == 102 /*f*/ && source::starts_with("rom", position::position() + 1) {
        position::step(4);
        let ch = commentWhitespace(true);
        readImportString(sStartPos, ch);
    } else {
        position::prev();
    }
}

// export var/let/const name = ...(, name = ...)+
// destructured initializations not currently supported (skipped for { or [)
// also, lexing names after variable equals is skipped (export var p = function () { ... }, q = 5 skips "q")
#[allow(non_snake_case)]
fn exportVarNames() {
    position::step(2);
    unsafe { lexer::facade = false };
    loop {
        position::next();
        let mut ch = commentWhitespace(true);
        let startPos = position::position();
        ch = lexer::readToWsOrPunctuator(ch);
        // dont yet handle [ { destructurings
        if ch == 123 /*{*/ || ch == 91 /*[*/ {
            position::prev();
            return;
        }
        if position::position() == startPos {
            return;
        }
        addExport(startPos, position::position());
        ch = commentWhitespace(true);
        /*=*/
        if ch == 61 {
            position::prev();
            return;
        }
        /*,*/
        if ch != 44 {
            break;
        }
    }
    position::prev();
}

#[allow(non_snake_case)]
pub fn readExportAs(startPos: i32, endPos: i32) -> i32 {
    let mut startPos = startPos;
    let mut endPos = endPos;
    let mut ch = source::charCodeAt(position::position());
    /*a*/
    if ch == 97 {
        position::step(2);
        ch = commentWhitespace(true);
        startPos = position::position();
        lexer::readToWsOrPunctuator(ch);
        endPos = position::position();
        ch = commentWhitespace(true);
    }
    if position::position() != startPos {
        addExport(startPos, endPos);
    }
    return ch;
}
