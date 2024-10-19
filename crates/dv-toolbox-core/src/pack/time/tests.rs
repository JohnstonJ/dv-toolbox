use rstest::rstest;
use serde_test::Token;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

const ZERO_TIMECODE: TitleTimecode = TitleTimecode {
    timecode: Timecode::<TimeValueWithRequiredFrame> {
        time: TimeValueWithRequiredFrame {
            hour: 0,
            minute: 0,
            second: 0,
            drop_frame: false,
            frame: 0,
        },
        color_frame: ColorFrame::Unsynchronized,
        polarity_correction: PolarityCorrection::Even,
        binary_group_flag: BinaryGroupFlag::TimeUnspecifiedGroupUnspecified,
    },
    blank_flag: BlankFlag::Discontinuous,
};

fn zero_tc<F>(f: F, ctx: PackContext) -> Option<Pack>
where
    F: FnOnce(&mut TitleTimecode),
{
    let mut tc = ZERO_TIMECODE;
    f(&mut tc);
    Some(Pack::TitleTimecode(validated(tc, ctx)))
}

test_all_test_cases_ran!(
    ("test_title_timecode_binary", &TITLE_TIMECODE_BINARY_TEST_CASES),
    ("test_aaux_recording_time_binary", &AAUX_RECORDING_TIME_BINARY_TEST_CASES),
    ("test_vaux_recording_time_binary", &VAUX_RECORDING_TIME_BINARY_TEST_CASES),
    ("test_title_timecode_validation", &TITLE_TIMECODE_VALIDATION_TEST_CASES),
    ("test_title_timecode_serde", &TITLE_TIMECODE_SERDE_TEST_CASES),
    ("test_title_timecode_deserialize_error", &TITLE_TIMECODE_DESERIALIZE_ERROR_TEST_CASES),
    ("test_recording_time_serde", &RECORDING_TIME_SERDE_TEST_CASES)
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

//
// Do most heavy testing on the TitleTimecode
//

static TITLE_TIMECODE_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    //
    // success NTSC, no TITLE BINARY pack, blank flag continuous
    "success_ntsc_basic",
    PackBinaryTestCase {
        input: "13 D5 B4 D7 D3",
        parsed: Some(Pack::TitleTimecode(validated(
            TitleTimecode {
                timecode: Timecode::<TimeValueWithRequiredFrame> {
                    time: TimeValueWithRequiredFrame {
                        hour: 13,
                        minute: 57,
                        second: 34,
                        drop_frame: true,
                        frame: 15,
                    },
                    color_frame: ColorFrame::Synchronized,
                    polarity_correction: PolarityCorrection::Odd,
                    binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
                },
                blank_flag: BlankFlag::Continuous,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success NTSC, drop frame True, do not drop 00:10:00;01
    "success_ntsc_do_not_drop",
    PackBinaryTestCase {
        input: "13 41 00 10 00",
        parsed: Some(Pack::TitleTimecode(validated(
            TitleTimecode {
                timecode: Timecode::<TimeValueWithRequiredFrame> {
                    time: TimeValueWithRequiredFrame {
                        hour: 0,
                        minute: 10,
                        second: 0,
                        drop_frame: true,
                        frame: 1,
                    },
                    color_frame: ColorFrame::Unsynchronized,
                    polarity_correction: PolarityCorrection::Even,
                    binary_group_flag: BinaryGroupFlag::TimeUnspecifiedGroupUnspecified,
                },
                blank_flag: BlankFlag::Discontinuous,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success PAL, drop frame False, do not drop 00:09:00:01
    "success_pal_do_not_drop",
    PackBinaryTestCase {
        input: "13 01 00 09 00",
        parsed: Some(Pack::TitleTimecode(validated(
            TitleTimecode {
                timecode: Timecode::<TimeValueWithRequiredFrame> {
                    time: TimeValueWithRequiredFrame {
                        hour: 0,
                        minute: 9,
                        second: 0,
                        drop_frame: false,
                        frame: 1,
                    },
                    color_frame: ColorFrame::Unsynchronized,
                    polarity_correction: PolarityCorrection::Even,
                    binary_group_flag: BinaryGroupFlag::TimeUnspecifiedGroupUnspecified,
                },
                blank_flag: BlankFlag::Discontinuous,
            },
            *PAL
        ))),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // max bounds NTSC, no TITLE BINARY pack
    "max_bounds_ntsc",
    PackBinaryTestCase {
        input: "13 29 D9 D9 E3",
        parsed: Some(Pack::TitleTimecode(validated(
            TitleTimecode {
                timecode: Timecode::<TimeValueWithRequiredFrame> {
                    time: TimeValueWithRequiredFrame {
                        hour: 23,
                        minute: 59,
                        second: 59,
                        drop_frame: false,
                        frame: 29,
                    },
                    color_frame: ColorFrame::Unsynchronized,
                    polarity_correction: PolarityCorrection::Odd,
                    binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
                },
                blank_flag: BlankFlag::Discontinuous,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // max bounds PAL, no TITLE BINARY pack
    "max_bounds_pal",
    PackBinaryTestCase {
        input: "13 A4 D9 D9 E3",
        parsed: Some(Pack::TitleTimecode(validated(
            TitleTimecode {
                timecode: Timecode::<TimeValueWithRequiredFrame> {
                    time: TimeValueWithRequiredFrame {
                        hour: 23,
                        minute: 59,
                        second: 59,
                        drop_frame: false,
                        frame: 24,
                    },
                    color_frame: ColorFrame::Synchronized,
                    polarity_correction: PolarityCorrection::Odd,
                    binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
                },
                blank_flag: BlankFlag::Continuous,
            },
            *PAL
        ))),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: INVALID BINARY-CODED DECIMAL AND TOO-HIGH VALUES =====
    //
    // high frame tens
    "high_frame_tens",
    PackBinaryTestCase {
        input: "13 30 59 59 23",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.frame: frame number 30 is greater than 29, which is the maximum \
               valid frame number for system 525-60\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high frame units
    "high_frame_units",
    PackBinaryTestCase {
        input: "13 1A 59 59 23",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the time's frame number\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high frame PAL
    "high_frame_pal",
    PackBinaryTestCase {
        input: "13 25 59 59 23",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.frame: frame number 25 is greater than 24, which is the maximum \
               valid frame number for system 625-50\n"
        ),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // high second tens
    "high_second_tens",
    PackBinaryTestCase {
        input: "13 29 60 59 23",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.second: greater than 59\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high second units
    "high_second_units",
    PackBinaryTestCase {
        input: "13 29 4A 59 23",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the time's second\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high minute tens
    "high_minute_tens",
    PackBinaryTestCase {
        input: "13 29 59 60 23",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.minute: greater than 59\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high minute units
    "high_minute_units",
    PackBinaryTestCase {
        input: "13 29 59 4A 23",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the time's minute\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high hour tens
    "high_hour_tens",
    PackBinaryTestCase {
        input: "13 29 59 59 30",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.hour: greater than 23\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high hour units
    "high_hour_units",
    PackBinaryTestCase {
        input: "13 29 59 59 1A",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the time's hour\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: DROP FRAME VALIDATION =====
    //
    // dropped frame number 0 provided
    "dropped_frame_number_0_provided",
    PackBinaryTestCase {
        input: "13 40 00 01 00", // 00:01:00;00
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.frame: the drop frame flag was set, but a dropped frame number 0 \
            was provided\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // dropped frame number 1 provided
    "dropped_frame_number_1_provided",
    PackBinaryTestCase {
        input: "13 41 00 01 00", // 00:01:00;01
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> timecode.time.frame: the drop frame flag was set, but a dropped frame number 1 \
            was provided\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: OBVIOUS TAPE DROPOUTS =====
    // Dropouts should look like 0xFF bytes.
    //
    // no dropout
    "no_dropout",
    PackBinaryTestCase {
        input: "13 00 00 00 00",
        parsed: Some(Pack::TitleTimecode(validated(ZERO_TIMECODE, *NTSC))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // obvious dropout: 1 byte
    "obvious_dropout_1",
    PackBinaryTestCase {
        input: "13 00 00 00 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: hour/minute/second time fields must be \
            fully present or fully absent"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // obvious dropout: 2 byte
    "obvious_dropout_2",
    PackBinaryTestCase {
        input: "13 00 00 FF FF",
        err: Some(
            "Pack failed deserialization of raw bytes: hour/minute/second time fields must be \
            fully present or fully absent"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // obvious dropout: 3 byte
    "obvious_dropout_3",
    PackBinaryTestCase {
        input: "13 00 FF FF FF",
        err: Some(
            "Pack failed deserialization of raw bytes: frame number cannot be given if \
            the rest of the time is missing"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // obvious dropout: 4 byte
    "obvious_dropout_4",
    PackBinaryTestCase {
        input: "13 FF FF FF FF",
        err: Some("Pack failed deserialization of raw bytes: required timecode value is missing"),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // frame number is missing
    "missing_frame_number",
    PackBinaryTestCase {
        input: "13 FF 00 00 00",
        err: Some("Pack failed deserialization of raw bytes: required frame number is missing"),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== DETAILED SUCCESS CASES: INDIVIDUAL BIT TESTING =====
    // Try setting the various high bits one by one to make sure that each field is processed
    // correctly.  This especially matters since there are different NTSC and PAL structures.
    //
    // CF NTSC
    "cf_ntsc",
    PackBinaryTestCase {
        input: "13 80 00 00 00",
        parsed: zero_tc(
            |t| {
                t.timecode.color_frame = ColorFrame::Synchronized;
                t.blank_flag = BlankFlag::Continuous;
            },
            *NTSC
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // CF PAL
    "cf_pal",
    PackBinaryTestCase {
        input: "13 80 00 00 00",
        parsed: zero_tc(
            |t| {
                t.timecode.color_frame = ColorFrame::Synchronized;
                t.blank_flag = BlankFlag::Continuous;
            },
            *PAL
        ),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // DF NTSC
    "df_ntsc",
    PackBinaryTestCase {
        input: "13 40 00 00 00",
        parsed: zero_tc(|t| t.timecode.time.drop_frame = true, *NTSC),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // DF PAL
    "df_pal",
    PackBinaryTestCase {
        input: "13 40 00 00 00",
        parsed: zero_tc(|t| t.timecode.time.drop_frame = true, *PAL),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // PC NTSC
    "pc_ntsc",
    PackBinaryTestCase {
        input: "13 00 80 00 00",
        parsed: zero_tc(|t| t.timecode.polarity_correction = PolarityCorrection::Odd, *NTSC),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // PC PAL
    "pc_pal",
    PackBinaryTestCase {
        input: "13 00 00 00 80",
        parsed: zero_tc(|t| t.timecode.polarity_correction = PolarityCorrection::Odd, *PAL),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // BGF0 NTSC
    "bgf0_ntsc",
    PackBinaryTestCase {
        input: "13 00 00 80 00",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeUnspecifiedGroup8BitCodes,
            *NTSC
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // BGF0 PAL
    "bgf0_pal",
    PackBinaryTestCase {
        input: "13 00 80 00 00",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeUnspecifiedGroup8BitCodes,
            *PAL
        ),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // BGF1 NTSC
    "bgf1_ntsc",
    PackBinaryTestCase {
        input: "13 00 00 00 40",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeClockGroupUnspecified,
            *NTSC
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // BGF1 PAL
    "bgf1_pal",
    PackBinaryTestCase {
        input: "13 00 00 00 40",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeClockGroupUnspecified,
            *PAL
        ),
        ctx: *PAL,
        ..Default::default()
    },
    //
    // BGF2 NTSC
    "bgf2_ntsc",
    PackBinaryTestCase {
        input: "13 00 00 00 80",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeUnspecifiedGroupDateTimeZone,
            *NTSC
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // BGF2 PAL
    "bgf2_pal",
    PackBinaryTestCase {
        input: "13 00 00 80 00",
        parsed: zero_tc(
            |t| t.timecode.binary_group_flag = BinaryGroupFlag::TimeUnspecifiedGroupDateTimeZone,
            *PAL
        ),
        ctx: *PAL,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::success_ntsc_basic(function_name!())]
#[case::success_ntsc_do_not_drop(function_name!())]
#[case::success_pal_do_not_drop(function_name!())]
#[case::max_bounds_ntsc(function_name!())]
#[case::max_bounds_pal(function_name!())]
#[case::high_frame_tens(function_name!())]
#[case::high_frame_units(function_name!())]
#[case::high_frame_pal(function_name!())]
#[case::high_second_tens(function_name!())]
#[case::high_second_units(function_name!())]
#[case::high_minute_tens(function_name!())]
#[case::high_minute_units(function_name!())]
#[case::high_hour_tens(function_name!())]
#[case::high_hour_units(function_name!())]
#[case::dropped_frame_number_0_provided(function_name!())]
#[case::dropped_frame_number_1_provided(function_name!())]
#[case::no_dropout(function_name!())]
#[case::obvious_dropout_1(function_name!())]
#[case::obvious_dropout_2(function_name!())]
#[case::obvious_dropout_3(function_name!())]
#[case::obvious_dropout_4(function_name!())]
#[case::missing_frame_number(function_name!())]
#[case::cf_ntsc(function_name!())]
#[case::cf_pal(function_name!())]
#[case::df_ntsc(function_name!())]
#[case::df_pal(function_name!())]
#[case::pc_ntsc(function_name!())]
#[case::pc_pal(function_name!())]
#[case::bgf0_ntsc(function_name!())]
#[case::bgf0_pal(function_name!())]
#[case::bgf1_ntsc(function_name!())]
#[case::bgf1_pal(function_name!())]
#[case::bgf2_ntsc(function_name!())]
#[case::bgf2_pal(function_name!())]
fn test_title_timecode_binary(#[case] test_function_name: &str) {
    let tc = TITLE_TIMECODE_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

//
// Do some testing on the recording time packs, especially around optional values.
// Otherwise, we'll rely on the existing test cases of the TitleTimecode pack, above.
//

static AAUX_RECORDING_TIME_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    //
    // success NTSC, no TITLE BINARY pack
    "success_ntsc_basic",
    PackBinaryTestCase {
        input: "53 D5 B4 D7 D3",
        parsed: Some(Pack::AAUXRecordingTime(validated(
            RecordingTime {
                time: Some(TimeValueWithOptionalFrame {
                    hour: 13,
                    minute: 57,
                    second: 34,
                    drop_frame: true,
                    frame: Some(15),
                }),
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success NTSC, no frames
    "success_ntsc_no_frames",
    PackBinaryTestCase {
        input: "53 FF B4 D7 D3",
        parsed: Some(Pack::AAUXRecordingTime(validated(
            RecordingTime {
                time: Some(TimeValueWithOptionalFrame {
                    hour: 13,
                    minute: 57,
                    second: 34,
                    drop_frame: true,
                    frame: None,
                }),
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success, no time at all
    "success_no_time",
    PackBinaryTestCase {
        input: "53 FF FF FF FF",
        parsed: Some(Pack::AAUXRecordingTime(validated(
            RecordingTime {
                time: None,
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::success_ntsc_basic(function_name!())]
#[case::success_ntsc_no_frames(function_name!())]
#[case::success_no_time(function_name!())]
fn test_aaux_recording_time_binary(#[case] test_function_name: &str) {
    let tc = AAUX_RECORDING_TIME_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

static VAUX_RECORDING_TIME_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    //
    // success NTSC, no TITLE BINARY pack
    "success_ntsc_basic",
    PackBinaryTestCase {
        input: "63 D5 B4 D7 D3",
        parsed: Some(Pack::VAUXRecordingTime(validated(
            RecordingTime {
                time: Some(TimeValueWithOptionalFrame {
                    hour: 13,
                    minute: 57,
                    second: 34,
                    drop_frame: true,
                    frame: Some(15),
                }),
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success NTSC, no frames
    "success_ntsc_no_frames",
    PackBinaryTestCase {
        input: "63 FF B4 D7 D3",
        parsed: Some(Pack::VAUXRecordingTime(validated(
            RecordingTime {
                time: Some(TimeValueWithOptionalFrame {
                    hour: 13,
                    minute: 57,
                    second: 34,
                    drop_frame: true,
                    frame: None,
                }),
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // success, no time at all
    "success_no_time",
    PackBinaryTestCase {
        input: "63 FF FF FF FF",
        parsed: Some(Pack::VAUXRecordingTime(validated(
            RecordingTime {
                time: None,
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::success_ntsc_basic(function_name!())]
#[case::success_ntsc_no_frames(function_name!())]
#[case::success_no_time(function_name!())]
fn test_vaux_recording_time_binary(#[case] test_function_name: &str) {
    let tc = VAUX_RECORDING_TIME_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static TITLE_TIMECODE_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<TitleTimecode, PackContext>,
> = test_case_map!(
    //
    // blank flag mismatch
    "blank_flag_mismatch",
    ValidateFailureTestCase {
        value: {
            let mut v = ZERO_TIMECODE;
            v.blank_flag = BlankFlag::Continuous;
            v
        },
        err: "blank_flag: Blank flag integer value of 1 must be equal to the color frame flag \
        integer value of 0 because they occupy the same physical bit positions on the tape.  \
        Change one value to match the other.\n",
        ctx: *NTSC
    },
    //
    // color frame mismatch
    "color_frame_mismatch",
    ValidateFailureTestCase {
        value: {
            let mut v = ZERO_TIMECODE;
            v.timecode.color_frame = ColorFrame::Synchronized;
            v
        },
        err: "blank_flag: Blank flag integer value of 0 must be equal to the color frame flag \
        integer value of 1 because they occupy the same physical bit positions on the tape.  \
        Change one value to match the other.\n",
        ctx: *NTSC
    }
);

#[googletest::test]
#[rstest]
#[case::blank_flag_mismatch(function_name!())]
#[case::color_frame_mismatch(function_name!())]
fn test_title_timecode_validation(#[case] test_function_name: &str) {
    let tc = TITLE_TIMECODE_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}

// ==================== SERIALIZATION TESTING ====================
// Tests the serde Serialize / Deserialize implementations to ensure we get the desired tokens.

static TITLE_TIMECODE_SERDE_TEST_CASES: LazyTestCases<SerDeTestCase<TitleTimecode>> = test_case_map!(
    //
    // test successful (de)serialization with drop frame timecode format
    "drop_frame",
    SerDeTestCase {
        value: TitleTimecode {
            timecode: Timecode::<TimeValueWithRequiredFrame> {
                time: TimeValueWithRequiredFrame {
                    hour: 3,
                    minute: 4,
                    second: 5,
                    drop_frame: true,
                    frame: 6
                },
                color_frame: ColorFrame::Synchronized,
                polarity_correction: PolarityCorrection::Even,
                binary_group_flag: BinaryGroupFlag::TimeUnspecifiedGroupDateTimeZone,
            },
            blank_flag: BlankFlag::Continuous,
        },
        tokens: &[
            Token::Map { len: None },
            Token::Str("time"),
            Token::Str("03:04:05;06"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Synchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Even" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant {
                name: "BinaryGroupFlag",
                variant: "TimeUnspecifiedGroupDateTimeZone"
            },
            Token::Str("blank_flag"),
            Token::UnitVariant { name: "BlankFlag", variant: "Continuous" },
            Token::MapEnd
        ],
    },
    //
    // test successful (de)serialization without drop frame timecode format
    "no_drop_frame",
    SerDeTestCase {
        value: TitleTimecode {
            timecode: Timecode::<TimeValueWithRequiredFrame> {
                time: TimeValueWithRequiredFrame {
                    hour: 23,
                    minute: 45,
                    second: 57,
                    drop_frame: false,
                    frame: 22,
                },
                color_frame: ColorFrame::Unsynchronized,
                polarity_correction: PolarityCorrection::Odd,
                binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
            },
            blank_flag: BlankFlag::Discontinuous,
        },
        tokens: &[
            Token::Map { len: None },
            Token::Str("time"),
            Token::Str("23:45:57:22"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Unsynchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Odd" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant { name: "BinaryGroupFlag", variant: "TimeClockGroupPageLine" },
            Token::Str("blank_flag"),
            Token::UnitVariant { name: "BlankFlag", variant: "Discontinuous" },
            Token::MapEnd
        ],
    }
);

#[googletest::test]
#[rstest]
#[case::drop_frame(function_name!())]
#[case::no_drop_frame(function_name!())]
fn test_title_timecode_serde(#[case] test_function_name: &str) {
    let tc = TITLE_TIMECODE_SERDE_TEST_CASES.get_test_case(test_function_name);
    serde_test::assert_tokens(&tc.value, tc.tokens);
}

static TITLE_TIMECODE_DESERIALIZE_ERROR_TEST_CASES: LazyTestCases<DeserializeErrorTestCase> = test_case_map!(
    //
    // time string does not match the regular expression
    "invalid_format",
    DeserializeErrorTestCase {
        tokens: &[
            Token::Map { len: None },
            Token::Str("time"),
            Token::Str("03.04:05;06"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Synchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Even" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant {
                name: "BinaryGroupFlag",
                variant: "TimeUnspecifiedGroupDateTimeZone"
            },
            Token::Str("blank_flag"),
            Token::UnitVariant { name: "BlankFlag", variant: "Continuous" },
            Token::MapEnd
        ],
        err: "invalid value: string \"03.04:05;06\", expected a time with optional frame number",
    },
    //
    // time string is valid, but has no frame number, and TitleTimecode must have a frame number
    "no_frame_number",
    DeserializeErrorTestCase {
        tokens: &[
            Token::Map { len: None },
            Token::Str("time"),
            Token::Str("03:04:05"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Synchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Even" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant {
                name: "BinaryGroupFlag",
                variant: "TimeUnspecifiedGroupDateTimeZone"
            },
            Token::Str("blank_flag"),
            Token::UnitVariant { name: "BlankFlag", variant: "Continuous" },
            Token::MapEnd
        ],
        err: "input is missing a frame number",
    }
);

#[googletest::test]
#[rstest]
#[case::invalid_format(function_name!())]
#[case::no_frame_number(function_name!())]
fn test_title_timecode_deserialize_error(#[case] test_function_name: &str) {
    let tc = TITLE_TIMECODE_DESERIALIZE_ERROR_TEST_CASES.get_test_case(test_function_name);
    serde_test::assert_de_tokens_error::<TitleTimecode>(tc.tokens, tc.err);
}

static RECORDING_TIME_SERDE_TEST_CASES: LazyTestCases<SerDeTestCase<RecordingTime>> = test_case_map!(
    //
    // test successful (de)serialization with drop frame timecode format
    "drop_frame",
    SerDeTestCase {
        value: RecordingTime {
            time: Some(TimeValueWithOptionalFrame {
                hour: 3,
                minute: 4,
                second: 5,
                drop_frame: true,
                frame: Some(6)
            }),
            color_frame: ColorFrame::Synchronized,
            polarity_correction: PolarityCorrection::Even,
            binary_group_flag: BinaryGroupFlag::TimeUnspecifiedGroupDateTimeZone,
        },
        tokens: &[
            Token::Struct { name: "Timecode", len: 4 },
            Token::Str("time"),
            Token::Some,
            Token::Str("03:04:05;06"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Synchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Even" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant {
                name: "BinaryGroupFlag",
                variant: "TimeUnspecifiedGroupDateTimeZone"
            },
            Token::StructEnd
        ],
    },
    //
    // test successful (de)serialization without drop frame timecode format
    "no_drop_frame",
    SerDeTestCase {
        value: RecordingTime {
            time: Some(TimeValueWithOptionalFrame {
                hour: 23,
                minute: 45,
                second: 57,
                drop_frame: false,
                frame: Some(22),
            }),
            color_frame: ColorFrame::Unsynchronized,
            polarity_correction: PolarityCorrection::Odd,
            binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
        },
        tokens: &[
            Token::Struct { name: "Timecode", len: 4 },
            Token::Str("time"),
            Token::Some,
            Token::Str("23:45:57:22"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Unsynchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Odd" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant { name: "BinaryGroupFlag", variant: "TimeClockGroupPageLine" },
            Token::StructEnd
        ],
    },
    //
    // test successful (de)serialization without any frame number
    "no_frame_number",
    SerDeTestCase {
        value: RecordingTime {
            time: Some(TimeValueWithOptionalFrame {
                hour: 23,
                minute: 45,
                second: 57,
                drop_frame: true,
                frame: None,
            }),
            color_frame: ColorFrame::Unsynchronized,
            polarity_correction: PolarityCorrection::Odd,
            binary_group_flag: BinaryGroupFlag::TimeClockGroupPageLine,
        },
        tokens: &[
            Token::Struct { name: "Timecode", len: 4 },
            Token::Str("time"),
            Token::Some,
            Token::Str("23:45:57"),
            Token::Str("color_frame"),
            Token::UnitVariant { name: "ColorFrame", variant: "Unsynchronized" },
            Token::Str("polarity_correction"),
            Token::UnitVariant { name: "PolarityCorrection", variant: "Odd" },
            Token::Str("binary_group_flag"),
            Token::UnitVariant { name: "BinaryGroupFlag", variant: "TimeClockGroupPageLine" },
            Token::StructEnd
        ],
    }
);

#[googletest::test]
#[rstest]
#[case::drop_frame(function_name!())]
#[case::no_drop_frame(function_name!())]
#[case::no_frame_number(function_name!())]
fn test_recording_time_serde(#[case] test_function_name: &str) {
    let tc = RECORDING_TIME_SERDE_TEST_CASES.get_test_case(test_function_name);
    serde_test::assert_tokens(&tc.value, tc.tokens);
}
