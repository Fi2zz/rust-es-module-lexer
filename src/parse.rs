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
        if lexer::openTokenDepth != 0 || import::dynamicImportStackLen() != 0 {
            syntaxError();
        }
    }
    return (import::getImports(), export::getExports(), lexer::getFacade());
}

#[allow(non_snake_case)]
fn moduleOnlyParse() {
    // as soon as we hit a non-module token, we go to main parser
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        if ch == 32 || (ch < 14 && ch > 8) {
            continue;
        }
        match ch {
            /*e*/
            101 => {
                if unsafe { lexer::openTokenDepth } == 0
                    && helper::keywordStart()
                    && source::starts_with("xport", position::position() + 1)
                {
                    tryParseExportStatement();
                    // export might have been a non-pure declaration
                    if unsafe { !lexer::facade } {
                        unsafe { lexer::lastTokenPos = position::position() };
                        break;
                    }
                }
            }
            /*i*/
            105 => {
                if source::charCodeAt(position::position() + 1) == 109 /*m*/
                    && helper::keywordStart()
                    && source::starts_with("port", position::position() + 2)
                {
                    tryParseImportStatement();
                }
            }
            /*;*/
            59 => {}
            // 47 is /
            47 => {
                let next_ch = source::charCodeAt(position::position() + 1);
                if next_ch == 47 {
                    lineComment();
                    // dont update lastToken
                    continue;
                } else if next_ch == 42 {
                    blockComment(true);
                    // dont update lastToken
                    continue;
                }
                // fallthrough
                unsafe { lexer::facade = false };
                position::prev();
                break;
            }
            _ => {
                unsafe { lexer::facade = false };
                position::prev();
                break;
            }
        }
        unsafe { lexer::lastTokenPos = position::position() };
    }
}

#[allow(non_snake_case)]
fn mainParse() {
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        if ch == 32 || (ch < 14 && ch > 8) {
            continue;
        }
        consumeToken(ch);
    }
}

// Consume one token at the current ch/pos, updating the global tokenizer state.
// The single source of tokenization: the main loop and skipExpression both call
// it, so the regex/keyword/import rules never diverge. Comments do not advance
// lastTokenPos. Syntax errors panic (C returns false up to parse()).
#[allow(non_snake_case)]
pub fn consumeToken(ch: i32) {
    let mut isComment = false;
    match ch {
        /*e*/
        101 => {
            if unsafe { lexer::openTokenDepth } == 0
                && helper::keywordStart()
                && source::starts_with("xport", position::position() + 1)
            {
                tryParseExportStatement();
            } else {
                skipTokenRun();
            }
        }
        /*i*/
        105 => {
            if source::charCodeAt(position::position() + 1) == 109 /*m*/
                && helper::keywordStart()
                && source::starts_with("port", position::position() + 2)
            {
                tryParseImportStatement();
            } else {
                skipTokenRun();
            }
        }
        /*c*/
        99 => {
            if source::charCodeAt(position::position() + 1) == 108 /*l*/
                && helper::keywordStart()
                && source::starts_with("ass", position::position() + 2)
                && helper::isBrOrWs(source::charCodeAt(position::position() + 5))
            {
                unsafe { lexer::nextBraceIsClass = true };
            }
            skipTokenRun();
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
                syntaxError();
            }
            lexer::openTokenDepth -= 1;
        },
        /*,*/
        44 => commaToken(),
        /*)*/
        41 => closeParen(),
        /*{*/
        123 => openBrace(),
        /*}*/
        125 => unsafe {
            if lexer::openTokenDepth == 0 {
                syntaxError();
            }
            lexer::openTokenDepth -= 1;
            if lexer::openTokenStack[lexer::openTokenDepth as usize].token
                == OpenTokenState::TemplateBrace
            {
                templateString();
            }
        },
        /*'*/ /*"*/
        39 | 34 => stringIteral(ch),
        // 47 is /
        47 => {
            isComment = handleSlash();
        }
        /*`*/
        96 => {
            unsafe {
                lexer::openTokenStack[lexer::openTokenDepth as usize] =
                    OpenToken { token: OpenTokenState::Template, pos: lexer::lastTokenPos };
                lexer::openTokenDepth += 1;
            }
            templateString();
        }
        _ => {
            if helper::isTokenRunChar(ch) {
                skipTokenRun();
            }
        }
    }
    if !isComment {
        unsafe { lexer::lastTokenPos = position::position() };
    }
}

#[allow(non_snake_case)]
fn skipTokenRun() {
    while helper::isTokenRunChar(source::charCodeAt(position::position() + 1)) {
        position::next();
    }
}

#[allow(non_snake_case)]
fn commaToken() {
    unsafe {
        if import::dynamicImportStackLen() > 0
            && lexer::openTokenDepth > 0
            && lexer::openTokenStack[(lexer::openTokenDepth - 1) as usize].token
                == OpenTokenState::ImportParen
        {
            let cur = import::topDynamicImport();
            if import::getState(cur).end == 0 {
                import::updateState(cur, |i| i.end = lexer::lastTokenPos + 1);
                position::next();
                crate::comment::commentWhitespace(true);
                let attrPos = position::position();
                import::updateState(cur, |i| i.attr_index = attrPos);
                position::prev();
            }
        }
    }
}

#[allow(non_snake_case)]
fn closeParen() {
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
            if import::getState(cur).end == 0 {
                import::updateState(cur, |i| i.end = lexer::lastTokenPos + 1);
            }
            let se = position::position() + 1;
            import::updateState(cur, |i| i.statement_end = se);
            import::popDynamicImport();
        }
    }
}

#[allow(non_snake_case)]
fn openBrace() {
    unsafe {
        // dynamic import followed by { is not a dynamic import (so remove)
        // this is a sneaky way to get around { import () {} } v { import () }
        // block / object ambiguity without a parser (assuming source is valid)
        // statement_end (the char after the closing paren) identifies that paren;
        // end is moved before the first comma for import(a, b), so it can't be used here
        if source::charCodeAt(lexer::lastTokenPos) == 41 /*)*/
            && import::importsLen() > 0
            && import::getState(import::importsLen() - 1).statement_end == lexer::lastTokenPos + 1
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

// Division / regex ambiguity + comment dispatch, shared so skipExpression
// resolves '/' with the exact main-loop logic. Returns true for a comment
// (caller must not update lastTokenPos).
#[allow(non_snake_case)]
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
    // match the main loop exactly.
    let baseDepth = unsafe { lexer::openTokenDepth };
    let mut lastWasValue = false;
    unsafe { lexer::lastTokenPos = position::position() };
    while source::posIncLtEnd() {
        let ch = source::charCodeAt(position::position());
        if helper::isWsNotBr(ch) {
            continue;
        }
        if unsafe { lexer::openTokenDepth } == baseDepth {
            if ch == 44 /*,*/ || ch == 59 /*;*/ || ch == 41 /*)*/ || ch == 93 /*]*/ || ch == 125
            /*}*/
            {
                return ch;
            }
            if asi && lastWasValue && helper::isBr(ch) {
                return ch;
            }
        }
        if helper::isBr(ch) {
            continue;
        }
        let before = unsafe { lexer::lastTokenPos };
        consumeToken(ch);
        if unsafe { lexer::lastTokenPos } == before {
            // a comment: a line comment can land on the ASI-terminating line break
            if asi
                && unsafe { lexer::openTokenDepth } == baseDepth
                && lastWasValue
                && helper::isBr(source::charCodeAt(position::position()))
            {
                return source::charCodeAt(position::position());
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
    return 0;
}
