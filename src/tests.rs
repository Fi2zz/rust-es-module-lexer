#[test]
pub fn test_parse() {
    use crate::parse::parse;
    use crate::source::setSource;
    use std::fs::read;
    let source = read("main.js").unwrap();
    setSource(&source);
    let (imports, exports, facade) = parse();
    println!("\n\n");
    println!("lexer.facade {} \n", facade);
    println!("lexer.exports\n{:?} \n", exports);
    println!("lexer.imports\n{:?} \n", imports);
}

#[test]
pub fn test_demo() {
    let hello = "hello".chars();
    let bytes = "hello".as_bytes();
    println!("hello {:?}", hello);
    println!("bytes {:?}", bytes);
    println!("tostring {:?}", String::from_utf8(bytes.to_vec()).unwrap());

    println!("0x2028 {:?}", 0x2028.to_string());
    println!("0x2029 {:?}", 0x2029.to_string())
}
