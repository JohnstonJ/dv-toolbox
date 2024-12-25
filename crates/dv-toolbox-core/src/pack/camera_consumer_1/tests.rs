use arbitrary_int::{u2, u4};
use insta::{assert_debug_snapshot, with_settings};
use itertools::Itertools;
use rstest::rstest;
use rust_decimal_macros::dec;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_camera_consumer_1_binary", &CAMERA_CONSUMER_1_BINARY_TEST_CASES,),
    ("test_camera_consumer_1_validation", &CAMERA_CONSUMER_1_VALIDATION_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

static CAMERA_CONSUMER_1_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    // These are from real tape transfers
    //
    // basic success case: from my Sony DCR-TRV460
    "basic_success_dcr_trv460",
    PackBinaryTestCase {
        input: "70 C5 07 1F FF",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
                iris: Some(dec!(1.5)),
                auto_gain_control: Some(u4::new(7)),
                white_balance_mode: Some(WhiteBalanceMode::Automatic),
                white_balance: None,
                focus_mode: FocusMode::Manual,
                focus_position: None,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // basic success case: from a tape originating from some old Digital8 camcorder 20 years ago
    "basic_success_old_camcorder",
    PackBinaryTestCase {
        input: "70 DE 01 1F FF",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
                iris: Some(dec!(13.5)),
                auto_gain_control: Some(u4::new(1)),
                white_balance_mode: Some(WhiteBalanceMode::Automatic),
                white_balance: None,
                focus_mode: FocusMode::Manual,
                focus_position: None,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ADDITIONAL CONTRIVED/SYNTHETIC TEST CASES =====
    //
    // everything's kind of in the middle here
    "all_values_in_range",
    PackBinaryTestCase {
        input: "70 E5 37 44 56",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::IrisPriority),
                iris: Some(dec!(24.7)),
                auto_gain_control: Some(u4::new(7)),
                white_balance_mode: Some(WhiteBalanceMode::OnePush),
                white_balance: Some(WhiteBalance::Sunlight),
                focus_mode: FocusMode::Automatic,
                focus_position: Some(2100),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // maximum values that are calculated (especially iris/focus position)
    "max_calculated_values",
    PackBinaryTestCase {
        input: "70 FC 3E 44 7E",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::IrisPriority),
                iris: Some(dec!(181.0)),
                auto_gain_control: Some(u4::new(14)),
                white_balance_mode: Some(WhiteBalanceMode::OnePush),
                white_balance: Some(WhiteBalance::Sunlight),
                focus_mode: FocusMode::Automatic,
                focus_position: Some(3100),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // wide open iris
    "wide_open_iris",
    PackBinaryTestCase {
        input: "70 FD 37 44 7E",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::IrisPriority),
                iris: Some(dec!(0.0)),
                auto_gain_control: Some(u4::new(7)),
                white_balance_mode: Some(WhiteBalanceMode::OnePush),
                white_balance: Some(WhiteBalance::Sunlight),
                focus_mode: FocusMode::Automatic,
                focus_position: Some(3100),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // closed iris
    "closed_iris",
    PackBinaryTestCase {
        input: "70 FE 37 44 7E",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::IrisPriority),
                iris: Some(dec!(999.9)),
                auto_gain_control: Some(u4::new(7)),
                white_balance_mode: Some(WhiteBalanceMode::OnePush),
                white_balance: Some(WhiteBalance::Sunlight),
                focus_mode: FocusMode::Automatic,
                focus_position: Some(3100),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // all bits set
    "all_bits_set",
    PackBinaryTestCase {
        input: "70 FF FF FF FF",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: None,
                iris: None,
                auto_gain_control: None,
                white_balance_mode: None,
                white_balance: None,
                focus_mode: FocusMode::Manual,
                focus_position: None,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // all bits clear
    "all_bits_clear",
    PackBinaryTestCase {
        input: "70 00 00 00 00",
        parsed: Some(Pack::CameraConsumer1(validated(
            CameraConsumer1 {
                auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
                iris: Some(dec!(1.0)),
                auto_gain_control: Some(u4::new(0)),
                white_balance_mode: Some(WhiteBalanceMode::Automatic),
                white_balance: Some(WhiteBalance::Candle),
                focus_mode: FocusMode::Automatic,
                focus_position: Some(0),
                reserved: u2::new(0x0),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success_dcr_trv460(function_name!())]
#[case::basic_success_old_camcorder(function_name!())]
#[case::all_values_in_range(function_name!())]
#[case::max_calculated_values(function_name!())]
#[case::wide_open_iris(function_name!())]
#[case::closed_iris(function_name!())]
#[case::all_bits_set(function_name!())]
#[case::all_bits_clear(function_name!())]
fn test_camera_consumer_1_binary(#[case] test_function_name: &str) {
    let tc = CAMERA_CONSUMER_1_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static CAMERA_CONSUMER_1_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<CameraConsumer1, PackContext>,
> = test_case_map!(
    //
    // unsupported iris value
    "invalid_iris",
    ValidateFailureTestCase {
        value: CameraConsumer1 {
            auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
            iris: Some(dec!(50.2)),
            auto_gain_control: Some(u4::new(7)),
            white_balance_mode: Some(WhiteBalanceMode::Automatic),
            white_balance: None,
            focus_mode: FocusMode::Manual,
            focus_position: None,
            reserved: u2::new(0x3),
        },
        err: "iris: iris value 50.2 not supported: only iris values returned by the \
            valid_iris_values function are supported\n",
        ctx: *NTSC
    },
    //
    // unsupported iris value
    "invalid_auto_gain_control",
    ValidateFailureTestCase {
        value: CameraConsumer1 {
            auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
            iris: Some(dec!(1.5)),
            auto_gain_control: Some(u4::new(15)),
            white_balance_mode: Some(WhiteBalanceMode::Automatic),
            white_balance: None,
            focus_mode: FocusMode::Manual,
            focus_position: None,
            reserved: u2::new(0x3),
        },
        err: "auto_gain_control: instead of specifying Some(0xF), use None to \
            indicate no information\n",
        ctx: *NTSC
    },
    //
    // unsupported focus position
    "invalid_focus_position",
    ValidateFailureTestCase {
        value: CameraConsumer1 {
            auto_exposure_mode: Some(AutoExposureMode::FullAutomatic),
            iris: Some(dec!(1.5)),
            auto_gain_control: Some(u4::new(7)),
            white_balance_mode: Some(WhiteBalanceMode::Automatic),
            white_balance: None,
            focus_mode: FocusMode::Manual,
            focus_position: Some(1234),
            reserved: u2::new(0x3),
        },
        err: "focus_position: focus position 1234 not supported: only focus positions \
            returned by the valid_focus_positions function are supported\n",
        ctx: *NTSC
    }
);

#[googletest::test]
#[rstest]
#[case::invalid_iris(function_name!())]
#[case::invalid_auto_gain_control(function_name!())]
#[case::invalid_focus_position(function_name!())]
fn test_camera_consumer_1_validation(#[case] test_function_name: &str) {
    let tc = CAMERA_CONSUMER_1_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}

// ==================== VALID VALUES SNAPSHOT TESTING ====================
// Tests that the valid values match a snapshot, which should be compared vs the specified values in
// the IEC specification.

#[googletest::test]
fn test_valid_iris_values() {
    let displayed_iris_values: Vec<String> =
        CameraConsumer1::valid_iris_values().iter().sorted().map(|i| format!("{}", i)).collect();
    with_settings!({prepend_module_to_snapshot => false}, {
        assert_debug_snapshot!(displayed_iris_values);
    });
}

#[googletest::test]
fn test_valid_focus_positions() {
    let displayed_focus_positions: Vec<String> = CameraConsumer1::valid_focus_positions()
        .iter()
        .sorted()
        .map(|i| format!("{}", i))
        .collect();
    with_settings!({prepend_module_to_snapshot => false}, {
        assert_debug_snapshot!(displayed_focus_positions);
    });
}
