use crate::comment::{blockComment, lineComment};
use crate::export::{self, tryParseExportStatement};
use crate::helper;
use crate::helper::ParseResult;
use crate::helper::syntaxError;
use crate::import::{self, tryParseImportStatement};
use crate::lexer::{self, regularExpression, stringIteral, templateString, OpenToken, OpenTokenState};
use crate::position;
use crate::source;

// Note: parsing is based on the _assumption_ that the source is already valid
pub fn parse() -> ParseResult {
    position::setPos(-1);
    lexer::reset();
    import::reset();
    export::reset();

    // start with a pure "module-only" parser
    moduleOnlyParse();
    // then the main parser, continuing from the same position
    mainParse();

    unsafe {
        if !lexer::jsxTolerantEof
            && (lexer::openTokenDepth != 0 || import::dynamicImportStackLen() != 0)
        {
            syntaxError();
        }
    }
    return (import::getImports(), export::getExports(), lexer::getFacade());
}

#[allow(non_snake_case)]
fn moduleOnlyParse() {
    // as soon as we hit a non-module token, we go to main parser
    // 热循环内用局部 pos 游标，只在调用子解析器前后同步全局 _pos；
    // pos 在 [0, end] 内，charCodeAtUnchecked 安全
    let end = source::end();
    let mut pos = position::position();
    let mut brk = false;
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let ch = source::charCodeAtUnchecked(pos);
        if ch == 32 || (ch < 14 && ch > 8) {
            continue;
        }
        position::setPos(pos);
        match ch {
            /*e*/
            101 => {
                if unsafe { lexer::openTokenDepth } == 0
                    && helper::keywordStart(pos)
                    && source::starts_with("xport", pos + 1)
                {
                    tryParseExportStatement();
                    // export might have been a non-pure declaration
                    if unsafe { !lexer::facade } {
                        unsafe { lexer::lastTokenPos = position::position() };
                        brk = true;
                    }
                }
            }
            /*i*/
            105 => {
                if source::charCodeAt(pos + 1) == 109 /*m*/
                    && helper::keywordStart(pos)
                    && source::starts_with("port", pos + 2)
                {
                    tryParseImportStatement();
                }
            }
            /*;*/
            59 => {}
            // 47 is /
            47 => {
                let next_ch = source::charCodeAt(pos + 1);
                if next_ch == 47 {
                    lineComment();
                    // dont update lastToken
                    pos = position::position();
                    continue;
                } else if next_ch == 42 {
                    blockComment(true);
                    // dont update lastToken
                    pos = position::position();
                    continue;
                }
                // fallthrough
                unsafe { lexer::facade = false };
                position::prev();
                brk = true;
            }
            _ => {
                unsafe { lexer::facade = false };
                position::prev();
                brk = true;
            }
        }
        if brk {
            pos = position::position();
            break;
        }
        unsafe { lexer::lastTokenPos = position::position() };
        pos = position::position();
    }
    position::setPos(pos);
}

#[allow(non_snake_case)]
fn mainParse() {
    // 热循环内用局部 pos 游标，只在 consumeToken 前后同步全局 _pos
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let ch = source::charCodeAtUnchecked(pos);
        if ch == 32 || (ch < 14 && ch > 8) {
            continue;
        }
        // 快路径（查表）：consumeToken 的 default 分支内联，token run 整段
        // 本地跳过，省掉每 token 的全局 _pos 同步与函数调用
        if !helper::isConsumeCaseChar(ch) {
            if helper::isTokenRunChar(ch) {
                while pos < end && helper::isTokenRunChar(source::charCodeAtUnchecked(pos + 1)) {
                    pos += 1;
                }
            }
            unsafe { lexer::lastTokenPos = pos };
            continue;
        }
        pos = consumeToken(pos, ch);
    }
    position::setPos(pos);
}

// Consume one token at the given ch/pos, updating the global tokenizer state.
// The single source of tokenization: the main loop and skipExpression both call
// it, so the regex/keyword/import rules never diverge. Comments do not advance
// lastTokenPos. Syntax errors panic (C returns false up to parse()).
//
// pos 以寄存器方式进出（避免每 token 的全局 _pos 读写）；只有调用子扫描器
// （tryParse*/stringIteral/templateString/handleSlash）的分支才临时同步全局
// _pos，纯栈操作的分支（( ) [ ] { } ,）完全不碰 _pos。
#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
pub fn consumeToken(pos: i32, ch: i32) -> i32 {
    let mut pos = pos;
    let mut isComment = false;
    match ch {
        /*e*/
        101 => {
            if unsafe { lexer::openTokenDepth } == 0
                && helper::keywordStart(pos)
                && source::starts_with("xport", pos + 1)
            {
                position::setPos(pos);
                tryParseExportStatement();
                pos = position::position();
            } else {
                pos = skipTokenRunFrom(pos);
            }
        }
        /*i*/
        105 => {
            if source::charCodeAt(pos + 1) == 109 /*m*/
                && helper::keywordStart(pos)
                && source::starts_with("port", pos + 2)
            {
                position::setPos(pos);
                tryParseImportStatement();
                pos = position::position();
            } else {
                pos = skipTokenRunFrom(pos);
            }
        }
        /*c*/
        99 => {
            if source::charCodeAt(pos + 1) == 108 /*l*/
                && helper::keywordStart(pos)
                && source::starts_with("ass", pos + 2)
                && helper::isBrOrWs(source::charCodeAt(pos + 5))
            {
                unsafe { lexer::nextBraceIsClass = true };
            }
            pos = skipTokenRunFrom(pos);
        }
        /*(*/
        40 => unsafe {
            lexer::openTokenStack[lexer::openTokenDepth as usize] =
                OpenToken { token: OpenTokenState::AnyParen, pos: lexer::lastTokenPos };
            lexer::openTokenDepth += 1;
        },
        /*[*/
        91 => unsafe {
            lexer::openTokenStack[lexer::openTokenDepth as usize] =
                OpenToken { token: OpenTokenState::AnyBracket, pos: lexer::lastTokenPos };
            lexer::openTokenDepth += 1;
        },
        /*]*/
        93 => unsafe {
            if lexer::openTokenDepth == 0 {
                position::setPos(pos);
                syntaxError();
            }
            lexer::openTokenDepth -= 1;
        },
        /*,*/
        44 => pos = commaToken(pos),
        /*)*/
        41 => closeParen(pos),
        /*<*/ // JSX：仅当 '<' 处于表达式位置时进入（合法 JS 不受影响）
        60 => {
            if ltStartsJsx() {
                position::setPos(pos);
                crate::jsx::jsxScanTolerant();
                pos = position::position();
            }
        }
        /*{*/
        123 => openBrace(),
        /*}*/
        125 => {
            let isTemplateBrace = unsafe {
                if lexer::openTokenDepth == 0 {
                    position::setPos(pos);
                    syntaxError();
                }
                lexer::openTokenDepth -= 1;
                lexer::openTokenStack[lexer::openTokenDepth as usize].token
                    == OpenTokenState::TemplateBrace
            };
            if isTemplateBrace {
                position::setPos(pos);
                templateString();
                pos = position::position();
            }
        }
        /*'*/ /*"*/
        39 | 34 => {
            position::setPos(pos);
            stringIteral(ch);
            pos = position::position();
        }
        // 47 is /
        47 => {
            position::setPos(pos);
            isComment = handleSlash();
            pos = position::position();
        }
        /*`*/
        96 => {
            unsafe {
                lexer::openTokenStack[lexer::openTokenDepth as usize] =
                    OpenToken { token: OpenTokenState::Template, pos: lexer::lastTokenPos };
                lexer::openTokenDepth += 1;
            }
            position::setPos(pos);
            templateString();
            pos = position::position();
        }
        _ => {
            if helper::isTokenRunChar(ch) {
                pos = skipTokenRunFrom(pos);
            }
        }
    }
    if !isComment {
        unsafe { lexer::lastTokenPos = pos };
    }
    return pos;
}

// C: while (isTokenRunChar(*(pos + 1))) pos++;
// pos == end 时 C 读到 null 终止符（0），不是 token run char，同样停止
#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
fn skipTokenRunFrom(pos: i32) -> i32 {
    let end = source::end();
    let mut pos = pos;
    while pos < end && helper::isTokenRunChar(source::charCodeAtUnchecked(pos + 1)) {
        pos += 1;
    }
    return pos;
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
fn commaToken(pos: i32) -> i32 {
    unsafe {
        if import::dynamicImportStackLen() > 0
            && lexer::openTokenDepth > 0
            && lexer::openTokenStack[(lexer::openTokenDepth - 1) as usize].token
                == OpenTokenState::ImportParen
        {
            let cur = import::topDynamicImport();
            if import::importEnd(cur) == 0 {
                import::updateState(cur, |i| i.end = lexer::lastTokenPos + 1);
                // C: pos++; ch = commentWhitespace(true); attr_index = pos; pos--;
                position::setPos(pos + 1);
                crate::comment::commentWhitespace(true);
                let attrPos = position::position();
                import::updateState(cur, |i| i.attr_index = attrPos);
                return position::position() - 1;
            }
        }
    }
    return pos;
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
fn closeParen(pos: i32) {
    unsafe {
        if lexer::openTokenDepth == 0 {
            syntaxError();
        }
        lexer::openTokenDepth -= 1;
        if import::dynamicImportStackLen() > 0
            && lexer::openTokenStack[lexer::openTokenDepth as usize].token
                == OpenTokenState::ImportParen
        {
            let cur = import::topDynamicImport();
            if import::importEnd(cur) == 0 {
                import::updateState(cur, |i| i.end = lexer::lastTokenPos + 1);
            }
            import::updateState(cur, |i| i.statement_end = pos + 1);
            import::popDynamicImport();
        }
    }
}

#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
fn openBrace() {
    unsafe {
        // dynamic import followed by { is not a dynamic import (so remove)
        // this is a sneaky way to get around { import () {} } v { import () }
        // block / object ambiguity without a parser (assuming source is valid)
        // statement_end (the char after the closing paren) identifies that paren;
        // end is moved before the first comma for import(a, b), so it can't be used here
        if source::charCodeAt(lexer::lastTokenPos) == 41 /*)*/
            && import::importsLen() > 0
            && import::importStatementEnd(import::importsLen() - 1) == lexer::lastTokenPos + 1
        {
            import::popImport();
        }
        let token = if lexer::nextBraceIsClass {
            OpenTokenState::ClassBrace
        } else {
            OpenTokenState::AnyBrace
        };
        lexer::openTokenStack[lexer::openTokenDepth as usize] =
            OpenToken { token, pos: lexer::lastTokenPos };
        lexer::openTokenDepth += 1;
        lexer::nextBraceIsClass = false;
    }
}

// JSX 入口判定（jsx 扩展，上游无此逻辑）：合法 JS 中 '<' 作为二元运算符必有
// 左操作数，即其前一个 token 必为值；所以表达式位置（与正则起点同一套判定）
// 出现的 '<' 只会是 JSX。`<<` 移位除外（前一字符是 '<'）。
#[allow(non_snake_case)]
fn ltStartsJsx() -> bool {
    if source::charCodeAt(unsafe { lexer::lastTokenPos }) == 60 {
        return false;
    }
    slashStartsRegex() || export::lastExportCovers(unsafe { lexer::lastTokenPos })
}

// Division / regex ambiguity + comment dispatch, shared so skipExpression
// resolves '/' with the exact main-loop logic. Returns true for a comment
// (caller must not update lastTokenPos).
#[allow(non_snake_case)]
#[cfg_attr(target_arch = "wasm32", inline(always))]
fn handleSlash() -> bool {
    let next_ch = source::charCodeAt(position::position() + 1);
    if next_ch == 47 {
        lineComment();
        return true;
    }
    if next_ch == 42 {
        blockComment(true);
        return true;
    }
    if slashStartsRegex() || export::lastExportCovers(unsafe { lexer::lastTokenPos }) {
        regularExpression();
        unsafe { lexer::lastSlashWasDivision = false };
    } else {
        divisionLookback();
    }
    return false;
}

// Division / regex ambiguity handling based on checking backtrack analysis of:
// - what token came previously (lastToken)
// - if a closing brace or paren, what token came before the corresponding
//   opening brace or paren (lastOpenTokenIndex)
#[allow(non_snake_case)]
fn slashStartsRegex() -> bool {
    unsafe {
        let lastTokenPos = lexer::lastTokenPos;
        let lastToken = source::charCodeAt(lastTokenPos);
        let prevToken = source::charCodeAt(lastTokenPos - 1);
        let openToken = lexer::openTokenStack[lexer::openTokenDepth as usize];
        return (helper::isExpressionPunctuator(lastToken)
            && !(lastToken == 46 /*.*/ && prevToken >= 48 /*0*/ && prevToken <= 57 /*9*/)
            && !(lastToken == 43 /*+*/ && prevToken == 43 /*+*/)
            && !(lastToken == 45 /*-*/ && prevToken == 45 /*-*/))
            || (lastToken == 41 /*)*/ && helper::isParenKeyword(openToken.pos))
            || (lexer::openTokenDepth > 0
                && lexer::openTokenStack[(lexer::openTokenDepth - 1) as usize].token
                    == OpenTokenState::AnyParen
                && lastToken == 102 /*f*/
                && prevToken == 111 /*o*/
                && helper::isForOfBinding(lastTokenPos - 2)
                && helper::readPrecedingKeywordn(
                    lexer::openTokenStack[(lexer::openTokenDepth - 1) as usize].pos,
                    "for",
                ))
            || (lastToken == 125 /*}*/
                && (helper::isExpressionTerminator(openToken.pos)
                    || openToken.token == OpenTokenState::ClassBrace))
            || helper::isExpressionKeyword(lastTokenPos)
            || (lastToken == 47 && lexer::lastSlashWasDivision) // 47 is /
            || lastToken == 0;
    }
}

// Not a regex: walk lastTokenPos back over the current token run, and when that
// run is whitespace-separated from a break/continue keyword, it was a regex
// after all (ASI). Otherwise this was a division.
#[allow(non_snake_case)]
fn divisionLookback() {
    unsafe {
        // C: while (lastTokenPos > source && !isBrOrWsOrPunctuatorNotDot(*(--lastTokenPos)));
        while lexer::lastTokenPos > 0 {
            lexer::lastTokenPos -= 1;
            if helper::isBrOrWsOrPunctuatorNotDot(source::charCodeAt(lexer::lastTokenPos)) {
                break;
            }
        }
        if helper::isWsNotBr(source::charCodeAt(lexer::lastTokenPos)) {
            // C: while (lastTokenPos > source && isWsNotBr(*(--lastTokenPos)));
            while lexer::lastTokenPos > 0 {
                lexer::lastTokenPos -= 1;
                if !helper::isWsNotBr(source::charCodeAt(lexer::lastTokenPos)) {
                    break;
                }
            }
            if helper::isBreakOrContinue(lexer::lastTokenPos) {
                regularExpression();
                lexer::lastSlashWasDivision = false;
                return;
            }
        }
        lexer::lastSlashWasDivision = true;
    }
}

// Skips an initializer or default-value expression, returning the depth-0
// terminator (',' or ';', or an enclosing ')'/']'/'}' that the expression did
// not open) and leaving pos AT it. With `asi` set, a line break following a
// value also terminates, so the statement after an automatic semicolon is never
// read as another binding. Entry: pos AT the char before the expression (the
// '=' of an initializer, or the '[' of a computed key).
#[allow(non_snake_case)]
pub fn skipExpression(asi: bool) -> i32 {
    // Rides consumeToken (the single tokenizer) so the regex/keyword/import rules
    // match the main loop exactly. 局部游标扫描，consumeToken 前后同步全局 _pos
    let baseDepth = unsafe { lexer::openTokenDepth };
    let mut lastWasValue = false;
    let end = source::end();
    let mut pos = position::position();
    unsafe { lexer::lastTokenPos = pos };
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            return 0;
        }
        let ch = source::charCodeAtUnchecked(pos);
        if helper::isWsNotBr(ch) {
            continue;
        }
        if unsafe { lexer::openTokenDepth } == baseDepth {
            if ch == 44 /*,*/ || ch == 59 /*;*/ || ch == 41 /*)*/ || ch == 93 /*]*/ || ch == 125
            /*}*/
            {
                position::setPos(pos);
                return ch;
            }
            if asi && lastWasValue && helper::isBr(ch) {
                position::setPos(pos);
                return ch;
            }
        }
        if helper::isBr(ch) {
            continue;
        }
        let before = unsafe { lexer::lastTokenPos };
        pos = consumeToken(pos, ch);
        if unsafe { lexer::lastTokenPos } == before {
            // a comment: a line comment can land on the ASI-terminating line break
            if asi
                && unsafe { lexer::openTokenDepth } == baseDepth
                && lastWasValue
                && helper::isBr(source::charCodeAt(pos))
            {
                return source::charCodeAt(pos);
            }
        } else {
            lastWasValue = if ch == 47 {
                unsafe { !lexer::lastSlashWasDivision }
            } else {
                helper::isValueChar(ch)
                    || ch == 41 || ch == 93 || ch == 125
                    || ch == 39 || ch == 34 || ch == 96
            };
        }
    }
}
