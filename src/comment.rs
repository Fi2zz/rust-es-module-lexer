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
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let ch = source::charCodeAtUnchecked(pos);
        //10 /*\n*/ //13 /*\r*/
        if ch == 10 || ch == 13 {
            break;
        }
    }
    position::setPos(pos);
}
#[allow(non_snake_case)]
pub fn blockComment(br: bool) {
    let end = source::end();
    let mut pos = position::position() + 1;
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let ch = source::charCodeAtUnchecked(pos);
        if !br && helper::isBr(ch) {
            break;
        }
        /*  42 is * & 47 is / */
        if ch == 42 && source::charCodeAtUnchecked(pos + 1) == 47 {
            pos += 1;
            break;
        }
    }
    position::setPos(pos);
}
