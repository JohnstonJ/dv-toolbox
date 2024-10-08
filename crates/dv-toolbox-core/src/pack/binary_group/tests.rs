use arbitrary_int::u4;
use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_title_binary_group_binary", &TITLE_BINARY_GROUP_BINARY_TEST_CASES),
    ("test_aaux_binary_group_binary", &AAUX_BINARY_GROUP_BINARY_TEST_CASES),
    ("test_vaux_binary_group_binary", &VAUX_BINARY_GROUP_BINARY_TEST_CASES)
);

static TITLE_BINARY_GROUP_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    "basic_test",
    PackBinaryTestCase {
        input: "14 12 34 56 78",
        parsed: Some(Pack::TitleBinaryGroup(validated(
            BinaryGroup {
                group_data: [
                    u4::new(2),
                    u4::new(1),
                    u4::new(4),
                    u4::new(3),
                    u4::new(6),
                    u4::new(5),
                    u4::new(8),
                    u4::new(7)
                ]
            },
            *NTSC
        ))),
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_test(function_name!())]
fn test_title_binary_group_binary(#[case] test_function_name: &str) {
    let tc = TITLE_BINARY_GROUP_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

static AAUX_BINARY_GROUP_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    "basic_test",
    PackBinaryTestCase {
        input: "54 12 34 56 78",
        parsed: Some(Pack::AAUXBinaryGroup(validated(
            BinaryGroup {
                group_data: [
                    u4::new(2),
                    u4::new(1),
                    u4::new(4),
                    u4::new(3),
                    u4::new(6),
                    u4::new(5),
                    u4::new(8),
                    u4::new(7)
                ]
            },
            *PAL
        ))),
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_test(function_name!())]
fn test_aaux_binary_group_binary(#[case] test_function_name: &str) {
    let tc = AAUX_BINARY_GROUP_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

static VAUX_BINARY_GROUP_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    "basic_test",
    PackBinaryTestCase {
        input: "64 12 34 56 78",
        parsed: Some(Pack::VAUXBinaryGroup(validated(
            BinaryGroup {
                group_data: [
                    u4::new(2),
                    u4::new(1),
                    u4::new(4),
                    u4::new(3),
                    u4::new(6),
                    u4::new(5),
                    u4::new(8),
                    u4::new(7)
                ]
            },
            *NTSC
        ))),
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_test(function_name!())]
fn test_vaux_binary_group_binary(#[case] test_function_name: &str) {
    let tc = VAUX_BINARY_GROUP_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}
