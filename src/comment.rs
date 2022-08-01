use crate::helper;
use crate::position;
use crate::source;
#[allow(non_snake_case)]
pub fn commentWhitespace(br: bool) -> i32 {
    let mut ch: i32 = -1;
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        ch = source::charCodeAt(pos) as i32;
        if ch == 47 {
            let next_ch = source::charCodeAt(pos + 1) as i32;
            // line_comment
            if next_ch == 47 {
                lineComment()
            }
            // block_commnent
            else if next_ch == 42 {
                blockComment(br)
            } else {
                return ch;
            }
        } else {
            if br {
                if !helper::is_br_or_whitespace(ch as usize) {
                    return ch;
                }
            } else if !helper::is_whitespace_not_br(ch as usize) {
                return ch;
            }
        }
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
pub fn lineComment() {
    position::next();
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        let ch = source::charCodeAt(pos);
        //10 /*\n*/
        //13 /*\r*/
        if ch == 10 || ch == 13 {
            return;
        }
    }
}
#[allow(non_snake_case)]
pub fn blockComment(br: bool) {
    position::next();
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        let ch = source::charCodeAt(pos);
        if !br && helper::is_br(ch) {
            return;
        }
        let next_ch = source::charCodeAt(pos + 1);
        /*  42 is * & 47 is / */
        if ch == 42 && next_ch == 47 {
            position::next();
            return;
        }
    }
}
