use crate::comment::commentWhitespace;
use crate::helper;
use crate::helper::{Import, Imports};
use crate::lexer;
use crate::position;
use crate::source;

#[allow(non_upper_case_globals)]
static mut imports: Imports = Vec::new();
// index into imports, -1 = null (JS: curDynamicImport)
#[allow(non_upper_case_globals)]
static mut curDynamicImport: i32 = -1;

pub fn reset() {
    unsafe {
        imports = Vec::new();
        curDynamicImport = -1;
    }
}

#[allow(non_snake_case)]
pub fn addImport(ss: i32, s: i32, e: i32, d: i32) -> usize {
    let se = if d == -2 {
        e
    } else if d == -1 {
        e + 1
    } else {
        0
    };
    let impt = Import {
        n: None,
        ss,
        se,
        s,
        e,
        a: -1,
        d,
    };
    unsafe {
        imports.push(impt);
        return imports.len() - 1;
    }
}
#[allow(non_snake_case)]
pub fn getImports() -> Imports {
    unsafe { imports.to_vec() }
}
#[allow(non_snake_case)]
pub fn getImport(index: usize) -> Import {
    unsafe { imports[index].clone() }
}
#[allow(non_snake_case)]
pub fn updateImport<F: FnOnce(&mut Import)>(index: usize, update: F) {
    unsafe {
        update(&mut imports[index]);
    }
}
#[allow(non_snake_case)]
pub fn popImport() {
    unsafe {
        imports.pop();
    }
}
#[allow(non_snake_case)]
pub fn importsLen() -> usize {
    unsafe { imports.len() }
}
#[allow(non_snake_case)]
pub fn getCurDynamicImport() -> i32 {
    unsafe { curDynamicImport }
}
#[allow(non_snake_case)]
pub fn setCurDynamicImport(index: i32) {
    unsafe { curDynamicImport = index }
}

#[allow(non_snake_case)]
pub fn readName(index: usize) {
    let impt = getImport(index);
    let mut s = impt.s;
    if impt.d != -1 {
        s += 1;
    }
    let n = lexer::readString(s, source::charCodeAt(s - 1));
    updateImport(index, |impt| impt.n = Some(n));
}

#[allow(non_snake_case)]
pub fn tryParseImportStatement() {
    let startPos = position::position();
    position::step(6);
    let mut ch = commentWhitespace(true);
    match ch {
        // dynamic import
        /*(*/
        40 => {
            unsafe {
                lexer::openTokenPosStack[lexer::openTokenDepth as usize] = startPos;
                lexer::openTokenDepth += 1;
            }
            if source::charCodeAt(unsafe { lexer::lastTokenPos }) == 46
            /*.*/
            {
                return;
            }
            // dynamic import indicated by positive d
            let impt = addImport(startPos, position::position() + 1, 0, startPos);
            setCurDynamicImport(impt as i32);
            // try parse a string, to record a safe dynamic import string
            position::next();
            ch = commentWhitespace(true);
            if ch == 39 /*'*/ || ch == 34 /*"*/ {
                lexer::stringIteral(ch);
            } else {
                position::prev();
                return;
            }
            position::next();
            ch = commentWhitespace(true);
            /*,*/
            if ch == 44 {
                updateImport(impt, |i| i.e = position::position());
                position::next();
                commentWhitespace(true);
                updateImport(impt, |i| i.a = position::position());
                readName(impt);
                position::prev();
            /*)*/
            } else if ch == 41 {
                unsafe { lexer::openTokenDepth -= 1 };
                updateImport(impt, |i| {
                    i.e = position::position();
                    i.se = position::position();
                });
                readName(impt);
            } else {
                position::prev();
            }
            return;
        }
        // import.meta
        /*.*/
        46 => {
            position::next();
            ch = commentWhitespace(true);
            // import.meta indicated by d === -2
            if ch == 109 /*m*/
                && source::starts_with("eta", position::position() + 1)
                && source::charCodeAt(unsafe { lexer::lastTokenPos }) != 46
            /*.*/
            {
                addImport(startPos, startPos, position::position() + 4, -2);
            }
            return;
        }
        _ => {
            let statementStart = ch == 34 /*"*/ || ch == 39 /*'*/ || ch == 123 /*{*/ || ch == 42
            /***/;
            // no space after "import" -> not an import keyword
            if !statementStart && position::position() == startPos + 6 {
                return;
            }
            // import statement only permitted at base-level
            if unsafe { lexer::openTokenDepth } != 0 {
                position::prev();
                return;
            }
            while position::position() < source::end() {
                ch = source::charCodeAt(position::position());
                if ch == 39 /*'*/ || ch == 34 /*"*/ {
                    readImportString(startPos, ch);
                    return;
                }
                position::next();
            }
            helper::syntaxError();
        }
    }
}

#[allow(non_snake_case)]
pub fn readImportString(ss: i32, ch: i32) {
    let startPos = position::position() + 1;
    if ch == 39 /*'*/ || ch == 34 /*"*/ {
        lexer::stringIteral(ch);
    } else {
        helper::syntaxError();
    }
    let impt = addImport(ss, startPos, position::position(), -1);
    readName(impt);
    position::next();
    let mut ch = commentWhitespace(false);
    if ch != 97 /*a*/ || !source::starts_with("ssert", position::position() + 1) {
        position::prev();
        return;
    }
    let assertIndex = position::position();

    position::step(6);
    ch = commentWhitespace(true);
    if ch != 123
    /*{*/
    {
        position::setPos(assertIndex);
        return;
    }
    let assertStart = position::position();
    loop {
        position::next();
        ch = commentWhitespace(true);
        if ch == 39 /*'*/ || ch == 34 /*"*/ {
            lexer::stringIteral(ch);
            position::next();
            ch = commentWhitespace(true);
        } else {
            ch = lexer::readToWsOrPunctuator(ch);
        }
        if ch != 58
        /*:*/
        {
            position::setPos(assertIndex);
            return;
        }
        position::next();
        ch = commentWhitespace(true);
        if ch == 39 /*'*/ || ch == 34 /*"*/ {
            lexer::stringIteral(ch);
        } else {
            position::setPos(assertIndex);
            return;
        }
        position::next();
        ch = commentWhitespace(true);
        /*,*/
        if ch == 44 {
            position::next();
            ch = commentWhitespace(true);
            if ch == 125 {
                break;
            }
            continue;
        }
        /*}*/
        if ch == 125 {
            break;
        }
        position::setPos(assertIndex);
        return;
    }
    updateImport(impt, |i| {
        i.a = assertStart;
        i.se = position::position() + 1;
    });
}
