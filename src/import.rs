use crate::comment::commentWhitespace;
use crate::helper;
use crate::helper::{Import, Imports};
use crate::lexer;
use crate::position;
use crate::source;

// C: dynamic == STANDARD_IMPORT / IMPORT_META sentinels, else the '(' position
pub const STANDARD_IMPORT: i32 = -1;
pub const IMPORT_META: i32 = -2;

// phase types (C enum ImportType)
pub const STATIC: i32 = 1;
pub const DYNAMIC: i32 = 2;
pub const META: i32 = 3;
pub const STATIC_SOURCE_PHASE: i32 = 4;
pub const DYNAMIC_SOURCE_PHASE: i32 = 5;
pub const STATIC_DEFER_PHASE: i32 = 6;
pub const DYNAMIC_DEFER_PHASE: i32 = 7;

// raw parse-time shape; converted to the public Import in getImports()
#[derive(Debug, Clone)]
pub struct ImportState {
    pub start: i32,
    pub end: i32,            // 0 = unset
    pub statement_start: i32,
    pub statement_end: i32,  // 0 = unset
    pub attr_index: i32,     // 0 = unset
    pub dynamic: i32,        // -1 static, -2 import.meta, else '(' pos
    pub safe: bool,
    pub import_ty: i32,
    pub attributes: Vec<(i32, i32, i32, i32)>, // key_start, key_end, value_start, value_end
}

#[allow(non_upper_case_globals)]
static mut imports: Vec<ImportState> = Vec::new();
#[allow(non_upper_case_globals)]
static mut dynamicImportStack: Vec<usize> = Vec::new();

pub fn reset() {
    unsafe {
        imports = Vec::new();
        dynamicImportStack = Vec::new();
    }
}

#[allow(non_snake_case)]
pub fn addImport(ss: i32, s: i32, e: i32, d: i32) -> usize {
    let (se, t) = if d == IMPORT_META {
        (e, META)
    } else if d == STANDARD_IMPORT {
        (e + 1, STATIC)
    } else {
        (0, DYNAMIC)
    };
    let impt = ImportState {
        start: s,
        end: e,
        statement_start: ss,
        statement_end: se,
        attr_index: 0,
        dynamic: d,
        safe: d == STANDARD_IMPORT,
        import_ty: t,
        attributes: Vec::new(),
    };
    unsafe {
        imports.push(impt);
        return imports.len() - 1;
    }
}
#[allow(non_snake_case)]
pub fn getState(index: usize) -> ImportState {
    unsafe { imports[index].clone() }
}
#[allow(non_snake_case)]
pub fn updateState<F: FnOnce(&mut ImportState)>(index: usize, update: F) {
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
pub fn dynamicImportStackLen() -> usize {
    unsafe { dynamicImportStack.len() }
}
#[allow(non_snake_case)]
pub fn pushDynamicImport(index: usize) {
    unsafe { dynamicImportStack.push(index) }
}
#[allow(non_snake_case)]
pub fn popDynamicImport() {
    unsafe {
        dynamicImportStack.pop();
    }
}
#[allow(non_snake_case)]
pub fn topDynamicImport() -> usize {
    unsafe { *dynamicImportStack.last().unwrap() }
}

// converts to the output shape, mirroring the WASM getters + JS wrapper
#[allow(non_snake_case)]
pub fn getImports() -> Imports {
    unsafe { imports.iter().map(convertImport).collect() }
}

#[allow(non_snake_case)]
fn convertImport(impt: &ImportState) -> Import {
    let e = if impt.end == 0 { -1 } else { impt.end };
    let se = if impt.statement_end == 0 { -1 } else { impt.statement_end };
    let a = if impt.attr_index == 0 { -1 } else { impt.attr_index };
    // n: the wrapper evals the quoted specifier when the import is safe;
    // static slices include the quotes (s - 1 .. e + 1), dynamic slices too
    // (s .. e with s AT the quote/backtick)
    let n = if impt.safe {
        if impt.dynamic == STANDARD_IMPORT {
            lexer::evalLiteral(impt.start - 1, impt.end + 1)
        } else {
            lexer::evalLiteral(impt.start, impt.end)
        }
    } else {
        None
    };
    let at = if impt.attributes.is_empty() {
        None
    } else {
        Some(
            impt.attributes
                .iter()
                .map(|&(ks, ke, vs, ve)| (decodeAttr(ks, ke), decodeAttr(vs, ve)))
                .collect(),
        )
    };
    Import { n, t: impt.import_ty, ss: impt.statement_start, se, s: impt.start, e, a, d: impt.dynamic, at }
}

// the wrapper's s(): quoted slices are eval-decoded, falling back to the raw
// slice when the decode is empty/fails; unquoted slices pass through
#[allow(non_snake_case)]
pub fn decodeAttr(start: i32, end: i32) -> String {
    let q = source::charCodeAt(start);
    if q == 39/*'*/ || q == 34/*"*/ {
        if let Some(decoded) = lexer::evalLiteral(start, end) {
            if !decoded.is_empty() {
                return decoded;
            }
        }
    }
    source::stringify(start, end)
}

#[allow(non_snake_case)]
pub fn tryParseImportStatement() {
    let startPos = position::position();
    position::step(6);
    let mut ch = commentWhitespace(true);

    let maybePhasePos = position::position();
    let mut phase_keyword = 0;

    /*.*/
    if ch == 46 {
        position::next();
        ch = commentWhitespace(true);
        let notDot = helper::isSpread(unsafe { lexer::lastTokenPos })
            || source::charCodeAt(unsafe { lexer::lastTokenPos }) != 46;
        // import.meta indicated by d == -2
        if ch == 109 /*m*/ && source::starts_with("eta", position::position() + 1) && notDot {
            addImport(startPos, startPos, position::position() + 4, IMPORT_META);
            return;
        } else if ch == 115 /*s*/ && source::starts_with("ource", position::position() + 1) && notDot
        {
            phase_keyword = 1;
            position::step(6);
            ch = commentWhitespace(true);
        } else if ch == 100 /*d*/ && source::starts_with("efer", position::position() + 1) && notDot
        {
            phase_keyword = 2;
            position::step(5);
            ch = commentWhitespace(true);
        } else {
            return;
        }
    // import source ...
    } else if position::position() > startPos + 6
        && ch == 115 /*s*/
        && source::starts_with("ource", position::position() + 1)
        && helper::isBrOrWs(source::charCodeAt(position::position() + 6))
    {
        phase_keyword = 1;
        position::step(6);
        ch = commentWhitespace(true);
        // need a space after the source keyword, and must not be followed by from keyword
        let followedByFrom = ch == 102 /*f*/
            && source::starts_with("rom", position::position() + 1)
            && helper::isBrOrWsOrPunctuatorNotDot(source::charCodeAt(position::position() + 4));
        if position::position() == maybePhasePos + 6 || followedByFrom {
            position::setPos(maybePhasePos);
            phase_keyword = 0;
        }
    // import defer ...
    } else if position::position() > startPos + 5
        && ch == 100 /*d*/
        && source::starts_with("efer", position::position() + 1)
        && helper::isBrOrWs(source::charCodeAt(position::position() + 5))
    {
        phase_keyword = 2;
        position::step(5);
        ch = commentWhitespace(true);
        // need a * after the defer keyword
        if ch != 42
        /***/
        {
            position::setPos(maybePhasePos);
            phase_keyword = 0;
        }
    }

    // dynamic import
    if ch == 40
    /*(*/
    {
        unsafe {
            lexer::openTokenStack[lexer::openTokenDepth as usize] = lexer::OpenToken {
                token: lexer::OpenTokenState::ImportParen,
                pos: position::position(),
            };
            lexer::openTokenDepth += 1;
        }
        if source::charCodeAt(unsafe { lexer::lastTokenPos }) == 46
        /*.*/
        {
            return;
        }
        // dynamic import indicated by positive d
        let dynamicPos = position::position();
        // try parse a string, to record a safe dynamic import string
        position::next();
        ch = commentWhitespace(true);
        let impt = addImport(startPos, position::position(), 0, dynamicPos);
        if phase_keyword > 0 {
            let t = if phase_keyword == 1 { DYNAMIC_SOURCE_PHASE } else { DYNAMIC_DEFER_PHASE };
            updateState(impt, |i| i.import_ty = t);
        }
        pushDynamicImport(impt);
        if ch == 39 /*'*/ || ch == 34 /*"*/ {
            lexer::stringIteral(ch);
        } else if ch == 96 && lexer::noSubstitutionTemplate() {
            // A no-substitution template literal is a constant string, so it is
            // a safe specifier exactly like a quoted one.
        } else {
            position::prev();
            return;
        }
        position::next();
        let endPos = position::position();
        ch = commentWhitespace(true);
        /*,*/
        if ch == 44 {
            position::next();
            commentWhitespace(true);
            let attrPos = position::position();
            updateState(impt, |i| {
                i.end = endPos;
                i.attr_index = attrPos;
                i.safe = true;
            });
            position::prev();
        /*)*/
        } else if ch == 41 {
            unsafe { lexer::openTokenDepth -= 1 };
            let se = position::position() + 1;
            updateState(impt, |i| {
                i.end = endPos;
                i.statement_end = se;
                i.safe = true;
            });
            popDynamicImport();
        } else {
            position::prev();
        }
        return;
    }

    if ch == 123 /*{*/ && phase_keyword == 0 {
        // import statement only permitted at base-level
        if unsafe { lexer::openTokenDepth } != 0 {
            position::prev();
            return;
        }
        while position::position() < source::end() {
            ch = commentWhitespace(true);
            if helper::isQuote(ch) {
                lexer::stringIteral(ch);
            } else if ch == 125
            /*}*/
            {
                position::next();
                break;
            }
            position::next();
        }

        ch = commentWhitespace(true);
        if ch == 102 /*f*/ && !source::starts_with("rom", position::position() + 1) {
            helper::syntaxError();
        }

        position::step(4);
        ch = commentWhitespace(true);

        if !helper::isQuote(ch) {
            helper::syntaxError();
        }

        readImportString(startPos, ch, 0);
    } else {
        if !(ch == 34 /*"*/ || ch == 39 /*'*/ || ch == 42
        /***/)
        {
            // no space after "import" -> not an import keyword
            if position::position() == startPos + 6 {
                position::prev();
                return;
            }
        }
        // import defer * as foo mandates *;
        // import statement only permitted at base-level
        if phase_keyword == 2 && ch != 42 || unsafe { lexer::openTokenDepth } != 0 {
            position::prev();
            return;
        }
        while position::position() < source::end() {
            ch = source::charCodeAt(position::position());
            if helper::isQuote(ch) {
                readImportString(startPos, ch, phase_keyword);
                return;
            }
            position::next();
        }
        helper::syntaxError();
    }
}

#[allow(non_snake_case)]
pub fn readImportString(ss: i32, ch: i32, phase_keyword: i32) {
    let startPos = position::position() + 1;
    if ch == 39 /*'*/ || ch == 34 /*"*/ {
        lexer::stringIteral(ch);
    } else {
        helper::syntaxError();
    }
    let impt = addImport(ss, startPos, position::position(), STANDARD_IMPORT);
    if phase_keyword > 0 {
        let t = if phase_keyword == 1 { STATIC_SOURCE_PHASE } else { STATIC_DEFER_PHASE };
        updateState(impt, |i| i.import_ty = t);
    }
    position::next();
    let mut ch = commentWhitespace(false);
    // with ...
    let with = ch == 119 /*w*/
        && source::charCodeAt(position::position() + 1) == 105 /*i*/
        && source::charCodeAt(position::position() + 2) == 116 /*t*/
        && source::charCodeAt(position::position() + 3) == 104 /*h*/;
    if !with {
        position::prev();
        return;
    }
    let attrIndex = position::position();
    position::step(4);
    ch = commentWhitespace(true);
    if ch != 123
    /*{*/
    {
        position::setPos(attrIndex);
        return;
    }
    let attrStart = position::position();
    loop {
        position::next();
        ch = commentWhitespace(true);
        let key_start;
        let key_end;
        if ch == 39 /*'*/ || ch == 34 /*"*/ {
            key_start = position::position();
            lexer::stringIteral(ch);
            key_end = position::position() + 1;
            position::next();
            ch = commentWhitespace(true);
        } else {
            key_start = position::position();
            ch = lexer::readToWsOrPunctuator(ch);
            key_end = position::position();
        }
        if ch != 58
        /*:*/
        {
            position::setPos(attrIndex);
            return;
        }
        position::next();
        ch = commentWhitespace(true);
        let value_start;
        let value_end;
        if ch == 39 /*'*/ || ch == 34 /*"*/ {
            value_start = position::position();
            lexer::stringIteral(ch);
            value_end = position::position() + 1;
        } else {
            position::setPos(attrIndex);
            return;
        }
        updateState(impt, |i| i.attributes.push((key_start, key_end, value_start, value_end)));
        position::next();
        ch = commentWhitespace(true);
        /*,*/
        if ch == 44 {
            position::next();
            continue;
        }
        /*}*/
        if ch == 125 {
            break;
        }
        position::setPos(attrIndex);
        return;
    }
    let se = position::position() + 1;
    updateState(impt, |i| {
        i.attr_index = attrStart;
        i.statement_end = se;
    });
}
