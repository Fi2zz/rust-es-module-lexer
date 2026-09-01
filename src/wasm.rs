//! wasm-bindgen 绑定：把 lexer 暴露给 JS（Node + 浏览器通用，同步可用，
//! 无上游的 init 异步引导）。只在 wasm32 目标上编译。
//!
//! 语法错误：wasm32 的预编译 std 是 panic=abort（catch_unwind 无效），
//! 由 helper::syntaxError 直接 throw_str 抛 JS Error。

use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use serde::Serialize as _;

/// 解析 JS/JSX 模块源码，返回 `[imports, exports, facade]`。
/// imports 字段：n/t/ss/se/s/e/a/d/at；exports 字段：s/e/ls/le/ss/n/ln；
/// 缺失值序列化为 null。语法错误抛 JS Error。
#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {
    // 全局 static mut 架构下 WASM 是单实例运行，无并发问题；panic 兜成 JS 异常
    let result = std::panic::catch_unwind(|| {
        crate::source::setSource(source);
        crate::parse::parse()
    });
    match result {
        Ok(parsed) => {
            let serializer =
                serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
            parsed
                .serialize(&serializer)
                .map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "Parse error".to_string());
            Err(js_sys::Error::new(&msg).into())
        }
    }
}

#[allow(non_upper_case_globals)]
static mut LEX_BUF: Option<Vec<u16>> = None;

/// 返回 wasm 内存的 JsValue，供 JS 侧构造 Uint16Array/Buffer 视图
///（不能叫 memory：与 wasm 线性内存的导出重名）
#[wasm_bindgen]
pub fn lex_memory() -> JsValue {
    wasm_bindgen::memory()
}

/// 快速输入通道：分配 units 个 UTF-16 码元的缓冲（外加哨兵位），返回其
/// 指针（字节地址）。契约：lex_alloc → JS 写入 units 个码元 → lex_parse_at。
/// 缓冲区在下一次 lex_alloc/lex_parse_at 时被回收复用。
/// 只分配不 memset：JS 侧随后全量覆写 [0, units)，哨兵位由 setSourceUtf16 清零
#[wasm_bindgen]
pub fn lex_alloc(units: usize) -> *mut u16 {
    let mut v: Vec<u16> = Vec::with_capacity(units + 4);
    let ptr = v.as_mut_ptr();
    unsafe {
        v.set_len(units + 4);
        LEX_BUF = Some(v);
    }
    ptr
}

/// 快速输入通道：解析 lex_alloc 缓冲里的 units 个 UTF-16 码元。
/// 返回 JSON 字符串（JS 侧 JSON.parse 后形状与 parse() 相同）——
/// 逐字段构造 JS 对象意味着每字段一次 wasm↔JS 边界调用（大文件数百条目
/// 时开销可观），JSON 串只有一次边界拷贝，V8 的 JSON.parse 极快。
/// ptr 必须与最近一次 lex_alloc 的返回值一致（缓冲所有权在此移交给 lexer 内核）。
#[wasm_bindgen]
pub fn lex_parse_at(ptr: *const u16, units: usize) -> Result<String, JsValue> {
    let buf = unsafe { LEX_BUF.take() }
        .filter(|b| b.as_ptr() == ptr as *const u16 && b.len() == units + 4);
    let buf = match buf {
        Some(b) => b,
        None => {
            return Err(js_sys::Error::new("lex_parse_at: stale ptr, call lex_alloc first").into())
        }
    };
    let result = std::panic::catch_unwind(|| {
        crate::source::setSourceUtf16(buf, units);
        crate::parse::parse()
    });
    match result {
        Ok((imports, exports, facade)) => Ok(writeResultJson(&imports, &exports, facade)),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "Parse error".to_string());
            Err(js_sys::Error::new(&msg).into())
        }
    }
}

// 手写 JSON writer：字段名固定，名字只需转义 " \ 与控制字符
#[allow(non_snake_case)]
fn writeResultJson(imports: &[crate::helper::Import], exports: &[crate::helper::Export], facade: bool) -> String {
    let mut out = String::with_capacity(256 + imports.len() * 96 + exports.len() * 80);
    out.push_str("[[");
    for (i, im) in imports.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"n\":");
        writeJsonOptStr(&mut out, &im.n);
        out.push_str(",\"t\":");
        push_i32(&mut out, im.t);
        out.push_str(",\"ss\":");
        push_i32(&mut out, im.ss);
        out.push_str(",\"se\":");
        push_i32(&mut out, im.se);
        out.push_str(",\"s\":");
        push_i32(&mut out, im.s);
        out.push_str(",\"e\":");
        push_i32(&mut out, im.e);
        out.push_str(",\"a\":");
        push_i32(&mut out, im.a);
        out.push_str(",\"d\":");
        push_i32(&mut out, im.d);
        out.push_str(",\"at\":");
        match &im.at {
            None => out.push_str("null"),
            Some(at) => {
                out.push('[');
                for (j, (k, v)) in at.iter().enumerate() {
                    if j > 0 {
                        out.push(',');
                    }
                    out.push('[');
                    writeJsonStr(&mut out, k);
                    out.push(',');
                    writeJsonStr(&mut out, v);
                    out.push(']');
                }
                out.push(']');
            }
        }
        out.push('}');
    }
    out.push_str("],[");
    for (i, e) in exports.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"s\":");
        push_i32(&mut out, e.s);
        out.push_str(",\"e\":");
        push_i32(&mut out, e.e);
        out.push_str(",\"ls\":");
        push_i32(&mut out, e.ls);
        out.push_str(",\"le\":");
        push_i32(&mut out, e.le);
        out.push_str(",\"ss\":");
        push_i32(&mut out, e.ss);
        out.push_str(",\"n\":");
        writeJsonOptStr(&mut out, &e.n);
        out.push_str(",\"ln\":");
        writeJsonOptStr(&mut out, &e.ln);
        out.push('}');
    }
    out.push_str("],");
    out.push_str(if facade { "true" } else { "false" });
    out.push(']');
    out
}

fn push_i32(out: &mut String, v: i32) {
    // 栈上 itoa，避免每字段一次 String 分配
    let mut buf = [0u8; 12];
    let mut i = buf.len();
    let mut n = (v as i64).unsigned_abs();
    loop {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    if v < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    // buf 内容全为 ASCII
    out.push_str(unsafe { std::str::from_utf8_unchecked(&buf[i..]) });
}

#[allow(non_snake_case)]
fn writeJsonOptStr(out: &mut String, s: &Option<String>) {
    match s {
        None => out.push_str("null"),
        Some(s) => writeJsonStr(out, s),
    }
}

#[allow(non_snake_case)]
fn writeJsonStr(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u");
                let n = c as u32;
                out.push(char::from_digit(n >> 12 & 15, 16).unwrap());
                out.push(char::from_digit(n >> 8 & 15, 16).unwrap());
                out.push(char::from_digit(n >> 4 & 15, 16).unwrap());
                out.push(char::from_digit(n & 15, 16).unwrap());
            }
            c => out.push(c),
        }
    }
    out.push('"');
}


