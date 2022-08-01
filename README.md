# ES Module Lexer in Rust

Inspired by [es-module-lexer](https://github.com/guybedford/es-module-lexer)

A JS module syntax lexer in Rust

### Usage

```
npm install es-module-lexer
```

For use in CommonJS:

```js
const { init, parse } = require("es-module-lexer");

(async () => {
	// either await init, or call parse asynchronously
	// this is necessary for the Web Assembly boot
	await init;

	const [imports, exports] = parse("export var p = 5");
	exports[0] === "p";
})();
```

An ES module version is also available:

```js
import { init, parse } from "es-module-lexer";

(async () => {
	await init;

	const source = `
    import { name } from 'mod\\u1011';
    import json from './json.json' assert { type: 'json' }
    export var p = 5;
    export function q () {

    };

    // Comments provided to demonstrate edge cases
    import /*comment!*/ (  'asdf', { assert: { type: 'json' }});
    import /*comment!*/.meta.asdf;
  `;

	const [imports, exports] = parse(source, "optional-sourcename");

	// Returns "modထ"
	imports[0].n;
	// Returns "mod\u1011"
	source.substring(imports[0].s, imports[0].e);
	// "s" = start
	// "e" = end

	// Returns "import { name } from 'mod'"
	source.substring(imports[0].ss, imports[0].se);
	// "ss" = statement start
	// "se" = statement end

	// Returns "{ type: 'json' }"
	source.substring(imports[1].a, imports[1].se);
	// "a" = assert, -1 for no assertion

	// Returns "p,q"
	exports.toString();

	// Dynamic imports are indicated by imports[2].d > -1
	// In this case the "d" index is the start of the dynamic import bracket
	// Returns true
	imports[2].d > -1;

	// Returns "asdf" (only for string literal dynamic imports)
	imports[2].n;
	// Returns "import /*comment!*/ (  'asdf', { assert: { type: 'json' } })"
	source.substring(imports[2].ss, imports[2].se);
	// Returns "'asdf'"
	source.substring(imports[2].s, imports[2].e);
	// Returns "(  'asdf', { assert: { type: 'json' } })"
	source.substring(imports[2].d, imports[2].se);
	// Returns "{ assert: { type: 'json' } }"
	source.substring(imports[2].a, imports[2].se - 1);

	// For non-string dynamic import expressions:
	// - n will be undefined
	// - a is currently -1 even if there is an assertion
	// - e is currently the character before the closing )

	// For nested dynamic imports, the se value of the outer import is -1 as end tracking does not
	// currently support nested dynamic immports

	// import.meta is indicated by imports[2].d === -2
	// Returns true
	imports[2].d === -2;
	// Returns "import /*comment!*/.meta"
	source.substring(imports[2].s, imports[2].e);
	// ss and se are the same for import meta
})();
```

### Escape Sequences

To handle escape sequences in specifier strings, the `.n` field of imported specifiers will be provided where possible.

For dynamic import expressions, this field will be empty if not a valid JS string.

### Facade Detection

Facade modules that only use import / export syntax can be detected via the third return value:

```js
const [, , facade] = parse(`
  export * from 'external';
  import * as ns from 'external2';
  export { a as b } from 'external3';
  export { ns };
`);
facade === true;
```

### Grammar Support

- Token state parses all line comments, block comments, strings, template strings, blocks, parens and punctuators.
- Division operator / regex token ambiguity is handled via backtracking checks against punctuator prefixes, including closing brace or paren backtracking.
- Always correctly parses valid JS source, but may parse invalid JS source without errors.

### Limitations

The lexing approach is designed to deal with the full language grammar including RegEx / division operator ambiguity through backtracking and paren / brace tracking.

The only limitation to the reduced parser is that the "exports" list may not correctly gather all export identifiers in the following edge cases:

```js
// Only "a" is detected as an export, "q" isn't
export var a = "asdf",
	q = z;

// "b" is not detected as an export
export var { a: b } = asdf;
```

The above cases are handled gracefully in that the lexer will keep going fine, it will just not properly detect the export names above.

### License

MIT
