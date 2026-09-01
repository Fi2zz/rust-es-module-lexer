use crate::comment::commentWhitespace;
use crate::helper;
use crate::helper::{Export, Exports};
use crate::import::{decodeAttr, readImportString};
use crate::lexer;
use crate::parse::skipExpression;
use crate::position;
use crate::source;

// raw parse-time shape; local_* / statement_start use -1 for the C NULL
#[derive(Debug, Clone)]
pub struct ExportState {
    pub start: i32,
    pub end: i32,
    pub local_start: i32,
    pub local_end: i32,
    pub statement_start: i32,
}

#[allow(non_upper_case_globals)]
static mut exports: Vec<ExportState> = Vec::new();
#[allow(non_upper_case_globals)]
static mut exportStatementStart: i32 = -1;

pub fn reset() {
    unsafe {
        exports = Vec::new();
        exportStatementStart = -1;
    }
}
#[allow(non_snake_case)]
pub fn addExport(start: i32, end: i32, local_start: i32, local_end: i32) {
    unsafe {
        exports.push(ExportState {
            start,
            end,
            local_start,
            local_end,
            statement_start: exportStatementStart,
        });
    }
}
#[allow(non_snake_case)]
pub fn exportsLen() -> usize {
    unsafe { exports.len() }
}
#[allow(non_snake_case)]
pub fn updateExport<F: FnOnce(&mut ExportState)>(index: usize, update: F) {
    unsafe {
        update(&mut exports[index]);
    }
}
// C: export_write_head->start/end of the most recently added export
#[allow(non_snake_case)]
pub fn lastExportCovers(pos: i32) -> bool {
    unsafe {
        match exports.last() {
            Some(e) => pos >= e.start && pos <= e.end,
            None => false,
        }
    }
}

#[allow(non_snake_case)]
pub fn getExports() -> Exports {
    unsafe {
        exports
            .iter()
            .map(|e| Export {
                s: e.start,
                e: e.end,
                ls: e.local_start,
                le: e.local_end,
                ss: e.statement_start,
                n: Some(decodeAttr(e.start, e.end)),
                ln: if e.local_start < 0 {
                    None
                } else {
                    Some(decodeAttr(e.local_start, e.local_end))
                },
            })
            .collect()
    }
}

#[allow(non_snake_case)]
pub fn tryParseExportStatement() {
    let sStartPos = position::position();
    let prevExportCount = exportsLen();

    position::step(6);
    let curPos = position::position();
    let mut ch = commentWhitespace(true);
    if position::position() == curPos && !helper::isPunctuator(ch) {
        return;
    }

    // Only commit the statement start once this is a real export: skipExpression
    // re-enters here for an `export`-prefixed identifier (e.g. `exports`) in an
    // initializer, which would otherwise clobber the start for later bindings.
    unsafe { exportStatementStart = sStartPos };

    if ch == 123
    /*{*/
    {
        position::next();
        ch = commentWhitespace(true);
        loop {
            let startPos = position::position();
            if !helper::isQuote(ch) {
                lexer::readToWsOrPunctuator(ch);
            } else {
                lexer::stringIteral(ch);
                position::next();
            }
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
    // export *
    // export * as X
    } else if ch == 42
    /***/
    {
        position::next();
        commentWhitespace(true);
        readExportAs(position::position(), position::position());
        ch = commentWhitespace(true);
    } else {
        unsafe { lexer::facade = false };
        match ch {
            // export default ...
            /*d*/
            100 => {
                exportDefault();
                return;
            }
            // export async? function*? name () {
            /*a*/ /*f*/
            97 | 102 => {
                if ch == 97 {
                    position::step(5);
                    commentWhitespace(false);
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
                addExport(startPos, position::position(), startPos, position::position());
                position::prev();
                return;
            }
            /*c*/
            99 => {
                // export class name ...
                if source::starts_with("lass", position::position() + 1)
                    && helper::isBrOrWsOrPunctuatorNotDot(source::charCodeAt(
                        position::position() + 5,
                    ))
                {
                    position::step(5);
                    ch = commentWhitespace(true);
                    let startPos = position::position();
                    lexer::readToWsOrPunctuator(ch);
                    addExport(startPos, position::position(), startPos, position::position());
                    position::prev();
                    return;
                }
                position::step(2);
                exportBindings();
                return;
            }
            // export var/let/const binding (, binding)*
            /*v*/ /*l*/
            118 | 108 => {
                exportBindings();
                return;
            }
            _ => return,
        }
    }

    // from ...
    if ch == 102 /*f*/ && source::starts_with("rom", position::position() + 1) {
        position::step(4);
        let ch = commentWhitespace(true);
        readImportString(sStartPos, ch, 0);

        // There were no local names.
        for index in prevExportCount..exportsLen() {
            updateExport(index, |e| {
                e.local_start = -1;
                e.local_end = -1;
            });
        }
    } else {
        position::prev();
    }
}

// export default ...
#[allow(non_snake_case)]
fn exportDefault() {
    let startPos = position::position();
    position::step(7);
    let mut ch = commentWhitespace(true);
    let mut localName = false;
    match ch {
        // export default async? function*? name? (){}
        /*a*/
        97 => {
            if source::starts_with("sync", position::position() + 1)
                && helper::isWsNotBr(source::charCodeAt(position::position() + 5))
            {
                position::step(5);
                commentWhitespace(false);
                localName = exportDefaultFunctionTail();
            }
        }
        /*f*/
        102 => {
            localName = exportDefaultFunctionTail();
        }
        /*c*/
        99 => {
            // export default class name? {}
            if source::starts_with("lass", position::position() + 1) {
                let after = source::charCodeAt(position::position() + 5);
                if helper::isBrOrWs(after) || after == 123
                /*{*/
                {
                    position::step(5);
                    ch = commentWhitespace(true);
                    if ch != 123 {
                        localName = true;
                    }
                }
            }
        }
        _ => {}
    }
    if localName {
        let localStartPos = position::position();
        lexer::readToWsOrPunctuator(ch);
        if position::position() > localStartPos {
            addExport(startPos, startPos + 7, localStartPos, position::position());
            position::prev();
            return;
        }
    }
    addExport(startPos, startPos + 7, -1, -1);
    position::setPos(startPos + 6);
}

// the `function*? name?` tail of an export default; returns true when a local
// name follows (pos AT its first char)
#[allow(non_snake_case)]
fn exportDefaultFunctionTail() -> bool {
    if source::starts_with("unction", position::position() + 1) {
        let after = source::charCodeAt(position::position() + 8);
        if helper::isBrOrWs(after) || after == 42 || after == 40 {
            position::step(8);
            let mut ch = commentWhitespace(true);
            /***/
            if ch == 42 {
                position::next();
                ch = commentWhitespace(true);
            }
            if ch == 40
            /*(*/
            {
                return false;
            }
            return true;
        }
    }
    return false;
}

// export var/let/const binding (, binding)*  — each binding is an
// identifier or a destructuring pattern, optionally `= initializer`.
// Initializers and defaults are skipped expression-aware (see
// skipExpression) so a comma inside them does not split the binding list,
// and the list ends at ';', EOF or an ASI line break — never reading into
// the following statement.
#[allow(non_snake_case)]
fn exportBindings() {
    position::step(3);
    unsafe { lexer::facade = false };
    let mut ch = commentWhitespace(true);
    while position::position() <= source::end() {
        let bindingStart = position::position();
        ch = readBindingTarget(ch);
        if position::position() == bindingStart {
            break;
        }
        /*=*/
        if ch == 61 {
            ch = skipExpression(true);
        }
        /*,*/
        if ch != 44 {
            break;
        }
        position::next();
        ch = commentWhitespace(true);
    }
    position::prev();
}

// pos AT a binding target: an identifier or a nested '{'/'[' destructuring
// pattern. Adds the bound name(s), then skips trailing whitespace/comments and
// returns the next significant char with pos AT it. pos is left unchanged when
// no target is present (malformed input or the end of a binding list).
#[allow(non_snake_case)]
fn readBindingTarget(ch: i32) -> i32 {
    if ch == 123 /*{*/ || ch == 91 /*[*/ {
        readBindingPattern();
        position::next();
    } else {
        let nameStart = position::position();
        lexer::readToWsOrPunctuator(ch);
        if position::position() > nameStart {
            addExport(nameStart, position::position(), nameStart, position::position());
        }
    }
    return commentWhitespace(true);
}

// pos AT '{' or '['. Adds every identifier bound by the destructuring pattern,
// resolving aliases ({ a: b } adds b), defaults ({ a = 1 } adds a), rest
// (...rest adds rest) and arbitrary nesting. Leaves pos AT the matching closer.
#[allow(non_snake_case)]
fn readBindingPattern() {
    let isObject = source::charCodeAt(position::position()) == 123;
    let close = if isObject { 125 } else { 93 };
    position::next();
    let mut ch = commentWhitespace(true);
    while ch != close && position::position() <= source::end() {
        // ...rest element
        if ch == 46 && source::charCodeAt(position::position() + 1) == 46
            && source::charCodeAt(position::position() + 2) == 46
        {
            position::step(3);
            ch = commentWhitespace(true);
            ch = readBindingTarget(ch);
            continue;
        }
        if isObject {
            let keyStart = position::position();
            let mut keyEnd = position::position();
            /*[*/
            if ch == 91 {
                skipExpression(false); // computed key: pos AT matching ']'
                position::next();
                ch = commentWhitespace(true);
            } else if helper::isQuote(ch) {
                lexer::stringIteral(ch);
                position::next();
                ch = commentWhitespace(true);
            } else if ch >= 48 && ch <= 57 {
                position::next();
                ch = source::charCodeAt(position::position());
                while (ch >= 48 && ch <= 57) || ch == 46 || ch == 95
                    || ch == 101 || ch == 69 || ch == 110
                    || ch == 120 || ch == 88 || ch == 98 || ch == 66 || ch == 111 || ch == 79
                    || (ch >= 97 && ch <= 102) || (ch >= 65 && ch <= 70)
                    || ((ch == 43 || ch == 45) && {
                        let prev = source::charCodeAt(position::position() - 1);
                        prev == 101 || prev == 69
                    })
                {
                    position::next();
                    ch = source::charCodeAt(position::position());
                }
                ch = commentWhitespace(true);
            } else {
                lexer::readToWsOrPunctuator(ch);
                keyEnd = position::position();
                ch = commentWhitespace(true);
            }
            // { key: target } binds target; shorthand { key } / { key = default }
            // binds the key. A computed ([expr]) or string key has no shorthand.
            /*:*/
            if ch == 58 {
                position::next();
                ch = commentWhitespace(true);
                ch = readBindingTarget(ch);
            } else if keyEnd > keyStart {
                addExport(keyStart, keyEnd, keyStart, keyEnd);
            }
        /*,*/
        } else if ch == 44 {
            // array elision
            position::next();
            ch = commentWhitespace(true);
            continue;
        } else {
            ch = readBindingTarget(ch);
        }
        /*=*/
        if ch == 61 {
            ch = skipExpression(false); // default value
        }
        /*,*/
        if ch == 44 {
            position::next();
            ch = commentWhitespace(true);
        } else {
            break;
        }
    }
}

#[allow(non_snake_case)]
pub fn readExportAs(startPos: i32, endPos: i32) -> i32 {
    let mut startPos = startPos;
    let mut endPos = endPos;
    let hasLocal = startPos != endPos;
    let localStartPos = if hasLocal { startPos } else { -1 };
    let localEndPos = if hasLocal { endPos } else { -1 };
    let mut ch = source::charCodeAt(position::position());
    /*a*/
    if ch == 97 {
        position::step(2);
        ch = commentWhitespace(true);
        startPos = position::position();
        if !helper::isQuote(ch) {
            lexer::readToWsOrPunctuator(ch);
        } else {
            lexer::stringIteral(ch);
            position::next();
        }
        endPos = position::position();
        ch = commentWhitespace(true);
    }
    if position::position() != startPos {
        addExport(startPos, endPos, localStartPos, localEndPos);
    }
    return ch;
}
