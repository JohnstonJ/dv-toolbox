use arbitrary_int::{u1, u2, u3, u7, Number};
use bitbybit::{bitenum, bitfield};
use garde::Validate;
use serde::{Deserialize, Serialize};

use super::{RawCompressionCount, RawCopyProtection, RawInputSource, RawSourceSituation};

#[cfg(test)]
mod tests;

super::util::required_enum! {
    /// Indicates the recording mode of the video.
    ///
    /// New video content can be dubbed onto existing audio at a later time.  This flag is
    /// supposed to indicate whether that has happened.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum VAUXRecordingMode {
        /// All video was recorded at the same time as the audio.
        Original = 0x0,

        /// The video was updated with new content, while the audio block channels were left alone.
        Insert = 0x2,

        InvalidRecording = 0x3,

        Reserved = 0x1,
    }

    #[bitenum(u2, exhaustive = true)]
    enum RawVAUXRecordingMode;
}

super::util::required_enum! {
    /// Indicates whether both fields are output in order or only one of them is output twice
    /// during one frame period.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum FrameField {
        /// Only one of two fields is output twice
        OnlyOne = 0x0,

        /// Both fields are output in order
        Both = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawFrameField;
}

super::util::required_enum! {
    /// Indicates whether the picture of the current frame is the same as the previous frame.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum FrameChange {
        /// The current frame has the same picture as the previous frame.
        SameAsPrevious = 0x0,

        /// The current frame has a new picture that is different from the previous frame.
        DifferentFromPrevious = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawFrameChange;
}

super::util::required_enum! {
    /// Indicates the time difference between the two fields within a frame.
    ///
    /// The value shall be the same for at least three consecutive frames.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum StillFieldPicture {
        /// No time elapsed between fields in a frame
        NoGap = 0x0,

        /// 1001/60 (NTSC) or 1/50 (PAL/SECAM) seconds elapsed between fields
        HalfFrameTime = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    enum RawStillFieldPicture;
}

/// Contains some metadata about the video stream.
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 9.2 - Source Control (VAUX)
/// - SMPTE 306M-2002 Section 8.9.2 - VAUX source control pack (VSC)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct VAUXSourceControl {
    // Display format / aspect ratio
    //
    /// Broadcast system type
    ///
    /// Refer to the table in IEC 61834-4 for the full meaning of this and how it relates to the
    /// [`VAUXSourceControl::display_mode`] field.
    ///
    /// - `0`:  Type 0, see IEC 61880, EIA-608
    /// - `1`:  Type 1, see prETS 300 294
    /// - Other values:  reserved
    #[garde(skip)]
    pub broadcast_system: u2,

    /// Display select mode
    ///
    /// Refer to the table in IEC 61834-4 for the full meaning of this and how it relates to the
    /// [`VAUXSourceControl::broadcast_system`] field.
    #[garde(skip)]
    pub display_mode: u3,

    // Frame structure
    //
    /// Indicates whether both fields are output in order or only one of them is output twice
    /// during one frame period.
    #[garde(skip)]
    pub frame_field: FrameField,

    /// Indicates which field to output during field 1 period.  The value may be `1` or `2`, for
    /// field 1 or field 2, respectively.
    #[garde(range(min = 1, max = 2))]
    first_second: u8,

    /// Indicates whether the picture of the current frame is the same as the previous frame.
    #[garde(skip)]
    frame_change: FrameChange,

    /// Indicates whether the data of two fields within one frame are interlaced or non-interlaced.
    #[garde(skip)]
    interlaced: bool,

    /// Indicates the time difference between the two fields within a frame.
    ///
    /// The value shall be the same for at least three consecutive frames.
    #[garde(skip)]
    still_field_picture: StillFieldPicture,

    /// Indicates that the frame is a still camera picture.
    ///
    /// Still camera pictures must have five consecutive frames of the same picture.  Tape decks
    /// might automatically pause tape travel at this position.
    #[garde(skip)]
    still_camera_picture: bool,

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
    /// The value should be repeated for a full second's worth of frames: 30 frames or 25 frames,
    /// depending on the system.
    #[garde(skip)]
    pub recording_start_point: bool,

    /// Indicates the recording mode of the video.
    #[garde(skip)]
    pub recording_mode: VAUXRecordingMode,

    /// Defines the genre or category of the video source.
    ///
    /// The value corresponds to a massive enumeration of dozens of TV genres.  See
    /// IEC 61834-4:1998 Section 3.3 - Timer Activation Date (CONTROL) for the full list.
    #[garde(custom(super::check_genre_category))]
    pub genre_category: Option<u7>,

    /// Reserved bits; should normally be set to 0x7.
    #[garde(skip)]
    pub reserved: u3,
}

#[bitfield(u32)]
struct RawVAUXSourceControl {
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
    disp: u3,
    #[bits([11, 14, 31], rw)] // also part of PC4
    reserved: u3,
    #[bits(12..=13, rw)]
    rec_mode: RawVAUXRecordingMode,
    #[bit(15, rw)]
    rec_st: bool,

    // PC3
    #[bits(16..=17, rw)]
    bcsys: u2,
    #[bit(18, rw)]
    sc: bool,
    #[bit(19, rw)]
    st: RawStillFieldPicture,
    #[bit(20, rw)]
    il: bool,
    #[bit(21, rw)]
    fc: RawFrameChange,
    #[bit(22, rw)]
    fs: u1,
    #[bit(23, rw)]
    ff: RawFrameField,

    // PC4
    #[bits(24..=30, rw)]
    genre_category: u7,
}

impl super::PackData for VAUXSourceControl {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawVAUXSourceControl::new_with_raw_value(u32::from_le_bytes(*raw));
        Ok(Self {
            broadcast_system: raw.bcsys(),
            display_mode: raw.disp(),
            frame_field: raw.ff().into(),
            first_second: match raw.fs().value() {
                0x0 => 2,
                0x1 => 1,
                _ => panic!("code was supposed to be unreachable"),
            },
            frame_change: raw.fc().into(),
            interlaced: raw.il(),
            still_field_picture: raw.st().into(),
            still_camera_picture: !raw.sc(),
            copy_protection: raw.cgms().into(),
            source_situation: raw.ss().into(),
            input_source: raw.isr().into(),
            compression_count: raw.cmp().into(),
            recording_start_point: !raw.rec_st(),
            recording_mode: raw.rec_mode().into(),
            genre_category: if raw.genre_category() == u7::MAX {
                None
            } else {
                Some(raw.genre_category())
            },
            reserved: raw.reserved(),
        })
    }
}

impl super::ValidPackDataTrait<VAUXSourceControl> for super::ValidPack<VAUXSourceControl> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        // the panics in this function should not actually happen because the structure is validated
        RawVAUXSourceControl::builder()
            .with_ss(self.source_situation.into())
            .with_cmp(self.compression_count.into())
            .with_isr(self.input_source.into())
            .with_cgms(self.copy_protection.into())
            .with_disp(self.display_mode)
            .with_reserved(self.reserved)
            .with_rec_mode(self.recording_mode.into())
            .with_rec_st(!self.recording_start_point)
            .with_bcsys(self.broadcast_system)
            .with_sc(!self.still_camera_picture)
            .with_st(self.still_field_picture.into())
            .with_il(self.interlaced)
            .with_fc(self.frame_change.into())
            .with_fs(u1::new(match self.first_second {
                2 => 0x0,
                1 => 0x1,
                _ => panic!("code was suppposed to be unreachable in validated structure"),
            }))
            .with_ff(self.frame_field.into())
            .with_genre_category(self.genre_category.unwrap_or(u7::MAX))
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
