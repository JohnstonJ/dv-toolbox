use std::num::TryFromIntError;

use derive_more::{Deref, DerefMut};
use num::rational::Ratio;
use rsmpeg::avutil::{self, AVRational};
use snafu::prelude::*;

/// Newtype class for converting between num-rational Ratio and FFmpeg AVRational types.  Values
/// will automatically be reduced when making the conversion.
#[derive(Debug, Deref, DerefMut)]
pub struct AVRationalConverter(pub AVRational);

/// Errors that can occur when converting to or from an [`AVRational`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Snafu)]
pub enum AVRationalConversionError {
    /// The underlying [`AVRational`] denominator was zero. Most likely the field is entirely
    /// missing.
    #[snafu(display("zero value denominator"))]
    DenominatorZero,

    /// Checked conversion failed when converting the rational to another numeric type.
    #[snafu(display("rational cannot be converted to another numeric type"))]
    TryFromRational {
        /// Underlying conversion error.
        source: TryFromIntError,
    },
}

impl TryFrom<AVRationalConverter> for Ratio<i32> {
    type Error = AVRationalConversionError;
    fn try_from(value: AVRationalConverter) -> Result<Self, Self::Error> {
        if value.den == 0 {
            return Err(AVRationalConversionError::DenominatorZero);
        }
        Ok((value.num, value.den).into())
    }
}

impl From<Ratio<i32>> for AVRationalConverter {
    fn from(value: Ratio<i32>) -> Self {
        let reduced = value.reduced();
        Self(avutil::ra(*reduced.numer(), *reduced.denom()))
    }
}

impl AVRationalConverter {
    /// Try to convert to a [`Ratio`] with the given type.
    pub fn try_into_other_ratio<T>(&self) -> Result<Ratio<T>, AVRationalConversionError>
    where
        T: TryFrom<i32, Error = TryFromIntError> + Clone + num::Integer,
    {
        if self.den == 0 {
            return Err(AVRationalConversionError::DenominatorZero);
        }
        Ok((
            T::try_from(self.num).context(TryFromRationalSnafu)?,
            T::try_from(self.den).context(TryFromRationalSnafu)?,
        )
            .into())
    }

    /// Try to convert from a [`Ratio`] with the given type.
    pub fn try_from_other_ratio<T>(value: Ratio<T>) -> Result<Self, AVRationalConversionError>
    where
        T: TryInto<i32, Error = TryFromIntError> + Clone + num::Integer,
    {
        let reduced = value.reduced();
        let n: i32 = reduced.numer().clone().try_into().context(TryFromRationalSnafu)?;
        let d: i32 = reduced.denom().clone().try_into().context(TryFromRationalSnafu)?;
        Ok(Self(avutil::ra(n, d)))
    }
}

#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    #[googletest::test]
    fn test_rational_converter() {
        // AVRational to Rational32 conversion

        expect_that!(
            Ratio::<i32>::try_from(AVRationalConverter(avutil::ra(6, 8))),
            ok(eq(Ratio::<i32>::new(3, 4)))
        );
        expect_that!(
            Ratio::<i32>::try_from(AVRationalConverter(avutil::ra(6, 0))),
            err(pat!(AVRationalConversionError::DenominatorZero { .. }))
        );

        expect_that!(
            AVRationalConverter(avutil::ra(6, 8)).try_into_other_ratio::<u32>(),
            ok(eq(Ratio::<u32>::new(3, 4)))
        );
        expect_that!(
            AVRationalConverter(avutil::ra(6, 0)).try_into_other_ratio::<u32>(),
            err(pat!(AVRationalConversionError::DenominatorZero { .. }))
        );
        expect_that!(
            AVRationalConverter(avutil::ra(-6, 8)).try_into_other_ratio::<u32>(),
            err(pat!(AVRationalConversionError::TryFromRational { .. }))
        );

        // Rational32 to ffmpeg::Rational conversion

        let expected = avutil::ra(3, 4);
        let actual = *AVRationalConverter::from(Ratio::<i32>::new_raw(6, 8));
        // AVRational doesn't implement PartialEq
        expect_that!(actual.num, eq(expected.num));
        expect_that!(actual.den, eq(expected.den));

        let actual =
            AVRationalConverter::try_from_other_ratio(Ratio::<u32>::new_raw(6, 8)).unwrap();
        expect_that!(actual.num, eq(expected.num));
        expect_that!(actual.den, eq(expected.den));

        let actual = AVRationalConverter::try_from_other_ratio(Ratio::<u32>::new_raw(u32::MAX, 8));
        expect_that!(actual, err(pat!(AVRationalConversionError::TryFromRational { .. })));
    }
}
