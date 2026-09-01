use crate::helper;
use crate::position;
use crate::source;

#[allow(non_snake_case)]
pub fn commentWhitespace(br: bool) -> i32 {
    let mut ch: i32;
    loop {
        ch = source::charCodeAt(position::position());
        // 47 is /
        if ch == 47 {
            let next_ch = source::charCodeAt(position::position() + 1);
            if next_ch == 47 {
                lineComment()
            } else if next_ch == 42 {
                blockComment(br)
            } else {
                return ch;
            }
        } else {
            let skipped = if br {
                helper::isBrOrWs(ch)
            } else {
                helper::isWsNotBr(ch)
            };
            if !skipped {
                return ch;
            }
        }
        if !source::posIncLtEnd() {
            break;
        }
    }
    return ch;
}
#[allow(non_snake_case)]
pub fn lineComment() {
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        //10 /*\n*/ //13 /*\r*/
        if ch == 10 || ch == 13 {
            return;
        }
    }
}
#[allow(non_snake_case)]
pub fn blockComment(br: bool) {
    position::next();
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        if !br && helper::isBr(ch) {
            return;
        }
        /*  42 is * & 47 is / */
        if ch == 42 && source::charCodeAt(position::position() + 1) == 47 {
            position::next();
            return;
        }
    }
}
