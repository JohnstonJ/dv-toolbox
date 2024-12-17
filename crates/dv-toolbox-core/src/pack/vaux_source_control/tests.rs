use arbitrary_int::{u2, u3, u7};
use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_vaux_source_control_binary", &VAUX_SOURCE_CONTROL_BINARY_TEST_CASES,),
    ("test_vaux_source_control_validation", &VAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

static VAUX_SOURCE_CONTROL_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    // These are from real tape transfers
    //
    // basic success case: from my Sony DCR-TRV460
    "basic_success",
    PackBinaryTestCase {
        input: "61 03 80 FC FF",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x0),
                display_mode: u3::new(0x0),
                frame_field: FrameField::Both,
                first_second: 1,
                frame_change: FrameChange::DifferentFromPrevious,
                interlaced: true,
                still_field_picture: StillFieldPicture::HalfFrameTime,
                still_camera_picture: false,
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_mode: VAUXRecordingMode::Original,
                genre_category: None,
                reserved: u3::new(0x4),
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
        input: "61 33 C8 FC FF",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x0),
                display_mode: u3::new(0x0),
                frame_field: FrameField::Both,
                first_second: 1,
                frame_change: FrameChange::DifferentFromPrevious,
                interlaced: true,
                still_field_picture: StillFieldPicture::HalfFrameTime,
                still_camera_picture: false,
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: None,
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_mode: VAUXRecordingMode::Original,
                genre_category: None,
                reserved: u3::new(0x7),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ADDITIONAL CONTRIVED/SYNTHETIC TEST CASES =====
    //
    // various values (1)
    "various_values_1",
    PackBinaryTestCase {
        input: "61 0A ED AA 2A",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x2),
                display_mode: u3::new(0x5),
                frame_field: FrameField::Both,
                first_second: 2,
                frame_change: FrameChange::DifferentFromPrevious,
                interlaced: false,
                still_field_picture: StillFieldPicture::HalfFrameTime,
                still_camera_picture: true,
                copy_protection: CopyProtection::NoRestriction,
                source_situation: Some(SourceSituation::SourceWithAudienceRestrictions),
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed3OrMore),
                recording_start_point: false,
                recording_mode: VAUXRecordingMode::Insert,
                genre_category: Some(u7::new(0x2A)),
                reserved: u3::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // various values (2)
    "various_values_2",
    PackBinaryTestCase {
        input: "61 93 02 55 BB",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x1),
                display_mode: u3::new(0x2),
                frame_field: FrameField::OnlyOne,
                first_second: 1,
                frame_change: FrameChange::SameAsPrevious,
                interlaced: true,
                still_field_picture: StillFieldPicture::NoGap,
                still_camera_picture: false,
                copy_protection: CopyProtection::OneGenerationOnly,
                source_situation: None,
                input_source: Some(InputSource::Digital),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: true,
                recording_mode: VAUXRecordingMode::Original,
                genre_category: Some(u7::new(0x3B)),
                reserved: u3::new(0x4),
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
        input: "61 FF FF FF FF",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x3),
                display_mode: u3::new(0x7),
                frame_field: FrameField::Both,
                first_second: 1,
                frame_change: FrameChange::DifferentFromPrevious,
                interlaced: true,
                still_field_picture: StillFieldPicture::HalfFrameTime,
                still_camera_picture: false,
                copy_protection: CopyProtection::NotPermitted,
                source_situation: None,
                input_source: None,
                compression_count: None,
                recording_start_point: false,
                recording_mode: VAUXRecordingMode::InvalidRecording,
                genre_category: None,
                reserved: u3::new(0x7),
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
        input: "61 00 00 00 00",
        parsed: Some(Pack::VAUXSourceControl(validated(
            VAUXSourceControl {
                broadcast_system: u2::new(0x0),
                display_mode: u3::new(0x0),
                frame_field: FrameField::OnlyOne,
                first_second: 2,
                frame_change: FrameChange::SameAsPrevious,
                interlaced: false,
                still_field_picture: StillFieldPicture::NoGap,
                still_camera_picture: true,
                copy_protection: CopyProtection::NoRestriction,
                source_situation: Some(SourceSituation::ScrambledSourceWithAudienceRestrictions),
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: true,
                recording_mode: VAUXRecordingMode::Original,
                genre_category: Some(u7::new(0x00)),
                reserved: u3::new(0x0),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success(function_name!())]
#[case::dvcpro50(function_name!())]
#[case::various_values_1(function_name!())]
#[case::various_values_2(function_name!())]
#[case::all_bits_set(function_name!())]
#[case::all_bits_clear(function_name!())]
fn test_vaux_source_control_binary(#[case] test_function_name: &str) {
    let tc = VAUX_SOURCE_CONTROL_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static VAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<VAUXSourceControl, PackContext>,
> = test_case_map!(
    //
    // first/second flag is too low
    "first_second_low",
    ValidateFailureTestCase {
        value: VAUXSourceControl {
            broadcast_system: u2::new(0x0),
            display_mode: u3::new(0x0),
            frame_field: FrameField::Both,
            first_second: 0,
            frame_change: FrameChange::DifferentFromPrevious,
            interlaced: true,
            still_field_picture: StillFieldPicture::HalfFrameTime,
            still_camera_picture: false,
            copy_protection: CopyProtection::NoRestriction,
            source_situation: None,
            input_source: Some(InputSource::Analog),
            compression_count: Some(CompressionCount::Compressed1),
            recording_start_point: false,
            recording_mode: VAUXRecordingMode::Original,
            genre_category: None,
            reserved: u3::new(0x0),
        },
        err: "first_second: lower than 1\n",
        ctx: *NTSC
    },
    //
    // first/second flag is too high
    "first_second_high",
    ValidateFailureTestCase {
        value: VAUXSourceControl {
            broadcast_system: u2::new(0x0),
            display_mode: u3::new(0x0),
            frame_field: FrameField::Both,
            first_second: 3,
            frame_change: FrameChange::DifferentFromPrevious,
            interlaced: true,
            still_field_picture: StillFieldPicture::HalfFrameTime,
            still_camera_picture: false,
            copy_protection: CopyProtection::NoRestriction,
            source_situation: None,
            input_source: Some(InputSource::Analog),
            compression_count: Some(CompressionCount::Compressed1),
            recording_start_point: false,
            recording_mode: VAUXRecordingMode::Original,
            genre_category: None,
            reserved: u3::new(0x0),
        },
        err: "first_second: greater than 2\n",
        ctx: *NTSC
    },
    //
    // invalid genre category
    "invalid_genre_category",
    ValidateFailureTestCase {
        value: VAUXSourceControl {
            broadcast_system: u2::new(0x0),
            display_mode: u3::new(0x0),
            frame_field: FrameField::Both,
            first_second: 2,
            frame_change: FrameChange::DifferentFromPrevious,
            interlaced: true,
            still_field_picture: StillFieldPicture::HalfFrameTime,
            still_camera_picture: false,
            copy_protection: CopyProtection::NoRestriction,
            source_situation: None,
            input_source: Some(InputSource::Analog),
            compression_count: Some(CompressionCount::Compressed1),
            recording_start_point: false,
            recording_mode: VAUXRecordingMode::Original,
            genre_category: Some(u7::new(0x7F)), // should be None instead
            reserved: u3::new(0x0),
        },
        err: "genre_category: instead of specifying Some(0x7F), use None to indicate \
            no information\n",
        ctx: *NTSC
    }
);

#[googletest::test]
#[rstest]
#[case::first_second_low(function_name!())]
#[case::first_second_high(function_name!())]
#[case::invalid_genre_category(function_name!())]
fn test_vaux_source_control_validation(#[case] test_function_name: &str) {
    let tc = VAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}
