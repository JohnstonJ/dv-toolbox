use core::str;
use std::fs::File;

use googletest::prelude::*;
use num::rational::Ratio;
use rstest::rstest;
use stdext::function_name;

use super::*;
use crate::testutil::*;

test_all_test_cases_ran!(
    ("test_info_read", &INFO_READ_TEST_CASES),
    ("test_info_validation", &INFO_VALIDATION_TEST_CASES),
    ("test_info_check_similar", &INFO_CHECK_SIMILAR_TEST_CASES)
);

#[derive(Debug)]
struct DerivedFields {
    video_frame_count: u64,
    video_frame_size: u32,
    video_frame_channel_count: u8,
    video_frame_dif_sequence_count: u8,
    system: System,
    ideal_audio_samples_per_frame: Option<Ratio<u32>>,
}

impl DerivedFields {
    fn assert_info(&self, actual_info: &ValidInfo) {
        expect_that!(actual_info.video_frame_count(), eq(self.video_frame_count));
        expect_that!(actual_info.video_frame_size(), eq(self.video_frame_size));
        expect_that!(actual_info.video_frame_channel_count(), eq(self.video_frame_channel_count));
        expect_that!(
            actual_info.video_frame_dif_sequence_count(),
            eq(self.video_frame_dif_sequence_count)
        );
        expect_that!(actual_info.system(), eq(self.system));
        expect_that!(
            actual_info.ideal_audio_samples_per_frame(),
            eq(self.ideal_audio_samples_per_frame)
        );
    }
}

#[derive(Debug)]
struct InfoReadTestCase<'a> {
    filename: &'a str,
    info: Info,
    derived: DerivedFields,
}

static INFO_READ_TEST_CASES: LazyTestCases<InfoReadTestCase> = test_case_map!(
    "sony_good_quality",
    InfoReadTestCase {
        filename: "dv_multiframe/sony_good_quality.dv",
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000), // 5 frames in input file
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        derived: DerivedFields {
            video_frame_count: 5,
            video_frame_size: 120_000,
            video_frame_channel_count: 1,
            video_frame_dif_sequence_count: 10,
            system: System::Sys525_60,
            ideal_audio_samples_per_frame: Some(Ratio::<u32>::new(16_016, 15)),
        },
    }
);

#[googletest::test]
#[rstest]
#[case::sony_good_quality(function_name!())]
fn test_info_read(#[case] test_function_name: &str) {
    let tc = INFO_READ_TEST_CASES.get_test_case(test_function_name);

    let file = File::open(test_resource(tc.filename)).unwrap();
    let info = ValidInfo::read(Rc::new(RefCell::new(file))).unwrap();

    expect_that!(*info, eq(tc.info));
    tc.derived.assert_info(&info);
}

#[derive(Debug)]
struct InfoValidationTestCase<'a> {
    info: Info,
    err: Option<&'a str>,
    derived: Option<DerivedFields>,
}

static INFO_VALIDATION_TEST_CASES: LazyTestCases<InfoValidationTestCase> = test_case_map!(
    "ntsc_48k",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000), // 5 frames in input file
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: None,
        derived: Some(DerivedFields {
            video_frame_count: 5,
            video_frame_size: 120_000,
            video_frame_channel_count: 1,
            video_frame_dif_sequence_count: 10,
            system: System::Sys525_60,
            ideal_audio_samples_per_frame: Some(Ratio::<u32>::new(16_016, 15)),
        }),
    },
    "pal_44_1k",
    InfoValidationTestCase {
        info: Info {
            file_size: 1_008_000,
            video_frame_rate: Ratio::<u32>::from(25),
            video_duration: Ratio::<u128>::new(7, 25),
            audio_stereo_stream_count: 1,
            audio_sample_rate: Some(44_100),
        },
        err: None,
        derived: Some(DerivedFields {
            video_frame_count: 7,
            video_frame_size: 144_000,
            video_frame_channel_count: 1,
            video_frame_dif_sequence_count: 12,
            system: System::Sys625_50,
            ideal_audio_samples_per_frame: Some(Ratio::<u32>::from(1_764)),
        }),
    },
    "ntsc_no_audio_2_channel",
    InfoValidationTestCase {
        info: Info {
            file_size: 960_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 4, 30_000),
            audio_stereo_stream_count: 0,
            audio_sample_rate: None,
        },
        err: None,
        derived: Some(DerivedFields {
            video_frame_count: 4,
            video_frame_size: 240_000,
            video_frame_channel_count: 2,
            video_frame_dif_sequence_count: 10,
            system: System::Sys525_60,
            ideal_audio_samples_per_frame: None,
        }),
    },
    "pal_no_audio_2_channel",
    InfoValidationTestCase {
        info: Info {
            file_size: 1_728_000,
            video_frame_rate: Ratio::<u32>::from(25),
            video_duration: Ratio::<u128>::new(6, 25),
            audio_stereo_stream_count: 0,
            audio_sample_rate: None,
        },
        err: None,
        derived: Some(DerivedFields {
            video_frame_count: 6,
            video_frame_size: 288_000,
            video_frame_channel_count: 2,
            video_frame_dif_sequence_count: 12,
            system: System::Sys625_50,
            ideal_audio_samples_per_frame: None,
        }),
    },
    "non_integer_frame_count",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5 + 1, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some(
            "video_duration: Total video frame count 5006/1001 is not an integer; \
                it resulted from multiplying video frame rate 30000/1001 by video \
                duration 2503/15000\n",
        ),
        derived: None,
    },
    "zero_length",
    InfoValidationTestCase {
        info: Info {
            file_size: 0,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::from(0),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some(
            "video_duration: Video frame count is zero, so cannot calculate the frame size\n",
        ),
        derived: None,
    },
    "weird_file_size",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_001,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some(
            "video_duration: File size 600001 is not evenly divisible by video frame count 5\n",
        ),
        derived: None,
    },
    "unsupported_frame_size",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_005,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some("video_duration: Unsupported frame size 120001\n"),
        derived: None,
    },
    "unsupported_frame_rate",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_001, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_001),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some(
            "video_frame_rate: Video frame rate 30001/1001 is not a \
            supported NTSC/PAL/SECAM rate\n",
        ),
        derived: None,
    },
    "unsupported_sample_rate",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_001),
        },
        err: Some("audio_sample_rate: Unsupported audio sample rate 32001\n"),
        derived: None,
    },
    "missing_sample_rate",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: None,
        },
        err: Some("audio_sample_rate: Could not detect sample rate for audio streams\n"),
        derived: None,
    },
    "unexpected_sample_rate",
    InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 0,
            audio_sample_rate: Some(48_000),
        },
        err: Some(
            "audio_sample_rate: Audio sample rate cannot be provided \
            if there are no audio streams\n",
        ),
        derived: None,
    }
);

#[googletest::test]
#[rstest]
#[case::ntsc_48k(function_name!())]
#[case::pal_44_1k(function_name!())]
#[case::ntsc_no_audio_2_channel(function_name!())]
#[case::pal_no_audio_2_channel(function_name!())]
#[case::non_integer_frame_count(function_name!())]
#[case::zero_length(function_name!())]
#[case::weird_file_size(function_name!())]
#[case::unsupported_frame_size(function_name!())]
#[case::unsupported_frame_rate(function_name!())]
#[case::unsupported_sample_rate(function_name!())]
#[case::missing_sample_rate(function_name!())]
#[case::unexpected_sample_rate(function_name!())]
fn test_info_validation(#[case] test_function_name: &str) {
    let tc = INFO_VALIDATION_TEST_CASES.get_test_case(test_function_name);

    let res = UnvalidatedInfo::from(tc.info).validate();
    if let Some(ref derived) = tc.derived {
        derived.assert_info(&res.unwrap());
    } else {
        expect_that!(res, err(displays_as(eq(tc.err.unwrap()))));
    }
}

#[derive(Debug)]
struct InfoCheckSimilarTestCase<'a> {
    expected: Info,
    comparison: Info,
    err: Option<&'a str>,
}

static INFO_CHECK_SIMILAR_TEST_CASES: LazyTestCases<InfoCheckSimilarTestCase> = test_case_map!(
    "matches",
    InfoCheckSimilarTestCase {
        expected: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        comparison: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: None,
    },
    "frame_rate_mismatch",
    InfoCheckSimilarTestCase {
        expected: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        comparison: Info {
            file_size: 720_000,
            video_frame_rate: Ratio::<u32>::from(25),
            video_duration: Ratio::<u128>::new(5, 25),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some("Video frame rate 25 does not match 30000/1001"),
    },
    "frame_size_mismatch",
    InfoCheckSimilarTestCase {
        expected: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        comparison: Info {
            file_size: 1_200_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some("Video frame size 240000 does not match 120000"),
    },
    "audio_stream_count_mismatch",
    InfoCheckSimilarTestCase {
        expected: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        comparison: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 1,
            audio_sample_rate: Some(32_000),
        },
        err: Some("Audio stereo stream count 1 does not match 2"),
    },
    "audio_sample_rate_mismatch",
    InfoCheckSimilarTestCase {
        expected: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        comparison: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(48_000),
        },
        err: Some("Audio sample rate 48000 does not match 32000"),
    }
);

#[googletest::test]
#[rstest]
#[case::matches(function_name!())]
#[case::frame_rate_mismatch(function_name!())]
#[case::frame_size_mismatch(function_name!())]
#[case::audio_stream_count_mismatch(function_name!())]
#[case::audio_sample_rate_mismatch(function_name!())]
fn test_info_check_similar(#[case] test_function_name: &str) {
    let tc = INFO_CHECK_SIMILAR_TEST_CASES.get_test_case(test_function_name);

    let expected = UnvalidatedInfo::from(tc.expected).validate().unwrap();
    let comparison = UnvalidatedInfo::from(tc.comparison).validate().unwrap();
    match tc.err {
        Some(e) => expect_that!(expected.check_similar(&comparison), err(displays_as(eq(e)))),
        None => expect_that!(expected.check_similar(&comparison), ok(eq(&()))),
    };
}
