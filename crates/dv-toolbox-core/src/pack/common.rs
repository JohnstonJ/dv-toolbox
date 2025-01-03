//! Common data types shared between multiple packs.

use arbitrary_int::{u7, Number};
use bitbybit::bitenum;
use serde::{Deserialize, Serialize};

use crate::file::{System, ValidInfoMethods};

super::util::required_enum! {
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
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
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

    #[bitenum(u5, exhaustive = true)]
    pub(crate) enum RawSourceType;
}

super::util::required_enum! {
    /// Copy protection flags
    ///
    /// This flag is used by equipment to restrict copies from being made.  From the days before
    /// copy protection used encryption.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum CopyProtection {
        /// The content may be copied without restriction.  Copies shall also have this flag.
        NoRestriction = 0x0,

        #[allow(missing_docs)]
        Reserved = 0x1,

        /// Only one copy of the content may be made.  Copies made of this content shall be flagged
        /// with [`CopyProtection::NotPermitted`].
        OneGenerationOnly = 0x2,

        /// No copies of the content shall be allowed to be made.
        NotPermitted = 0x3,
    }

    #[bitenum(u2, exhaustive = true)]
    pub(crate) enum RawCopyProtection;
}

super::util::optional_enum! {
    /// Indicates whether the source was scrambled and whether it was descrambled when recorded.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum SourceSituation {
        /// The source was scrambled with audience restrictions, and was recorded without
        /// descrambling.
        ScrambledSourceWithAudienceRestrictions = 0b00,

        /// The source was scrambled without audience restrictions, and was recorded without
        /// descrambling.
        ScrambledSourceWithoutAudienceRestrictions = 0b01,

        /// The source has audience restrictions.  If it was scrambled, then it was descrambled.  In
        /// this scenario, the `TitleKey` pack should be recorded in the AAUX optional area.
        SourceWithAudienceRestrictions = 0b10,
    }

    #[bitenum(u2, exhaustive = true)]
    pub(crate) enum RawSourceSituation {
        NoInfo = 0b11,
    }
}

super::util::optional_enum! {
    /// Input source of the recorded content.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum InputSource {
        /// Analog input was used to record the content.
        Analog = 0b00,

        /// Digital input was used to record the content.
        Digital = 0b01,

        #[allow(missing_docs)]
        Reserved = 0b10,
    }

    #[bitenum(u2, exhaustive = true)]
    pub(crate) enum RawInputSource {
        NoInfo = 0b11,
    }
}

super::util::optional_enum! {
    /// The number of times the content has been compressed.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum CompressionCount {
        /// One generation of compression.
        Compressed1 = 0b00,

        /// Two generations of compression.
        Compressed2 = 0b01,

        /// Three or more generations of compression.
        Compressed3OrMore = 0b10,
    }

    #[bitenum(u2, exhaustive = true)]
    pub(crate) enum RawCompressionCount {
        NoInfo = 0b11,
    }
}

/// Validate that the field count is 50 or 60, and matches with the system.
pub(crate) fn check_field_count(field_count: &u8, ctx: &super::PackContext) -> garde::Result {
    let system = ctx.file_info.system();
    let expected_field_count = match system {
        System::Sys525_60 => 60,
        System::Sys625_50 => 50,
    };
    if *field_count != expected_field_count {
        Err(garde::Error::new(format!(
            "field count of {field_count} does not match the expected value of \
            {expected_field_count} for system {system}"
        )))
    } else {
        Ok(())
    }
}

/// Ensure that no information genre category values are specified as None, instead of Some(0x7F).
pub(crate) fn check_genre_category(
    genre_category: &Option<u7>,
    _ctx: &super::PackContext,
) -> garde::Result {
    if *genre_category == Some(u7::MAX) {
        Err(garde::Error::new(
            "instead of specifying Some(0x7F), use None to indicate no information",
        ))
    } else {
        Ok(())
    }
}
