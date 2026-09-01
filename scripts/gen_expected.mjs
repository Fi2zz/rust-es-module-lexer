// 生成测试基准：用参考实现 lexer.js 解析语料，输出 JSON 供 Rust 测试断言
// 用法: node scripts/gen_expected.mjs > testdata/expected.json
import { parse } from "../lexer.js";
import { readFileSync } from "node:fs";

const cases = JSON.parse(readFileSync(new URL("../testdata/cases.json", import.meta.url), "utf8"));

const results = cases.map(({ name, source }) => {
  try {
    const [imports, exports, facade] = parse(source);
    return {
      name,
      ok: true,
      imports: imports.map((i) => ({
        n: i.n === undefined ? null : i.n,
        ss: i.ss,
        se: i.se,
        s: i.s,
        e: i.e,
        a: i.a,
        d: i.d,
      })),
      exports,
      facade,
    };
  } catch (err) {
    return { name, ok: false, error: String(err.message || err) };
  }
});

console.log(JSON.stringify(results, null, 2));
