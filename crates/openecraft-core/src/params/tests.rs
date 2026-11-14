use super::*;

const PARAMS: &[Param] = &[param("page", Kind::Str, "Page"), optional("ids", Kind::Ids, "Items"), optional("points", Kind::Points, "Points"), optional("mirror", Kind::Bool, "Mirror")];

fn map(v: Value) -> Map<String, Value> {
    v.as_object().unwrap().clone()
}

#[test]
fn check_rejects_unknown_missing_and_mistyped_arguments() {
    assert!(check(PARAMS, &map(json!({ "page": "1", "ids": [1, 2] }))).is_ok());
    assert_eq!(check(PARAMS, &map(json!({}))).unwrap_err(), "Missing parameter: page");
    assert_eq!(check(PARAMS, &map(json!({ "page": 1 }))).unwrap_err(), "Parameter page has the wrong type");
    assert_eq!(check(PARAMS, &map(json!({ "page": "1", "x": 1 }))).unwrap_err(), "Unknown parameter: x");
    assert!(check(PARAMS, &map(json!({ "page": "1", "points": [[0, 0], [1]] }))).is_err());
}

#[test]
fn schema_has_types_and_required_parameters() {
    let s = schema(PARAMS);
    assert_eq!(s["required"], json!(["page"]));
    assert_eq!(s["properties"]["ids"]["items"]["type"], "integer");
    assert_eq!(s["properties"]["mirror"]["description"], "Mirror");
}

#[test]
fn values_are_parsed_by_kind() {
    assert_eq!(Kind::Ids.parse("3, 4,5"), Some(json!([3, 4, 5])));
    assert_eq!(Kind::Points.parse("0,0 8,-4"), Some(json!([[0.0, 0.0], [8.0, -4.0]])));
    assert_eq!(Kind::Bool.parse("yes"), Some(json!(true)));
    assert_eq!(Kind::Int.parse("1.5"), None);
    assert_eq!(Kind::Object.parse("[1]"), None);
    let pair = |n: &str, v: Option<&str>| (n.to_string(), v.map(String::from));
    let m = from_cli(PARAMS, Some(r#"{"page": "1"}"#), &[pair("page", Some("2")), pair("mirror", None)]).unwrap();
    assert_eq!(Value::Object(m), json!({ "page": "2", "mirror": true }));
}

#[test]
fn params_of_external_plugins_are_read_from_json() {
    let p: Param = serde_json::from_value(json!({ "name": "length", "kind": "num" })).unwrap();
    assert!(p.required);
    assert_eq!(p.kind, Kind::Num);
    assert!(matches!(p.name, Cow::Owned(_)));
}

#[test]
fn integers_may_be_written_with_a_zero_fraction() {
    const P: &[Param] = &[optional("rotation", Kind::Int, "Rotation"), optional("ids", Kind::Ids, "Items")];
    assert!(check(P, &map(json!({ "rotation": 90.0, "ids": [3.0, 4] }))).is_ok());
    assert!(check(P, &map(json!({ "rotation": 90.5 }))).is_err());
    assert!(check(P, &map(json!({ "ids": [-1] }))).is_err());
    let m = map(json!({ "rotation": 90.0, "ids": [3.0, 4] }));
    let a = Args(&m);
    assert_eq!((a.int("rotation"), a.ids("ids")), (Some(90), vec![3, 4]));
}

#[test]
fn command_line_words_and_pairs() {
    let argv: Vec<String> = "rename -p a.oec --old -K1 --new=-K2 --help x --discard -h".split(' ').map(String::from).collect();
    let pair = |n: &str, v: Option<&str>| CliWord::Pair(n.to_string(), v.map(String::from));
    assert_eq!(
        cli_words(&argv, &["help"], &["p"]),
        vec![
            CliWord::Plain("rename".into()),
            pair("p", Some("a.oec")),
            pair("old", Some("-K1")),
            pair("new", Some("-K2")),
            pair("help", None),
            CliWord::Plain("x".into()),
            pair("discard", Some("-h")),
        ]
    );
    // A value never starts with two dashes; without `short` a single dash is a word.
    assert_eq!(cli_words(&argv[1..3], &[], &[]), vec![CliWord::Plain("-p".into()), CliWord::Plain("a.oec".into())]);
    let two: Vec<String> = vec!["--a".into(), "--b".into()];
    assert_eq!(cli_words(&two, &[], &[]), vec![pair("a", None), pair("b", None)]);
}
