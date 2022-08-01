use crate::position;
use crate::source;
#[derive(Debug, Clone)]
pub struct Import {
    /// name
    pub n: String,
    // statement start
    pub ss: i32,
    // statement end
    pub se: i32,
    // start
    pub s: i32,
    // end
    pub e: i32,
    // "a" = assert, -1 for no assertion
    pub a: i32,
    pub d: i32,
}
#[derive(Debug, Clone, Copy)]
pub struct DynamicImport {
    /// name
    //  pub n: String,
    // statement start
    pub ss: i32,
    // statement end
    pub se: i32,
    // start
    pub s: i32,
    // end
    pub e: i32,
    // "a" = assert, -1 for no assertion
    pub a: i32,
    pub d: i32,
}

pub type Imports = Vec<Import>;
pub type Exports = Vec<String>;
pub type Facade = bool;
pub type ParseResult = (Imports, Exports, Facade);
pub type VChars = Vec<char>;
pub type Source = Vec<u8>;
static SYNTAX_ERROR: &str = "Syntax Error";
static KEYWORD_EXPORT: &str = "export";
static KEYWORD_IMPORT: &str = "import";
static KEYWORD_CLASSS: &str = "class";
static KEYWORD_CONST: &str = "const";
static KEYWORD_LET: &str = "let";
static KEYWORD_VAR: &str = "var";
static KEYWORD_FROM: &str = "from";
static KEYWORD_DEFAULT: &str = "default";
static KEYWORD_META: &str = "meta";
pub fn create_import(ss: i32, s: i32, e: i32, d: i32) -> Import {
    let se = if d == -2 {
        e
    } else if d == -1 {
        e + 1
    } else {
        0
    };
    let import = Import {
        ss,
        s,
        e,
        d,
        a: -1,
        n: String::from(""),
        se,
    };
    import
}

pub fn iskeyword_meta(str: &str) -> bool {
    return str == KEYWORD_META;
}
pub fn iskeyword_export(str: &str) -> bool {
    return str == KEYWORD_EXPORT;
}

pub fn iskeyword_import(str: &str) -> bool {
    return str == KEYWORD_IMPORT;
}

pub fn iskeyword_from(str: &str) -> bool {
    return str == KEYWORD_FROM;
}

pub fn iskeyword_default(str: &str) -> bool {
    return str == KEYWORD_DEFAULT;
}
pub fn iskeyword_let(str: &str) -> bool {
    return str == KEYWORD_LET;
}

pub fn iskeyword_var(str: &str) -> bool {
    return str == KEYWORD_VAR;
}

pub fn create_imports() -> Vec<Import> {
    return Vec::new();
}

pub fn create_exports() -> Vec<String> {
    return Vec::new();
}
pub fn create_chars() -> Vec<char> {
    return Vec::new();
}

pub fn create_vec_with_length(size: i32) -> Vec<i32> {
    return vec![0, size];
}
#[allow(non_snake_case)]
pub fn syntaxError() -> () {
    panic!("{}", SYNTAX_ERROR);
}

pub fn print_cannot_empty_file() {
    println!("Cannot parse empty file")
}

pub fn array_map(array: &mut Vec<i32>, callback: fn(v: i32, index: i32) -> i32) {
    for index in 0..array.len() {
        let result = callback(array[index], index as i32);
        array[index] = result;
    }
}

// Note: non-asii BR and whitespace checks omitted for perf / footprint
// if there is a significant user need this can be reconsidered
pub fn is_br(c: usize) -> bool {
    return c == 13/*\r*/ || c == 10/*\n*/;
}
pub fn is_whitespace(c: usize) -> bool {
    return c == 9 || c == 11 || c == 12 || c == 32 || c == 160;
}
pub fn is_br_or_whitespace(c: usize) -> bool {
    return c > 8 && c < 14 || c == 32 || c == 160;
}
pub fn is_punctuator(ch: usize) -> bool {
    // 23 possible punctuator endings: !%&()*+,-./:;<=>?[]^{}|~
    return ch == 33/* *!*/ || ch == 37/*%*/ || ch == 38/*&*/ ||
    ch > 39 && ch < 48 || ch > 57 && ch < 64 ||
    ch == 91/*[*/ || ch == 93/*]*/ || ch == 94/*^*/ ||
    ch > 122 && ch < 127;
}

pub fn is_br_or_ws_or_punctuator_not_dot(c: usize) -> bool {
    return c > 8 && c < 14 || c == 32 || c == 160 || is_punctuator(c) && c != 46/*.*/;
}
pub fn at(str: &String) -> usize {
    let bt = str.as_bytes();
    return bt[0] as usize;
}
pub fn is_whitespace_not_br(c: usize) -> bool {
    return c == 9 || c == 11 || c == 12 || c == 32 || c == 160;
}

pub fn is_export() -> bool {
    let str = source::stringify(position::position(), position::dry(6));
    iskeyword_export(&str)
}
pub fn is_import() -> bool {
    let str = source::stringify(position::position(), position::dry(6));
    println!("iskeyword_import {:?}", str);
    iskeyword_import(&str)
}
pub fn iskeyword_start(ch: usize) -> bool {
    if ch != 0 {
        let start = position::dry(-1);
        if start < 0 {
            return false;
        }
        let code = source::charCodeAt(start);
        return is_br_or_ws_or_punctuator_not_dot(code);
    }
    return true;
}

pub fn iskeyword_class(start: i32, end: i32) -> bool {
    let str = source::stringify(start, end);
    println!("class {:?}", str);
    return str == KEYWORD_CLASSS;
}

pub fn iskeyword_const(start: i32, end: i32) -> bool {
    let str = source::stringify(start, end);
    println!("class {:?}", str);
    return str == KEYWORD_CONST;
}

pub fn compare(target: i32, value: i32, ctype: &str) -> bool {
    match ctype {
        ">" => {
            return target > value;
        }
        ">=" => {
            return target >= value;
        }
        "<" => {
            return target < value;
        }
        "<=" => {
            return target <= value;
        }
        "!" => {
            return target != value;
        }
        _ => {
            return target == value;
        }
    }
}
