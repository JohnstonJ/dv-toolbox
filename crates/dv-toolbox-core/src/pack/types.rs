//! Lists all pack types and contains dispatching logic for serialization.

use garde::Validate;
use serde::{Deserialize, Serialize};

use super::ValidPackDataTrait;
#[cfg(test)]
mod tests;

/// Creates the [`Type`] enum, along with [`From`] trait implementations that handle unknown
/// values and map them to/from the [`Type::Unknown`] value.
macro_rules! type_macro {
    (
        $($(#[$attr:meta])* $name:ident ($val:expr, $data_type:ty),)*
    ) => {
        /// Possible DV pack types
        ///
        /// Use the [`From`] trait implementations to map to/from a raw pack header byte.
        ///
        /// For more information about the contents of a pack, refer to the documentation of
        /// the corresponding struct type wrapped by the enumeration.  The enum variants themselves
        /// only have minimal documentation.
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
        #[serde(tag = "pack_type", content = "unknown_value")]
        pub enum Type {
            $($(#[$attr])* $name,)*

            /// DV pack type that is not yet supported by this tool.
            Unknown(u8),
        }

        impl From<u8> for Type {
            /// Converts a raw pack header byte to the [`Type`] enum.  Unrecognized values are
            /// mapped to the [`Type::Unknown`] value.
            fn from(value: u8) -> Self {
                match value {
                    $($val => Self::$name,)*
                    unk => Self::Unknown(unk),
                }
            }
        }

        impl From<Type> for u8 {
            /// Convert a [`Type`] to a raw pack header value.
            fn from(value: Type) -> Self {
                match value {
                    $(Type::$name => $val,)*
                    Type::Unknown(unk) => unk,
                }
            }
        }

        /// DV pack data of any type.
        ///
        /// For more information about the contents of a pack, refer to the documentation of
        /// the corresponding struct type wrapped by the enumeration.  The enum variants themselves
        /// only have minimal documentation.
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum Pack {
            $($(#[$attr])* $name(super::ValidPack<$data_type>),)*

            /// Holds the contents of an invalid pack.
            ///
            /// In this scenario, the pack type was known to this package, but the contents
            /// of the pack could not be correctly deserialized and validated for some reason.
            Invalid(Type, super::ValidPack<Unparsed>),

            /// Holds the contents of an unknown pack.
            ///
            /// In this scenario, the pack type was not known to this package.  The pack is
            /// likely a valid pack, but we just haven't implemented support for it yet.
            Unknown(u8, super::ValidPack<Unparsed>),
        }

        impl Pack {
            /// Serialize the DV pack to binary suitable for writing to a DV file.
            pub fn to_raw(&self, ctx: &super::PackContext) -> super::RawPack {
                match *self {
                    $(
                        Self::$name(data) => {
                            let ser = data.to_raw(ctx);
                            [self.pack_type().into(), ser[0], ser[1], ser[2], ser[3]]
                        },
                    )*
                    Self::Invalid(ty, data) => {
                        let ser = data.to_raw(ctx);
                        [ty.into(), ser[0], ser[1], ser[2], ser[3]]
                    },
                    Self::Unknown(ty, data) => {
                        let ser = data.to_raw(ctx);
                        [ty, ser[0], ser[1], ser[2], ser[3]]
                    },
                }
            }

            /// Deserialize the DV pack from binary bytes obtained from a DV file.
            ///
            /// The function is guaranteed to return a [`Pack`] value.  However, if there are
            /// problems parsing or validating the payload, the return enum variant will be
            /// [`Pack::Invalid`], and the parsing error will also be returned.
            ///
            /// This mechanism ensues that invalid pack values are preserved through
            /// deserialization/serialization.  Users who wish to remove invalid pack values can
            /// do so independently of this function.
            pub fn from_raw(
                raw: &super::RawPack, ctx: &super::PackContext
            ) -> (Self, Option<super::RawError>) {
                // Parse the pack header byte into an enum
                let ty = Type::from(raw[0]);
                match ty {
                    // For each pack header type
                    $(
                        Type::$name => {
                            // Try to deserialize the pack
                            let deser = super::ValidPack::<$data_type>::try_from_raw(
                                raw[1..].try_into().unwrap(), ctx
                            );
                            deser.map_or_else(
                                |err| {
                                    // If the deserialization failed, then map to Invalid pack,
                                    // which is guaranteed not to panic
                                    let invalid_pack = super::ValidPack::<Unparsed>::try_from_raw(
                                        raw[1..].try_into().unwrap(), ctx
                                    ).unwrap();
                                    (Self::Invalid(ty, invalid_pack), Some(err))
                                },
                                // Success case: return the deserialized pack
                                |val| (Self::$name(val), None)
                            )
                        },
                    )*
                    Type::Unknown(unk) => {
                        // We've never seen this pack type before, so convert to Unknown
                        (
                            Self::Unknown(unk, super::ValidPack::<Unparsed>::try_from_raw(
                                raw[1..].try_into().unwrap(), ctx
                            ).unwrap()),
                            None,
                        )
                    },
                }
            }

            /// Get the pack type for this pack.
            pub fn pack_type(&self) -> Type {
                match *self {
                    $(Self::$name(_) => Type::$name,)*
                    Self::Invalid(ty, _) => ty,
                    Self::Unknown(ty, _) => Type::Unknown(ty),
                }
            }
        }
    };
}

// List all the possible pack types
type_macro! {
    /// The timecode data showing elapsed time in the title at the tape position where this
    /// was recorded.
    ///
    /// - IEC 61834-4:1998 Section 4.4 - Time Code (TITLE)
    /// - SMPTE 306M-2002 Section 9.2.1 - Time code pack (TC)
    TitleTimecode(0x13, super::TitleTimecode),
    /// Additional binary group data associated with the [`Pack::TitleTimecode`] pack.
    ///
    /// - IEC 61834-4:1998 Section 4.5 - Binary Group (TITLE)
    /// - SMPTE 306M-2002 Section 9.2.2 - Binary group pack (BG)
    TitleBinaryGroup(0x14, super::BinaryGroup),

    /// The date when audio data is recorded.
    ///
    /// - IEC 61834-4:1998 Section 8.3 - Rec Date (AAUX)
    AAUXRecordingDate(0x52, super::RecordingDate),
    /// The time when audio data is recorded.
    ///
    /// - IEC 61834-4:1998 Section 8.4 - Rec Time (AAUX)
    AAUXRecordingTime(0x53, super::RecordingTime),
    /// Additional binary group data associated with the [`Pack::AAUXRecordingTime`] pack.
    ///
    /// - IEC 61834-4:1998 Section 8.5 - Binary Group (AAUX)
    AAUXBinaryGroup(0x54, super::BinaryGroup),

    /// The date when video data is recorded.
    ///
    /// - IEC 61834-4:1998 Section 9.3 - Rec Date (VAUX)
    VAUXRecordingDate(0x62, super::RecordingDate),
    /// The time when video data is recorded.
    ///
    /// - IEC 61834-4:1998 Section 9.4 - Rec Time (VAUX)
    VAUXRecordingTime(0x63, super::RecordingTime),
    /// Additional binary group data associated with the [`Pack::VAUXRecordingTime`] pack.
    ///
    /// - IEC 61834-4:1998 Section 9.5 - Binary Group (VAUX)
    VAUXBinaryGroup(0x64, super::BinaryGroup),

    /// No information
    ///
    /// There is no pack in this position, or it dropped out.
    ///
    /// - IEC 61834-4:1998 Section 12.16 - No Info: No information (SOFT MODE)
    NoInfo(0xFF, NoInfo),
}

/// No information pack
///
/// Indicates that this pack has no information.  In other words, there's nothing here.  There's
/// really no pack to begin with.  It's empty.
///
/// This also very commonly indicates a dropout: there could have originally been information here,
/// but the tape deck failed to read it.
///
/// - IEC 61834-4:1998 Section 12.16 - No Info: No information (SOFT MODE)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct NoInfo {}

impl super::PackData for NoInfo {
    fn try_from_raw(
        _raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        // The standard says that pack_bytes will always be 0xFFFFFFFFFF.  In practice, you'll also
        // "get" this pack as a result of dropouts from other packs: if the leading pack header is
        // lost and becomes 0xFF (this pack type), but the rest of the pack is not lost, then we'd
        // see other non-0xFF bytes here.  Unfortunately, in such a scenario, since the pack header
        // was lost, we don't know what pack that data is supposed to go with.  So we'll just let
        // this pack discard those bytes as it's probably not worth trying to preserve them.
        Ok(NoInfo {})
    }
}

impl super::ValidPackDataTrait<NoInfo> for super::ValidPack<NoInfo> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        [0xFF; 4]
    }
}

/// Holds the contents of an invalid or unknown pack.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct Unparsed {
    /// Contents of the invalid or unknown pack.
    #[garde(skip)]
    pub data: [u8; 4],
}

impl super::PackData for Unparsed {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        Ok(Unparsed { data: *raw })
    }
}

impl super::ValidPackDataTrait<Unparsed> for super::ValidPack<Unparsed> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        self.data
    }
}
