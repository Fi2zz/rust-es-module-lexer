use crate::comment::{blockComment, lineComment};
use crate::export::tryParseExportStatement;
use crate::helper;
use crate::helper::syntaxError;
use crate::helper::ParseResult;
use crate::import::{getImportByIndex, importsCount, popImport, tryParseImportStatement};
use crate::lexer;
use crate::lexer::getCurDynamicImport;
use crate::lexer::setCurDynamicImport;
use crate::position;
use crate::source;
use std::i32::MIN;

use crate::export;
use crate::import;
use crate::lexer::getFacade;
#[allow(non_snake_case)]
fn getResult() -> ParseResult {
    (import::getImports(), export::getExports(), getFacade())
}
pub fn parse() -> helper::ParseResult {
    let ch: usize = pre_parse();
    main_parse(ch);
    getResult()
}

fn pre_parse() -> usize {
    let mut pos = position::position();
    let mut ch: usize = 0;
    while source::ifNotEnd(pos) {
        pos = position::next();
        ch = source::charCodeAt(pos);
        if lexer::skip(ch) {
            continue;
        };
        match ch {
            /*e*/
            101 => {
                //export
                if lexer::openTokenDepthEqual(0)
                    && helper::iskeyword_start(ch)
                    && helper::is_export()
                {
                    tryParseExportStatement();
                    if !lexer::getFacade() {
                        lexer::setLastTokenPos(pos as i32);
                    }
                }
            }
            /*i*/
            105 => {
                if helper::iskeyword_start(ch) && helper::is_import() {
                    tryParseImportStatement();
                }
            }
            /*;*/
            59 => {}
            /* / */
            47 => {
                let next_ch = source::charCodeAt(position::position());
                /* / */
                if next_ch == 47 {
                    lineComment();
                    // dont update lastToken
                    continue;
                } else
                /***/
                if next_ch == 42 {
                    blockComment(true);
                    // dont update lastToken
                    continue;
                }
            }
            _ => lexer::setFacade(false),
        };
        lexer::setLastTokenPos(pos as i32);
    }

    ch
}

fn main_parse(ch: usize) {
    let mut ch = ch;
    let mut pos = position::position();
    while source::ifNotEnd(pos) {
        pos = position::next();
        ch = source::charCodeAt(pos);
        if lexer::skip(ch) {
            continue;
        };
        match ch {
            /*e*/
            101 => {
                //export
                if lexer::openTokenDepthEqual(0)
                    && helper::iskeyword_start(ch)
                    && helper::is_export()
                {
                    tryParseExportStatement();
                }
            }
            /*i*/
            105 => {
                if helper::iskeyword_start(ch) && helper::is_import() {
                    tryParseImportStatement();
                }
            }
            /*(*/
            40 => {
                lexer::setOpenTokenDepth(1);
                let mut stack = lexer::getOpenClassPosStack();
                for index in 0..stack.len() {
                    if lexer::openTokenDepthEqual(index as i32) {
                        stack[index] = lexer::getLastTokenPos()
                    }
                }
                lexer::setOpenClassPosStack(stack)
                //  break;
            }
            /*)*/
            41 => {
                if lexer::openTokenDepthEqual(1) {
                    syntaxError();
                }
                let depth = lexer::setOpenTokenDepth(1);
                let mut dynamic_import = getCurDynamicImport();
                let mut found: i32 = MIN;
                let stack = lexer::getOpenClassPosStack();
                for index in 0..stack.len() {
                    if index != depth as usize {
                        continue;
                    }
                    found = stack[index];
                }

                // initial struct
                if dynamic_import.d != -99 && found != MIN && dynamic_import.d == found {
                    if dynamic_import.e == 1 {
                        dynamic_import.e = pos;
                    }
                    dynamic_import.se = pos;
                    setCurDynamicImport(dynamic_import)
                }
                break;
            }
            99 => {
                if helper::iskeyword_start(ch) && helper::iskeyword_class(pos, pos + 5) {
                    let code = source::charCodeAt(pos + 5);
                    if helper::is_br_or_whitespace(code) {
                        lexer::setNextBraceIsClass(true);
                    }
                }
                break;
            }

            /*{*/
            123 => {
                let last_token_pos = lexer::getLastTokenPos();
                // dynamic import followed by { is not a dynamic import (so remove)
                // this is a sneaky way to get around { import () {} } v { import () }
                // block / object ambiguity without a parser (assuming source is valid)
                if source::charCodeAt(last_token_pos) == 41 && importsCount() > 0 {
                    let last = getImportByIndex(-1);
                    if last.e == last_token_pos {
                        popImport()
                    }
                }

                let mut open_class_pos_stack = lexer::getOpenClassPosStack();
                for index in 0..open_class_pos_stack.len() {
                    if lexer::openTokenDepthEqual(index as i32) {
                        open_class_pos_stack[index] = lexer::nextBraceIsClassToInt();
                    }
                }
                lexer::setOpenClassPosStack(open_class_pos_stack);
                lexer::setNextBraceIsClass(false);
            }

            125 => {
                if lexer::openTokenDepthEqual(0) {
                    syntaxError()
                }
                let depth = lexer::setOpenTokenDepth(-1);
                let template_depth = lexer::getTemplateDepth();
                if depth == template_depth {
                    lexer::setOpenTokenDepth(-1);
                    let depth: usize = lexer::getTemplateDepth() as usize;
                    let mut template_stack = lexer::getTemplateStack();
                    let value = template_stack.iter_mut().nth(depth);
                    match value {
                        Some(depth) => {
                            lexer::setTemplateDepth(*depth);
                            lexer::templateString();
                        }
                        _ => {}
                    }
                } else {
                    if helper::compare(template_depth, -1, "!")
                        && lexer::openTokenDepthCompare(template_depth, "<")
                    {
                        syntaxError();
                    }
                }
            }
            39 | 34 => lexer::stringIteral(ch),
            96 => lexer::templateString(),
            _ => {}
        }
        lexer::setLastTokenPos(pos);
    }
}
