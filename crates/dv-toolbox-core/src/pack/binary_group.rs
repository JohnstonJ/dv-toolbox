use arbitrary_int::u4;
use bitbybit::bitfield;
use garde::Validate;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Title, AAUX, or VAUX binary group pack
///
/// Binary groups are used to store additional time code data, such as time zones, as well as
/// large amounts of other character or user data.  The type of data is determined by the
/// `binary_group_flags` value in the corresponding timecode pack.
///
/// It's probably not safe to assume that these binary group values remain the same
/// throughout a single frame.
///
/// At this time, given the wide variety of possible values, this package makes no attempt to
/// further parse the binary groups.
///
/// In the context of DV, this data is defined at:
///
/// - Title binary group
///   - IEC 61834-4:1998 Section 4.5 - Binary Group (TITLE)
///   - SMPTE 306M-2002 Section 9.2.2 - Binary group pack (BG)
/// - AAUX binary group
///   - IEC 61834-4:1998 Section 8.5 - Binary Group (AAUX)
/// - VAUX binary group
///   - IEC 61834-4:1998 Section 9.5 - Binary Group (VAUX)
///
/// The format of the fields within this group are defined at:
///
/// - IEC 60461:2011 Section 7.4 - Use of the binary groups
/// - SMPTE 12M
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct BinaryGroup {
    /// The raw data of the binary groups.
    #[garde(skip)]
    pub group_data: [u4; 8],
}

#[bitfield(u32)]
struct RawBinaryGroup {
    #[bits(0..=3, rw)]
    group_data: [u4; 8],
}

impl super::PackData for BinaryGroup {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawBinaryGroup::new_with_raw_value(u32::from_le_bytes(*raw));
        Ok(BinaryGroup {
            group_data: {
                let mut data: [u4; 8] = [Default::default(); 8];
                for (i, d) in data.iter_mut().enumerate() {
                    *d = raw.group_data(i);
                }
                data
            },
        })
    }
}

impl super::ValidPackDataTrait<BinaryGroup> for super::ValidPack<BinaryGroup> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        RawBinaryGroup::builder().with_group_data(self.group_data).build().raw_value().to_le_bytes()
    }
}
