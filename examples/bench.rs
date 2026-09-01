use rust_demo::{parse, source};
use std::time::Instant;

// 性能基准：对每个样本文件解析 n 次取平均
// 用法: cargo run --release --example bench -- <file.js>...
fn main() {
    let n = 25;
    let mut total_size = 0usize;
    let mut total_ms = 0f64;
    for path in std::env::args().skip(1) {
        let text = std::fs::read_to_string(&path).expect("read file");
        total_size += text.len();
        let mut elapsed = 0f64;
        for _ in 0..n {
            source::setSource(&text);
            let start = Instant::now();
            let _ = parse::parse();
            elapsed += start.elapsed().as_secs_f64() * 1e3;
        }
        let avg = elapsed / n as f64;
        total_ms += avg;
        println!("{path} ({} KiB): {avg:.2}ms", text.len() / 1000);
    }
    println!("total ({} KiB): {total_ms:.2}ms", total_size / 1000);
}
