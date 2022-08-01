use crate::comment::commentWhitespace;
use crate::helper;
use crate::helper::{Import, Imports};
use crate::lexer;
use crate::position;
use crate::source;
#[allow(non_snake_case)]
pub fn tryParseImportStatement() {
    let start_pos = position::position();
    position::step(6);
    let mut ch = commentWhitespace(true) as usize;
    match ch {
        // dynamic import
        40 => {}
        //import.meta
        46 => {
            position::next();
            ch = commentWhitespace(true) as usize;
            // import.meta indicated by d === -2
            if ch == 109 {
                let last_token_pos = lexer::getLastTokenPos();
                let pos = position::position();
                let meta = source::stringify(pos, pos + 4);
                let last = source::charCodeAt(last_token_pos);
                if helper::iskeyword_meta(&meta) && last != 46 {
                    addImport(start_pos, start_pos, pos + 4, -2);
                }
            }
            return;
        }
        34 | 39 | 123 | 42 => {
            if lexer::openTokenDepthNotEqual(0) {
                position::step(-1);
                return;
            }
            let mut pos = position::position();
            while source::ifNotEnd(pos) {
                ch = source::charCodeAt(pos);
                if ch == 39 || ch == 34 {
                    readImportString(start_pos, ch);
                    return;
                }
                pos = position::next();
            }
            helper::syntaxError();
        }
        _ => if position::position() == start_pos + 6 {},
    }
}

#[allow(non_upper_case_globals)]
pub static mut imports: helper::Imports = Vec::new();
#[allow(non_snake_case)]
pub fn readImportString(ss: i32, ch: usize) {
    let start_pos = position::next();
    if ch == 39 || ch == 34 {
        lexer::stringIteral(ch)
    } else {
        helper::syntaxError();
        return;
    }
    addImport(ss, start_pos, position::position(), -1);
}

#[allow(non_snake_case)]
pub fn getImports() -> Imports {
    unsafe { imports.to_vec() }
}
#[allow(non_snake_case)]
pub fn setImports(input: Import) {
    unsafe {
        let mut _imports = getImports();
        _imports.push(input);
        imports = _imports;
    }
}
#[allow(non_snake_case)]
pub fn addImport(ss: i32, s: i32, e: i32, d: i32) {
    let mut _import = helper::create_import(ss, s, e, d);
    setImports(_import);
}
#[allow(non_snake_case)]
pub fn popImport() {
    unsafe {
        let mut _imports = getImports();
        _imports.pop();
        imports = _imports;
    }
}
#[allow(non_snake_case)]
pub fn getImportByIndex(index: i32) -> Import {
    let impts = getImports().to_vec();
    let mut po = index as usize;
    if index as i32 == -1 {
        po = impts.len() - 1;
    }
    return impts.get(po).unwrap().clone();
}

pub fn len() -> usize {
    let im = getImports();
    im.len()
}
#[allow(non_snake_case)]
pub fn importsCount() -> usize {
    let im = getImports();
    im.len()
}
