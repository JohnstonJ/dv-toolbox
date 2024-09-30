use std::ffi::CStr;

use googletest::prelude::*;
use rsmpeg::{avcodec::AVCodec, avformat::AVOutputFormat};
use rstest::rstest;

use super::*;

#[googletest::test]
#[rstest]
#[case::supported(c"matroska", c"pcm_u8", Compliance::Normal, Ok(true))]
// can't store raw PCM audio in raw DV video container
#[case::unsupported(c"dv", c"pcm_u8", Compliance::Normal, Ok(false))]
fn test_format_supports_codec(
    #[case] format: &CStr,
    #[case] codec: &CStr,
    #[case] compliance: Compliance,
    #[case] expected_result: rsmpeg::error::Result<bool>,
) {
    let format = AVOutputFormat::guess_format(Some(format), None, None).unwrap();
    let codec = AVCodec::find_encoder_by_name(codec).unwrap();

    let result = format_supports_codec(&format, &codec, compliance);
    expect_that!(&result, eq(&expected_result));
}
