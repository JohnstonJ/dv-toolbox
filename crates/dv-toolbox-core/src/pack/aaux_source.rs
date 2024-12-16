use arbitrary_int::{u1, u2, u3, u4, u6};
use bitbybit::{bitenum, bitfield};
use garde::Validate;
use serde::{Deserialize, Serialize};
use snafu::{whatever, OptionExt};

use super::RawSourceType;
use crate::file::{System, ValidInfoMethods};

#[cfg(test)]
mod tests;

super::util::required_enum! {
    /// Quantization format of audio samples.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum AudioQuantization {
        /// 16-bit linear samples (standard PCM)
        Linear16Bit = 0x0,

        /// Special 12-bit non-linear samples
        NonLinear12Bit = 0x1,

        /// 20-bit linear samples
        Linear20Bit = 0x2,

        Reserved3 = 0x3,
        Reserved4 = 0x4,
        Reserved5 = 0x5,
        Reserved6 = 0x6,
        Reserved7 = 0x7,
    }

    #[bitenum(u3, exhaustive = true)]
    enum RawAudioQuantization;
}

super::util::required_enum! {
    /// Whether the audio clock was locked to the video clock.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum LockedMode {
        /// The audio clock of the recording device was locked to the video clock.
        ///
        /// This value would typically be seen only in professional recording equipment.  It means
        /// that the number of audio samples per frame will always be the same, and audio will not
        /// become desynchronized from the video over time.
        Locked = 0x0,

        /// The audio clock of the recording device was _not_ locked to the video clock.
        ///
        /// This value is typical of consumer camcorders.  It means that the number of audio samples
        /// per frame will drift.  To avoid audio/video desynchronization during post-processing, it
        /// is recommended that you resample the audio in each video frame before using it.
        Unlocked = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawLockedMode;
}

super::util::required_enum! {
    /// Partially determines the channel layout.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum StereoMode {
        MultiStereoAudio = 0x0,
        LumpedAudio = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawStereoMode;
}

super::util::required_enum! {
    /// Whether the audio in audio block channel CH1 (CH3) is related to audio in audio block
    /// channel CH2 (CH4).
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum AudioBlockPairing {
        /// The audio block channels are paired with each other.
        Paired = 0x0,

        /// The audio block channels are independent of each other.
        Independent = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawAudioBlockPairing;
}

super::util::required_enum! {
    /// Time constant of audio pre-emphasis.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum EmphasisTimeConstant {
        /// Audio pre-emphasis of 50/15 microseconds
        Emphasis50_15 = 0x1,

        #[allow(missing_docs)]
        Reserved = 0x0,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawEmphasisTimeConstant;
}

/// Contains information about the audio stream.
///
/// The values are only unique within a given audio block channel: i.e. the first 5 or 6 DIF blocks
/// in a given sequence, or the second half of the DIF blocks in the sequence.
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 8.1 - Source (AAUX)
/// - SMPTE 306M-2002 Section 7.4.1 - AAUX source pack (AS)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct AAUXSource {
    // Basic audio information for a channel
    //
    /// Audio sample rate in Hz.
    ///
    /// Currently this is limited to 32000, 44100, and 48000 Hz.
    #[garde(custom(check_audio_sample_rate))]
    pub audio_sample_rate: u32,

    /// Quantization format of audio samples.
    #[garde(skip)]
    pub quantization: AudioQuantization,

    /// Number of audio samples in this video frame.
    ///
    /// Applies to all channels.
    #[garde(custom(check_audio_frame_size(self)))]
    pub audio_frame_size: u16,

    /// Whether the audio clock was locked to the video clock.
    #[garde(skip)]
    pub locked_mode: LockedMode,

    // Audio channel layout
    //
    /// Partially determines the channel layout, along with
    /// [`AAUXSource::audio_block_channel_count`] and [`AAUXSource::audio_mode`].  See the table
    /// in IEC 61834-4:1998 Section 8.1 - Source (AAUX).
    #[garde(skip)]
    pub stereo_mode: StereoMode,

    /// Number of audio channels within an audio block channel.
    ///
    /// Valid values are 1 and 2.
    ///
    /// For example, the first 5 or 6 DIF blocks in a sequence/track forms an audio block channel.
    /// These may in turn hold 1 or 2 audio channels.  This would be followed by additional DIF
    /// blocks forming additional audio block channels.
    ///
    /// Partially determines the channel layout, along with [`AAUXSource::stereo_mode`] and
    /// [`AAUXSource::audio_mode`].  See the table in IEC 61834-4:1998 Section 8.1 - Source (AAUX).
    #[garde(range(min = 1, max = 2))]
    pub audio_block_channel_count: u8,

    /// Defines the layout of the audio signal channels (what is left/right/center/surround, etc.).
    ///
    /// The values are too complex to encapsulate in an enum.  See the table in
    /// IEC 61834-4:1998 Section 8.1 - Source (AAUX).
    ///
    /// This value partially determines the channel layout, along with [`AAUXSource::stereo_mode`]
    /// and [`AAUXSource::audio_block_channel_count`].
    #[garde(skip)]
    pub audio_mode: u4,

    /// Whether the audio in audio block channel CH1 (CH3) is related to audio in audio block
    /// channel CH2 (CH4).
    #[garde(skip)]
    pub audio_block_pairing: AudioBlockPairing,

    /// Indicates whether there are multiple languages.
    ///
    /// This could mean that other audio blocks have channels in alternative languages.  It could
    /// also indicate the presence of alternative programming in the same language, such as
    /// director's commentary.
    #[garde(skip)]
    pub multi_language: bool,

    // Other information
    //
    /// Determines which video system type is in use.
    ///
    /// The video system is also determined in conjunction with the [`AAUXSource::field_count`]
    /// pack field.
    #[garde(skip)]
    pub source_type: super::SourceType,

    /// The number of fields per frame.
    ///
    /// Valid values are 50 (PAL/SECAM) and 60 (NTSC).
    #[garde(custom(super::check_field_count))]
    pub field_count: u8,

    /// Whether audio pre-emphasis is enabled.
    #[garde(skip)]
    pub emphasis_on: bool,

    /// Time constant of audio pre-emphasis.
    #[garde(skip)]
    pub emphasis_time_constant: EmphasisTimeConstant,

    /// Reserved bits; should normally be set to 0x3.
    #[garde(skip)]
    pub reserved: u2,
}

fn allowed_audio_frame_size_range(system: System, audio_sample_rate: u32) -> Option<(u16, u16)> {
    match system {
        System::Sys525_60 => match audio_sample_rate {
            32_000 => Some((1_053, 1_080)),
            44_100 => Some((1_452, 1_489)),
            48_000 => Some((1_580, 1_620)),
            _ => None,
        },
        System::Sys625_50 => match audio_sample_rate {
            32_000 => Some((1_264, 1_296)),
            44_100 => Some((1_742, 1_786)),
            48_000 => Some((1_896, 1_944)),
            _ => None,
        },
    }
}

/// Validate that the audio sample rate is 32000, 44100, or 48000 Hz.
fn check_audio_sample_rate(audio_sample_rate: &u32, _ctx: &super::PackContext) -> garde::Result {
    match audio_sample_rate {
        32_000 | 44_100 | 48_000 => Ok(()),
        sr => Err(garde::Error::new(format!(
            "audio sample rate of {sr} is not one of the supported values of 32000, 44100, \
            or 48000 Hz"
        ))),
    }
}

/// Check that the number of audio samples in the video frame is valid for the video system and
/// audio sample rate.
fn check_audio_frame_size(
    aaux_source: &AAUXSource,
) -> impl FnOnce(&u16, &super::PackContext) -> garde::Result + '_ {
    |audio_frame_size, ctx| {
        let system = ctx.file_info.system();
        let audio_sample_rate = aaux_source.audio_sample_rate;
        let (min, max) =
            allowed_audio_frame_size_range(system, audio_sample_rate).ok_or_else(|| {
                garde::Error::new(
                    "cannot validate the audio frame size because the audio sample rate \
                    is unsupported",
                )
            })?;
        if *audio_frame_size < min {
            Err(garde::Error::new(format!(
                "video frame contains {audio_frame_size} audio samples, which is below the \
                required minimum of {min} for video system {system} and audio sample rate \
                {audio_sample_rate} Hz"
            )))
        } else if *audio_frame_size > max {
            Err(garde::Error::new(format!(
                "video frame contains {audio_frame_size} audio samples, which is above the \
                maximum of {max} for video system {system} and audio sample rate \
                {audio_sample_rate} Hz"
            )))
        } else {
            Ok(())
        }
    }
}

#[bitfield(u32)]
struct RawAAUXSource {
    // PC1
    #[bits(0..=5, rw)]
    af_size: u6,
    #[bits([6, 23],rw)] // also part of PC3
    reserved: u2,
    #[bit(7, rw)]
    lf: RawLockedMode,

    // PC2
    #[bits(8..=11, rw)]
    audio_mode: u4,
    #[bit(12, rw)]
    pa: RawAudioBlockPairing,
    #[bits(13..=14, rw)]
    chn: u2,
    #[bit(15, rw)]
    sm: RawStereoMode,

    // PC3
    #[bits(16..=20, rw)]
    stype: RawSourceType,
    #[bit(21, rw)]
    field_count: u1,
    #[bit(22, rw)]
    ml: bool,

    // PC4
    #[bits(24..=26, rw)]
    qu: RawAudioQuantization,
    #[bits(27..=29, rw)]
    smp: u3,
    #[bit(30, rw)]
    tc: RawEmphasisTimeConstant,
    #[bit(31, rw)]
    ef: bool,
}

impl super::PackData for AAUXSource {
    fn try_from_raw(
        raw: &super::RawPackData,
        ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawAAUXSource::new_with_raw_value(u32::from_le_bytes(*raw));
        let system = ctx.file_info.system();
        let audio_sample_rate = match raw.smp().value() {
            0x0 => 48_000,
            0x1 => 44_100,
            0x2 => 32_000,
            smp => whatever!("smp value of {smp} does not correspond to a known audio sample rate"),
        };
        let (min, _) = allowed_audio_frame_size_range(system, audio_sample_rate)
            .whatever_context("audio sample rate is unsupported")?;
        Ok(Self {
            audio_sample_rate,
            quantization: raw.qu().into(),
            audio_frame_size: min + u16::from(raw.af_size()),
            locked_mode: raw.lf().into(),
            stereo_mode: raw.sm().into(),
            audio_block_channel_count: match raw.chn().value() {
                0x0 => 1,
                0x1 => 2,
                chn => whatever!(
                    "chn value of {chn} does not correspond to a known number of channels \
                    per audio block"
                ),
            },
            audio_mode: raw.audio_mode(),
            audio_block_pairing: raw.pa().into(),
            multi_language: !raw.ml(),
            source_type: raw.stype().into(),
            field_count: match raw.field_count().value() {
                0x0 => 60,
                0x1 => 50,
                _ => panic!("code was supposed to be unreachable"),
            },
            emphasis_on: !raw.ef(),
            emphasis_time_constant: raw.tc().into(),
            reserved: raw.reserved(),
        })
    }
}

impl super::ValidPackDataTrait<AAUXSource> for super::ValidPack<AAUXSource> {
    fn to_raw(&self, ctx: &super::PackContext) -> super::RawPackData {
        // the panics in this function should not actually happen because the structure is validated
        let system = ctx.file_info.system();
        let (min, _) = allowed_audio_frame_size_range(system, self.audio_sample_rate).unwrap();
        RawAAUXSource::builder()
            .with_af_size(u6::new(u8::try_from(self.audio_frame_size - min).unwrap()))
            .with_reserved(self.reserved)
            .with_lf(self.locked_mode.into())
            .with_audio_mode(self.audio_mode)
            .with_pa(self.audio_block_pairing.into())
            .with_chn(match self.audio_block_channel_count {
                1 => u2::new(0x0),
                2 => u2::new(0x1),
                _ => panic!("code was suppposed to be unreachable in validated structure"),
            })
            .with_sm(self.stereo_mode.into())
            .with_stype(self.source_type.into())
            .with_field_count(match self.field_count {
                60 => u1::new(0x0),
                50 => u1::new(0x1),
                _ => panic!("code was suppposed to be unreachable in validated structure"),
            })
            .with_ml(!self.multi_language)
            .with_qu(self.quantization.into())
            .with_smp(match self.audio_sample_rate {
                48_000 => u3::new(0x0),
                44_100 => u3::new(0x1),
                32_000 => u3::new(0x2),
                _ => panic!("code was suppposed to be unreachable in validated structure"),
            })
            .with_tc(self.emphasis_time_constant.into())
            .with_ef(!self.emphasis_on)
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
