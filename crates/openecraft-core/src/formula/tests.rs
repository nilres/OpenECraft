use super::*;

fn values(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn text(formula: &str, pairs: &[(&str, &str)]) -> String {
    eval(formula, &values(pairs)).unwrap_or_else(|e| panic!("{formula}: {e}")).text()
}

#[test]
fn rated_current_of_a_motor() {
    let motor = [("POWER_KW", "7.5"), ("VOLTAGE", "400"), ("COS_PHI", "0.85"), ("EFFICIENCY", "0.87")];
    assert_eq!(text("round(POWER_KW * 1000 / (sqrt(3) * VOLTAGE * COS_PHI * EFFICIENCY), 1)", &motor), "14.6");
    assert_eq!(text("2 ^ 3 ^ 2", &[]), "512", "powers bind to the right");
    assert_eq!(text("-2 ^ 2 + 10 / 4", &[]), "-1.5");
    assert_eq!(text("1.5e3 + .5", &[]), "1500.5");
    assert_eq!(text("round(2.345, 2) & \" A\"", &[]), "2.35 A");
}

#[test]
fn steps_pick_the_first_limit_not_exceeded() {
    let range = "step(I, 10, \"6.3 - 10 A\", 16, \"10 - 16 A\", \"\")";
    assert_eq!(text(range, &[("I", "14.6")]), "10 - 16 A");
    assert_eq!(text(range, &[("I", "10")]), "6.3 - 10 A");
    assert_eq!(text(range, &[("I", "40")]), "", "the last value is the otherwise");
    let e = eval("step(I, 10, 1.5, 16, 2.5)", &values(&[("I", "40")])).unwrap_err();
    assert!(e.contains("40 is above the last step"), "{e}");
}

#[test]
fn conditions_compare_numbers_and_texts() {
    assert_eq!(text("if(REPORTS = \"yes\", 4, 0)", &[("REPORTS", "yes")]), "4");
    assert_eq!(text("if(P > 5.5, \"star-delta\", \"direct\")", &[("P", "7.5")]), "star-delta");
    assert_eq!(text("(1 <= 2) + (2 != 2) + (\"a\" != \"b\")", &[]), "2");
    assert_eq!(text("max(1, 4, 2) - min(3, 1)", &[]), "3");
    assert_eq!(text("ceil(1.2) & floor(1.8) & abs(-3)", &[]), "213");
}

#[test]
fn mistakes_are_named() {
    let err = |f: &str| eval(f, &values(&[("T", "text")])).unwrap_err();
    assert!(err("X + 1").contains("Unknown name X"));
    assert!(err("T * 2").contains("text is not a number"));
    assert!(err("1 / 0").contains("Division by zero"));
    assert!(err("1 +").contains("ends too early"));
    assert!(err("(1 + 2").contains("ends too early"));
    assert!(err("1 2").contains("at 2"));
    assert!(err("1 # 2").contains("at #"));
    assert!(err("cos(1)").contains("Unknown function cos"));
    assert!(err("sqrt(1, 2)").contains("Wrong number of values for sqrt"));
    assert!(err("\"open").contains("at \""));
}

#[test]
fn names_of_a_formula() {
    assert_eq!(names("round(P * 1000 / (sqrt(3) * U), 1) & UNIT & P"), ["P", "U", "UNIT"]);
    assert!(names("\"open").is_empty(), "nothing of a formula that cannot be read");
}
