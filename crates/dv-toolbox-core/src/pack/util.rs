use std::fmt::Display;

use arbitrary_int::{u4, Number};
use snafu::prelude::*;

#[cfg(test)]
mod tests;

/// Convert binary-coded decimal value into a normal number.
///
/// If every bit in the digits is set, then the number is assumed to be absent (None).
// TODO: remove this annotation after using the function.
#[allow(dead_code)]
pub(crate) fn from_bcd_hundreds<THundreds>(
    hundreds: THundreds,
    tens: u4,
    units: u4,
) -> Result<Option<u16>, FastWhatever>
where
    THundreds: Display + PartialOrd + Copy + Number,
    u8: From<THundreds> + From<u4>,
{
    if hundreds == THundreds::MAX && tens == u4::MAX && units == u4::MAX {
        return Ok(None);
    }
    if u8::from(hundreds) > 9u8 {
        whatever!("hundreds place value of {} is greater than 9", hundreds);
    }
    if tens.value() > 9u8 {
        whatever!("tens place value of {} is greater than 9", tens);
    }
    if units.value() > 9u8 {
        whatever!("units place value of {} is greater than 9", units);
    }
    Ok(Some(
        u16::from(u8::from(hundreds)) * 100
            + u16::from(tens.value()) * 10
            + u16::from(units.value()),
    ))
}

/// Convert binary-coded decimal value into a normal number.
///
/// If every bit in the digits is set, then the number is assumed to be absent (None).
pub(crate) fn from_bcd_tens<TTens>(tens: TTens, units: u4) -> Result<Option<u8>, FastWhatever>
where
    TTens: Display + PartialOrd + Copy + Number,
    u8: From<TTens>,
{
    if tens == TTens::MAX && units == u4::MAX {
        return Ok(None);
    }
    if u8::from(tens) > 9u8 {
        whatever!("tens place value of {} is greater than 9", tens);
    }
    if units.value() > 9u8 {
        whatever!("units place value of {} is greater than 9", units);
    }
    Ok(Some(u8::from(tens) * 10 + units.value()))
}

/// Error type similar to [`snafu::Whatever`] but without the (slow to gather) backtrace.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub(crate) enum FastWhatever {
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
        // There is intentionally not a backtrace here, since they are slow and we could encounter
        // a lot of these errors when reading bad videotapes.
    },
}
