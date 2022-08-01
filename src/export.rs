use crate::comment::commentWhitespace;
use crate::helper;
use crate::lexer;
use crate::position;
use crate::source;
#[allow(non_snake_case)]
pub fn tryParseExportStatement() {
    #[allow(unused_variables)]
    let s_start_pos = position::position();
    let cur_pos = position::step(6);
    let mut ch = commentWhitespace(true);
    if position::position() == cur_pos && !helper::is_punctuator(ch as usize) {
        return;
    }
    // let end = lexer::get_end();
    match ch {
        //export *
        //export * as X
        42 => {
            let pos = position::next();
            commentWhitespace(true);
            ch = readExportAs(pos, pos) as i32;
            ch = commentWhitespace(true);
        }
        // export async? function*? name () {
        /*a*/
        97 => {
            position::step(5);
            commentWhitespace(true);
        }
        // export class xxx
        // export const xxx
        /* c */
        99 => {
            let start = position::position();
            //  export class SomeClass
            //  const some
            let maybe_space = source::charCodeAt(position::dry(5));
            let matched = helper::is_br_or_ws_or_punctuator_not_dot(maybe_space);
            if !matched {
                return;
            }
            //  "class".len() ==5
            //  "const".len() ==5
            if helper::iskeyword_class(start, start + 5)
                || helper::iskeyword_const(start, start + 5)
            {
                position::step(5);
                ch = commentWhitespace(true);
                let start = position::position();
                ch = lexer::readToWhitespaceOrPunctuator(ch as usize) as i32;
                let end = position::position();
                addExport(start, end);
            }
        }
        //export default
        /* d */
        100 => {
            let start = position::position();
            let end = start + 7;
            let str = source::stringify(start, end);
            if helper::iskeyword_default(&str) {
                addExport(start, end)
            }
        }
        // export function
        // fallthrough
        /*f*/
        102 => {
            position::step(8);
            ch = commentWhitespace(true) as i32;
            /***/
            if ch == 42 {
                position::next();
                ch = commentWhitespace(true);
            }
            let start_pos = position::position();
            ch = lexer::readToWhitespaceOrPunctuator(ch as usize) as i32;
            let end = position::position();
            addExport(start_pos, end);
            position::step(-1);
        }
        109 => {
            println!("109 {}", ch)
        }
        // export let/var xxx =
        // 108  - l
        // 118  - v
        108 | 118 => {
            position::step(2);
            lexer::setFacade(false);
            loop {
                position::next();
                ch = commentWhitespace(true);
                let start_pos = position::position() as i32;
                ch = lexer::readToWhitespaceOrPunctuator(ch as usize) as i32;
                if ch == 123 || ch == 91 {
                    position::step(-1);
                    return;
                }
                let pos = position::position();
                if pos == start_pos {
                    return;
                };
                addExport(start_pos, pos);
                ch = commentWhitespace(true);
                if ch == 61 {
                    position::step(-1);
                    return;
                }
                if ch != 44 {
                    break;
                }
            }
            position::step(-1);
            return;
        }

        // export {...}
        /*{*/
        123 => {
            position::next();
            ch = commentWhitespace(true);
            /*}*/
            loop {
                let start_pos = position::position();
                ch = lexer::readToWhitespaceOrPunctuator(ch as usize) as i32;
                let end_pos = position::position();
                commentWhitespace(true);
                ch = readExportAs(start_pos, end_pos) as i32;
                // export { /* var */ }
                if ch == 47 {
                    return helper::syntaxError();
                }
                // ,
                if ch == 44 {
                    position::next();
                    ch = commentWhitespace(true);
                }
                if ch == 125 {
                    break;
                }
                let pos = position::position();
                if pos == start_pos {
                    return helper::syntaxError();
                }
                if source::ifEnd(pos) {
                    return helper::syntaxError();
                }
            }
            position::next();
            ch = commentWhitespace(true);
        }

        _ => {
            println!("default case {}", ch);
        }
    }
    {
        let start = position::position();
        let end = position::dry(5);
        let str = source::stringify(start, end);
        if ch == 42 && helper::iskeyword_from(&str) {
            position::step(5);
        // readImportString(sStartPos, commentWhitespace(true));
        // self.rei
        } else {
            position::step(-1);
        }
    }
}

use crate::helper::Exports;

#[allow(non_upper_case_globals)]
static mut exports: helper::Exports = Vec::new();
#[allow(non_snake_case)]
pub fn readExportAs(_start_pos: i32, _end_pos: i32) -> usize {
    let mut start_pos = _start_pos;
    let mut end_pos = _end_pos;
    let mut ch: usize = source::charCodeAt(position::position());
    /* a */
    if ch == 97 {
        position::step(2);
        ch = commentWhitespace(true) as usize;
        start_pos = position::position();
        ch = lexer::readToWhitespaceOrPunctuator(ch);
        end_pos = position::position();
        ch = commentWhitespace(true) as usize;
    }
    if position::position() != start_pos {
        addExport(start_pos, end_pos);
    }
    return ch;
}

#[allow(non_snake_case)]
pub fn getExports() -> Exports {
    unsafe { exports.to_vec() }
}
#[allow(non_snake_case)]
pub fn addExport(start: i32, end: i32) {
    let export = source::stringify(start, end);
    let mut _exports = getExports();
    _exports.push(export);
    unsafe { exports = _exports }
}
#[allow(non_snake_case)]
pub fn isExport() -> bool {
    let str = source::stringify(position::position(), position::dry(6));
    helper::iskeyword_export(&str)
}
