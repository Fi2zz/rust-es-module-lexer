use rust_demo::{parse, source};
use serde_json::json;

// 差分工具：解析文件并输出与 scripts/extract_upstream_cases.mjs 相同形状的 JSON
// 用法: cargo run --example parse_file -- <file.js>
fn main() {
    // evalLiteral 用 catch_unwind 模拟 eval 抛错；吞掉这类预期内的 panic 输出
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(msg) = info.payload().downcast_ref::<String>() {
            if msg.starts_with("Parse error at") {
                return;
            }
        }
        default_hook(info);
    }));
    let path = std::env::args().nth(1).expect("usage: parse_file <file.js>");
    let text = std::fs::read_to_string(&path).expect("read file");
    source::setSource(&text);
    let (imports, exports, facade) = parse::parse();
    let imports: Vec<_> = imports
        .iter()
        .map(|i| {
            json!({
                "n": i.n, "t": i.t, "ss": i.ss, "se": i.se,
                "s": i.s, "e": i.e, "a": i.a, "d": i.d, "at": i.at,
            })
        })
        .collect();
    let exports: Vec<_> = exports
        .iter()
        .map(|e| {
            json!({
                "s": e.s, "e": e.e, "ls": e.ls, "le": e.le,
                "ss": e.ss, "n": e.n, "ln": e.ln,
            })
        })
        .collect();
    println!("{}", json!({ "imports": imports, "exports": exports, "facade": facade }));
}
