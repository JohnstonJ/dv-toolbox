use std::{fs::File, sync::LazyLock};

use googletest::prelude::*;
use num::rational::Ratio;
use rstest::rstest;

use super::*;
use crate::testutil;

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

static INFO_READ_SONY_GOOD_QUALITY: LazyLock<InfoReadTestCase> =
    LazyLock::new(|| InfoReadTestCase {
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
    });

#[googletest::test]
#[rstest]
#[case::sony_good_quality(&*INFO_READ_SONY_GOOD_QUALITY)]
fn test_info_read(#[case] tc: &InfoReadTestCase) {
    let file = File::open(testutil::test_resource(tc.filename)).unwrap();
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

static INFO_VALIDATION_NTSC_48K: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_PAL_44_1K: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_NTSC_NO_AUDIO_2_CHANNEL: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_PAL_NO_AUDIO_2_CHANNEL: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_NON_INTEGER_FRAME_COUNT: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_ZERO_LENGTH: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_WEIRD_FILE_SIZE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_UNSUPPORTED_FRAME_SIZE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
        info: Info {
            file_size: 600_005,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_000),
        },
        err: Some("video_duration: Unsupported frame size 120001\n"),
        derived: None,
    });

static INFO_VALIDATION_UNSUPPORTED_FRAME_RATE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

static INFO_VALIDATION_UNSUPPORTED_SAMPLE_RATE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: Some(32_001),
        },
        err: Some("audio_sample_rate: Unsupported audio sample rate 32001\n"),
        derived: None,
    });

static INFO_VALIDATION_MISSING_SAMPLE_RATE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
        info: Info {
            file_size: 600_000,
            video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
            video_duration: Ratio::<u128>::new(1_001 * 5, 30_000),
            audio_stereo_stream_count: 2,
            audio_sample_rate: None,
        },
        err: Some("audio_sample_rate: Could not detect sample rate for audio streams\n"),
        derived: None,
    });

static INFO_VALIDATION_UNEXPECTED_SAMPLE_RATE: LazyLock<InfoValidationTestCase> =
    LazyLock::new(|| InfoValidationTestCase {
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
    });

#[googletest::test]
#[rstest]
#[case::ntsc_48k(&*INFO_VALIDATION_NTSC_48K)]
#[case::pal_44_1k(&*INFO_VALIDATION_PAL_44_1K)]
#[case::ntsc_no_audio_2_channel(&*INFO_VALIDATION_NTSC_NO_AUDIO_2_CHANNEL)]
#[case::pal_no_audio_2_channel(&*INFO_VALIDATION_PAL_NO_AUDIO_2_CHANNEL)]
#[case::non_integer_frame_count(&*INFO_VALIDATION_NON_INTEGER_FRAME_COUNT)]
#[case::zero_length(&*INFO_VALIDATION_ZERO_LENGTH)]
#[case::weird_file_size(&*INFO_VALIDATION_WEIRD_FILE_SIZE)]
#[case::unsupported_frame_size(&*INFO_VALIDATION_UNSUPPORTED_FRAME_SIZE)]
#[case::unsupported_frame_rate(&*INFO_VALIDATION_UNSUPPORTED_FRAME_RATE)]
#[case::unsupported_sample_rate(&*INFO_VALIDATION_UNSUPPORTED_SAMPLE_RATE)]
#[case::missing_sample_rate(&*INFO_VALIDATION_MISSING_SAMPLE_RATE)]
#[case::unexpected_sample_rate(&*INFO_VALIDATION_UNEXPECTED_SAMPLE_RATE)]
fn test_info_validation(#[case] tc: &InfoValidationTestCase) {
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

static INFO_CHECK_SIMILAR_MATCHES: LazyLock<InfoCheckSimilarTestCase> =
    LazyLock::new(|| InfoCheckSimilarTestCase {
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
    });

static INFO_CHECK_SIMILAR_FRAME_RATE_MISMATCH: LazyLock<InfoCheckSimilarTestCase> =
    LazyLock::new(|| InfoCheckSimilarTestCase {
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
    });

static INFO_CHECK_SIMILAR_FRAME_SIZE_MISMATCH: LazyLock<InfoCheckSimilarTestCase> =
    LazyLock::new(|| InfoCheckSimilarTestCase {
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
    });

static INFO_CHECK_SIMILAR_AUDIO_STREAM_COUNT_MISMATCH: LazyLock<InfoCheckSimilarTestCase> =
    LazyLock::new(|| InfoCheckSimilarTestCase {
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
    });

static INFO_CHECK_SIMILAR_AUDIO_SAMPLE_RATE_MISMATCH: LazyLock<InfoCheckSimilarTestCase> =
    LazyLock::new(|| InfoCheckSimilarTestCase {
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
    });

#[googletest::test]
#[rstest]
#[case::matches(&*INFO_CHECK_SIMILAR_MATCHES)]
#[case::frame_rate_mismatch(&*INFO_CHECK_SIMILAR_FRAME_RATE_MISMATCH)]
#[case::frame_size_mismatch(&*INFO_CHECK_SIMILAR_FRAME_SIZE_MISMATCH)]
#[case::audio_stream_count_mismatch(&*INFO_CHECK_SIMILAR_AUDIO_STREAM_COUNT_MISMATCH)]
#[case::audio_sample_rate_mismatch(&*INFO_CHECK_SIMILAR_AUDIO_SAMPLE_RATE_MISMATCH)]
fn test_info_check_similar(#[case] tc: &InfoCheckSimilarTestCase) {
    let expected = UnvalidatedInfo::from(tc.expected).validate().unwrap();
    let comparison = UnvalidatedInfo::from(tc.comparison).validate().unwrap();
    match tc.err {
        Some(e) => expect_that!(expected.check_similar(&comparison), err(displays_as(eq(e)))),
        None => expect_that!(expected.check_similar(&comparison), ok(eq(&()))),
    };
}
