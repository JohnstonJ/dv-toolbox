use arbitrary_int::{u1, u7};
use insta::{assert_debug_snapshot, with_settings};
use itertools::Itertools;
use num::rational::Ratio;
use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_aaux_source_control_binary", &AAUX_SOURCE_CONTROL_BINARY_TEST_CASES,),
    ("test_aaux_source_control_validation", &AAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

static AAUX_SOURCE_CONTROL_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    // These are from real tape transfers
    //
    // basic success case: from my Sony DCR-TRV460, first audio channel block
    "basic_success_first_audio_block",
    PackBinaryTestCase {
        input: "51 03 CF A0 FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::ONE),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // basic success case: from my Sony DCR-TRV460, second (empty) audio channel block
    "basic_success_second_empty_audio_block",
    PackBinaryTestCase {
        input: "51 03 FF A0 FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::InvalidRecording,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::ONE),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ADDITIONAL CONTRIVED/SYNTHETIC TEST CASES: TEST PLAYBACK SPEEDS =====
    //
    // stopped playback speed
    "stopped_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF 80 FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::ZERO),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // super slow < 1/16 playback speed
    "super_slow_1_16_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF 81 FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::new(1, 32)),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // 0 + 1/4 playback speed
    "x_0_and_1_4_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF 8E FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::new(1, 4)),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // 1/2 + 3/32 playback speed
    "x_1_2_and_3_32_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF 93 FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::new(19, 32)),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // 32 + 28 playback speed
    "x_32_and_28_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF FE FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::from(60)),
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // unknown playback speed
    "unknown_playback_speed",
    PackBinaryTestCase {
        input: "51 03 CF FF FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: None,
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: None,
                reserved: u1::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ADDITIONAL CONTRIVED/SYNTHETIC TEST CASES: OTHER TESTS =====
    //
    // various values (1)
    "various_values_1",
    PackBinaryTestCase {
        input: "51 0A 8D 20 7F",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: Some(SourceSituation::SourceWithAudienceRestrictions),
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed3OrMore),
                recording_start_point: false,
                recording_end_point: true,
                recording_mode: AAUXRecordingMode::Original,
                insert_channel: Some(AAUXInsertChannel::Channels3_4),
                genre_category: None,
                direction: Direction::Reverse,
                playback_speed: Some(Ratio::<u8>::ONE),
                reserved: u1::new(0x0),
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
        input: "51 93 6F C3 AA",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::OneGenerationOnly,
                source_situation: None,
                input_source: Some(InputSource::Digital),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: true,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::TwoChannelInsert,
                insert_channel: None,
                genre_category: Some(u7::new(0x2A)),
                direction: Direction::Forward,
                playback_speed: Some(Ratio::<u8>::from(4) + Ratio::<u8>::new(3, 4)),
                reserved: u1::new(0x1),
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
        input: "51 FF FF FF FF",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NotPermitted,
                source_situation: None,
                input_source: None,
                compression_count: None,
                recording_start_point: false,
                recording_end_point: false,
                recording_mode: AAUXRecordingMode::InvalidRecording,
                insert_channel: None,
                genre_category: None,
                direction: Direction::Forward,
                playback_speed: None,
                reserved: u1::new(0x1),
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
        input: "51 00 00 00 00",
        parsed: Some(Pack::AAUXSourceControl(validated(
            AAUXSourceControl {
                copy_protection: CopyProtection::NoRestriction,
                source_situation: Some(SourceSituation::ScrambledSourceWithAudienceRestrictions),
                input_source: Some(InputSource::Analog),
                compression_count: Some(CompressionCount::Compressed1),
                recording_start_point: true,
                recording_end_point: true,
                recording_mode: AAUXRecordingMode::Reserved0,
                insert_channel: Some(AAUXInsertChannel::Channel1),
                genre_category: Some(u7::new(0x00)),
                direction: Direction::Reverse,
                playback_speed: Some(Ratio::<u8>::ZERO),
                reserved: u1::new(0x0),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success_first_audio_block(function_name!())]
#[case::basic_success_second_empty_audio_block(function_name!())]
#[case::stopped_playback_speed(function_name!())]
#[case::super_slow_1_16_playback_speed(function_name!())]
#[case::x_0_and_1_4_playback_speed(function_name!())]
#[case::x_1_2_and_3_32_playback_speed(function_name!())]
#[case::x_32_and_28_playback_speed(function_name!())]
#[case::unknown_playback_speed(function_name!())]
#[case::various_values_1(function_name!())]
#[case::various_values_2(function_name!())]
#[case::all_bits_set(function_name!())]
#[case::all_bits_clear(function_name!())]
fn test_aaux_source_control_binary(#[case] test_function_name: &str) {
    let tc = AAUX_SOURCE_CONTROL_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static AAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<AAUXSourceControl, PackContext>,
> = test_case_map!(
    //
    // invalid genre category
    "invalid_genre_category",
    ValidateFailureTestCase {
        value: AAUXSourceControl {
            copy_protection: CopyProtection::NoRestriction,
            source_situation: Some(SourceSituation::ScrambledSourceWithAudienceRestrictions),
            input_source: Some(InputSource::Analog),
            compression_count: Some(CompressionCount::Compressed1),
            recording_start_point: true,
            recording_end_point: true,
            recording_mode: AAUXRecordingMode::Reserved0,
            insert_channel: Some(AAUXInsertChannel::Channel1),
            genre_category: Some(u7::new(0x7F)), // should be None instead
            direction: Direction::Reverse,
            playback_speed: Some(Ratio::<u8>::ZERO),
            reserved: u1::new(0x0),
        },
        err: "genre_category: instead of specifying Some(0x7F), use None to indicate \
            no information\n",
        ctx: *NTSC
    },
    //
    // invalid playback speed
    "invalid_playback_speed",
    ValidateFailureTestCase {
        value: AAUXSourceControl {
            copy_protection: CopyProtection::NoRestriction,
            source_situation: Some(SourceSituation::ScrambledSourceWithAudienceRestrictions),
            input_source: Some(InputSource::Analog),
            compression_count: Some(CompressionCount::Compressed1),
            recording_start_point: true,
            recording_end_point: true,
            recording_mode: AAUXRecordingMode::Reserved0,
            insert_channel: Some(AAUXInsertChannel::Channel1),
            genre_category: None,
            direction: Direction::Reverse,
            playback_speed: Some(Ratio::<u8>::new(99, 100)), // no way to represent this in binary
            reserved: u1::new(0x0),
        },
        err: "playback_speed: playback speed 99/100 not supported: only playback speeds \
            returned by the valid_playback_speeds function are supported\n",
        ctx: *NTSC
    }
);

#[googletest::test]
#[rstest]
#[case::invalid_genre_category(function_name!())]
#[case::invalid_playback_speed(function_name!())]
fn test_aaux_source_control_validation(#[case] test_function_name: &str) {
    let tc = AAUX_SOURCE_CONTROL_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}

// ==================== VALID PLAYBACK SPEEDS SNAPSHOT TESTING ====================
// Tests that the valid playback speeds match a snapshot, which should be compared vs the table in
// the IEC specification.

#[googletest::test]
fn test_valid_playback_speeds() {
    let displayed_speeds: Vec<String> = AAUXSourceControl::valid_playback_speeds()
        .iter()
        .sorted()
        .map(|s| format!("{} {}", s.trunc(), s.fract()))
        .collect();
    with_settings!({prepend_module_to_snapshot => false}, {
        assert_debug_snapshot!(displayed_speeds);
    });
}
