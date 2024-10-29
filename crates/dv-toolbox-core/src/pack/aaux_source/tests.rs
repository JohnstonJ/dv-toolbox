use arbitrary_int::{u2, u4};
use rstest::rstest;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_aaux_source_binary", &AAUX_SOURCE_BINARY_TEST_CASES,),
    ("test_aaux_source_validation", &AAUX_SOURCE_VALIDATION_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

static AAUX_SOURCE_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    // These are from real tape transfers
    //
    // basic success case: from my Sony DCR-TRV460, first audio channel block
    "basic_success_first_audio_block",
    PackBinaryTestCase {
        input: "50 CE 30 C0 D1",
        parsed: Some(Pack::AAUXSource(validated(
            AAUXSource {
                audio_sample_rate: 32_000,
                quantization: AudioQuantization::NonLinear12Bit,
                audio_frame_size: 1_067,
                locked_mode: LockedMode::Unlocked,
                stereo_mode: StereoMode::MultiStereoAudio,
                audio_block_channel_count: 2,
                audio_mode: u4::new(0x0),
                audio_block_pairing: AudioBlockPairing::Independent,
                multi_language: false,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                emphasis_on: false,
                emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // basic success case: from my Sony DCR-TRV460, second (empty) audio channel block
    "basic_success_second_audio_block",
    PackBinaryTestCase {
        input: "50 CE 3F C0 D1",
        parsed: Some(Pack::AAUXSource(validated(
            AAUXSource {
                audio_sample_rate: 32_000,
                quantization: AudioQuantization::NonLinear12Bit,
                audio_frame_size: 1_067,
                locked_mode: LockedMode::Unlocked,
                stereo_mode: StereoMode::MultiStereoAudio,
                audio_block_channel_count: 2,
                audio_mode: u4::new(0xF), // no information (I guess means this channel is empty)
                audio_block_pairing: AudioBlockPairing::Independent,
                multi_language: false,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 60,
                emphasis_on: false,
                emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
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
    // various values (1)
    "various_values_1",
    PackBinaryTestCase {
        input: "50 00 B5 E0 C8",
        parsed: Some(Pack::AAUXSource(validated(
            AAUXSource {
                audio_sample_rate: 44_100,
                quantization: AudioQuantization::Linear16Bit,
                audio_frame_size: 1_742,
                locked_mode: LockedMode::Locked,
                stereo_mode: StereoMode::LumpedAudio,
                audio_block_channel_count: 2,
                audio_mode: u4::new(0x5),
                audio_block_pairing: AudioBlockPairing::Independent,
                multi_language: false,
                source_type: SourceType::StandardDefinitionCompressedChroma,
                field_count: 50,
                emphasis_on: false,
                emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
                reserved: u2::new(0x2),
            },
            *PAL
        ))),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // various values (2)
    "various_values_2",
    PackBinaryTestCase {
        input: "50 E8 0A 02 02",
        parsed: Some(Pack::AAUXSource(validated(
            AAUXSource {
                audio_sample_rate: 48_000,
                quantization: AudioQuantization::Linear20Bit,
                audio_frame_size: 1_620,
                locked_mode: LockedMode::Unlocked,
                stereo_mode: StereoMode::MultiStereoAudio,
                audio_block_channel_count: 1,
                audio_mode: u4::new(0xA),
                audio_block_pairing: AudioBlockPairing::Paired,
                multi_language: true,
                source_type: SourceType::AnalogHighDefinition1125_1250,
                field_count: 60,
                emphasis_on: true,
                emphasis_time_constant: EmphasisTimeConstant::Reserved,
                reserved: u2::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: VARIOUS VALIDATION FAILURES =====
    //
    // invalid audio sample rate
    "invalid_audio_sample_rate",
    PackBinaryTestCase {
        input: "50 CE 30 C0 D9",
        err: Some(
            "Pack failed deserialization of raw bytes: smp value of 3 does not correspond to a \
            known audio sample rate"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    "invalid_audio_frame_size",
    PackBinaryTestCase {
        input: "50 DC 30 C0 D1",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> audio_frame_size: video frame contains 1081 audio samples, which is above the \
               maximum of 1080 for video system 525-60 and audio sample rate 32000 Hz\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    "invalid_audio_block_channel_count",
    PackBinaryTestCase {
        input: "50 CE 50 C0 D1",
        err: Some(
            "Pack failed deserialization of raw bytes: chn value of 2 does not correspond to \
            a known number of channels per audio block"
        ),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success_first_audio_block(function_name!())]
#[case::basic_success_second_audio_block(function_name!())]
#[case::various_values_1(function_name!())]
#[case::various_values_2(function_name!())]
#[case::invalid_audio_sample_rate(function_name!())]
#[case::invalid_audio_frame_size(function_name!())]
#[case::invalid_audio_block_channel_count(function_name!())]
fn test_aaux_source_binary(#[case] test_function_name: &str) {
    let tc = AAUX_SOURCE_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static AAUX_SOURCE_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<AAUXSource, PackContext>,
> = test_case_map!(
    //
    // invalid audio sample rate
    "invalid_audio_sample_rate",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 14_000, // not valid
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_067,
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 2,
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "audio_frame_size: cannot validate the audio frame size because the audio \
            sample rate is unsupported\n\
            audio_sample_rate: audio sample rate of 14000 is not one of the supported values \
            of 32000, 44100, or 48000 Hz\n",
        ctx: *NTSC
    },
    //
    // low audio frame size
    "low_audio_frame_size",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 32_000,
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_052, // too low
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 2,
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "audio_frame_size: video frame contains 1052 audio samples, which is below the \
            required minimum of 1053 for video system 525-60 and audio sample rate 32000 Hz\n",
        ctx: *NTSC
    },
    //
    // high audio frame size
    "high_audio_frame_size",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 32_000,
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_081, // too high
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 2,
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "audio_frame_size: video frame contains 1081 audio samples, which is above the \
            maximum of 1080 for video system 525-60 and audio sample rate 32000 Hz\n",
        ctx: *NTSC
    },
    //
    // low audio block channel count
    "low_audio_block_channel_count",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 32_000,
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_067,
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 0, // too low
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "audio_block_channel_count: lower than 1\n",
        ctx: *NTSC
    },
    //
    // high audio block channel count
    "high_audio_block_channel_count",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 32_000,
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_067,
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 3, // too high
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60,
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "audio_block_channel_count: greater than 2\n",
        ctx: *NTSC
    },
    //
    // invalid field count for NTSC
    "invalid_field_count_ntsc",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 32_000,
            quantization: AudioQuantization::NonLinear12Bit,
            audio_frame_size: 1_067,
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 2,
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 50, // invalid
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "field_count: field count of 50 does not match the expected value of 60 \
            for system 525-60\n",
        ctx: *NTSC
    },
    //
    // invalid field count for PAL
    "invalid_field_count_pal",
    ValidateFailureTestCase {
        value: AAUXSource {
            audio_sample_rate: 44_100,
            quantization: AudioQuantization::Linear16Bit,
            audio_frame_size: 1_742,
            locked_mode: LockedMode::Unlocked,
            stereo_mode: StereoMode::MultiStereoAudio,
            audio_block_channel_count: 2,
            audio_mode: u4::new(0x0),
            audio_block_pairing: AudioBlockPairing::Independent,
            multi_language: false,
            source_type: SourceType::StandardDefinitionCompressedChroma,
            field_count: 60, // invalid
            emphasis_on: false,
            emphasis_time_constant: EmphasisTimeConstant::Emphasis50_15,
            reserved: u2::new(0x3),
        },
        err: "field_count: field count of 60 does not match the expected value of 50 \
            for system 625-50\n",
        ctx: *PAL
    }
);

#[googletest::test]
#[rstest]
#[case::invalid_audio_sample_rate(function_name!())]
#[case::low_audio_frame_size(function_name!())]
#[case::high_audio_frame_size(function_name!())]
#[case::low_audio_block_channel_count(function_name!())]
#[case::high_audio_block_channel_count(function_name!())]
#[case::invalid_field_count_ntsc(function_name!())]
#[case::invalid_field_count_pal(function_name!())]
fn test_aaux_source_validation(#[case] test_function_name: &str) {
    let tc = AAUX_SOURCE_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}
