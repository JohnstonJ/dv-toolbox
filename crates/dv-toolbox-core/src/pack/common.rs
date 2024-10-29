use bitbybit::bitenum;
use serde::{Deserialize, Serialize};

/// Common data types shared between multiple packs.

/// Determines which video system type is in use.
///
/// The video system is also determined in conjunction with the `field_count` pack field.
///
/// DV standards:
///
/// - AAUX source
///   - IEC 61834-4:1998 Section 8.1 - Source (AAUX)
///   - SMPTE 306M-2002 Section 7.4.1 - AAUX source pack (AS)
/// - VAUX source
///   - IEC 61834-4:1998 Section 9.1 - Source (VAUX)
///   - SMPTE 306M-2002 Section 8.9.1 - VAUX source pack (VS)
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[bitenum(u5, exhaustive = true)]
#[allow(missing_docs)]
pub enum SourceType {
    /// Defines a 525 line, 60 field system, or a 625 line, 50 field system
    ///
    /// 25 mbps bitrate, 4:1:1 chroma subsampling on NTSC
    ///
    /// Relevant standards:
    ///
    /// - IEC 61834-2:1998 - SD format for 525-60 and 625-50 systems
    /// - SMPTE 306M-2002 - 6.35-mm Type D-7 Component Format - Video Compression at 25 Mb/s and
    ///   50 Mb/s - 525/60 and 625/50
    StandardDefinitionCompressedChroma = 0x00,

    Reserved1 = 0x01,

    /// Defines a 1125 line, 60 field system, or a 1250, 50 field system
    ///
    /// Relevant standards:
    ///
    /// - IEC 61834-3:1999 - HD format for 1125-60 and 1250-50 systems
    AnalogHighDefinition1125_1250 = 0x02,

    Reserved3 = 0x03,

    /// Defines a higher-bitrate 525 line, 60 field system, or a 625 line, 50 field system
    ///
    /// 50 mbps bitrate, 4:2:2 chroma subsampling in SMPTE 306M
    ///
    /// Relevant standards:
    ///
    /// - SMPTE 306M-2002 - 6.35-mm Type D-7 Component Format - Video Compression at 25 Mb/s and
    ///   50 Mb/s - 525/60 and 625/50
    StandardDefinitionMoreChroma = 0x04,

    Reserved5 = 0x05,
    Reserved6 = 0x06,
    Reserved7 = 0x07,
    Reserved8 = 0x08,
    Reserved9 = 0x09,
    Reserved10 = 0x0A,
    Reserved11 = 0x0B,
    Reserved12 = 0x0C,
    Reserved13 = 0x0D,
    Reserved14 = 0x0E,
    Reserved15 = 0x0F,
    Reserved16 = 0x10,
    Reserved17 = 0x11,
    Reserved18 = 0x12,
    Reserved19 = 0x13,
    Reserved20 = 0x14,
    Reserved21 = 0x15,
    Reserved22 = 0x16,
    Reserved23 = 0x17,
    Reserved24 = 0x18,
    Reserved25 = 0x19,
    Reserved26 = 0x1A,
    Reserved27 = 0x1B,
    Reserved28 = 0x1C,
    Reserved29 = 0x1D,
    Reserved30 = 0x1E,
    Reserved31 = 0x1F,
}
