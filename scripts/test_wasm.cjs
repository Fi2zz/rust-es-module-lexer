// 端到端验证：用 wasm-pack 的 Node 产物（pkg/nodejs）跑全部测试语料，
// 与 testdata/ 下的期望 JSON 逐字段比对（含 ok:false 应抛异常的用例）。
// 两个入口都验证：parse(source) 与 lex_alloc/lex_parse_at 快速通道。
// 用法: node scripts/test_wasm.cjs   （需先 wasm-pack build --target nodejs）
const assert = require("node:assert");
const { readFileSync } = require("node:fs");
const wasm = require("../pkg/nodejs");

// UTF-16 直通道：JS string 本就是 UTF-16，直接写进 wasm 内存；
// lex_parse_at 返回 JSON 字符串
const parseFast = (source) => {
  const units = source.length;
  const ptr = wasm.lex_alloc(units);
  Buffer.from(wasm.lex_memory().buffer, ptr, units * 2).write(source, "utf16le");
  return JSON.parse(wasm.lex_parse_at(ptr, units));
};

const suites = [
  ["testdata/cases.json", "testdata/expected.json"],
  ["testdata/jsx_cases.json", "testdata/jsx_expected.json"],
];

const eq = (a, b) => assert.deepStrictEqual(a, b);

let passed = 0;
for (const parse of [wasm.parse, parseFast]) {
  for (const [casesPath, expectedPath] of suites) {
    const cases = JSON.parse(readFileSync(casesPath, "utf8"));
    const expected = JSON.parse(readFileSync(expectedPath, "utf8"));
    assert.strictEqual(cases.length, expected.length, `${casesPath} count mismatch`);
    for (let i = 0; i < cases.length; i++) {
      const { name, source } = cases[i];
      const exp = expected[i];
      assert.strictEqual(name, exp.name, "case order mismatch");
      let got = null;
      let err = null;
      try {
        got = parse(source);
      } catch (e) {
        err = e;
      }
      if (!exp.ok) {
        assert.ok(err instanceof Error, `${name}: expected a parse error`);
        passed++;
        continue;
      }
      assert.ok(err === null, `${name}: unexpected parse error: ${err && err.message}`);
      const [imports, exports, facade] = got;
      eq(facade, exp.facade);
      // undefined 与 null 在本基准里同义（上游 n/at 的 undefined 记为 null）
      const norm = (v) => (v === undefined ? null : v);
      eq(
        imports.map((i) => ({
          n: norm(i.n), t: i.t, ss: i.ss, se: i.se, s: i.s, e: i.e, a: i.a, d: i.d, at: norm(i.at),
        })),
        exp.imports
      );
      eq(
        exports.map((e) => ({
          s: e.s, e: e.e, ls: e.ls, le: e.le, ss: e.ss, n: norm(e.n), ln: norm(e.ln),
        })),
        exp.exports
      );
      passed++;
    }
  }
}
console.log(`wasm e2e: ${passed} cases passed (both entry points)`);
