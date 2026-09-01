use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    name: String,
    source: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ExpectedImport {
    n: Option<String>,
    t: Option<i32>,
    ss: i32,
    se: i32,
    s: i32,
    e: i32,
    a: i32,
    d: i32,
    at: Option<Vec<(String, String)>>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ExpectedExport {
    s: i32,
    e: i32,
    ls: i32,
    le: i32,
    ss: Option<i32>,
    n: Option<String>,
    ln: Option<String>,
}

#[derive(Deserialize)]
struct Expected {
    name: String,
    ok: bool,
    imports: Vec<ExpectedImport>,
    exports: Vec<ExpectedExport>,
    facade: Option<bool>,
}

// The lexer keeps global static state, so all cases run serially in one test.
#[test]
fn test_parse_matches_reference() {
    // parse errors are delivered as panics; keep test output clean, but let
    // assertion failures print as usual
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(msg) = info.payload().downcast_ref::<String>() {
            if msg.starts_with("Parse error at") {
                return;
            }
        }
        default_hook(info);
    }));

    let cases: Vec<Case> = serde_json::from_str(
        &std::fs::read_to_string("testdata/cases.json").expect("read cases.json"),
    )
    .expect("parse cases.json");
    let expected: Vec<Expected> = serde_json::from_str(
        &std::fs::read_to_string("testdata/expected.json").expect("read expected.json"),
    )
    .expect("parse expected.json");
    assert_eq!(cases.len(), expected.len(), "case count mismatch");

    let mut all: Vec<(Case, Expected)> = cases.into_iter().zip(expected).collect();
    // JSX 扩展用例（无上游基准，期望值为人工核对后的本实现输出）
    let jsx_cases: Vec<Case> = serde_json::from_str(
        &std::fs::read_to_string("testdata/jsx_cases.json").expect("read jsx_cases.json"),
    )
    .expect("parse jsx_cases.json");
    let jsx_expected: Vec<Expected> = serde_json::from_str(
        &std::fs::read_to_string("testdata/jsx_expected.json").expect("read jsx_expected.json"),
    )
    .expect("parse jsx_expected.json");
    assert_eq!(jsx_cases.len(), jsx_expected.len(), "jsx case count mismatch");
    all.extend(jsx_cases.into_iter().zip(jsx_expected));

    for (case, exp) in all.iter() {
        assert_eq!(case.name, exp.name, "case order mismatch");

        let source = case.source.clone();
        let result = std::panic::catch_unwind(move || {
            crate::source::setSource(&source);
            crate::parse::parse()
        });

        if !exp.ok {
            assert!(result.is_err(), "{}: expected a parse error", case.name);
            continue;
        }
        let (imports, exports, facade) =
            result.unwrap_or_else(|_| panic!("{}: unexpected parse error", case.name));

        assert_eq!(Some(facade), exp.facade, "{}: facade", case.name);

        let got_exports: Vec<ExpectedExport> = exports
            .iter()
            .map(|e| ExpectedExport {
                s: e.s,
                e: e.e,
                ls: e.ls,
                le: e.le,
                ss: Some(e.ss),
                n: e.n.clone(),
                ln: e.ln.clone(),
            })
            .collect();
        assert_eq!(got_exports, exp.exports, "{}: exports", case.name);

        let got_imports: Vec<ExpectedImport> = imports
            .iter()
            .map(|i| ExpectedImport {
                n: i.n.clone(),
                t: Some(i.t),
                ss: i.ss,
                se: i.se,
                s: i.s,
                e: i.e,
                a: i.a,
                d: i.d,
                at: i.at.clone(),
            })
            .collect();
        assert_eq!(got_imports, exp.imports, "{}: imports", case.name);
    }
}
