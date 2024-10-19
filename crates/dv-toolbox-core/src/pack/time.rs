use std::sync::LazyLock;

use arbitrary_int::{u2, u3, u4, Number};
use bitbybit::{bitenum, bitfield};
use garde::{Unvalidated, Validate};
use regex::Regex;
use serde::{de, de::Unexpected, Deserialize, Serialize};
use snafu::{whatever, OptionExt, ResultExt};

use crate::file::{System, ValidInfoMethods};

#[cfg(test)]
mod tests;

/// Contains a time address for a frame.
///
/// The timecode may have a required or optional frame/ number, depending on the generic parameter
/// `FrameType`.  Type aliases are defined to aid in the two valid types for `FrameType`:
/// - [`TimeValueWithRequiredFrame`]: the frame number is required
/// - [`TimeValueWithOptionalFrame`]: the frame number is optional
///
/// When a timecode value reaches 24 hours, it will wrap back to the 00 hour, similar to how normal
/// clock time works.  The maximum value of [`TimeValue::hour`] is 23.
///
/// When the structure is serialized to a string, the following string formats apply:
/// - `hh:mm:ss`: normal hours/minutes/seconds time notation, with no frame number.  (Only applies
///   to [`TimeValueWithOptionalFrame`].)
/// - `hh:mm:ss:ff`: timecode with zero-based frame number.  Drop frame mode does not apply: all
///   frame numbers are counted.
/// - `hh:mm:ss;ff`: timecode with zero-based frame number.  Drop frame mode is enabled.  (Only
///   applies to NTSC systems.)
///
/// See the [`TimeValue::drop_frame`] field for more information about drop frame.
///
/// General timecode standards:
///
/// - IEC 60461:2010 (entire standard) - Time and control code
/// - SMPTE 12M (entire standard) - Time and Control Code
#[derive(Debug, PartialEq, Eq, Clone, Copy, Validate)]
#[garde(context(super::PackContext))]
pub struct TimeValue<FrameType: FrameTypeTrait> {
    /// The hour of the timecode, in range `[0, 23]`.
    #[garde(range(min = 0, max = 23))]
    pub hour: u8,

    /// The minute of the timecode, in range `[0, 59]`.
    #[garde(range(min = 0, max = 59))]
    pub minute: u8,

    /// The second of the timecode, in range `[0, 59]`.
    #[garde(range(min = 0, max = 59))]
    pub second: u8,

    /// Whether drop frame mode is enabled.
    ///
    /// NTSC systems have a non-integer frame rate of approximately 29.970 Hz.  However, the
    /// [`TimeValue::frame`] field is an integer, and counts a full 30 frames per second.  In order
    /// to prevent the timecode value from drifting too far from the true clock time, drop frame
    /// is used to skip counting some frame numbers within the timecodes of a video.
    ///
    /// Drop frame works by skipping frames numbered 00 and 01 from the first second of every
    /// minute, except for the minutes which are evenly divisible by 10.  For example, suppose you
    /// have a frame with timecode `01:23:59;29`.  The following frame would have timecode
    /// `01:24:00;02`: frames 00 and 01 are skipped.  However, if a frame has timecode
    /// `01:29:59;29`, the following frame would have timecode `01:30:00;00`, because the minute
    /// is divisible by 10.
    #[garde(skip)]
    pub drop_frame: bool,

    /// The frame number of the timecode.  It may be optional or required, depending on the
    /// particular pack type:
    ///
    /// - [`TimeValueWithRequiredFrame`]: the frame number is required
    /// - [`TimeValueWithOptionalFrame`]: the frame number is optional
    ///
    /// The minimum value is generally 0.  However, special rules apply for NTSC systems when
    /// [`TimeValue::drop_frame`] is enabled.
    ///
    /// The maximum value will be 29 or 24, depending on the system in use (NTSC/PAL/SECAM).
    #[garde(custom(FrameType::check_frame_number(&self)))]
    pub frame: FrameType,
}

/// A time address for a frame where the frame number is required.
pub type TimeValueWithRequiredFrame = TimeValue<u8>;

/// A time address for a frame where the frame number is optional.
pub type TimeValueWithOptionalFrame = TimeValue<Option<u8>>;

/// Trait used for validation purposes of a [`TimeValue::frame`] value.  Not for public use outside
/// of this module, and no compatibility guarantees are made.
pub trait FrameTypeTrait
where
    Self: Sized,
{
    /// Return a custom validation function of the frame number for use with [`garde`].
    fn check_frame_number(
        time_value: &TimeValue<Self>,
    ) -> impl FnOnce(&Self, &super::PackContext) -> garde::Result + '_;
}

impl FrameTypeTrait for u8 {
    /// Validate the maximum frame number, which varies depending on the system in use.
    ///
    /// Also validates the minimum frame number when drop frame is in use to ensure that we aren't
    /// choosing a frame number that is supposed to be skipped.
    fn check_frame_number(
        time_value: &TimeValue<Self>,
    ) -> impl FnOnce(&Self, &super::PackContext) -> garde::Result + '_ {
        move |frame_number, ctx| {
            let system = ctx.file_info.system();
            match system {
                // NTSC: maximum frame number is 29.  Also check drop frame.
                System::Sys525_60 => {
                    if *frame_number > 29 {
                        return Err(garde::Error::new(format!(
                            "frame number {frame_number} is greater than 29, which is the \
                            maximum valid frame number for system {system}"
                        )));
                    }
                    // IEC 60461:2010 Section 4.2.3 - Drop frame - NTSC time compensated mode
                    // Frame numbers 0 and 1 are skipped at the start of each minute, except for
                    // minutes 00, 10, 20, 30, 40, and 50.  Make sure we weren't given a frame
                    // number that was supposed to be skipped.
                    if time_value.drop_frame
                        && time_value.minute % 10 > 0
                        && time_value.second == 0
                        && *frame_number < 2
                    {
                        return Err(garde::Error::new(format!(
                            "the drop frame flag was set, but a dropped frame number \
                            {frame_number} was provided"
                        )));
                    }
                    Ok(())
                }
                // PAL/SECAM: maximum frame number is 24.  Simple/straightforward validation.
                System::Sys625_50 => {
                    if *frame_number > 24 {
                        return Err(garde::Error::new(format!(
                            "frame number {frame_number} is greater than 24, which is the \
                            maximum valid frame number for system {system}"
                        )));
                    }
                    Ok(())
                }
            }
        }
    }
}

impl FrameTypeTrait for Option<u8> {
    /// Validate the frame number only if one is present.
    fn check_frame_number(
        time_value: &TimeValue<Self>,
    ) -> impl FnOnce(&Self, &super::PackContext) -> garde::Result + '_ {
        // Reuse the implementation from the TimeValue variation that has required frame numbers.
        move |frame_number, ctx| {
            frame_number.map_or(Ok(()), |f| {
                u8::check_frame_number(&TimeValue::<u8> {
                    hour: time_value.hour,
                    minute: time_value.minute,
                    second: time_value.second,
                    drop_frame: time_value.drop_frame,
                    frame: f,
                })(&f, ctx)
            })
        }
    }
}

impl Serialize for TimeValueWithOptionalFrame {
    /// Serialize the time value to a string.  The string format is defined in the documentation
    /// for [`TimeValue`].
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let time_str = match self.frame {
            None => format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second),
            Some(frame) => match self.drop_frame {
                true => {
                    format!("{:02}:{:02}:{:02};{:02}", self.hour, self.minute, self.second, frame)
                }
                false => {
                    format!("{:02}:{:02}:{:02}:{:02}", self.hour, self.minute, self.second, frame)
                }
            },
        };
        serializer.serialize_str(time_str.as_str())
    }
}

impl Serialize for TimeValueWithRequiredFrame {
    /// Serialize the time value to a string.  The string format is defined in the documentation
    /// for [`TimeValue`].
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Reuse the implementation from the version with optional frame numbers.
        TimeValueWithOptionalFrame {
            hour: self.hour,
            minute: self.minute,
            second: self.second,
            drop_frame: self.drop_frame,
            frame: Some(self.frame),
        }
        .serialize(serializer)
    }
}

struct TimeValueWithOptionalFrameVisitor;

static TIME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?P<hour>\d+):(?P<minute>\d+):(?P<second>\d+)((?P<frame_separator>[:;])(?P<frame>\d+))?$"
    ).unwrap()
});

impl<'de> de::Visitor<'de> for TimeValueWithOptionalFrameVisitor {
    type Value = TimeValueWithOptionalFrame;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a time with optional frame number")
    }

    /// Deserialize the time value from a string.  The string format is defined in the documentation
    /// for [`TimeValue`].
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let capture = TIME_RE
            .captures(v)
            .ok_or(E::invalid_value(Unexpected::Str(v), &"a time with optional frame number"))?;
        Ok(TimeValueWithOptionalFrame {
            hour: capture["hour"].parse().map_err(E::custom)?,
            minute: capture["minute"].parse().map_err(E::custom)?,
            second: capture["second"].parse().map_err(E::custom)?,
            drop_frame: match capture.name("frame_separator").map(|s| s.as_str()) {
                Some(";") => true,
                Some(":") => false,
                // If the frames are missing, we'll just set the DF bit since that's how I've
                // observed it happening in practice on a VAUX Rec Date pack from my camera.
                // This is also the value we'd want to set if the time is missing completely.
                None => true,
                // normally the regex should prevent us from getting here
                Some(_) => panic!("unexpected frame separator character"),
            },
            frame: capture
                .name("frame")
                .map(|s| s.as_str().parse().map_err(E::custom))
                .transpose()?,
        })
    }
}

impl<'de> Deserialize<'de> for TimeValueWithOptionalFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TimeValueWithOptionalFrameVisitor)
    }
}

impl<'de> Deserialize<'de> for TimeValueWithRequiredFrame {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // First deserialize to a time value with optional frame number.  Then convert the type
        // to one with a required frame number, and fail if the frame number was missing.
        let time_value = TimeValueWithOptionalFrame::deserialize(deserializer)?;
        Ok(TimeValueWithRequiredFrame {
            hour: time_value.hour,
            minute: time_value.minute,
            second: time_value.second,
            drop_frame: time_value.drop_frame,
            frame: time_value.frame.ok_or(de::Error::custom("input is missing a frame number"))?,
        })
    }
}

/// Indicates whether color frame identification was intentionally applied to the timecode by
/// the original source.
///
/// - IEC 60461:2010 Section 7.3.3 - Colour frame flag
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[bitenum(u1, exhaustive = true)]
pub enum ColorFrame {
    /// No relationship between color frame sequence and the time address.
    Unsynchronized = 0x0,

    /// Color frames are identified by the time address, as applied by the original video source.
    Synchronized = 0x1,
}

/// Used in linear time code (LTC) to ensure that the code word has an even number of 0 bits.
/// For vertical interval time code (VITC), this indicates the field flag.
///
/// - IEC 60461:2010 Section 8.2.6 - Biphase mark polarity correction
/// - IEC 60461:2010 Section 9.2.5 - Field mark flag
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[bitenum(u1, exhaustive = true)]
pub enum PolarityCorrection {
    /// Bit value of zero
    Even = 0x0,

    /// Bit value of one
    Odd = 0x1,
}

/// Indicates the contents of the associated binary group pack.
///
/// - IEC 60461:2010 Section 7.4.1 - Binary group flag assignments
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[bitenum(u3, exhaustive = true)]
pub enum BinaryGroupFlag {
    /// The time is not referenced to an external clock, and the contents of the binary group
    /// are unspecified.
    ///
    /// - IEC 60461:2010 Section 7.4.2 - Character set not specified and unspecified clock time
    TimeUnspecifiedGroupUnspecified = 0b000,

    /// The time is not referenced to an external clock, and the contents of the binary group
    /// contain an 8-bit ISO/IEC 646 or ISO/IEC 2022 character set.
    ///
    /// - IEC 60461:2010 Section 7.4.3 - Eight-bit character set and unspecified clock time
    TimeUnspecifiedGroup8BitCodes = 0b001,

    /// The time is not referenced to an external clock, and the contents of the binary group
    /// contain time zone information according to SMPTE 309M.
    ///
    /// - IEC 60461:2010 Section 7.4.4 - Date/time zone and unspecified clock time
    TimeUnspecifiedGroupDateTimeZone = 0b100,

    /// The time is not referenced to an external clock, and the contents of the binary group
    /// contain complex data formatted according to SMPTE 262M.
    ///
    /// - IEC 60461:2010 Section 7.4.5 - Page/line multiplex system and unspecified clock time
    TimeUnspecifiedGroupPageLine = 0b101,

    /// The time is referenced to an external clock, and the contents of the binary group
    /// are unspecified.
    ///
    /// - IEC 60461:2010 Section 7.4.6 - Clock time specified and unspecified character set
    TimeClockGroupUnspecified = 0b010,

    /// This combination is reserved for future use and should not be used.
    ///
    /// - IEC 60461:2010 Section 7.4.7 - Unassigned binary group usage and unassigned clock time
    TimeUnassignedGroupReserved = 0b011,

    /// The time is referenced to an external clock, and the contents of the binary group
    /// contain time zone information according to SMPTE 309M.
    ///
    /// - IEC 60461:2010 Section 7.4.8 - Date/time zone and clock time
    TimeClockGroupDateTimeZone = 0b110,

    /// The time is referenced to an external clock, and the contents of the binary group
    /// contain complex data formatted according to SMPTE 262M.
    ///
    /// - IEC 60461:2010 Section 7.4.9 - Specified clock time and page/line multiplex system
    TimeClockGroupPageLine = 0b111,
}

/// Indicates whethere there is a timecode discontinuity prior to where this recording was
/// started.
///
/// A discontinuity means that the timecode did not continuously increase one frame at a time for
/// all frames prior to this one.  For example, the timecode might change from `01:33:53;15` to
/// `00:00:00;00` at some location in the videotape.  In that example, the `00:00:00;00` frame
/// and all following frames would have a flag value of [`BlankFlag::Discontinuous`].
///
/// - IEC 61834-4:1998 Section 4.4 - Time Code (TITLE)
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[bitenum(u1, exhaustive = true)]
pub enum BlankFlag {
    /// A timecode discontinuity exists somewhere prior to the current tape position
    Discontinuous = 0x0,

    /// A timecode discontinuity does not exist somewhere prior to the current tape position
    Continuous = 0x1,
}

#[bitfield(u32)]
struct Raw525_60Timecode {
    // PC1
    #[bits(0..=3, rw)]
    frame_units: u4,
    #[bits(4..=5, rw)]
    frame_tens: u2,
    #[bit(6, rw)]
    df: bool,
    #[bit(7, rw)]
    cf: ColorFrame,

    // PC2
    #[bits(8..=11, rw)]
    second_units: u4,
    #[bits(12..=14, rw)]
    second_tens: u3,
    #[bit(15, rw)]
    pc: PolarityCorrection,

    // PC3
    #[bits(16..=19, rw)]
    minute_units: u4,
    #[bits(20..=22, rw)]
    minute_tens: u3,

    // PC4
    #[bits(24..=27, rw)]
    hour_units: u4,
    #[bits(28..=29, rw)]
    hour_tens: u2,

    #[bits([23, 30, 31], rw)]
    bgf: BinaryGroupFlag,
}

#[bitfield(u32)]
struct Raw625_50Timecode {
    // PC1
    #[bits(0..=3, rw)]
    frame_units: u4,
    #[bits(4..=5, rw)]
    frame_tens: u2,
    #[bit(6, rw)]
    df: bool,
    #[bit(7, rw)]
    cf: ColorFrame,

    // PC2
    #[bits(8..=11, rw)]
    second_units: u4,
    #[bits(12..=14, rw)]
    second_tens: u3,

    // PC3
    #[bits(16..=19, rw)]
    minute_units: u4,
    #[bits(20..=22, rw)]
    minute_tens: u3,

    // PC4
    #[bits(24..=27, rw)]
    hour_units: u4,
    #[bits(28..=29, rw)]
    hour_tens: u2,
    #[bit(31, rw)]
    pc: PolarityCorrection,

    #[bits([15, 30, 23], rw)]
    bgf: BinaryGroupFlag,
}

/// Title timecode, AAUX recording time, or VAUX recording time
///
/// See the [`TitleTimecode`] and [`RecordingTime`] type documentation for more details.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct Timecode<TimeType>
where
    TimeType: Validate<Context = super::PackContext>,
{
    /// Contains a time address for a frame.  The value is required for [`TitleTimecode`], and is
    /// optional for [`RecordingTime`].
    #[garde(dive)]
    pub time: TimeType,

    /// Indicates whether color frame identification was intentionally applied to the timecode by
    /// the original source.
    ///
    /// The field is only applicable when there is a subsequent binary group pack.  If this is not
    /// the case, the value should be set to binary 1, which is [`ColorFrame::Synchronized`].
    ///
    /// When the [`Timecode`] is inside a [`TitleTimecode`], the field occupies the same physical
    /// space as [`TitleTimecode::blank_flag`], so if you change this field, then you must also
    /// change that field as well to the same binary value.
    ///
    /// - IEC 60461:2010 Section 7.3.3 - Colour frame flag
    #[garde(skip)]
    pub color_frame: ColorFrame,

    /// Used in linear time code (LTC) to ensure that the code word has an even number of 0 bits.
    /// For vertical interval time code (VITC), this indicates the field flag.
    ///
    /// The field is only applicable when there is a subsequent binary group pack.  If this is not
    /// the case, the value should be set to binary 1, which is [`PolarityCorrection::Odd`].
    ///
    /// - IEC 60461:2010 Section 8.2.6 - Biphase mark polarity correction
    /// - IEC 60461:2010 Section 9.2.5 - Field mark flag
    #[garde(skip)]
    pub polarity_correction: PolarityCorrection,

    /// Indicates the contents of the associated binary group pack.
    ///
    /// The field is only applicable when there is a subsequent binary group pack.  If this is not
    /// the case, the value should be set to binary 111, which is
    /// [`BinaryGroupFlag::TimeClockGroupPageLine`].
    ///
    /// - IEC 60461:2010 Section 7.4.1 - Binary group flag assignments
    #[garde(skip)]
    pub binary_group_flag: BinaryGroupFlag,
}

impl super::PackData for Timecode<Option<TimeValueWithOptionalFrame>> {
    fn try_from_raw(
        raw: &super::RawPackData,
        ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        // The two systems have different binary layouts, but otherwise basically have the same
        // fields.  Transform them into a common structure for subsequent processing.
        let raw = match ctx.file_info.system() {
            System::Sys525_60 => Raw525_60Timecode::new_with_raw_value(u32::from_le_bytes(*raw)),
            System::Sys625_50 => {
                let raw = Raw625_50Timecode::new_with_raw_value(u32::from_le_bytes(*raw));
                Raw525_60Timecode::builder()
                    .with_frame_units(raw.frame_units())
                    .with_frame_tens(raw.frame_tens())
                    .with_df(raw.df())
                    .with_cf(raw.cf())
                    .with_second_units(raw.second_units())
                    .with_second_tens(raw.second_tens())
                    .with_pc(raw.pc())
                    .with_minute_units(raw.minute_units())
                    .with_minute_tens(raw.minute_tens())
                    .with_hour_units(raw.hour_units())
                    .with_hour_tens(raw.hour_tens())
                    .with_bgf(raw.bgf())
                    .build()
            }
        };

        // Convert binary-coded decimal to regular numbers
        let hour = super::util::from_bcd_tens(raw.hour_tens(), raw.hour_units())
            .whatever_context("couldn't read the time's hour")?;
        let minute = super::util::from_bcd_tens(raw.minute_tens(), raw.minute_units())
            .whatever_context("couldn't read the time's minute")?;
        let second = super::util::from_bcd_tens(raw.second_tens(), raw.second_units())
            .whatever_context("couldn't read the time's second")?;
        let frame = super::util::from_bcd_tens(raw.frame_tens(), raw.frame_units())
            .whatever_context("couldn't read the time's frame number")?;

        // Main time part must be fully present or fully absent
        let time_absent = hour.is_none() || minute.is_none() || second.is_none();
        let time_present = hour.is_some() || minute.is_some() || second.is_some();
        if time_absent == time_present {
            whatever!("hour/minute/second time fields must be fully present or fully absent");
        }
        // Don't allow specifying frames if there's no other time
        if time_absent && frame.is_some() {
            whatever!("frame number cannot be given if the rest of the time is missing");
        }

        // Construct final output
        Ok(Self {
            time: if time_present {
                Some(TimeValue {
                    hour: hour.unwrap(),
                    minute: minute.unwrap(),
                    second: second.unwrap(),
                    drop_frame: raw.df(),
                    frame,
                })
            } else {
                None
            },

            color_frame: raw.cf(),
            polarity_correction: raw.pc(),
            binary_group_flag: raw.bgf(),
        })
    }
}

impl super::PackData for Timecode<TimeValueWithRequiredFrame> {
    fn try_from_raw(
        raw: &super::RawPackData,
        ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let parsed = Timecode::<Option<TimeValueWithOptionalFrame>>::try_from_raw(raw, ctx)?;
        let time = parsed.time.whatever_context("required timecode value is missing")?;
        let frame = time.frame.whatever_context("required frame number is missing")?;
        Ok(Self {
            time: TimeValue {
                hour: time.hour,
                minute: time.minute,
                second: time.second,
                drop_frame: time.drop_frame,
                frame,
            },
            color_frame: parsed.color_frame,
            polarity_correction: parsed.polarity_correction,
            binary_group_flag: parsed.binary_group_flag,
        })
    }
}

impl super::ValidPackDataTrait<Timecode<Option<TimeValueWithOptionalFrame>>>
    for super::ValidPack<Timecode<Option<TimeValueWithOptionalFrame>>>
{
    fn to_raw(&self, ctx: &super::PackContext) -> super::RawPackData {
        let raw = Raw525_60Timecode::builder()
            .with_frame_units(self.time.and_then(|t| t.frame).map_or(u4::MAX, |f| u4::new(f % 10)))
            .with_frame_tens(self.time.and_then(|t| t.frame).map_or(u2::MAX, |f| u2::new(f / 10)))
            .with_df(self.time.map(|t| t.drop_frame).unwrap_or(true))
            .with_cf(self.color_frame)
            .with_second_units(self.time.map_or(u4::MAX, |t| u4::new(t.second % 10)))
            .with_second_tens(self.time.map_or(u3::MAX, |t| u3::new(t.second / 10)))
            .with_pc(self.polarity_correction)
            .with_minute_units(self.time.map_or(u4::MAX, |t| u4::new(t.minute % 10)))
            .with_minute_tens(self.time.map_or(u3::MAX, |t| u3::new(t.minute / 10)))
            .with_hour_units(self.time.map_or(u4::MAX, |t| u4::new(t.hour % 10)))
            .with_hour_tens(self.time.map_or(u2::MAX, |t| u2::new(t.hour / 10)))
            .with_bgf(self.binary_group_flag)
            .build();
        match ctx.file_info.system() {
            System::Sys525_60 => raw.raw_value().to_le_bytes(),
            System::Sys625_50 => Raw625_50Timecode::builder()
                .with_frame_units(raw.frame_units())
                .with_frame_tens(raw.frame_tens())
                .with_df(raw.df())
                .with_cf(raw.cf())
                .with_second_units(raw.second_units())
                .with_second_tens(raw.second_tens())
                .with_minute_units(raw.minute_units())
                .with_minute_tens(raw.minute_tens())
                .with_hour_units(raw.hour_units())
                .with_hour_tens(raw.hour_tens())
                .with_pc(raw.pc())
                .with_bgf(raw.bgf())
                .build()
                .raw_value()
                .to_le_bytes(),
        }
    }
}

impl super::ValidPackDataTrait<Timecode<TimeValueWithRequiredFrame>>
    for super::ValidPack<Timecode<TimeValueWithRequiredFrame>>
{
    fn to_raw(&self, ctx: &super::PackContext) -> super::RawPackData {
        super::ValidPack(
            Unvalidated::<Timecode<Option<TimeValueWithOptionalFrame>>>::new(Timecode::<
                Option<TimeValueWithOptionalFrame>,
            > {
                time: Some(TimeValueWithOptionalFrame {
                    hour: self.time.hour,
                    minute: self.time.minute,
                    second: self.time.second,
                    drop_frame: self.time.drop_frame,
                    frame: Some(self.time.frame),
                }),
                color_frame: self.color_frame,
                polarity_correction: self.polarity_correction,
                binary_group_flag: self.binary_group_flag,
            })
            .validate_with(ctx)
            .unwrap(),
        )
        .to_raw(ctx)
    }
}

/// Title timecode
///
/// The timecode indicates the elapsed time at the tape position of this pack.
///
/// The pack is typically repeated many times throughout a frame.  The value is supposed to be the
/// same everywhere within the frame.  However, if the tape contains errors, it might not be, and
/// a few packs within a frame may even contain values from neighboring frames.
///
/// IEC 61834-4 defines two modes of this pack.  If there's no associated
/// [`super::Pack::TitleBinaryGroup`] pack, then a simplified timecode structure is used.  In this
/// simplified structure, only the [`TitleTimecode::blank_flag`] and [`Timecode::time`] fields are
/// used.  The latter will also always be in drop frame format.  All other fields are to have
/// values where every bit has been set to 1.
///
/// Alternatively, when the [`super::Pack::TitleBinaryGroup`] pack is recorded, then a more advanced
/// pack format is used.  The format is based on IEC 60461.  In this format, all fields within the
/// nested [`TitleTimecode::timecode`] field apply, and the [`TitleTimecode::blank_flag`] is not
/// applicable.
///
/// Note that [`TitleTimecode::blank_flag`] and [`Timecode::color_frame`] both occupy the same
/// physical space in the pack.  If you change one value, then you must change the other value as
/// well (even if it's not applicable, per the previous two paragraphs).
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 4.4 - Time Code (TITLE)
/// - SMPTE 306M-2002 Section 9.2.1 - Time code pack (TC)
///
/// General timecode standards:
///
/// - IEC 60461:2010 (entire standard) - Time and control code
/// - SMPTE 12M (entire standard) - Time and Control Code
#[derive(Debug, PartialEq, Eq, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct TitleTimecode {
    /// Contains most of the title timecode
    #[serde(flatten)]
    #[garde(dive)]
    pub timecode: Timecode<TimeValueWithRequiredFrame>,

    /// Indicates whethere there is a timecode discontinuity prior to where this recording was
    /// started.
    ///
    /// The field is only applicable for consumer digital VCR when there is not a
    /// [`super::Pack::TitleBinaryGroup`].  It occupies the same physical space as
    /// [`Timecode::color_frame`], so if you change this field, then you must also change that
    /// field as well to the same binary value.
    #[garde(custom(check_blank_flag(&self)))]
    pub blank_flag: BlankFlag,
}

/// Enforces that [`TitleTimecode::blank_flag`] and [`Timecode::color_frame`] both have the same
/// binary value.  Different values would be ambiguous when serializing the structure to binary.
fn check_blank_flag(
    title_timecode: &TitleTimecode,
) -> impl FnOnce(&BlankFlag, &super::PackContext) -> garde::Result + '_ {
    move |blank_flag, _| {
        // These two fields physically overlap for different use cases: IEC 61834-4 shows two
        // different possible binary layouts that could be in use.
        if *blank_flag as u8 != title_timecode.timecode.color_frame as u8 {
            Err(garde::Error::new(format!(
                "Blank flag integer value of {} must be equal to the color frame flag integer \
                value of {} because they occupy the same physical bit positions on the tape.  \
                Change one value to match the other.",
                *blank_flag as u8, title_timecode.timecode.color_frame as u8
            )))
        } else {
            Ok(())
        }
    }
}

impl super::PackData for TitleTimecode {
    fn try_from_raw(
        raw: &super::RawPackData,
        ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        Ok(TitleTimecode {
            timecode: Timecode::<TimeValueWithRequiredFrame>::try_from_raw(raw, ctx)?,
            blank_flag: if (raw[0] & 0x80) >> 7 == 1 {
                BlankFlag::Continuous
            } else {
                BlankFlag::Discontinuous
            },
        })
    }
}

impl super::ValidPackDataTrait<TitleTimecode> for super::ValidPack<TitleTimecode> {
    fn to_raw(&self, ctx: &super::PackContext) -> super::RawPackData {
        // NOTE: We don't need to worry about writing blank_flag.  It occupies the same physical
        // space as color_frame, and our check_blank_flag validation function asserts that
        // color_frame has the same integer value as blank_flag.  Therefore, when Timecode writes
        // color_frame, we can be sure the correct value of blank_flag was also written out.
        super::ValidPack(
            Unvalidated::<Timecode<TimeValueWithRequiredFrame>>::new(self.timecode)
                .validate_with(ctx)
                .unwrap(),
        )
        .to_raw(ctx)
    }
}

/// AAUX or VAUX recording time
///
/// Indicates the time when audio or video data was recorded.
///
/// The pack is typically repeated many times throughout a frame.  The value is supposed to be the
/// same everywhere within the frame.
///
/// IEC 61834-4 defines two modes of this pack.  If there's no associated
/// [`super::Pack::AAUXBinaryGroup`] or [`super::Pack::VAUXBinaryGroup`] pack, then a simplified
/// timecode structure is used.  In this simplified structure, only the [`Timecode::time`] field
/// is used.  It will always be in drop frame format.  All other fields are to have values where
/// every bit has been set to 1.
///
/// Alternatively, when the [`super::Pack::AAUXBinaryGroup`] or [`super::Pack::VAUXBinaryGroup`]
/// pack is recorded, then a more advanced pack format is used.  The format is based on IEC 60461.
/// In this format, all fields within the [`Timecode`] structure apply.
///
/// DV standards:
///
/// - AAUX recording time
///   - IEC 61834-4:1998 Section 8.4 - Rec Time (AAUX)
/// - VAUX recording time
///   - IEC 61834-4:1998 Section 9.4 - Rec Time (VAUX)
///
/// General timecode standards:
///
/// - IEC 60461:2010 (entire standard) - Time and control code
/// - SMPTE 12M (entire standard) - Time and Control Code
pub type RecordingTime = Timecode<Option<TimeValueWithOptionalFrame>>;
