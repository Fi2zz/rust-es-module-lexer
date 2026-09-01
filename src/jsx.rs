// JSX 支持（上游 es-module-lexer 明确不支持，这是我们自己的扩展）。
//
// 设计要点：
// - 始终开启，无开关。合法 JS 中 '<' 作为二元运算符必有左操作数（前一 token
//   是值），所以"表达式位置"（与正则起点同一套判定，见 parse::ltStartsJsx）
//   出现的 '<' 在合法 JS 中不存在，只会是 JSX。
// - 目的边界：不报错、顶层 import/export 提取正确、内部字符串/模板/注释/正则
//   状态不被 JSX 内容污染。不校验标签配对，不产出 JSX 结构信息。
// - 状态机：标签区 / 子节点区两个模式。表达式容器 {...} 复用主词法的
//   openTokenStack 压栈与 consumeToken（容器内可出现字符串/模板 ${}/正则/
//   注释/嵌套 JSX——嵌套 JSX 由 consumeToken 的 '<' case 递归进 jsxScan）。
// - 容错：JSX 区内 EOF 或异常输入直接结束扫描，绝不 panic（catch_unwind 兜底）。
//
// 已知限制：TSX 泛型形态（如 const f = <T>(v) => v）的 '<' 跟在 '=' 后会
// 误入 JSX，可能吞掉后续内容——本期不支持。

use crate::comment::commentWhitespace;
use crate::helper;
use crate::lexer;
use crate::parse::consumeToken;
use crate::position;
use crate::source;

#[derive(Clone, Copy, PartialEq)]
enum JsxMode {
    Tag,
    Children,
}

// 从表达式位置的 '<' 进入：pos AT '<'。扫完整个 JSX 元素后返回，
// pos 停在 JSX 结束符（'>'）。任何 panic 都被吞掉（容错兜底）。
#[allow(non_snake_case)]
pub fn jsxScanTolerant() {
    if std::panic::catch_unwind(jsxScan).is_err() {
        // 扫描中出错（如未闭合字符串/模板）：放弃这次 JSX，状态可能已经
        // 不平衡，标记后让 parse 末尾的栈平衡检查放行
        unsafe { lexer::jsxTolerantEof = true };
    }
}

#[allow(non_snake_case)]
fn jsxScan() {
    let mut depth: i32 = 0;
    let mut mode = JsxMode::Tag;
    loop {
        let next = match mode {
            JsxMode::Tag => jsxTag(&mut depth),
            JsxMode::Children => jsxChildren(&mut depth),
        };
        match next {
            Some(next) => mode = next,
            None => return,
        }
    }
}

// 标签/属性名可用字符：标识符 + 成员 '.' + 连字符 '-' + 命名空间 ':'
#[allow(non_snake_case)]
fn isJsxNameChar(ch: i32) -> bool {
    helper::isTokenRunChar(ch) || ch == 46 || ch == 45 || ch == 58
}

#[allow(non_snake_case)]
fn skipJsxName() {
    while isJsxNameChar(source::charCodeAt(position::position())) {
        position::next();
    }
}

// pos AT '<'。扫描标签名与属性区：
//   Some(Children) —— 遇到 '>'（depth 已 +1）
//   None —— 根元素 '/>'（JSX 结束）或 EOF（容错）
#[allow(non_snake_case)]
fn jsxTag(depth: &mut i32) -> Option<JsxMode> {
    position::next();
    let mut ch = source::charCodeAt(position::position());
    // fragment '<>'
    if ch == 62 {
        *depth += 1;
        return Some(JsxMode::Children);
    }
    skipJsxName();
    // 属性区
    loop {
        ch = commentWhitespace(true);
        let pos = position::position();
        if pos >= source::end() {
            return None;
        }
        match ch {
            /*>*/
            62 => {
                *depth += 1;
                return Some(JsxMode::Children);
            }
            // 47 is /
            47 => {
                if source::charCodeAt(pos + 1) == 62 {
                    position::next();
                    // 根元素自闭合 → JSX 结束；否则是子元素，回到子节点区
                    if *depth == 0 {
                        return None;
                    }
                    return Some(JsxMode::Children);
                }
                // 容错：非 '/>' 的 '/' 直接跳过
                position::next();
            }
            /*{*/ // 展开属性 {...expr}
            123 => {
                if !jsxContainer() {
                    return None;
                }
                position::next();
            }
            _ => {
                let before = position::position();
                skipJsxName();
                ch = commentWhitespace(true);
                /*=*/
                if ch == 61 {
                    position::next();
                    ch = commentWhitespace(true);
                    /*"*/ /*'*/
                    if ch == 34 || ch == 39 {
                        jsxStringLiteral(ch);
                        position::next();
                    /*{*/ // {表达式} 属性值
                    } else if ch == 123 {
                        if !jsxContainer() {
                            return None;
                        }
                        position::next();
                    } else {
                        // 容错：无引号属性值，跳过一个 token run
                        while helper::isTokenRunChar(source::charCodeAt(position::position())) {
                            position::next();
                        }
                    }
                }
                // 容错：本轮未消耗任何字符时强制前进，保证终止
                if position::position() == before {
                    position::next();
                }
            }
        }
    }
}

// 属性区里的字符串：容忍未闭合/换行（不 panic），这是与 lexer::stringIteral
// 唯一的差别
#[allow(non_snake_case)]
fn jsxStringLiteral(quote: i32) {
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            break;
        }
        let mut ch = source::charCodeAtUnchecked(pos);
        if ch == quote || helper::isBr(ch) {
            break;
        }
        /*\*/
        if ch == 92 {
            pos += 1;
            ch = source::charCodeAtUnchecked(pos);
            if ch == 13 && source::charCodeAtUnchecked(pos + 1) == 10 {
                pos += 1;
            }
        }
    }
    position::setPos(pos);
}

// pos AT 子节点区内容。扫描文本/子元素/闭合标签/容器：
//   Some(Tag) —— 子元素开始（pos AT '<'）
//   None —— 根闭合（depth 归 0，JSX 结束）或 EOF（容错）
#[allow(non_snake_case)]
fn jsxChildren(depth: &mut i32) -> Option<JsxMode> {
    let end = source::end();
    loop {
        // 文本：原样跳过直到 '<' 或 '{'（文本里的引号、斜杠、撇号都不算语法）
        let mut pos = position::position();
        while pos < end {
            let ch = source::charCodeAtUnchecked(pos + 1);
            if ch == 60 || ch == 123 {
                break;
            }
            pos += 1;
        }
        pos += 1;
        position::setPos(pos);
        if pos > end {
            return None;
        }
        /*{*/ // 表达式容器
        if source::charCodeAtUnchecked(pos) == 123 {
            if !jsxContainer() {
                return None;
            }
            continue;
        }
        // '<'
        let next_ch = source::charCodeAt(pos + 1);
        // 47 is /
        if next_ch == 47 {
            // '</' 闭合标签：跳过量名直到 '>'（不校验配对）
            loop {
                position::next();
                let ch = source::charCodeAt(position::position());
                if ch == 62 || position::position() > end {
                    break;
                }
            }
            *depth -= 1;
            if *depth == 0 {
                return None;
            }
        } else if next_ch == 62 {
            // '<>' fragment 子元素
            position::next();
            *depth += 1;
        } else if helper::isTokenRunChar(next_ch) {
            // 子元素标签
            return Some(JsxMode::Tag);
        }
        // 否则：文本里的孤立 '<'，容错跳过
    }
}

// pos AT '{'。容器内容走完整的 consumeToken 词法（可含字符串/模板/正则/注释/
// 嵌套 JSX），直到匹配的 '}' 弹栈。返回 false 表示 EOF 容错退出。
// 注意容器不是块语句：压栈不做 openBrace 的 import-pop / class-brace 逻辑。
#[allow(non_snake_case)]
fn jsxContainer() -> bool {
    unsafe {
        lexer::openTokenStack[lexer::openTokenDepth as usize] = lexer::OpenToken {
            token: lexer::OpenTokenState::AnyBrace,
            pos: lexer::lastTokenPos,
        };
        lexer::openTokenDepth += 1;
    }
    let level = unsafe { lexer::openTokenDepth } - 1;
    let end = source::end();
    let mut pos = position::position();
    loop {
        pos += 1;
        if pos > end {
            position::setPos(pos);
            return false;
        }
        let ch = source::charCodeAtUnchecked(pos);
        if ch == 32 || (ch < 14 && ch > 8) {
            continue;
        }
        pos = consumeToken(pos, ch);
        // 只有容器的 '}' 会把深度弹回 level
        if unsafe { lexer::openTokenDepth } == level {
            break;
        }
    }
    position::setPos(pos);
    return true;
}
