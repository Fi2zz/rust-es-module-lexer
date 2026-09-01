// 我们的 WASM 产物 vs 上游 WASM 2.3.2，同机同口径对比
// ours 测两条路径：parse(source) 便捷路径与 lex_alloc/lex_parse_at UTF-16 直通
// 用法: node scripts/bench_wasm.cjs <file.js>...
const { readFileSync } = require("fs");
const ours = require("../pkg/nodejs/rust_demo.js");
const upstream = require("es-module-lexer");

const ROUNDS = 7;
const N = 25;

const stats = (fn, code) => {
  const mins = [];
  for (let r = 0; r < ROUNDS; r++) {
    let total = 0;
    for (let i = 0; i < N; i++) {
      const start = process.hrtime.bigint();
      fn(code);
      total += Number(process.hrtime.bigint() - start);
    }
    mins.push(total / N / 1e6);
  }
  mins.sort((a, b) => a - b);
  return { min: mins[0], median: mins[(ROUNDS - 1) / 2] };
};

// UTF-16 直通：JS string 本就是 UTF-16，Buffer.write 原生写入 wasm 内存；
// lex_parse_at 返回 JSON 字符串，JSON.parse 还原对象
const fastParse = (code) => {
  const units = code.length;
  const ptr = ours.lex_alloc(units);
  Buffer.from(ours.lex_memory().buffer, ptr, units * 2).write(code, "utf16le");
  JSON.parse(ours.lex_parse_at(ptr, units));
};

upstream.init.then(() => {
  let totals = { ours: 0, fast: 0, upstream: 0 };
  for (const path of process.argv.slice(2)) {
    const code = readFileSync(path, "utf8");
    // 交替测量，消掉系统负载漂移
    const a = stats(ours.parse, code);
    const f = stats(fastParse, code);
    const b = stats(upstream.parse, code);
    totals.ours += a.min;
    totals.fast += f.min;
    totals.upstream += b.min;
    console.log(
      `${path}\n  ours ${a.min.toFixed(2)}ms / ours-fast ${f.min.toFixed(2)}ms / upstream ${b.min.toFixed(2)}ms  (${(b.min / f.min).toFixed(2)}x)`
    );
  }
  console.log(
    `total: ours ${totals.ours.toFixed(2)}ms / ours-fast ${totals.fast.toFixed(2)}ms / upstream ${totals.upstream.toFixed(2)}ms (${(totals.upstream / totals.fast).toFixed(2)}x)`
  );
});
