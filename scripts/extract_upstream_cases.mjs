// 提取上游测试语料：以 mocha 全局桩运行上游 test/_unit.cjs，
// parse 调用被 .upstream-ref/dist/lexer.js 的包装器记录，最终导出基准 JSON。
// 用法: node scripts/extract_upstream_cases.mjs
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";

process.env.WASM = "1";

const setups = [];
const tests = [];
globalThis.suite = (_name, fn) => fn();
globalThis.setup = (fn) => setups.push(fn);
globalThis.test = (name, fn) => tests.push([name, fn]);

const require = createRequire(import.meta.url);
require("../.upstream-ref/test/_unit.cjs");

let failed = 0;
for (const [name, fn] of tests) {
  for (const s of setups) await s();
  try {
    await fn();
  } catch (err) {
    failed++;
    console.error(`上游测试未通过: ${name}\n  ${err.message}`);
  }
}

const seen = new Map();
for (const rec of globalThis.__recorded) {
  if (!seen.has(rec.source)) seen.set(rec.source, rec);
}

// 额外补充的自有用例，同样过一遍上游实现拿期望输出
const { init, parse } = await import("es-module-lexer");
await init;
const extras = JSON.parse(readFileSync(new URL("../testdata/extra_cases.json", import.meta.url), "utf8"));
for (const { source } of extras) {
  if (seen.has(source)) continue;
  try {
    const [imports, exports, facade] = parse(source);
    seen.set(source, { source, ok: true, imports, exports, facade });
  } catch (err) {
    seen.set(source, { source, ok: false, error: String(err && err.message || err) });
  }
}
const cases = [];
const expected = [];
for (const { source, ...rest } of seen.values()) {
  cases.push({ name: `upstream_${cases.length}`, source });
  expected.push({
    name: `upstream_${expected.length}`,
    ok: rest.ok,
    imports: rest.ok
      ? rest.imports.map((i) => ({ n: i.n ?? null, t: i.t ?? null, ss: i.ss, se: i.se, s: i.s, e: i.e, a: i.a, d: i.d, at: i.at ?? null }))
      : [],
    exports: rest.ok
      ? rest.exports.map((e) => ({ s: e.s, e: e.e, ls: e.ls, le: e.le, ss: e.ss ?? null, n: e.n ?? null, ln: e.ln ?? null }))
      : [],
    facade: rest.ok ? rest.facade : null,
    error: rest.ok ? null : rest.error,
  });
}

writeFileSync(new URL("../testdata/cases.json", import.meta.url), JSON.stringify(cases, null, 2));
writeFileSync(new URL("../testdata/expected.json", import.meta.url), JSON.stringify(expected, null, 2));
console.log(`tests: ${tests.length}, failed: ${failed}, unique parse calls: ${cases.length}`);
