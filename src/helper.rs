use crate::position;
use crate::source;
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// name (None = undefined / not a safe specifier)
    pub n: Option<String>,
    // phase type: 1=Static 2=Dynamic 3=ImportMeta 4=StaticSource 5=DynamicSource
    //   6=StaticDefer 7=DynamicDefer
    pub t: i32,
    // statement start
    pub ss: i32,
    // statement end
    pub se: i32,
    // start
    pub s: i32,
    // end
    pub e: i32,
    // "a" = attribute index, -1 for no attributes
    pub a: i32,
    // -1 static, -2 import.meta, otherwise pos of the dynamic import '('
    pub d: i32,
    // import attributes [[key, value], ...] (None = none)
    pub at: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    // exported name start/end
    pub s: i32,
    pub e: i32,
    // local name start/end (-1 when there is no local name, e.g. reexports)
    pub ls: i32,
    pub le: i32,
    // export statement start
    pub ss: i32,
    pub n: Option<String>,
    pub ln: Option<String>,
}

pub type Imports = Vec<Import>;
pub type Exports = Vec<Export>;
pub type Facade = bool;
pub type ParseResult = (Imports, Exports, Facade);

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
pub fn isSpread(pos: i32) -> bool {
    return source::charCodeAt(pos) == 46/*.*/
        && source::charCodeAt(pos - 1) == 46
        && source::charCodeAt(pos - 2) == 46;
}
#[allow(non_snake_case)]
pub fn isBrOrWsOrPunctuatorOrSpreadNotDot(pos: i32) -> bool {
    let c = source::charCodeAt(pos);
    return c > 8 && c < 14 || c == 32 || c == 160 || isPunctuator(c) && (isSpread(pos) || c != 46);
}
#[allow(non_snake_case)]
pub fn isQuote(ch: i32) -> bool {
    return ch == 39/*'*/ || ch == 34/*"*/;
}
// Fold ASCII case; NBSP is the only non-ASCII whitespace recognized here.
#[allow(non_snake_case)]
pub fn isTokenRunChar(ch: i32) -> bool {
    let folded = ((ch | 32) as u16).wrapping_sub(97) < 26;
    let digit = (ch as u16).wrapping_sub(48) < 10;
    return folded || digit
        || ch == 36/*$*/ || ch == 95/*_*/ || ch == 92/*\*/
        || ch > 127 && ch != 160;
}
// True for a char that can end a value. skipExpression uses this to tell
// division from a regex: a '/' right after a value is division.
#[allow(non_snake_case)]
pub fn isValueChar(c: i32) -> bool {
    return c >= 48 && c <= 57 || c >= 65 && c <= 90 || c >= 97 && c <= 122
        || c == 95/*_*/ || c == 36/*$*/ || c >= 128;
}
#[allow(non_snake_case)]
pub fn keywordStart() -> bool {
    let pos = position::position();
    return pos == 0 || isBrOrWsOrPunctuatorOrSpreadNotDot(pos - 1);
}
#[allow(non_snake_case)]
fn readPrecedingKeyword1(pos: i32, c1: i32) -> bool {
    if pos < 0 {
        return false;
    }
    return source::charCodeAt(pos) == c1
        && (pos == 0 || isBrOrWsOrPunctuatorNotDot(source::charCodeAt(pos - 1)));
}
#[allow(non_snake_case)]
pub fn readPrecedingKeywordn(pos: i32, m: &str) -> bool {
    let n = m.len() as i32;
    if pos - n + 1 < 0 {
        return false;
    }
    return source::starts_with(m, pos - n + 1)
        && (pos - n + 1 == 0 || isBrOrWsOrPunctuatorOrSpreadNotDot(pos - n));
}
// Detects whether the character sequence ending at `pos` (inclusive) ends a
// for-of binding. In valid JS, the for-of `of` keyword always follows a
// binding that ends with an identifier-tail char, ']', '}', or ')'.
#[allow(non_snake_case)]
pub fn isForOfBinding(pos: i32) -> bool {
    // 'of' must be a complete token: the char before 'o' must be whitespace
    // or a binding-terminator punctuator (excludes `proof / 2` etc.)
    let c = source::charCodeAt(pos);
    if !isBrOrWs(c) && c != 93/*]*/ && c != 125/*}*/ && c != 41/*)*/ {
        return false;
    }
    // Skip whitespace back to the binding's last char.
    let mut p = pos;
    while p > 0 && isBrOrWs(source::charCodeAt(p)) {
        p -= 1;
    }
    let c = source::charCodeAt(p);
    return c == 93 || c == 125 || c == 41 || !isPunctuator(c);
}
// Detects one of case, debugger, delete, do, else, in, instanceof, new,
//   return, throw, typeof, void, yield, await, break, continue
#[allow(non_snake_case)]
pub fn isExpressionKeyword(pos: i32) -> bool {
    match source::charCodeAt(pos) {
        /*d*/
        100 => match source::charCodeAt(pos - 1) {
            /*i*/ // void
            105 => readPrecedingKeywordn(pos - 2, "vo"),
            /*l*/ // yield
            108 => readPrecedingKeywordn(pos - 2, "yie"),
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
            116 => readPrecedingKeywordn(pos - 2, "dele"),
            /*u*/ // continue
            117 => readPrecedingKeywordn(pos - 2, "contin"),
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
                99 => readPrecedingKeywordn(pos - 4, "instan"),
                /*p*/ // typeof
                112 => readPrecedingKeywordn(pos - 4, "ty"),
                _ => false,
            }
        }
        /*k*/ // break
        107 => readPrecedingKeywordn(pos - 1, "brea"),
        /*n*/ // in, return
        110 => readPrecedingKeyword1(pos - 1, 105 /*i*/) || readPrecedingKeywordn(pos - 1, "retur"),
        /*o*/ // do
        111 => readPrecedingKeyword1(pos - 1, 100 /*d*/),
        /*r*/ // debugger
        114 => readPrecedingKeywordn(pos - 1, "debugge"),
        /*t*/ // await
        116 => readPrecedingKeywordn(pos - 1, "awai"),
        /*w*/
        119 => match source::charCodeAt(pos - 1) {
            /*e*/ // new
            101 => readPrecedingKeyword1(pos - 2, 110 /*n*/),
            /*o*/ // throw
            111 => readPrecedingKeywordn(pos - 2, "thr"),
            _ => false,
        },
        _ => false,
    }
}
#[allow(non_snake_case)]
pub fn isParenKeyword(curPos: i32) -> bool {
    return readPrecedingKeywordn(curPos, "while")
        || readPrecedingKeywordn(curPos, "for")
        || readPrecedingKeywordn(curPos, "if");
}
#[allow(non_snake_case)]
pub fn isBreakOrContinue(curPos: i32) -> bool {
    match source::charCodeAt(curPos) {
        /*k*/ // break
        107 => readPrecedingKeywordn(curPos - 1, "brea"),
        /*e*/
        101 => {
            if source::charCodeAt(curPos - 1) == 117
            /*u*/
            {
                // continue
                return readPrecedingKeywordn(curPos - 2, "contin");
            }
            false
        }
        _ => false,
    }
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
        /*h*/ // catch
        104 => readPrecedingKeywordn(curPos - 1, "catc"),
        /*y*/ // finally
        121 => readPrecedingKeywordn(curPos - 1, "finall"),
        /*e*/ // else
        101 => readPrecedingKeywordn(curPos - 1, "els"),
        _ => false,
    }
}
