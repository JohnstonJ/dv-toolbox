use arbitrary_int::u2;
use display_error_chain::ErrorChainExt;
use googletest::prelude::*;
use rstest::rstest;

use super::*;

#[googletest::test]
#[rstest]
#[case::valid_assorted_digits(u2::new(1), u4::new(6), u4::new(4), Some(164), None)]
#[case::valid_min(u2::new(0), u4::new(0), u4::new(0), Some(0), None)]
#[case::valid_max(u2::new(3), u4::new(9), u4::new(9), Some(399), None)]
#[case::absent(u2::new(3), u4::new(0xF), u4::new(0xF), None, None)]
#[case::overflow_tens(
    u2::new(0),
    u4::new(10),
    u4::new(0),
    None,
    Some("tens place value of 10 is greater than 9")
)]
#[case::overflow_units(
    u2::new(0),
    u4::new(0),
    u4::new(10),
    None,
    Some("units place value of 10 is greater than 9")
)]
fn test_from_bcd_hundreds(
    #[case] hundreds: u2,
    #[case] tens: u4,
    #[case] units: u4,
    #[case] expected: Option<u16>,
    #[case] error: Option<&str>,
) {
    let result = from_bcd_hundreds(hundreds, tens, units);
    match error {
        Some(e) => expect_that!(result.map_err(|e| e.chain().to_string()), err(eq(e))),
        None => expect_that!(result, ok(eq(&expected))),
    };
}

#[googletest::test]
fn test_from_bcd_hundreds_overflow_hundreds() {
    let result = from_bcd_hundreds(u4::new(10), u4::new(0), u4::new(0));
    expect_that!(
        result.map_err(|e| e.chain().to_string()),
        err(eq("hundreds place value of 10 is greater than 9"))
    )
}

#[googletest::test]
#[rstest]
#[case::valid_assorted_digits(u2::new(1), u4::new(4), Some(14), None)]
#[case::valid_min(u2::new(0), u4::new(0), Some(0), None)]
#[case::valid_max(u2::new(3), u4::new(9), Some(39), None)]
#[case::absent(u2::new(3), u4::new(0xF), None, None)]
#[case::overflow_units(
    u2::new(0),
    u4::new(10),
    None,
    Some("units place value of 10 is greater than 9")
)]
fn test_from_bcd_tens(
    #[case] tens: u2,
    #[case] units: u4,
    #[case] expected: Option<u8>,
    #[case] error: Option<&str>,
) {
    let result = from_bcd_tens(tens, units);
    match error {
        Some(e) => expect_that!(result.map_err(|e| e.chain().to_string()), err(eq(e))),
        None => expect_that!(result, ok(eq(&expected))),
    };
}

#[googletest::test]
fn test_from_bcd_tens_overflow_tens() {
    let result = from_bcd_tens(u4::new(10), u4::new(0));
    expect_that!(
        result.map_err(|e| e.chain().to_string()),
        err(eq("tens place value of 10 is greater than 9"))
    )
}
