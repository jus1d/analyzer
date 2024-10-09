use analyzer::is_identifier;

#[test]
fn test_zero() {
    assert_eq!(is_identifier("0"), false);
}

#[test]
fn test_positive() {
    assert_eq!(is_identifier("121"), false);
}

#[test]
fn test_ok() {
    assert_eq!(is_identifier("accum"), true);
}

#[test]
fn test_ok_with_numbers() {
    assert_eq!(is_identifier("accum2"), true);
}

#[test]
fn test_negative() {
    assert_eq!(is_identifier("-21"), false);
}

#[test]
fn test_empty() {
    assert_eq!(is_identifier(""), false);
}

#[test]
fn test_plus() {
    assert_eq!(is_identifier("+"), false);
}

#[test]
fn test_minus() {
    assert_eq!(is_identifier("-"), false);
}

#[test]
fn test_space() {
    assert_eq!(is_identifier(" "), false);
}

#[test]
fn test_with_letters() {
    assert_eq!(is_identifier("-123bba"), false);
}

#[test]
fn test_letters() {
    assert_eq!(is_identifier("pdd"), true);
}

#[test]
fn test_double() {
    assert_eq!(is_identifier("-13.69"), false);
}

#[test]
fn test_k() {
    assert_eq!(is_identifier("k"), true);
}
