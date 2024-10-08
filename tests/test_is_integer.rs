use analyzer::is_integer;

#[test]
fn test_zero() {
    assert_eq!(is_integer("0"), true);
}

#[test]
fn test_positive() {
    assert_eq!(is_integer("121"), true);
}

#[test]
fn test_negative() {
    assert_eq!(is_integer("-21"), true);
}

#[test]
fn test_empty() {
    assert_eq!(is_integer(""), false);
}

#[test]
fn test_plus() {
    assert_eq!(is_integer("+"), false);
}

#[test]
fn test_minus() {
    assert_eq!(is_integer("-"), false);
}

#[test]
fn test_space() {
    assert_eq!(is_integer(" "), false);
}

#[test]
fn test_with_letters() {
    assert_eq!(is_integer("-123bba"), false);
}

#[test]
fn test_letters() {
    assert_eq!(is_integer("pdd"), false);
}

#[test]
fn test_double() {
    assert_eq!(is_integer("-13.69"), false);
}
