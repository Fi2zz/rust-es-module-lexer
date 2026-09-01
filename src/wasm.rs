//! wasm-bindgen 绑定：把 lexer 暴露给 JS（Node + 浏览器通用，同步可用，
//! 无上游的 init 异步引导）。只在 wasm32 目标上编译。
//!
//! 注意：WASM 构建需要 `RUSTFLAGS="-C panic=unwind"`（默认 panic=abort 下
//! catch_unwind 不生效，语法错误会直接 trap）。

use wasm_bindgen::prelude::*;

#[allow(unused_imports)]
use serde::Serialize as _;

/// 解析 JS/JSX 模块源码，返回 `[imports, exports, facade]`。
/// imports 字段：n/t/ss/se/s/e/a/d/at；exports 字段：s/e/ls/le/ss/n/ln；
/// 缺失值序列化为 null。语法错误抛 JS Error。
#[wasm_bindgen]
pub fn parse(source: &str) -> Result<JsValue, JsValue> {
    let owned = source.to_owned();
    // 全局 static mut 架构下 WASM 是单实例运行，无并发问题；panic 兜成 JS 异常
    let result = std::panic::catch_unwind(move || {
        crate::source::setSource(&owned);
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
