use arbitrary_int::{u1, u2, u4, Number};
use bitbybit::{bitenum, bitfield};
use garde::Validate;
use serde::{Deserialize, Serialize};
use snafu::{whatever, ResultExt};

use super::RawSourceType;

#[cfg(test)]
mod tests;

/// Determines the input source of the original video signal.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum SourceCode {
    /// The video was recorded by a camera.
    Camera,

    /// The video was recorded from another input video signal.
    ///
    /// The signal uses
    /// [MUSE](https://en.wikipedia.org/wiki/Multiple_sub-Nyquist_sampling_encoding).
    LineMUSE,

    /// The video was recorded from another input video signal.
    Line,

    /// The video was recorded from cable television.
    Cable,

    /// The video was recorded from a TV tuner.
    Tuner,

    /// The video is prerecorded tape.
    PrerecordedTape,
}

super::util::required_enum! {
    /// Whether the video is black and white
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum BlackAndWhiteFlag {
        /// Video is black and white
        BlackAndWhite = 0x0,

        /// Video is color
        Color = 0x1,
    }

    #[bitenum(u1, exhaustive=true)]
    enum RawBlackAndWhiteFlag;
}

super::util::required_enum! {
    /// Color frames ID code.
    ///
    /// See ITU-R Report 624-4.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum ColorFramesID {
        /// Applicable to 525-60 or 625-50 system
        CLFColorFrameAOr1_2Field = 0x0,

        /// Applicable to 525-60 or 625-50 system
        CLFColorFrameBOr3_4Field = 0x1,

        /// Applicable to 625-50 system
        CLF5_6Field = 0x2,

        /// Applicable to 625-50 system
        CLF7_8Field = 0x3,
    }

    #[bitenum(u2, exhaustive=true)]
    enum RawColorFramesID;
}

/// Contains information about the video stream.
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 9.1 - Source (VAUX)
/// - SMPTE 306M-2002 Section 8.9.1 - VAUX source pack (VS)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct VAUXSource {
    // Origin information
    //
    /// Determines the input source of the original video signal.
    #[garde(custom(check_source_code(&self)))]
    pub source_code: Option<SourceCode>,

    /// TV channel of the original video signal, if applicable.
    ///
    /// The value is required if and only if [`VAUXSource::source_code`] is set to
    /// [`SourceCode::Cable`] or [`SourceCode::Tuner`].
    #[garde(range(min = 1, max = 999))]
    pub tv_channel: Option<u16>,

    /// Tuner category is basically the range of spectrum that the channels are taken from.
    ///
    /// The value only applies if [`VAUXSource::source_code`] is set to [`SourceCode::Tuner`].
    #[garde(custom(check_tuner_category))]
    pub tuner_category: Option<u8>,

    // Other information
    //
    /// Determines which video system type is in use.
    ///
    /// The video system is also determined in conjunction with the [`VAUXSource::field_count`]
    /// pack field.
    #[garde(skip)]
    pub source_type: super::SourceType,

    /// The number of fields per frame.
    ///
    /// Valid values are 50 (PAL/SECAM) and 60 (NTSC).
    #[garde(custom(super::check_field_count))]
    pub field_count: u8,

    /// Whether the video is black and white
    #[garde(skip)]
    pub bw_flag: BlackAndWhiteFlag,

    /// Color frames ID code.
    #[garde(skip)]
    pub color_frames_id: Option<ColorFramesID>,
}

fn check_source_code(
    vaux_source: &VAUXSource,
) -> impl FnOnce(&Option<SourceCode>, &super::PackContext) -> garde::Result + '_ {
    |source_code, _ctx| {
        // Check for proper presence of TV channel
        match *source_code {
            Some(
                SourceCode::Camera
                | SourceCode::LineMUSE
                | SourceCode::Line
                | SourceCode::PrerecordedTape,
            )
            | None => {
                if vaux_source.tv_channel.is_some() {
                    return Err(garde::Error::new(
                        "a TV channel number must not be provided for this source code value",
                    ));
                }
            }
            Some(SourceCode::Cable | SourceCode::Tuner) => {
                if vaux_source.tv_channel.is_none() {
                    return Err(garde::Error::new(
                        "a TV channel number is required for this source code value",
                    ));
                }
            }
        };
        // Check for proper presence of tuner category
        match *source_code {
            Some(
                SourceCode::Camera
                | SourceCode::LineMUSE
                | SourceCode::Line
                | SourceCode::Cable
                | SourceCode::PrerecordedTape,
            )
            | None => {
                if vaux_source.tuner_category.is_some() {
                    return Err(garde::Error::new(
                        "a tuner category must not be provided if the source code is not a tuner",
                    ));
                }
            }
            Some(SourceCode::Tuner) => {
                if vaux_source.tuner_category.is_none() {
                    return Err(garde::Error::new(
                        "a tuner category must be provided if the source code is a tuner",
                    ));
                }
            }
        };
        Ok(())
    }
}

fn check_tuner_category(tuner_category: &Option<u8>, _ctx: &super::PackContext) -> garde::Result {
    if *tuner_category == Some(u8::MAX) {
        Err(garde::Error::new(
            "instead of specifying Some(0xFF), use None to indicate no information",
        ))
    } else {
        Ok(())
    }
}

#[bitfield(u32)]
struct RawVAUXSource {
    // PC1
    #[bits(0..=3, rw)]
    tv_channel_units: u4,
    #[bits(4..=7, rw)]
    tv_channel_tens: u4,

    // PC2
    #[bits(8..=11, rw)]
    tv_channel_hundreds: u4,
    #[bits(12..=13, rw)]
    clf: RawColorFramesID,
    #[bit(14, rw)]
    en: u1,
    #[bit(15, rw)]
    bw: RawBlackAndWhiteFlag,

    // PC3
    #[bits(16..=20, rw)]
    stype: RawSourceType,
    #[bit(21, rw)]
    field_count: u1,
    #[bits(22..=23, rw)]
    source_code: u2,

    // PC4
    #[bits(24..=31, rw)]
    tuner_category: u8,
}

impl super::PackData for VAUXSource {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawVAUXSource::new_with_raw_value(u32::from_le_bytes(*raw));

        // Figure out the source code and TV channel, which all go together
        let channel_is_e = raw.tv_channel_hundreds() == u4::new(0xE)
            && raw.tv_channel_tens() == u4::new(0xE)
            && raw.tv_channel_units() == u4::new(0xE);
        let channel_is_f = raw.tv_channel_hundreds() == u4::new(0xF)
            && raw.tv_channel_tens() == u4::new(0xF)
            && raw.tv_channel_units() == u4::new(0xF);

        Ok(Self {
            source_code: match raw.source_code().value() {
                0b00 => {
                    if !channel_is_f {
                        whatever!("TV channel must be absent for source code Camera");
                    }
                    Some(SourceCode::Camera)
                }
                0b01 => {
                    if channel_is_e {
                        Some(SourceCode::LineMUSE)
                    } else if channel_is_f {
                        Some(SourceCode::Line)
                    } else {
                        whatever!("invalid TV channel code specified for line-based source code");
                    }
                }
                0b10 => Some(SourceCode::Cable),
                0b11 => {
                    if channel_is_e {
                        Some(SourceCode::PrerecordedTape)
                    } else if channel_is_f {
                        None
                    } else {
                        Some(SourceCode::Tuner)
                    }
                }
                _ => panic!("code was supposed to be unreachable"),
            },
            tv_channel: if channel_is_e {
                None
            } else {
                super::util::from_bcd_hundreds(
                    raw.tv_channel_hundreds(),
                    raw.tv_channel_tens(),
                    raw.tv_channel_units(),
                )
                .whatever_context("couldn't read the TV channel number")?
            },
            tuner_category: if raw.tuner_category() == u8::MAX {
                None
            } else {
                Some(raw.tuner_category())
            },
            source_type: raw.stype().into(),
            field_count: match raw.field_count().value() {
                0x0 => 60,
                0x1 => 50,
                _ => panic!("code was supposed to be unreachable"),
            },
            bw_flag: raw.bw().into(),
            color_frames_id: match raw.en().value() {
                0x0 => Some(raw.clf().into()),
                0x1 => None,
                _ => panic!("code was supposed to be unreachable"),
            },
        })
    }
}

impl super::ValidPackDataTrait<VAUXSource> for super::ValidPack<VAUXSource> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        // the panics in this function should not actually happen because the structure is validated
        let (tv_channel_hundreds, tv_channel_tens, tv_channel_units) = match self.source_code {
            Some(SourceCode::Camera | SourceCode::Line) | None => (u4::MAX, u4::MAX, u4::MAX),
            Some(SourceCode::LineMUSE | SourceCode::PrerecordedTape) => {
                (u4::new(0xE), u4::new(0xE), u4::new(0xE))
            }
            Some(SourceCode::Cable | SourceCode::Tuner) => (
                u4::new(u8::try_from(self.tv_channel.unwrap() / 100).unwrap()),
                u4::new(u8::try_from((self.tv_channel.unwrap() / 10) % 10).unwrap()),
                u4::new(u8::try_from(self.tv_channel.unwrap() % 10).unwrap()),
            ),
        };
        RawVAUXSource::builder()
            .with_tv_channel_units(tv_channel_units)
            .with_tv_channel_tens(tv_channel_tens)
            .with_tv_channel_hundreds(tv_channel_hundreds)
            .with_clf(self.color_frames_id.unwrap_or(ColorFramesID::CLF7_8Field).into())
            .with_en(u1::new(self.color_frames_id.map_or(0x1, |_| 0x0)))
            .with_bw(self.bw_flag.into())
            .with_stype(self.source_type.into())
            .with_field_count(match self.field_count {
                60 => u1::new(0x0),
                50 => u1::new(0x1),
                _ => panic!("code was suppposed to be unreachable in validated structure"),
            })
            .with_source_code(u2::new(match self.source_code {
                Some(SourceCode::Camera) => 0b00,
                Some(SourceCode::LineMUSE | SourceCode::Line) => 0b01,
                Some(SourceCode::Cable) => 0b10,
                Some(SourceCode::Tuner | SourceCode::PrerecordedTape) | None => 0b11,
            }))
            .with_tuner_category(self.tuner_category.unwrap_or(u8::MAX))
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
