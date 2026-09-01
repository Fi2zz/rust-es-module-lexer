use crate::position;
use crate::source;
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// name (None = undefined)
    pub n: Option<String>,
    // statement start
    pub ss: i32,
    // statement end
    pub se: i32,
    // start
    pub s: i32,
    // end
    pub e: i32,
    // "a" = assert, -1 for no assertion
    pub a: i32,
    pub d: i32,
}

pub type Imports = Vec<Import>;
pub type Exports = Vec<String>;
pub type Facade = bool;
pub type ParseResult = (Imports, Exports, Facade);
pub type Source = Vec<u8>;

#[allow(non_snake_case)]
pub fn syntaxError() -> ! {
    panic!("Parse error at {}", position::position());
}

// Note: non-asii BR and whitespace checks omitted for perf / footprint
// if there is a significant user need this can be reconsidered
#[allow(non_snake_case)]
pub fn isBr(c: i32) -> bool {
    return c == 13/*\r*/ || c == 10/*\n*/;
}
#[allow(non_snake_case)]
pub fn isWsNotBr(c: i32) -> bool {
    return c == 9 || c == 11 || c == 12 || c == 32 || c == 160;
}
#[allow(non_snake_case)]
pub fn isBrOrWs(c: i32) -> bool {
    return c > 8 && c < 14 || c == 32 || c == 160;
}
#[allow(non_snake_case)]
pub fn isPunctuator(ch: i32) -> bool {
    // 23 possible punctuator endings: !%&()*+,-./:;<=>?[]^{}|~
    return ch == 33/* !*/ || ch == 37/*%*/ || ch == 38/*&*/ ||
    ch > 39 && ch < 48 || ch > 57 && ch < 64 ||
    ch == 91/*[*/ || ch == 93/*]*/ || ch == 94/*^*/ ||
    ch > 122 && ch < 127;
}
#[allow(non_snake_case)]
pub fn isExpressionPunctuator(ch: i32) -> bool {
    // 20 possible expression endings: !%&(*+,-.:;<=>?[^{|~
    return ch == 33/* !*/ || ch == 37/*%*/ || ch == 38/*&*/ ||
    ch > 39 && ch < 47 && ch != 41/*)*/ || ch > 57 && ch < 64 ||
    ch == 91/*[*/ || ch == 94/*^*/ ||
    ch > 122 && ch < 127 && ch != 125/*}*/;
}
#[allow(non_snake_case)]
pub fn isBrOrWsOrPunctuatorNotDot(c: i32) -> bool {
    return c > 8 && c < 14 || c == 32 || c == 160 || isPunctuator(c) && c != 46/*.*/;
}
#[allow(non_snake_case)]
pub fn keywordStart() -> bool {
    let pos = position::position();
    return pos == 0 || isBrOrWsOrPunctuatorNotDot(source::charCodeAt(pos - 1));
}
#[allow(non_snake_case)]
fn readPrecedingKeyword(pos: i32, m: &str) -> bool {
    if pos < m.len() as i32 - 1 {
        return false;
    }
    return source::starts_with(m, pos - m.len() as i32 + 1)
        && (pos == 0 || isBrOrWsOrPunctuatorNotDot(source::charCodeAt(pos - m.len() as i32)));
}
#[allow(non_snake_case)]
fn readPrecedingKeyword1(pos: i32, ch: i32) -> bool {
    return source::charCodeAt(pos) == ch
        && (pos == 0 || isBrOrWsOrPunctuatorNotDot(source::charCodeAt(pos - 1)));
}
// Detects one of case, debugger, delete, do, else, in, instanceof, new,
//   return, throw, typeof, void, yield, await
#[allow(non_snake_case)]
pub fn isExpressionKeyword(pos: i32) -> bool {
    match source::charCodeAt(pos) {
        /*d*/
        100 => match source::charCodeAt(pos - 1) {
            /*i*/ // void
            105 => readPrecedingKeyword(pos - 2, "vo"),
            /*l*/ // yield
            108 => readPrecedingKeyword(pos - 2, "yie"),
            _ => false,
        },
        /*e*/
        101 => match source::charCodeAt(pos - 1) {
            /*s*/
            115 => match source::charCodeAt(pos - 2) {
                /*l*/ // else
                108 => readPrecedingKeyword1(pos - 3, 101 /*e*/),
                /*a*/ // case
                97 => readPrecedingKeyword1(pos - 3, 99 /*c*/),
                _ => false,
            },
            /*t*/ // delete
            116 => readPrecedingKeyword(pos - 2, "dele"),
            _ => false,
        },
        /*f*/
        102 => {
            if source::charCodeAt(pos - 1) != 111 /*o*/ || source::charCodeAt(pos - 2) != 101
            /*e*/
            {
                return false;
            }
            match source::charCodeAt(pos - 3) {
                /*c*/ // instanceof
                99 => readPrecedingKeyword(pos - 4, "instan"),
                /*p*/ // typeof
                112 => readPrecedingKeyword(pos - 4, "ty"),
                _ => false,
            }
        }
        /*n*/ // in, return
        110 => readPrecedingKeyword1(pos - 1, 105 /*i*/) || readPrecedingKeyword(pos - 1, "retur"),
        /*o*/ // do
        111 => readPrecedingKeyword1(pos - 1, 100 /*d*/),
        /*r*/ // debugger
        114 => readPrecedingKeyword(pos - 1, "debugge"),
        /*t*/ // await
        116 => readPrecedingKeyword(pos - 1, "awai"),
        /*w*/
        119 => match source::charCodeAt(pos - 1) {
            /*e*/ // new
            101 => readPrecedingKeyword1(pos - 2, 110 /*n*/),
            /*o*/ // throw
            111 => readPrecedingKeyword(pos - 2, "thr"),
            _ => false,
        },
        _ => false,
    }
}
#[allow(non_snake_case)]
pub fn isParenKeyword(curPos: i32) -> bool {
    return (source::charCodeAt(curPos) == 101 /*e*/ && source::starts_with("whil", curPos - 4))
        || (source::charCodeAt(curPos) == 114 /*r*/ && source::starts_with("fo", curPos - 2))
        || (source::charCodeAt(curPos - 1) == 105 /*i*/ && source::charCodeAt(curPos) == 102) /*f*/;
}
#[allow(non_snake_case)]
pub fn isExpressionTerminator(curPos: i32) -> bool {
    // detects:
    // => ; ) finally catch else
    // as all of these followed by a { will indicate a statement brace
    match source::charCodeAt(curPos) {
        /*>*/
        62 => source::charCodeAt(curPos - 1) == 61 /*=*/,
        /*;*/ /*)*/
        59 | 41 => true,
        /*h*/
        104 => source::starts_with("catc", curPos - 4),
        /*y*/
        121 => source::starts_with("finall", curPos - 6),
        /*e*/
        101 => source::starts_with("els", curPos - 3),
        _ => false,
    }
}
