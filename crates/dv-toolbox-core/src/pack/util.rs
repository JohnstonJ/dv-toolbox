use std::fmt::Display;

use arbitrary_int::{u4, Number};
use snafu::prelude::*;

#[cfg(test)]
mod tests;

/// Convert binary-coded decimal value into a normal number.
///
/// If every bit in the digits is set, then the number is assumed to be absent (None).
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

/// Create both a regular required enum and an exhaustive bitenum, along with [`From`] trait
/// implementations to convert between them.
macro_rules! required_enum {
    (
        // Specify the enum values that are public-facing.  Number the values for use with bitenum.
        $(#[$enum_attr:meta])*
        $enum_vis:vis enum $enum_name:ident {
            $($(#[$variant_attr:meta])* $variant_name:ident = $variant_raw_value:tt,)*
        }

        // Specify the raw bitenum enumeration.  The variants from the previous enum will be
        // automatically inserted into this one.
        $(#[$raw_enum_attr:meta])*
        $raw_enum_vis:vis enum $raw_enum_name:ident;
    ) => {
        // Create a public-facing enum that does not number its variants.
        $(#[$enum_attr])*
        $enum_vis enum $enum_name {
            $($(#[$variant_attr])* $variant_name,)*
        }

        // Create an enum used to serialize to/from binary.  It's intended that the user use an
        // exhaustive bitenum attribute with this.
        $(#[$raw_enum_attr])*
        $raw_enum_vis enum $raw_enum_name {
            $($(#[$variant_attr])* $variant_name = $variant_raw_value,)*
        }

        // Straightforward RawMyEnum --> MyEnum mapping.
        impl From<$raw_enum_name> for $enum_name {
            fn from(value: $raw_enum_name) -> Self {
                match value {
                    $($raw_enum_name::$variant_name => $enum_name::$variant_name,)*
                }
            }
        }

        // Straightforward MyEnum --> RawMyEnum mapping.
        impl From<$enum_name> for $raw_enum_name {
            fn from(value: $enum_name) -> Self {
                match value {
                    $($enum_name::$variant_name => $raw_enum_name::$variant_name,)*
                }
            }
        }
    };
}

pub(crate) use required_enum;

/// Create both a regular optional enum and an exhaustive bitenum, along with [`From`] trait
/// implementations to convert between them.
macro_rules! optional_enum {
    (
        // Specify the enum values that are public-facing.  Number the values for use with bitenum.
        $(#[$enum_attr:meta])*
        $enum_vis:vis enum $enum_name:ident {
            $($(#[$variant_attr:meta])* $variant_name:ident = $variant_raw_value:tt,)*
        }

        // Specify the raw bitenum enumeration, with a single "no info" variant.  The variants
        // from the previous enum will be automatically inserted into this one.
        $(#[$raw_enum_attr:meta])*
        $raw_enum_vis:vis enum $raw_enum_name:ident {
            $(#[$no_info_attr:meta])* $no_info_name:ident = $no_info_raw_value:tt,
        }
    ) => {
        // Create a public-facing enum that does not number its variants.
        $(#[$enum_attr])*
        $enum_vis enum $enum_name {
            $($(#[$variant_attr])* $variant_name,)*
        }

        // Create an enum used to serialize to/from binary, and contains a "no info" attribute.
        // It's intended that the user use an exhaustive bitenum attribute with this.
        $(#[$raw_enum_attr])*
        $raw_enum_vis enum $raw_enum_name {
            $($(#[$variant_attr])* $variant_name = $variant_raw_value,)*

            $(#[$no_info_attr])* $no_info_name = $no_info_raw_value,
        }

        // Straightforward RawMyEnum --> Option<MyEnum> mapping.  NoInfo raw values are converted
        // to None values.
        impl From<$raw_enum_name> for Option<$enum_name> {
            fn from(value: $raw_enum_name) -> Self {
                match value {
                    $($raw_enum_name::$variant_name => Some($enum_name::$variant_name),)*
                    $raw_enum_name::$no_info_name => None,
                }
            }
        }

        // Straightforward Option<MyEnum> --> RawMyEnum mapping.  None values are converted to
        // NoInfo raw values.
        impl From<Option<$enum_name>> for $raw_enum_name {
            fn from(value: Option<$enum_name>) -> Self {
                match value {
                    $(Some($enum_name::$variant_name) => $raw_enum_name::$variant_name,)*
                    None => $raw_enum_name::$no_info_name,
                }
            }
        }
    };
}

pub(crate) use optional_enum;
