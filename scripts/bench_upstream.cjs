// 上游 WASM 基准：与 examples/bench.rs 相同的口径（每文件 25 次取平均）
// 用法: node scripts/bench_upstream.cjs <file.js>...
const { init, parse } = require("es-module-lexer");
const { readFileSync } = require("fs");

init.then(() => {
  const n = 25;
  let totalSize = 0, totalMs = 0;
  for (const path of process.argv.slice(2)) {
    const code = readFileSync(path, "utf8");
    totalSize += code.length;
    let elapsed = 0;
    for (let i = 0; i < n; i++) {
      const start = process.hrtime.bigint();
      parse(code);
      elapsed += Number(process.hrtime.bigint() - start) / 1e6;
    }
    const avg = elapsed / n;
    totalMs += avg;
    console.log(`${path} (${Math.round(code.length / 1000)} KiB): ${avg.toFixed(2)}ms`);
  }
  console.log(`total (${Math.round(totalSize / 1000)} KiB): ${totalMs.toFixed(2)}ms`);
});
