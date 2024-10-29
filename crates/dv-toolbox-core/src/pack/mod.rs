//! Model structures for working with packs, as defined in
//! [IEC 61834-4](https://webstore.iec.ch/en/publication/5987) and other related standards.
//!
//! The main top-level data type in this module is the [`Pack`] enumeration, which has a
//! variant for all supported pack types.  To deserialize from binary to [`Pack`], use the
//! [`Pack::from_raw`] function.  To serialize from a [`Pack`] back to binary, use the
//! [`Pack::to_raw`] function.

use std::hash::Hash;

pub use aaux_source::*;
pub use binary_group::*;
pub use common::*;
pub use date::*;
use derive_more::derive::{AsMut, AsRef, Deref, DerefMut, From};
use garde::{
    validate::{Valid, Validate},
    Unvalidated,
};
use serde::{de::DeserializeOwned, Serialize};
use snafu::prelude::*;
pub use time::*;
pub use types::*;

use crate::file;

mod aaux_source;
mod binary_group;
mod common;
mod date;
mod time;
mod types;
mod util;

#[cfg(test)]
mod testutil;

/// Unvalidated contents of a DV data pack.
///
/// The data cannot be written to binary until it is validated into the form of a
/// [`garde::validate::Valid`] wrapper structure.  The latter must implement the
/// [`ValidPackDataTrait`] trait, which provides the binary serialization code.
///
/// The data can also be serialized into other destinations, such as databases, using the common
/// [`serde`] crate.
pub trait PackData:
    std::fmt::Debug
    + PartialEq
    + Eq
    + Hash
    + Clone
    + Copy
    + Validate<Context = PackContext>
    + Serialize
    + DeserializeOwned
{
    /// Read from a DV file by deserializing the pack from raw bytes.  The return value is not
    /// validated, although the function may still return whatever errors.
    ///
    /// Users should generally use the corresponding [`ValidPackDataTrait::try_from_raw`] function.
    fn try_from_raw(raw: &RawPackData, ctx: &PackContext) -> Result<Self, RawError>;
}

/// Raw bytes composing a DV data pack, excluding the pack header byte.
pub type RawPackData = [u8; 4];

/// Raw bytes composing a DV data pack, including the pack header byte.
pub type RawPack = [u8; 5];

/// Validated contents of a DV data pack.
///
/// To instantiate, follow the instructions in [`Valid`] to validate a [`PackData`].
///
/// (Newtype on the [`Valid`] type that restores some of the derived functionality.)
#[derive(Debug, Clone, Copy, AsRef, AsMut, Deref, DerefMut, From)]
pub struct ValidPack<T: PackData>(pub Valid<T>);

impl<T: PackData> PartialEq for ValidPack<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&*other.0)
    }
}

impl<T: PackData> Eq for ValidPack<T> {}

impl<T: PackData> Hash for ValidPack<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Trait implemented on the validated contents of a DV data pack.
pub trait ValidPackDataTrait<T: PackData> {
    /// Serialize the pack into raw bytes for writing to a DV file.
    fn to_raw(&self, ctx: &PackContext) -> RawPackData;

    /// Read from a DV file by deserializing the pack from raw bytes, and then validating that they
    /// are correct.
    fn try_from_raw(raw: &RawPackData, ctx: &PackContext) -> Result<ValidPack<T>, RawError> {
        let unvalidated = T::try_from_raw(raw, ctx)?;
        let valid =
            Unvalidated::new(unvalidated).validate_with(ctx).context(PackValidationSnafu)?;
        Ok(valid.into())
    }
}

/// Extra information that is used when serialization/deserializing and validating DV pack data.
///
/// It's common that packs will need to refer to information about the entire DV file, such as
/// which [`file::System`] is in use.
#[derive(Debug, Clone, Copy)]
pub struct PackContext {
    /// Information about the file that the pack was obtained from.
    pub file_info: file::ValidInfo,
}

/// Error type for when there is a problem deserializing the raw binary data of a DV pack.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum RawError {
    /// The raw bytes were read into an unvalidated [`PackData`] struct, but they then failed
    /// to be validated.
    #[snafu(display("Pack failed validation during deserialization of raw bytes"))]
    PackValidation { source: garde::Report },

    #[snafu(whatever, display("Pack failed deserialization of raw bytes: {message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
        // There is intentionally not a backtrace here, since they are slow and we could encounter
        // a lot of these errors when reading bad videotapes.
    },
}
