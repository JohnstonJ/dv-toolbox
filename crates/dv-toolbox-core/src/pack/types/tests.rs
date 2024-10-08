use arbitrary_int::u4;
use googletest::prelude::*;
use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_no_info_binary", &NO_INFO_BINARY_TEST_CASES),
    ("test_unknown_binary", &UNKNOWN_BINARY_TEST_CASES)
);

#[googletest::test]
fn test_type_from_u8() {
    expect_that!(Type::from(0x14u8), eq(Type::TitleBinaryGroup));

    expect_that!(Type::from(0xFEu8), eq(Type::Unknown(0xFE)));
}

#[googletest::test]
fn test_u8_from_type() {
    expect_that!(u8::from(Type::TitleBinaryGroup), eq(0x14u8));

    expect_that!(u8::from(Type::Unknown(0xFE)), eq(0xFEu8));
}

#[googletest::test]
fn test_pack_type_method() {
    let p = Pack::TitleBinaryGroup(validated(
        BinaryGroup {
            group_data: [
                u4::new(2),
                u4::new(1),
                u4::new(4),
                u4::new(3),
                u4::new(6),
                u4::new(5),
                u4::new(8),
                u4::new(7),
            ],
        },
        *NTSC,
    ));
    expect_that!(p.pack_type(), eq(Type::TitleBinaryGroup));

    let p = Pack::Unknown(0xFE, validated(Unparsed { data: [0x12, 0x34, 0x56, 0x78] }, *NTSC));
    expect_that!(p.pack_type(), eq(Type::Unknown(0xFE)));

    let p = Pack::Invalid(
        Type::Unknown(0xFE),
        validated(Unparsed { data: [0x12, 0x34, 0x56, 0x78] }, *NTSC),
    );
    expect_that!(p.pack_type(), eq(Type::Unknown(0xFE)));
}

static UNKNOWN_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    "basic_test",
    PackBinaryTestCase {
        input: "FE 12 34 56 78",
        parsed: Some(Pack::Unknown(
            0xFE,
            validated(Unparsed { data: [0x12, 0x34, 0x56, 0x78] }, *NTSC)
        )),
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_test(function_name!())]
fn test_unknown_binary(#[case] test_function_name: &str) {
    let tc = UNKNOWN_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

static NO_INFO_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    "basic_test",
    PackBinaryTestCase {
        input: "FF FF FF FF FF",
        parsed: Some(Pack::NoInfo(validated(NoInfo {}, *NTSC))),
        ..Default::default()
    },
    "random_bytes",
    PackBinaryTestCase {
        input: "FF 12 34 56 78",
        parsed: Some(Pack::NoInfo(validated(NoInfo {}, *NTSC))),
        output: Some("FF FF FF FF FF"),
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_test(function_name!())]
#[case::random_bytes(function_name!())]
fn test_no_info_binary(#[case] test_function_name: &str) {
    let tc = NO_INFO_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}
