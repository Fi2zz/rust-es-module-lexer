// 差分验证：对指定 JS 文件分别跑上游 WASM 实现与 Rust 移植，比较完整输出
// 用法: node scripts/diff_samples.mjs <file.js>...
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { init, parse } from "es-module-lexer";

await init;

const shape = (imports, exports, facade) => ({
  imports: imports.map((i) => ({
    n: i.n ?? null, t: i.t ?? null, ss: i.ss, se: i.se,
    s: i.s, e: i.e, a: i.a, d: i.d, at: i.at ?? null,
  })),
  exports: exports.map((e) => ({
    s: e.s, e: e.e, ls: e.ls, le: e.le, ss: e.ss ?? null, n: e.n ?? null, ln: e.ln ?? null,
  })),
  facade,
});

let failed = 0;
const sortKeys = (v) =>
  Array.isArray(v)
    ? v.map(sortKeys)
    : v && typeof v === "object"
      ? Object.fromEntries(Object.keys(v).sort().map((k) => [k, sortKeys(v[k])]))
      : v;
for (const path of process.argv.slice(2)) {
  const source = readFileSync(path, "utf8");
  const want = JSON.stringify(sortKeys(shape(...parse(source))));
  const gotRaw = execFileSync("cargo", ["run", "-q", "--example", "parse_file", "--", path], {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  }).trim();
  const got = JSON.stringify(sortKeys(JSON.parse(gotRaw)));
  const ok = want === got;
  if (!ok) failed++;
  console.log(`${ok ? "OK  " : "DIFF"} ${path}`);
}
process.exit(failed ? 1 : 0);
