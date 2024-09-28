use derive_more::{Deref, DerefMut};
use ffmpeg_next as ffmpeg;
use num::Rational32;

/// Newtype class for converting between num-rational Ratio and FFmpeg AVRational types.  Values
/// will automatically be reduced when making the conversion.
#[derive(Debug, Deref, DerefMut)]
pub struct AVRationalConverter(ffmpeg::Rational);

impl From<AVRationalConverter> for Rational32 {
    fn from(value: AVRationalConverter) -> Self {
        (value.numerator(), value.denominator()).into()
    }
}

impl From<Rational32> for AVRationalConverter {
    fn from(value: Rational32) -> Self {
        Self(value.reduced().into_raw().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[googletest::test]
    fn test_rational_converter() {
        // ffmpeg::Rational to Rational32 conversion
        expect_that!(Rational32::new(3, 4), eq(AVRationalConverter(ffmpeg::Rational(6, 8)).into()));

        // Rational32 to ffmpeg::Rational conversion
        expect_that!(
            ffmpeg::Rational::new(3, 4),
            eq(*AVRationalConverter::from(Rational32::new_raw(6, 8)))
        );
    }
}
