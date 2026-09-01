# ES Module Lexer in Rust

A Rust port of [es-module-lexer](https://github.com/guybedford/es-module-lexer),
behavior-aligned with upstream **2.3.2** (verified against its WASM build), with
one deliberate extension: **JSX source files are supported**.

Designed for fast import/export extraction from JS and JSX sources — no full
parse, no AST, just the module syntax.

## Usage

```rust
use rust_demo::{parse, source};

source::setSource("import mod from \"module\";\nexport var p = 5;");
let (imports, exports, facade) = parse::parse();

assert_eq!(imports[0].n.as_deref(), Some("module"));
assert_eq!(exports[0].n.as_deref(), Some("p"));
```

Positions are UTF-16 code unit offsets into the source, matching upstream.
On invalid source the lexer panics (`Parse error`); wrap with
`std::panic::catch_unwind` if you need to handle that gracefully.

To inspect a file from the command line:

```sh
cargo run --example parse_file -- path/to/file.jsx   # prints JSON
```

## Output Format

### Imports

| Field | Meaning |
|-------|---------|
| `n` | Specifier string with escapes decoded (`None` if not a safe literal) |
| `t` | Phase type: 1 static, 2 dynamic `import()`, 3 `import.meta`, 4 `import source`, 5 `import.source()`, 6 `import defer`, 7 `import.defer()` |
| `ss` / `se` | Statement start / end |
| `s` / `e` | Specifier start / end (inside the quotes) |
| `a` | Start of the attributes object, -1 when absent |
| `d` | -1 static, -2 `import.meta`, otherwise position of the dynamic import `(` |
| `at` | Import attributes as `[[key, value], ...]` (`with { type: 'json' }`), `None` when absent |

### Exports

| Field | Meaning |
|-------|---------|
| `n` / `ln` | Exported name / local name (`ln` is `None` for reexports) |
| `s` / `e` | Exported name start / end |
| `ls` / `le` | Local name start / end (-1 when there is no local name) |
| `ss` | Export statement start |

### Facade

`facade` is `true` when the module only uses import/export syntax (a pure
reexport shim), which allows such modules to be skipped when bundling.

## JSX Support

Unlike upstream es-module-lexer, JSX is supported and always enabled (no flag).
Rationale: in valid JS, `<` as a binary operator always follows a value token,
so a `<` in expression position (the same positions where a regex can start)
cannot occur in valid JS — it can only be JSX. The upstream baseline (valid JS)
serves as the no-regression guard for this rule.

The JSX scanner only guarantees: no parse errors, correct top-level
import/export extraction, and no contamination of string/template/comment/regex
token state by JSX content (e.g. apostrophes and slashes in JSX text). It does
not validate tag pairing and produces no JSX structure information.

Expression containers `{...}` (including spread attributes) reuse the main
brace-stack machinery, so strings, templates with `${...}`, comments, regexes
and nested JSX inside containers are all lexed normally.

## Performance

Roughly **1.4x the upstream WASM build** on the upstream sample corpus
(angular/d3/rollup/magic-string + minified, 3 MB total, ~3.9ms vs ~5.5ms,
average of 25 runs per file). To reproduce:

```sh
cargo build --release --example bench
./target/release/examples/bench .upstream-ref/test/samples/*.js
node scripts/bench_upstream.cjs .upstream-ref/test/samples/*.js
```

## Development

The test suite asserts field-by-field equality against the real upstream
behavior, captured from its WASM build:

- `cargo test` — 254 baseline cases (190 extracted from upstream's own
  `test/_unit.cjs` + 64 edge cases) plus 24 JSX cases
- `node scripts/extract_upstream_cases.mjs` — regenerate `testdata/cases.json`
  / `testdata/expected.json` (runs upstream's test suite with mocha stubs and
  records every `parse` call; requires `npm install` first)
- `node scripts/diff_samples.mjs <files...>` — full-output diff against
  upstream on real-world files

The benchmark and sample diffs need the upstream repo checked out locally:
`git clone --depth 1 https://github.com/guybedford/es-module-lexer .upstream-ref`

### JavaScript / WASM

The lexer is exposed to JS via wasm-bindgen (Node + browsers). Build both
targets (requires the `wasm32-unknown-unknown` Rust target and wasm-pack,
e.g. `npm install --save-dev wasm-pack`):

```sh
npx wasm-pack build --release --target nodejs --out-dir pkg/nodejs
npx wasm-pack build --release --target bundler --out-dir pkg/bundler
```

Node (CommonJS), synchronously — no async `init` like upstream:

```js
const { parse } = require("./pkg/nodejs");
const [imports, exports, facade] = parse("export const p = 5");
// imports: { n, t, ss, se, s, e, a, d, at }, exports: { s, e, ls, le, ss, n, ln }
```

Browser / bundler (ESM):

```js
import { parse } from "./pkg/bundler";
const [imports, exports, facade] = parse(source);
```

Fast path (recommended for bulk workloads): JS strings are already UTF-16, so
skip the UTF-8 roundtrip of `parse(source)` by writing straight into wasm
memory. `lex_parse_at` returns a JSON string (cheaper than constructing
hundreds of JS objects across the wasm↔JS boundary; `JSON.parse` in V8 is
very fast):

```js
const lex = require("./pkg/nodejs");

const units = source.length;            // UTF-16 code units
const ptr = lex.lex_alloc(units);       // allocate inside wasm memory
Buffer.from(lex.lex_memory().buffer, ptr, units * 2).write(source, "utf16le");
const [imports, exports, facade] = JSON.parse(lex.lex_parse_at(ptr, units));
```

With this path the wasm build matches upstream es-module-lexer's wasm
(`scripts/bench_wasm.cjs`: ~1.0x); the plain `parse(source)` path pays a
UTF-8 copy + transcode and is ~1.8x slower on large files.

API correspondence with upstream es-module-lexer 2.x: same `[imports, exports,
facade]` triple and field names; import phases (`t`), import attributes (`at`,
`with { ... }` only) and the `ls/le/ln/ss` export fields follow upstream 2.3.2.
Differences: JSX is supported (always on); parse errors throw a plain JS
`Error("Parse error at <offset>")` instead of the upstream `Parse error
name:line:col` + `idx` shape; no `hasModuleSyntax` fourth return value.

Run the end-to-end corpus against the Node build:

```sh
node scripts/test_wasm.cjs   # 278 cases x both entry points, field-by-field
```

### Limitations

- **TS/TSX is explicitly out of scope.** Plain `.tsx` type syntax is not
  handled. `useRef<T>(null)` is safe by accident (the `<` follows an
  identifier), but `const f = <T>(v) => v` enters JSX mode at `<` and may
  swallow the rest of the file.
- Unterminated JSX at EOF is tolerated silently.
- `assert { ... }` import syntax is not recognized (upstream dropped it in
  favor of `with { ... }` in 2.x).

## License

MIT
