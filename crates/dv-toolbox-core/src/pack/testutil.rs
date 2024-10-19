use std::sync::LazyLock;

use display_error_chain::ErrorChainExt;
use googletest::prelude::*;
use num::rational::Ratio;

use super::*;
use crate::{
    file::{Info, UnvalidatedInfo},
    testutil::from_hex,
};

pub(crate) static NTSC: LazyLock<PackContext> = LazyLock::new(|| PackContext {
    file_info: UnvalidatedInfo::new(Info {
        file_size: 120_000,
        video_frame_rate: Ratio::<u32>::new(30_000, 1_001),
        video_duration: Ratio::<u128>::new(1_001, 30_000),
        audio_stereo_stream_count: 1,
        audio_sample_rate: Some(48_000),
    })
    .validate()
    .unwrap(),
});

pub(crate) static PAL: LazyLock<PackContext> = LazyLock::new(|| PackContext {
    file_info: UnvalidatedInfo::new(Info {
        file_size: 144_000,
        video_frame_rate: Ratio::<u32>::from(25),
        video_duration: Ratio::<u128>::new(1, 25),
        audio_stereo_stream_count: 1,
        audio_sample_rate: Some(48_000),
    })
    .validate()
    .unwrap(),
});

/// Shorthand function useful for constructing a validated pack from an unvalidated pack literal.
pub(crate) fn validated<T: PackData>(unvalidated_pack: T, ctx: PackContext) -> ValidPack<T> {
    Unvalidated::new(unvalidated_pack).validate_with(&ctx).unwrap().into()
}

#[derive(Debug)]
pub(crate) struct PackBinaryTestCase<'a> {
    pub(crate) input: &'a str,
    pub(crate) parsed: Option<Pack>,
    pub(crate) err: Option<&'a str>,
    pub(crate) output: Option<&'a str>,
    pub(crate) ctx: PackContext,
}

impl<'a> Default for PackBinaryTestCase<'a> {
    fn default() -> Self {
        Self { input: "", parsed: None, err: None, output: None, ctx: *NTSC }
    }
}

/// Test round trip of a pack: deserialize from binary, check the result, and then back to binary.
pub(crate) fn run_pack_binary_test_case(tc: &PackBinaryTestCase) {
    // Parse input hex to bytes
    let input = from_hex(tc.input);

    // Deserialize the pack and check for expected error and deserialized pack contents
    let (deserialized, err) = Pack::from_raw(&input, &tc.ctx);
    match tc.err {
        None => expect_that!(err, none()),
        Some(msg) => expect_that!(err.map(|e| e.chain().to_string()), some(eq(msg))),
    };
    let expected_pack = match tc.parsed {
        // Leaving tc.parsed as None is a shortcut that avoids retyping the Invalid pack.
        None => Pack::Invalid(
            Type::from(input[0]),
            ValidPack::<Unparsed>::try_from_raw(input[1..].try_into().unwrap(), &NTSC).unwrap(),
        ),
        Some(p) => p,
    };
    expect_that!(deserialized, eq(expected_pack));

    // Serialize the pack and check for expected output
    let serialized = deserialized.to_raw(&tc.ctx);
    // Leaving tc.output as None is a shortcut for saying that the input is expected
    let expected_output = tc.output.map_or_else(|| input, from_hex);
    expect_that!(serialized, eq(expected_output));
}
