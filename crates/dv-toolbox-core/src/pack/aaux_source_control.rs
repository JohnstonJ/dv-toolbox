use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use arbitrary_int::{u1, u7, Number};
use bitbybit::{bitenum, bitfield};
use garde::Validate;
use num::rational::Ratio;
use serde::{Deserialize, Serialize};

use super::{RawCompressionCount, RawCopyProtection, RawInputSource, RawSourceSituation};

#[cfg(test)]
mod tests;

super::util::required_enum! {
    /// Indicates the recording mode of the audio.
    ///
    /// New audio content can be dubbed onto existing video at a later time.  This flag is
    /// supposed to indicate whether that has happened, and if so, onto which channels.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum AAUXRecordingMode {
        /// All audio was recorded at the same time as the video.
        Original = 0x1,

        /// One of the audio block channels was updated with new content.  The remaining audio block
        /// channels were recorded at the same time as the video.
        OneChannelInsert = 0x3,

        /// Two of the audio block channels were updated with new content.  The remaining audio block
        /// channels were recorded at the same time as the video.
        ///
        /// The inserted audio block channels must be either CH1 and CH2, or CH3 and CH4.
        TwoChannelInsert = 0x5,

        /// All four of the audio block channels were updated with new content at a later time after
        /// the video was recorded.
        FourChannelInsert = 0x4,

        InvalidRecording = 0x7,

        Reserved0 = 0x0,
        Reserved2 = 0x2,
        Reserved6 = 0x6,
    }

    #[bitenum(u3, exhaustive = true)]
    enum RawAAUXRecordingMode;
}

super::util::optional_enum! {
    /// Indicates which channels were inserted.
    ///
    /// The value is only meaningful for Memory in Cassette (MIC).  Packs recorded on tape are not
    /// supposed to specify a value here.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum AAUXInsertChannel {
        /// Audio block channel 1 inserted.
        Channel1 = 0b000,

        /// Audio block channel 2 inserted.
        Channel2 = 0b001,

        /// Audio block channel 3 inserted.
        Channel3 = 0b010,

        /// Audio block channel 4 inserted.
        Channel4 = 0b011,

        /// Audio block channels 1 and 2 inserted.
        Channels1_2 = 0b100,

        /// Audio block channels 3 and 4 inserted.
        Channels3_4 = 0b101,

        /// Audio block channels 1 through 4 inserted.
        Channels1_2_3_4 = 0b110,
    }

    #[bitenum(u3, exhaustive = true)]
    enum RawAAUXInsertChannel {
        NoInfo = 0b111,
    }
}

super::util::required_enum! {
    /// Playback direction.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum Direction {
        /// Tape playback was in a forward direction.
        Forward = 0x1,

        /// Tape playback was in a reverse direction.
        Reverse = 0x0,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawDirection;
}

/// Contains some metadata about the audio stream.
///
/// The structure from IEC 61834-4 is actually substantially different from
/// SMPTE 306M-2002 7.4.2 AAUX source control pack (ASC), which is yet still different from the
/// draft copy of SMPTE 314M I was able to locate.  This structure may need some updates if someone
/// reconciles it with a newer/final copy of the SMPTE standards and/or some real-life DV files.
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 8.2 - Source control (AAUX)
/// - SMPTE 306M-2002 Section 7.4.2 - AAUX source control pack (ASC)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct AAUXSourceControl {
    // Copy protection information
    //
    /// Copy protection flags
    #[garde(skip)]
    pub copy_protection: super::CopyProtection,

    /// Indicates whether the source was scrambled and whether it was descrambled when recorded.
    #[garde(skip)]
    pub source_situation: Option<super::SourceSituation>,

    // General metadata
    //
    /// Input source of the recorded content, if known.
    #[garde(skip)]
    pub input_source: Option<super::InputSource>,

    /// The number of times the content has been compressed, if known.
    #[garde(skip)]
    pub compression_count: Option<super::CompressionCount>,

    /// Indicates whether this pack marks the start of a new recording.
    ///
    /// The value should be repeated a full "audio block period" for each recording channel.
    #[garde(skip)]
    pub recording_start_point: bool,

    /// Indicates whether this pack marks the end of a recording.
    ///
    /// The value should be repeated a full "audio block period" for each recording channel.
    #[garde(skip)]
    pub recording_end_point: bool,

    /// Indicates the recording mode of the audio.
    #[garde(skip)]
    pub recording_mode: AAUXRecordingMode,

    /// Indicates which channels were inserted.
    ///
    /// The value is only meaningful for Memory in Cassette (MIC).  Packs recorded on tape are not
    /// supposed to specify a value here, and should be set to [`Option::None`].
    #[garde(skip)]
    pub insert_channel: Option<AAUXInsertChannel>,

    /// Defines the genre or category of the audio source.
    ///
    /// The value corresponds to a massive enumeration of dozens of TV genres.  See
    /// IEC 61834-4:1998 Section 3.3 - Timer Activation Date (CONTROL) for the full list.
    #[garde(custom(check_genre_category))]
    pub genre_category: Option<u7>,

    // Playback information
    //
    /// Playback direction.
    #[garde(skip)]
    pub direction: Direction,

    /// Playback speed, if known.
    ///
    /// Only specific fractional values are supported by this pack.  The list of valid values are
    /// returned by [`AAUXSourceControl::valid_playback_speeds`].
    ///
    /// Playback speed works as follows:
    ///
    /// - Videotape was recorded from a normal speed source (e.g. camera):
    ///   - Tape deck plays back at normal speed during transfer to computer: value is 1
    ///   - Tape deck plays at other speed during transfer to computer: value is the alternate speed
    /// - Videotape was recorded from another source that was playing back at a different speed:
    ///   - Tape deck plays back at normal speed during transfer to computer: value is the playback
    ///     speed from the previous device (i.e. the speed from the previous tape-to-tape transfer)
    ///   - Tape deck plays back at other speed during transfer to computer: value is None
    ///
    /// For more information, see IEC 61834-2:1998 Section 11.6 - Playback speed.
    #[garde(custom(check_playback_speed))]
    pub playback_speed: Option<Ratio<u8>>,

    /// Reserved bits; should normally be set to 0x1.
    #[garde(skip)]
    pub reserved: u1,
}

fn check_genre_category(genre_category: &Option<u7>, _ctx: &super::PackContext) -> garde::Result {
    if *genre_category == Some(u7::MAX) {
        Err(garde::Error::new(
            "instead of specifying Some(0x7F), use None to indicate no information",
        ))
    } else {
        Ok(())
    }
}

static PLAYBACK_SPEED_BITS_TO_RATIO: LazyLock<[Option<Ratio<u8>>; 128]> = LazyLock::new(|| {
    let mut speeds = [None; 128];

    // First row of playback speeds (coarse value 0) is special
    speeds[0x00] = Some(Ratio::<u8>::ZERO);
    speeds[0x01] = Some(Ratio::<u8>::new(1, 32)); // really just means "some speed slower than 1/16"
    for fine_bits in 0x2u8..=0xFu8 {
        speeds[usize::from(fine_bits)] = Some(Ratio::<u8>::new(1, 18 - fine_bits));
    }

    // Remaining rows of coarse speeds follows some simple exponential rules
    for coarse_bits in 0x1u8..=0x7u8 {
        // 1/2, 1, 2, 4, 8, 16, 32:
        let coarse_value = Ratio::<u8>::from(2).pow(-1 + i32::from(coarse_bits) - 1);
        for fine_bits in 0x0u8..=0xFu8 {
            let fine_value = Ratio::<u8>::from(fine_bits)
                / Ratio::<u8>::from(2).pow(6i32 - i32::from(coarse_bits));
            // Note that the very final cell value with all bits set means "no information"
            // or "unknown speed"
            let speed_bits = (coarse_bits << 4) | fine_bits;
            if speed_bits != u7::MAX.value() {
                speeds[usize::from(speed_bits)] = Some(coarse_value + fine_value);
            }
        }
    }
    speeds
});

static PLAYBACK_RATIO_TO_SPEED_BITS: LazyLock<HashMap<Option<Ratio<u8>>, u7>> =
    LazyLock::new(|| {
        HashMap::<Option<Ratio<u8>>, u7>::from_iter(
            PLAYBACK_SPEED_BITS_TO_RATIO
                .iter()
                .enumerate()
                .map(|(speed_bits, ratio)| (*ratio, u7::new(u8::try_from(speed_bits).unwrap()))),
        )
    });

static VALID_PLAYBACK_SPEEDS: LazyLock<HashSet<Ratio<u8>>> = LazyLock::new(|| {
    HashSet::<Ratio<u8>>::from_iter(PLAYBACK_RATIO_TO_SPEED_BITS.clone().into_keys().flatten())
});

fn check_playback_speed(
    playback_speed: &Option<Ratio<u8>>,
    _ctx: &super::PackContext,
) -> garde::Result {
    match playback_speed {
        Some(speed) => AAUXSourceControl::valid_playback_speeds()
            .contains(speed)
            .then_some(())
            .ok_or_else(|| {
                garde::Error::new(format!(
                    "playback speed {speed} not supported: only playback speeds returned by the \
                    valid_playback_speeds function are supported",
                ))
            }),
        None => Ok(()),
    }
}

#[bitfield(u32)]
struct RawAAUXSourceControl {
    // PC1
    #[bits(0..=1, rw)]
    ss: RawSourceSituation,
    #[bits(2..=3, rw)]
    cmp: RawCompressionCount,
    #[bits(4..=5, rw)]
    isr: RawInputSource,
    #[bits(6..=7, rw)]
    cgms: RawCopyProtection,

    // PC2
    #[bits(8..=10, rw)]
    insert_ch: RawAAUXInsertChannel,
    #[bits(11..=13, rw)]
    rec_mode: RawAAUXRecordingMode,
    #[bit(14, rw)]
    rec_end: bool,
    #[bit(15, rw)]
    rec_st: bool,

    // PC3
    #[bits(16..=22, rw)]
    speed: u7,
    #[bit(23, rw)]
    drf: RawDirection,

    // PC4
    #[bits(24..=30, rw)]
    genre_category: u7,
    #[bit(31, rw)]
    reserved: u1,
}

impl AAUXSourceControl {
    /// Returns the list of valid playback speeds that are recognized by
    /// [`AAUXSourceControl::playback_speed`].
    pub fn valid_playback_speeds() -> &'static HashSet<Ratio<u8>> {
        &VALID_PLAYBACK_SPEEDS
    }
}

impl super::PackData for AAUXSourceControl {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawAAUXSourceControl::new_with_raw_value(u32::from_le_bytes(*raw));
        Ok(Self {
            copy_protection: raw.cgms().into(),
            source_situation: raw.ss().into(),
            input_source: raw.isr().into(),
            compression_count: raw.cmp().into(),
            recording_start_point: !raw.rec_st(),
            recording_end_point: !raw.rec_end(),
            recording_mode: raw.rec_mode().into(),
            insert_channel: raw.insert_ch().into(),
            genre_category: if raw.genre_category() == u7::MAX {
                None
            } else {
                Some(raw.genre_category())
            },
            direction: raw.drf().into(),
            playback_speed: PLAYBACK_SPEED_BITS_TO_RATIO[usize::from(raw.speed().value())],
            reserved: raw.reserved(),
        })
    }
}

impl super::ValidPackDataTrait<AAUXSourceControl> for super::ValidPack<AAUXSourceControl> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        RawAAUXSourceControl::builder()
            .with_ss(self.source_situation.into())
            .with_cmp(self.compression_count.into())
            .with_isr(self.input_source.into())
            .with_cgms(self.copy_protection.into())
            .with_insert_ch(self.insert_channel.into())
            .with_rec_mode(self.recording_mode.into())
            .with_rec_end(!self.recording_end_point)
            .with_rec_st(!self.recording_start_point)
            .with_speed(PLAYBACK_RATIO_TO_SPEED_BITS[&self.playback_speed])
            .with_drf(self.direction.into())
            .with_genre_category(self.genre_category.unwrap_or(u7::MAX))
            .with_reserved(self.reserved)
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
