use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_vaux_source_binary", &VAUX_SOURCE_BINARY_TEST_CASES,),
    ("test_vaux_source_validation", &VAUX_SOURCE_VALIDATION_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

static VAUX_SOURCE_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    // These are from real tape transfers
    //
    // basic success case: from my Sony DCR-TRV460
    "basic_success",
    PackBinaryTestCase {
        input: "60 FF FF 00 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Camera),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // DVCPRO50 from https://archive.org/details/SMPTEColorBarsBadTracking
    "dvcpro50",
    PackBinaryTestCase {
        input: "60 FF FF 04 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Camera),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionMoreChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== SOURCE CODE TEST CASES =====
    //
    // Camera
    "source_code_camera",
    PackBinaryTestCase {
        input: "60 FF FF 00 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Camera),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Line MUSE
    "source_code_line_muse",
    PackBinaryTestCase {
        input: "60 EE FE 40 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::LineMUSE),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Line
    "source_code_line",
    PackBinaryTestCase {
        input: "60 FF FF 40 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Line),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Cable
    "source_code_cable",
    PackBinaryTestCase {
        input: "60 36 F4 80 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Cable),
                tv_channel: Some(436),
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Tuner
    "source_code_tuner",
    PackBinaryTestCase {
        input: "60 36 F4 C0 2B",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Tuner),
                tv_channel: Some(436),
                tuner_category: Some(0x2B),
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Prerecorded tape
    "source_code_prerecorded_tape",
    PackBinaryTestCase {
        input: "60 EE FE C0 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::PrerecordedTape),
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // No information
    "source_code_no_info",
    PackBinaryTestCase {
        input: "60 FF FF C0 FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: None,
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== BOUNDS TESTING: GENERAL =====
    //
    // maximum bounds
    "max_bounds",
    PackBinaryTestCase {
        input: "60 99 F9 C0 2B",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Tuner),
                tv_channel: Some(999),
                tuner_category: Some(0x2B),
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // minimum bounds
    "min_bounds",
    PackBinaryTestCase {
        input: "60 01 F0 C0 2B",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: Some(SourceCode::Tuner),
                tv_channel: Some(1),
                tuner_category: Some(0x2B),
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                bw_flag: BlackAndWhiteFlag::Color,
                color_frames_id: None,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ADDITIONAL CONTRIVED/SYNTHETIC TEST CASES =====
    //
    // Change a bunch of misc fields to different values from what we've been testing
    "misc_weird_fields",
    PackBinaryTestCase {
        input: "60 FF 1F EE FF",
        parsed: Some(Pack::VAUXSource(validated(
            VAUXSource {
                source_code: None,
                tv_channel: None,
                tuner_category: None,
                source_type: SourceType::Reserved14,
                field_count: 50,
                bw_flag: BlackAndWhiteFlag::BlackAndWhite,
                color_frames_id: Some(ColorFramesID::CLFColorFrameBOr3_4Field),
            },
            *PAL
        ))),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: TV CHANNEL IMPROPERLY PRESENT OR MISSING =====
    //
    // Camera: unexpected TV channel number
    "source_code_camera_with_tv_channel",
    PackBinaryTestCase {
        input: "60 12 F3 00 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: TV channel must be \
            absent for source code Camera"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Line: unexpected TV channel number
    "source_code_line_with_tv_channel",
    PackBinaryTestCase {
        input: "60 12 F3 40 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: invalid TV channel code \
            specified for line-based source code"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Cable: missing TV channel number
    "source_code_cable_without_tv_channel",
    PackBinaryTestCase {
        input: "60 FF FF 80 FF",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> source_code: a TV channel number is required for this source code value\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Cable: TV channel number is 0xE nibbles
    "source_code_cable_tv_channel_is_e",
    PackBinaryTestCase {
        input: "60 EE FE 80 FF",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> source_code: a TV channel number is required for this source code value\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: TUNER CATEGORY IMPROPERLY PRESENT OR MISSING =====
    //
    // Camera: unexpected tuner category
    "source_code_camera_with_tuner_category",
    PackBinaryTestCase {
        input: "60 FF FF 00 2B",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> source_code: a tuner category must not be provided if the source code \
               is not a tuner\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Tuner: missing tuner category
    "source_code_tuner_without_tuner_category",
    PackBinaryTestCase {
        input: "60 12 F3 C0 FF",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> source_code: a tuner category must be provided if the source code is a tuner\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: INVALID BINARY-CODED AND TOO-LOW VALUES =====
    //
    // high TV channel hundreds
    "high_tv_channel_hundreds",
    PackBinaryTestCase {
        input: "60 36 FA 80 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the TV channel number\n\
            Caused by:\n  \
            -> hundreds place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high TV channel tens
    "high_tv_channel_tens",
    PackBinaryTestCase {
        input: "60 A6 F4 80 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the TV channel number\n\
            Caused by:\n  \
            -> tens place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high TV channel units
    "high_tv_channel_units",
    PackBinaryTestCase {
        input: "60 3A F4 80 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the TV channel number\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // low TV channel
    "low_tv_channel",
    PackBinaryTestCase {
        input: "60 00 F0 80 FF",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> tv_channel: lower than 1\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success(function_name!())]
#[case::dvcpro50(function_name!())]
#[case::source_code_camera(function_name!())]
#[case::source_code_line_muse(function_name!())]
#[case::source_code_line(function_name!())]
#[case::source_code_cable(function_name!())]
#[case::source_code_tuner(function_name!())]
#[case::source_code_prerecorded_tape(function_name!())]
#[case::source_code_no_info(function_name!())]
#[case::max_bounds(function_name!())]
#[case::min_bounds(function_name!())]
#[case::misc_weird_fields(function_name!())]
#[case::source_code_camera_with_tv_channel(function_name!())]
#[case::source_code_line_with_tv_channel(function_name!())]
#[case::source_code_cable_without_tv_channel(function_name!())]
#[case::source_code_cable_tv_channel_is_e(function_name!())]
#[case::source_code_camera_with_tuner_category(function_name!())]
#[case::source_code_tuner_without_tuner_category(function_name!())]
#[case::high_tv_channel_hundreds(function_name!())]
#[case::high_tv_channel_tens(function_name!())]
#[case::high_tv_channel_units(function_name!())]
#[case::low_tv_channel(function_name!())]
fn test_vaux_source_binary(#[case] test_function_name: &str) {
    let tc = VAUX_SOURCE_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static VAUX_SOURCE_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<VAUXSource, PackContext>,
> = test_case_map!(
    //
    // ===== TEST TV CHANNEL IMPROPERLY PRESENT OR ABSENT =====
    //
    // source code of camera with a TV channel
    "source_code_camera_with_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Camera),
            tv_channel: Some(5),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number must not be provided for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of line MUSE with a TV channel
    "source_code_line_muse_with_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::LineMUSE),
            tv_channel: Some(5),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number must not be provided for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of line with a TV channel
    "source_code_line_with_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Line),
            tv_channel: Some(5),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number must not be provided for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of cable without a TV channel
    "source_code_cable_without_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Cable),
            tv_channel: None,
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number is required for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of tuner without a TV channel
    "source_code_tuner_without_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Tuner),
            tv_channel: None,
            tuner_category: Some(0x2B),
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number is required for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of prerecorded tape with a TV channel
    "source_code_prerecorded_tape_with_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::PrerecordedTape),
            tv_channel: Some(5),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number must not be provided for this source code value\n",
        ctx: *NTSC
    },
    //
    // source code of no info with a TV channel
    "source_code_no_info_with_tv_channel",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: None,
            tv_channel: Some(5),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "source_code: a TV channel number must not be provided for this source code value\n",
        ctx: *NTSC
    },
    //
    // ===== OTHER VALIDATIONS =====
    //
    // The value of 0 is tested in the binary conversion test cases above.  However, a
    // value >= 1000 can't be represented in 3-digit BCD, so we test it here.
    "tv_channel_high",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Cable),
            tv_channel: Some(1000),
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "tv_channel: greater than 999\n",
        ctx: *NTSC
    },
    //
    // tuner category of 0xFF needs to be represented as None, not Some(0xFF)
    "tuner_category_ff",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Tuner),
            tv_channel: Some(123),
            tuner_category: Some(0xFF),
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "tuner_category: instead of specifying Some(0xFF), use None to \
              indicate no information\n",
        ctx: *NTSC
    },
    //
    // invalid field count for NTSC
    "invalid_field_count_ntsc",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Camera),
            tv_channel: None,
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 50, // invalid
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "field_count: field count of 50 does not match the expected value of 60 \
              for system 525-60\n",
        ctx: *NTSC
    },
    //
    // invalid field count for PAL
    "invalid_field_count_pal",
    ValidateFailureTestCase {
        value: VAUXSource {
            source_code: Some(SourceCode::Camera),
            tv_channel: None,
            tuner_category: None,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60, // invalid
            bw_flag: BlackAndWhiteFlag::Color,
            color_frames_id: None,
        },
        err: "field_count: field count of 60 does not match the expected value of 50 \
              for system 625-50\n",
        ctx: *PAL
    }
);

#[googletest::test]
#[rstest]
#[case::source_code_camera_with_tv_channel(function_name!())]
#[case::source_code_line_muse_with_tv_channel(function_name!())]
#[case::source_code_line_with_tv_channel(function_name!())]
#[case::source_code_cable_without_tv_channel(function_name!())]
#[case::source_code_tuner_without_tv_channel(function_name!())]
#[case::source_code_prerecorded_tape_with_tv_channel(function_name!())]
#[case::source_code_no_info_with_tv_channel(function_name!())]
#[case::tv_channel_high(function_name!())]
#[case::tuner_category_ff(function_name!())]
#[case::invalid_field_count_ntsc(function_name!())]
#[case::invalid_field_count_pal(function_name!())]
fn test_vaux_source_validation(#[case] test_function_name: &str) {
    let tc = VAUX_SOURCE_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}
