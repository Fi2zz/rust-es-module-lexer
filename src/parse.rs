use crate::comment::{blockComment, lineComment};
use crate::export::{self, tryParseExportStatement};
use crate::helper;
use crate::helper::ParseResult;
use crate::helper::syntaxError;
use crate::import::{self, tryParseImportStatement};
use crate::lexer::{self, regularExpression, stringIteral, templateString};
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
        if lexer::templateDepth != -1 || lexer::openTokenDepth != 0 {
            syntaxError();
        }
        return (import::getImports(), export::getExports(), lexer::facade);
    }
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
                if helper::keywordStart() && source::starts_with("mport", position::position() + 1)
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
        match ch {
            /*e*/
            101 => {
                if unsafe { lexer::openTokenDepth } == 0
                    && helper::keywordStart()
                    && source::starts_with("xport", position::position() + 1)
                {
                    tryParseExportStatement();
                }
            }
            /*i*/
            105 => {
                if helper::keywordStart() && source::starts_with("mport", position::position() + 1)
                {
                    tryParseImportStatement();
                }
            }
            /*c*/
            99 => {
                if helper::keywordStart()
                    && source::starts_with("lass", position::position() + 1)
                    && helper::isBrOrWs(source::charCodeAt(position::position() + 5))
                {
                    unsafe { lexer::nextBraceIsClass = true };
                }
            }
            /*(*/
            40 => unsafe {
                lexer::openTokenPosStack[lexer::openTokenDepth as usize] = lexer::lastTokenPos;
                lexer::openTokenDepth += 1;
            },
            /*)*/
            41 => closeParen(),
            /*{*/
            123 => openBrace(),
            /*}*/
            125 => closeBrace(),
            /*'*/ /*"*/
            39 | 34 => stringIteral(ch),
            // 47 is /
            47 => {
                if skipComment() {
                    // dont update lastToken
                    continue;
                }
                if slashStartsRegex() {
                    regularExpression();
                    unsafe { lexer::lastSlashWasDivision = false };
                } else {
                    unsafe { lexer::lastSlashWasDivision = true };
                }
            }
            /*`*/
            96 => templateString(),
            _ => {}
        }
        unsafe { lexer::lastTokenPos = position::position() };
    }
}

#[allow(non_snake_case)]
fn closeParen() {
    unsafe {
        if lexer::openTokenDepth == 0 {
            syntaxError();
        }
        lexer::openTokenDepth -= 1;
        let cur = import::getCurDynamicImport();
        if cur >= 0
            && import::getImport(cur as usize).d
                == lexer::openTokenPosStack[lexer::openTokenDepth as usize]
        {
            let impt = cur as usize;
            if import::getImport(impt).e == 0 {
                import::updateImport(impt, |i| i.e = position::position());
            }
            import::updateImport(impt, |i| i.se = position::position());
            import::setCurDynamicImport(-1);
        }
    }
}

#[allow(non_snake_case)]
fn openBrace() {
    unsafe {
        // dynamic import followed by { is not a dynamic import (so remove)
        // this is a sneaky way to get around { import () {} } v { import () }
        // block / object ambiguity without a parser (assuming source is valid)
        if source::charCodeAt(lexer::lastTokenPos) == 41 /*)*/
            && import::importsLen() > 0
            && import::getImport(import::importsLen() - 1).e == lexer::lastTokenPos
        {
            import::popImport();
        }
        lexer::openClassPosStack[lexer::openTokenDepth as usize] = lexer::nextBraceIsClass;
        lexer::nextBraceIsClass = false;
        lexer::openTokenPosStack[lexer::openTokenDepth as usize] = lexer::lastTokenPos;
        lexer::openTokenDepth += 1;
    }
}

#[allow(non_snake_case)]
fn closeBrace() {
    unsafe {
        if lexer::openTokenDepth == 0 {
            syntaxError();
        }
        // JS: if (openTokenDepth-- === templateDepth)
        let closingDepth = lexer::openTokenDepth;
        lexer::openTokenDepth -= 1;
        if closingDepth == lexer::templateDepth {
            lexer::templateStackDepth -= 1;
            lexer::templateDepth = lexer::templateStack[lexer::templateStackDepth as usize];
            templateString();
        } else if lexer::templateDepth != -1 && lexer::openTokenDepth < lexer::templateDepth {
            syntaxError();
        }
    }
}

// returns true when the slash turned out to be a comment (and was consumed)
#[allow(non_snake_case)]
fn skipComment() -> bool {
    let next_ch = source::charCodeAt(position::position() + 1);
    if next_ch == 47 {
        lineComment();
        return true;
    }
    if next_ch == 42 {
        blockComment(true);
        return true;
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
        let lastToken = source::charCodeAt(lexer::lastTokenPos);
        let prevToken = source::charCodeAt(lexer::lastTokenPos - 1);
        let openTokenPos = lexer::openTokenPosStack[lexer::openTokenDepth as usize];
        return (helper::isExpressionPunctuator(lastToken)
            && !(lastToken == 46 /*.*/ && prevToken >= 48 /*0*/ && prevToken <= 57 /*9*/)
            && !(lastToken == 43 /*+*/ && prevToken == 43 /*+*/)
            && !(lastToken == 45 /*-*/ && prevToken == 45 /*-*/))
            || (lastToken == 41 /*)*/ && helper::isParenKeyword(openTokenPos))
            || (lastToken == 125 /*}*/
                && (helper::isExpressionTerminator(openTokenPos)
                    || lexer::openClassPosStack[lexer::openTokenDepth as usize]))
            || (lastToken == 47 && lexer::lastSlashWasDivision) // 47 is /
            || helper::isExpressionKeyword(lexer::lastTokenPos)
            || lastToken == 0;
    }
}
