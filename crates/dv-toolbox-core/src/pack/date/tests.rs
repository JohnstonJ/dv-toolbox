use arbitrary_int::{u2, Number};
use chrono::{FixedOffset, NaiveDate, Weekday};
use rstest::rstest;
use serde_test::Token;
use stdext::function_name;
use testutil::*;

use super::super::*;
use crate::{pack::testutil::PackBinaryTestCase, testutil::*};

test_all_test_cases_ran!(
    ("test_vaux_recording_date_binary", &VAUX_RECORDING_DATE_BINARY_TEST_CASES,),
    ("test_aaux_recording_date_binary", &AAUX_RECORDING_DATE_BINARY_TEST_CASES,),
    ("test_vaux_recording_date_validation", &VAUX_RECORDING_DATE_VALIDATION_TEST_CASES,),
    ("test_vaux_recording_date_serde", &VAUX_RECORDING_DATE_SERDE_TEST_CASES,),
    (
        "test_vaux_recording_date_deserialize_error",
        &VAUX_RECORDING_DATE_DESERIALIZE_ERROR_TEST_CASES
    )
);

// ==================== BINARY SERIALIZATION TESTING ====================
// Tests to/from actual/raw DV pack data.

//
// Do most heavy testing on the VAUXRecordingDate
//

static VAUX_RECORDING_DATE_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    //
    // ===== SOME BASIC SUCCESS CASES =====
    //
    // basic success case
    "basic_success",
    PackBinaryTestCase {
        input: "62 D9 E7 68 97",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(1997, 8, 27).unwrap()),
                weekday: Some(Weekday::Wed),
                timezone: Some(FixedOffset::east_opt(19 * 60 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // another basic success case
    "more_basic_success",
    PackBinaryTestCase {
        input: "62 85 97 85 63",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2063, 5, 17).unwrap()),
                weekday: Some(Weekday::Thu),
                timezone: Some(FixedOffset::east_opt(5 * 60 * 60 + 30 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x2),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== BOUNDS TESTING: Y2K =====
    //
    // Y2K rollover: last 20th century year, DST, 09:00 TZ, reserved 0x0
    "y2k_last_20th_century_year",
    PackBinaryTestCase {
        input: "62 49 17 25 99",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(1999, 5, 17).unwrap()),
                weekday: Some(Weekday::Mon),
                timezone: Some(FixedOffset::east_opt(9 * 60 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x0),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Y2K rollover: first 21st century year, DST, 09:00 TZ, reserved 0x1
    "y2k_first_21st_century_year",
    PackBinaryTestCase {
        input: "62 21 57 65 00",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2000, 5, 17).unwrap()),
                weekday: Some(Weekday::Wed),
                timezone: Some(FixedOffset::east_opt(21 * 60 * 60 + 30 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x1),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Y2K rollover: first 20th century year
    "y2k_first_20th_century_year",
    PackBinaryTestCase {
        input: "62 49 17 C5 75",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(1975, 5, 17).unwrap()),
                weekday: Some(Weekday::Sat),
                timezone: Some(FixedOffset::east_opt(9 * 60 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x0),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // Y2K rollover: last 21st century year
    "y2k_last_21st_century_year",
    PackBinaryTestCase {
        input: "62 49 17 85 74",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2074, 5, 17).unwrap()),
                weekday: Some(Weekday::Thu),
                timezone: Some(FixedOffset::east_opt(9 * 60 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x0),
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
        input: "62 23 31 32 74",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2074, 12, 31).unwrap()),
                weekday: Some(Weekday::Mon),
                timezone: Some(FixedOffset::east_opt(23 * 60 * 60 + 30 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x0),
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
        input: "62 C0 C1 61 75",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(1975, 1, 1).unwrap()),
                weekday: Some(Weekday::Wed),
                timezone: Some(FixedOffset::east_opt(0).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== BOUNDS TESTING: WEEKDAY =====
    //
    // minimum weekday
    "min_weekday",
    PackBinaryTestCase {
        input: "62 C0 D4 05 00",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2000, 5, 14).unwrap()),
                weekday: Some(Weekday::Sun),
                timezone: Some(FixedOffset::east_opt(0).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // maximum weekday
    "max_weekday",
    PackBinaryTestCase {
        input: "62 C0 E0 C5 00",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2000, 5, 20).unwrap()),
                weekday: Some(Weekday::Sat),
                timezone: Some(FixedOffset::east_opt(0).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== OPTIONALITY =====
    //
    // no time zone or week
    "no_time_zone_or_week",
    PackBinaryTestCase {
        input: "62 FF E1 E5 01",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(2001, 5, 21).unwrap()),
                weekday: None,
                timezone: None,
                daylight_saving_time: None,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // no date, but has time zone
    "no_date_but_has_time_zone",
    PackBinaryTestCase {
        input: "62 21 FF FF FF",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: None,
                weekday: None,
                timezone: Some(FixedOffset::east_opt(21 * 60 * 60 + 30 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::DaylightSavingTime),
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // empty pack
    "empty_pack",
    PackBinaryTestCase {
        input: "62 FF FF FF FF",
        parsed: Some(Pack::VAUXRecordingDate(validated(
            RecordingDate {
                date: None,
                weekday: None,
                timezone: None,
                daylight_saving_time: None,
                reserved: u2::new(0x3),
            },
            *NTSC
        ))),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: INVALID BINARY-CODED DECIMAL AND TOO-HIGH VALUES =====
    //
    // high time zone tens
    "high_time_zone_tens",
    PackBinaryTestCase {
        input: "62 30 31 52 74",
        err: Some("Pack failed deserialization of raw bytes: timezone hour 30 is out of range"),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high time zone units
    "high_time_zone_units",
    PackBinaryTestCase {
        input: "62 1A 31 52 74",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the date's timezone\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high day value
    "high_day_value",
    PackBinaryTestCase {
        input: "62 23 39 52 74",
        err: Some(
            "Pack failed deserialization of raw bytes: \
            the date 2074-12-39 is not a valid date"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high day units
    "high_day_units",
    PackBinaryTestCase {
        input: "62 23 2A 52 74",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the date's day\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high month value
    "high_month_value",
    PackBinaryTestCase {
        input: "62 23 31 53 74",
        err: Some(
            "Pack failed deserialization of raw bytes: \
            the date 2074-13-31 is not a valid date"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high month units
    "high_month_units",
    PackBinaryTestCase {
        input: "62 23 25 0A 74",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the date's month\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high year tens
    "high_year_tens",
    PackBinaryTestCase {
        input: "62 23 31 12 A0",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the date's year\n\
            Caused by:\n  \
            -> tens place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // high year units
    "high_year_units",
    PackBinaryTestCase {
        input: "62 23 31 12 7A",
        err: Some(
            "Pack failed deserialization of raw bytes: couldn't read the date's year\n\
            Caused by:\n  \
            -> units place value of 10 is greater than 9"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: OBVIOUS TAPE DROPOUTS =====
    // Dropouts should look like 0xFF bytes.
    //
    // obvious dropout: 1 byte
    "obvious_dropout_1",
    PackBinaryTestCase {
        input: "62 D9 E7 48 FF",
        err: Some(
            "Pack failed deserialization of raw bytes: year/month/day date fields must be \
            fully present or fully absent"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // obvious dropout: 2 byte
    "obvious_dropout_2",
    PackBinaryTestCase {
        input: "62 D9 E7 FF FF",
        err: Some(
            "Pack failed deserialization of raw bytes: year/month/day date fields must be \
            fully present or fully absent"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // ===== ERROR CASES: OTHER SCENARIOS =====
    //
    // weekday provided with no date
    "weekday_provided_with_no_date",
    PackBinaryTestCase {
        input: "62 FF FF 1F FF",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> weekday: a weekday must not be provided if the date is otherwise absent.\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    },
    //
    // specified weekday does not match the date's true weekday
    "weekday_mismatch",
    PackBinaryTestCase {
        input: "62 D9 E7 08 97",
        err: Some(
            "Pack failed validation during deserialization of raw bytes\n\
            Caused by:\n  \
            -> weekday: the weekday field value of Sun does not match the date weekday of \
               1997-08-27 which is Wed\n"
        ),
        ctx: *NTSC,
        ..Default::default()
    }
);

#[googletest::test]
#[rstest]
#[case::basic_success(function_name!())]
#[case::more_basic_success(function_name!())]
#[case::y2k_last_20th_century_year(function_name!())]
#[case::y2k_first_21st_century_year(function_name!())]
#[case::y2k_first_20th_century_year(function_name!())]
#[case::y2k_last_21st_century_year(function_name!())]
#[case::max_bounds(function_name!())]
#[case::min_bounds(function_name!())]
#[case::min_weekday(function_name!())]
#[case::max_weekday(function_name!())]
#[case::no_time_zone_or_week(function_name!())]
#[case::no_date_but_has_time_zone(function_name!())]
#[case::empty_pack(function_name!())]
#[case::high_time_zone_tens(function_name!())]
#[case::high_time_zone_units(function_name!())]
#[case::high_day_value(function_name!())]
#[case::high_day_units(function_name!())]
#[case::high_month_value(function_name!())]
#[case::high_month_units(function_name!())]
#[case::high_year_tens(function_name!())]
#[case::high_year_units(function_name!())]
#[case::obvious_dropout_1(function_name!())]
#[case::obvious_dropout_2(function_name!())]
#[case::weekday_provided_with_no_date(function_name!())]
#[case::weekday_mismatch(function_name!())]
fn test_vaux_recording_date_binary(#[case] test_function_name: &str) {
    let tc = VAUX_RECORDING_DATE_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

//
// Just a quick test to check that AAUXRecordingDate is also set up right
//

static AAUX_RECORDING_DATE_BINARY_TEST_CASES: LazyTestCases<PackBinaryTestCase> = test_case_map!(
    // basic success case
    "basic_success",
    PackBinaryTestCase {
        input: "52 D9 E7 68 97",
        parsed: Some(Pack::AAUXRecordingDate(validated(
            RecordingDate {
                date: Some(NaiveDate::from_ymd_opt(1997, 8, 27).unwrap()),
                weekday: Some(Weekday::Wed),
                timezone: Some(FixedOffset::east_opt(19 * 60 * 60).unwrap()),
                daylight_saving_time: Some(DaylightSavingTime::Normal),
                reserved: u2::new(0x3),
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
fn test_aaux_recording_date_binary(#[case] test_function_name: &str) {
    let tc = AAUX_RECORDING_DATE_BINARY_TEST_CASES.get_test_case(test_function_name);
    run_pack_binary_test_case(tc);
}

// ==================== VALIDATION TESTING ====================
// Tests on validation code that was not already tested as part of binary serialization.

static VAUX_RECORDING_DATE_VALIDATION_TEST_CASES: LazyTestCases<
    ValidateFailureTestCase<RecordingDate, PackContext>,
> = test_case_map!(
    //
    // year is too low and won't fit in 2-digit space
    "low_year",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: NaiveDate::from_ymd_opt(1974, 1, 1),
            weekday: None,
            timezone: None,
            daylight_saving_time: None,
            reserved: u2::MAX
        },
        err: "date: the year 1974 is before the minimum allowed year 1975\n",
        ctx: *NTSC
    },
    //
    // year is too high and won't fit in 2-digit space
    "high_year",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: NaiveDate::from_ymd_opt(2075, 1, 1),
            weekday: None,
            timezone: None,
            daylight_saving_time: None,
            reserved: u2::MAX
        },
        err: "date: the year 2075 is after the maximum allowed year 2074\n",
        ctx: *NTSC
    },
    //
    // negative time zone offsets are not allowed
    "negative_time_zone_offset",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: None,
            weekday: None,
            timezone: Some(FixedOffset::west_opt(60 * 60).unwrap()),
            daylight_saving_time: Some(DaylightSavingTime::Normal),
            reserved: u2::MAX
        },
        err: "timezone: the time zone offset of -3600 seconds must be a positive offset from GMT\n",
        ctx: *NTSC
    },
    //
    // unsupported time zone interval: it needs to be a multiple of 30 minutes
    "unsupported_time_zone_interval",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: None,
            weekday: None,
            timezone: Some(FixedOffset::east_opt(15 * 60).unwrap()), // GMT +15 minutes
            daylight_saving_time: Some(DaylightSavingTime::Normal),
            reserved: u2::MAX
        },
        err: "timezone: the time zone offset of 900 seconds must be a multiple of \
            30 minutes, or 1800 seconds\n",
        ctx: *NTSC
    },
    //
    // missing daylight saving time
    "missing_daylight_saving_time",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: None,
            weekday: None,
            timezone: Some(FixedOffset::east_opt(0).unwrap()),
            daylight_saving_time: None,
            reserved: u2::MAX
        },
        err: "daylight_saving_time: daylight saving time value must be specified if \
            time zone is present\n",
        ctx: *NTSC
    },
    //
    // unexpected daylight saving time
    "unexpected_daylight_saving_time",
    ValidateFailureTestCase {
        value: RecordingDate {
            date: None,
            weekday: None,
            timezone: None,
            daylight_saving_time: Some(DaylightSavingTime::Normal),
            reserved: u2::MAX
        },
        err: "daylight_saving_time: daylight saving time value must be not specified if \
            time zone is absent\n",
        ctx: *NTSC
    }
);

#[googletest::test]
#[rstest]
#[case::low_year(function_name!())]
#[case::high_year(function_name!())]
#[case::negative_time_zone_offset(function_name!())]
#[case::unsupported_time_zone_interval(function_name!())]
#[case::missing_daylight_saving_time(function_name!())]
#[case::unexpected_daylight_saving_time(function_name!())]
fn test_vaux_recording_date_validation(#[case] test_function_name: &str) {
    let tc = VAUX_RECORDING_DATE_VALIDATION_TEST_CASES.get_test_case(test_function_name);
    run_validate_failure_test_case(tc);
}

// ==================== SERIALIZATION TESTING ====================
// Tests the serde Serialize / Deserialize implementations to ensure we get the desired tokens.

static VAUX_RECORDING_DATE_SERDE_TEST_CASES: LazyTestCases<SerDeTestCase<RecordingDate>> = test_case_map!(
    //
    // test successful (de)serialization with time zone offset
    "with_time_zone",
    SerDeTestCase {
        value: RecordingDate {
            date: NaiveDate::from_ymd_opt(2000, 1, 2),
            weekday: Some(Weekday::Sat),
            timezone: Some(FixedOffset::east_opt(3 * 60 * 60).unwrap()),
            daylight_saving_time: Some(DaylightSavingTime::Normal),
            reserved: u2::new(0x3),
        },
        tokens: &[
            Token::Struct { name: "RecordingDate", len: 5 },
            Token::Str("date"),
            Token::Some,
            Token::Str("2000-01-02"),
            Token::Str("weekday"),
            Token::Some,
            Token::Str("Sat"),
            Token::Str("timezone"),
            Token::Some,
            Token::I32(3 * 60 * 60),
            Token::Str("daylight_saving_time"),
            Token::Some,
            Token::UnitVariant { name: "DaylightSavingTime", variant: "Normal" },
            Token::Str("reserved"),
            Token::U8(0x3),
            Token::StructEnd
        ],
    },
    //
    // test successful (de)serialization without time zone offset
    "without_time_zone",
    SerDeTestCase {
        value: RecordingDate {
            date: NaiveDate::from_ymd_opt(2000, 1, 2),
            weekday: Some(Weekday::Sat),
            timezone: None,
            daylight_saving_time: None,
            reserved: u2::new(0x3),
        },
        tokens: &[
            Token::Struct { name: "RecordingDate", len: 5 },
            Token::Str("date"),
            Token::Some,
            Token::Str("2000-01-02"),
            Token::Str("weekday"),
            Token::Some,
            Token::Str("Sat"),
            Token::Str("timezone"),
            Token::None,
            Token::Str("daylight_saving_time"),
            Token::None,
            Token::Str("reserved"),
            Token::U8(0x3),
            Token::StructEnd
        ],
    }
);

#[googletest::test]
#[rstest]
#[case::with_time_zone(function_name!())]
#[case::without_time_zone(function_name!())]
fn test_vaux_recording_date_serde(#[case] test_function_name: &str) {
    let tc = VAUX_RECORDING_DATE_SERDE_TEST_CASES.get_test_case(test_function_name);
    serde_test::assert_tokens(&tc.value, tc.tokens);
}

static VAUX_RECORDING_DATE_DESERIALIZE_ERROR_TEST_CASES: LazyTestCases<DeserializeErrorTestCase> = test_case_map!(
    //
    // time zone offset is outside the range of FixedOffset
    "time_zone_out_of_range",
    DeserializeErrorTestCase {
        tokens: &[
            Token::Struct { name: "RecordingDate", len: 5 },
            Token::Str("date"),
            Token::Some,
            Token::Str("2000-01-02"),
            Token::Str("weekday"),
            Token::Some,
            Token::Str("Sat"),
            Token::Str("timezone"),
            Token::Some,
            Token::I32(25 * 60 * 60), // too high
        ],
        err: "invalid value: integer `90000`, expected a time zone offset measured in seconds \
            and less than 24 hours",
    }
);

#[googletest::test]
#[rstest]
#[case::time_zone_out_of_range(function_name!())]
fn test_vaux_recording_date_deserialize_error(#[case] test_function_name: &str) {
    let tc = VAUX_RECORDING_DATE_DESERIALIZE_ERROR_TEST_CASES.get_test_case(test_function_name);
    serde_test::assert_de_tokens_error::<RecordingDate>(tc.tokens, tc.err);
}
