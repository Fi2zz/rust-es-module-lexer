use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    source: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ExpectedImport {
    n: Option<String>,
    ss: i32,
    se: i32,
    s: i32,
    e: i32,
    a: i32,
    d: i32,
}

#[derive(Deserialize)]
struct Expected {
    name: String,
    ok: bool,
    imports: Vec<ExpectedImport>,
    exports: Vec<String>,
    facade: bool,
}

// The lexer keeps global static state, so all cases run serially in one test.
#[test]
fn test_parse_matches_reference() {
    let cases: Vec<Case> = serde_json::from_str(
        &std::fs::read_to_string("testdata/cases.json").expect("read cases.json"),
    )
    .expect("parse cases.json");
    let expected: Vec<Expected> = serde_json::from_str(
        &std::fs::read_to_string("testdata/expected.json").expect("read expected.json"),
    )
    .expect("parse expected.json");
    assert_eq!(cases.len(), expected.len(), "case count mismatch");

    for (case, exp) in cases.iter().zip(expected.iter()) {
        assert_eq!(case.name, exp.name, "case order mismatch");
        assert!(exp.ok, "{}: reference itself failed", case.name);

        crate::source::setSource(&case.source.as_bytes().to_vec());
        let (imports, exports, facade) = crate::parse::parse();

        assert_eq!(facade, exp.facade, "{}: facade", case.name);
        assert_eq!(exports, exp.exports, "{}: exports", case.name);
        assert_eq!(
            imports.len(),
            exp.imports.len(),
            "{}: imports length (got {:?})",
            case.name,
            imports
        );
        for (index, want) in exp.imports.iter().enumerate() {
            let got = &imports[index];
            let got = ExpectedImport {
                n: got.n.clone(),
                ss: got.ss,
                se: got.se,
                s: got.s,
                e: got.e,
                a: got.a,
                d: got.d,
            };
            assert_eq!(&got, want, "{}: imports[{}]", case.name, index);
        }
    }
}
