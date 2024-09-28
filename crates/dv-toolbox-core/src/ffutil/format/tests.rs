use ffmpeg_next::{codec, encoder};
use googletest::prelude::*;
use rstest::rstest;

use super::*;
use crate::testutil;

#[googletest::test]
#[rstest]
#[case::valid("dv", true)]
#[case::invalid("garbage-format-not-real", false)]
#[case::with_null("dv\0", false)]
fn test_find_input_format(#[case] short_name: &str, #[case] matches: bool) {
    testutil::init_ffmpeg();

    let result = find_input_format(short_name);
    if matches {
        expect_that!(result.unwrap().name(), eq(short_name));
    } else {
        expect_true!(result.is_none());
    }
}

#[googletest::test]
#[rstest]
#[case::empty_params(None, None, None, None)]
// short name testing
#[case::valid_short_name(Some("dv"), None, None, Some("dv"))]
#[case::invalid_short_name(Some("garbage-format-not-real"), None, None, None)]
#[case::with_null_short_name(Some("dv\0"), None, None, None)]
// filename testing
#[case::valid_filename(None, Some("file.dv"), None, Some("dv"))]
#[case::invalid_filename(None, Some("file.garbageext"), None, None)]
#[case::with_null_filename(None, Some("file.dv\0"), None, None)]
// MIME type testing
#[case::valid_mime_type(None, None, Some("video/x-matroska"), Some("matroska"))]
#[case::invalid_mime_type(None, None, Some("application/x-garbage"), None)]
#[case::with_null_mime_type(None, None, Some("video/x-matroska\0"), None)]
fn test_guess_output_format(
    #[case] short_name: Option<&str>,
    #[case] filename: Option<&str>,
    #[case] mime_type: Option<&str>,
    #[case] matches: Option<&str>, // match with short name
) {
    testutil::init_ffmpeg();

    let result = guess_output_format(short_name, filename, mime_type);
    match matches {
        Some(match_name) => expect_that!(result.unwrap().name(), eq(match_name)),
        None => expect_true!(result.is_none()),
    }
}

#[googletest::test]
#[rstest]
#[case::supported("matroska", "pcm_u8", codec::Compliance::Normal, Ok(true))]
// can't store raw PCM audio in raw DV video container
#[case::unsupported("dv", "pcm_u8", codec::Compliance::Normal, Ok(false))]
fn test_format_supports_codec(
    #[case] format: &str,
    #[case] codec: &str,
    #[case] compliance: codec::Compliance,
    #[case] expected_result: std::result::Result<bool, ffmpeg::Error>,
) {
    testutil::init_ffmpeg();

    let format = guess_output_format(Some(format), None, None).unwrap();
    let codec = encoder::find_by_name(codec).unwrap();

    let result = format_supports_codec(&format, &codec, compliance);
    expect_that!(result, eq(expected_result));
}
