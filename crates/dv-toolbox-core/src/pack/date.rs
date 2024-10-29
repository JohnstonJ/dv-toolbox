use arbitrary_int::{u1, u2, u4, Number};
use bitbybit::{bitenum, bitfield};
use chrono::{Datelike, FixedOffset, NaiveDate, Weekday};
use garde::Validate;
use serde::{Deserialize, Serialize};
use snafu::{whatever, OptionExt, ResultExt};

#[cfg(test)]
mod tests;

/// Whether daylight saving time is in effect
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[bitenum(u1, exhaustive = true)]
pub enum DaylightSavingTime {
    /// Daylight saving time is in effect
    DaylightSavingTime = 0x0,

    /// Daylight saving time is not in effect
    Normal = 0x1,
}

/// AAUX or VAUX recording date
///
/// Indicates the date when audio or video data was recorded.
///
/// The pack is typically repeated many times throughout a frame.  The value is supposed to be the
/// same everywhere within the frame.
///
/// DV standards:
///
/// - AAUX recording date
///   - IEC 61834-4:1998 Section 8.3 - Rec Date (AAUX)
/// - VAUX recording date
///   - IEC 61834-4:1998 Section 9.3 - Rec Date (VAUX)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct RecordingDate {
    /// The date of recording.
    ///
    /// Note that the pack data only encodes a 2-digit year; we use 75 as the Y2K rollover
    /// threshold, similar to what
    /// [MediaInfoLib](https://github.com/MediaArea/MediaInfoLib/blob/abdbb218b07f6cc0d4504c863ac5b42ecfab6fc6/Source/MediaInfo/Multiple/File_DvDif_Analysis.cpp#L1225)
    /// chooses to do.
    #[garde(custom(check_date))]
    pub date: Option<NaiveDate>,

    /// The day of the week the recording took place.
    ///
    /// This is a value that is explicitly recorded on tape.  If present, the value needs to be in
    /// agreement with the date's actual weekday as returned by [`NaiveDate::weekday`].
    #[garde(custom(check_weekday(&self)))]
    pub weekday: Option<Weekday>,

    /// The time zone in which the recording took place.
    #[garde(custom(check_timezone))]
    #[serde(with = "timezone_serde")]
    pub timezone: Option<FixedOffset>,

    /// Whether daylight saving time was in effect when the recording took place.
    ///
    /// The field is required to be present if and only if [`RecordingDate::timezone`] is
    /// also present.
    #[garde(custom(check_dst(&self)))]
    pub daylight_saving_time: Option<DaylightSavingTime>,

    /// Reserved bits; should normally be set to 0x3.
    #[garde(skip)]
    pub reserved: u2,
}

/// Date packs are stored with a 2-digit year, so only a narrow range of years are allowed.
fn check_date(date: &Option<NaiveDate>, _ctx: &super::PackContext) -> garde::Result {
    match *date {
        Some(d) => {
            let year = d.year();
            match year {
                ..1975 => Err(garde::Error::new(format!(
                    "the year {year} is before the minimum allowed year 1975"
                ))),
                1975..2075 => Ok(()),
                2075.. => Err(garde::Error::new(format!(
                    "the year {year} is after the maximum allowed year 2074"
                ))),
            }
        }
        None => Ok(()),
    }
}

/// If a weekday is present, then a date must also be present, and the weekday must match the date's
/// weekday.
fn check_weekday(
    recording_date: &RecordingDate,
) -> impl FnOnce(&Option<Weekday>, &super::PackContext) -> garde::Result + '_ {
    |weekday, _ctx| match (recording_date.date, *weekday) {
        // No weekday specified
        (None, None) | (Some(_), None) => Ok(()),
        // Weekday specified without a date
        (None, Some(_)) => Err(garde::Error::new(
            "a weekday must not be provided if the date is otherwise absent.",
        )),
        // Both weekday and date is specified: does the weekday match the date's weekday?
        (Some(d), Some(w)) => {
            if d.weekday() == w {
                Ok(())
            } else {
                Err(garde::Error::new(format!(
                    "the weekday field value of {w} does not match the date \
                    weekday of {} which is {}",
                    d,
                    d.weekday()
                )))
            }
        }
    }
}

/// Only certain time zone offsets are supported by the date pack.  In particular, the offset must
/// be positive.  For example, New York is represented as +19 instead of -5.  It also has to be
/// 30 minute interval.
fn check_timezone(timezone: &Option<FixedOffset>, _ctx: &super::PackContext) -> garde::Result {
    match *timezone {
        Some(tz) => match tz.local_minus_utc() {
            offset @ ..0 => Err(garde::Error::new(format!(
                "the time zone offset of {offset} seconds must be a positive offset from GMT"
            ))),
            offset @ 0.. => {
                if offset % (30 * 60) == 0 {
                    Ok(())
                } else {
                    Err(garde::Error::new(format!(
                        "the time zone offset of {offset} seconds must be a multiple of 30 \
                        minutes, or 1800 seconds"
                    )))
                }
            }
        },
        None => Ok(()),
    }
}

/// The daylight_saving_time field may be set if and only if the [`RecordingDate::timezone`] field
/// is also present.
fn check_dst(
    recording_date: &RecordingDate,
) -> impl FnOnce(&Option<DaylightSavingTime>, &super::PackContext) -> garde::Result + '_ {
    |daylight_saving_time, _ctx| match (recording_date.timezone, *daylight_saving_time) {
        (None, None) | (Some(_), Some(_)) => Ok(()),
        // Timezone without daylight saving time
        (Some(_), None) => Err(garde::Error::new(
            "daylight saving time value must be specified if time zone is present",
        )),
        // Daylight saving time without timezone
        (None, Some(_)) => Err(garde::Error::new(
            "daylight saving time value must be not specified if time zone is absent",
        )),
    }
}

/// [`chrono::FixedOffset`] does not implement [`serde::Serialize`] or [`serde::Deserialize`], so
/// we have to provide our own implementation here.  We do so by transforming it to a signed integer
/// and serialize that instead.
mod timezone_serde {
    use chrono::FixedOffset;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub(crate) fn serialize<S>(
        timezone: &Option<FixedOffset>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        timezone.map(|tz| tz.local_minus_utc()).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<i32>::deserialize(deserializer)? {
            None => Ok(None),
            Some(offset) => match FixedOffset::east_opt(offset) {
                None => Err(de::Error::invalid_value(
                    de::Unexpected::Signed(offset.into()),
                    &"a time zone offset measured in seconds and less than 24 hours",
                )),
                Some(tz) => Ok(Some(tz)),
            },
        }
    }
}

#[bitfield(u32)]
struct RawRecordingDate {
    // PC1
    #[bits(0..=3, rw)]
    tz_hour_units: u4,
    #[bits(4..=5, rw)]
    tz_hour_tens: u2,
    #[bit(6, rw)]
    tm: u1,
    #[bit(7, rw)]
    ds: DaylightSavingTime,

    // PC2
    #[bits(8..=11, rw)]
    day_units: u4,
    #[bits(12..=13, rw)]
    day_tens: u2,
    #[bits(14..=15, rw)]
    reserved: u2,

    // PC3
    #[bits(16..=19, rw)]
    month_units: u4,
    #[bit(20, rw)]
    month_tens: u1,
    #[bits(21..=23, rw)]
    week: RawWeekday,

    // PC4
    #[bits(24..=27, rw)]
    year_units: u4,
    #[bits(28..=31, rw)]
    year_tens: u4,
}

#[derive(Debug, PartialEq, Eq)]
#[bitenum(u3, exhaustive = true)]
enum RawWeekday {
    Sunday = 0x0,
    Monday = 0x1,
    Tuesday = 0x2,
    Wednesday = 0x3,
    Thursday = 0x4,
    Friday = 0x5,
    Saturday = 0x6,
    NoInfo = 0x7,
}

impl super::PackData for RecordingDate {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawRecordingDate::new_with_raw_value(u32::from_le_bytes(*raw));

        // Convert binary-coded decimal to regular numbers
        let year = super::util::from_bcd_tens(raw.year_tens(), raw.year_units())
            .whatever_context("couldn't read the date's year")?
            // The year is a 2-digit year; convert it to a 4-digit year
            .map(|y| u16::from(y) + if y < 75 { 2000 } else { 1900 });
        let month = super::util::from_bcd_tens(raw.month_tens(), raw.month_units())
            .whatever_context("couldn't read the date's month")?;
        let day = super::util::from_bcd_tens(raw.day_tens(), raw.day_units())
            .whatever_context("couldn't read the date's day")?;
        let timezone = super::util::from_bcd_tens(raw.tz_hour_tens(), raw.tz_hour_units())
            .whatever_context("couldn't read the date's timezone")?
            .map(|tz_hour| {
                FixedOffset::east_opt(
                    i32::from(tz_hour) * 60 * 60 + i32::from(u8::from(!raw.tm())) * 30 * 60,
                )
                .with_whatever_context(|| format!("timezone hour {tz_hour} is out of range"))
            })
            .transpose()?;

        // Main date part must be fully present or fully absent; construct a date if found
        let date = match (year, month, day) {
            (None, None, None) => None,
            (Some(y), Some(m), Some(d)) => {
                Some(NaiveDate::from_ymd_opt(y.into(), m.into(), d.into()).with_whatever_context(
                    || format!("the date {y}-{m}-{d} is not a valid date"),
                )?)
            }
            _ => whatever!("year/month/day date fields must be fully present or fully absent"),
        };

        // Construct final output
        Ok(Self {
            date,
            weekday: match raw.week() {
                RawWeekday::Sunday => Some(Weekday::Sun),
                RawWeekday::Monday => Some(Weekday::Mon),
                RawWeekday::Tuesday => Some(Weekday::Tue),
                RawWeekday::Wednesday => Some(Weekday::Wed),
                RawWeekday::Thursday => Some(Weekday::Thu),
                RawWeekday::Friday => Some(Weekday::Fri),
                RawWeekday::Saturday => Some(Weekday::Sat),
                RawWeekday::NoInfo => None,
            },
            timezone,
            daylight_saving_time: timezone.map(|_| raw.ds()),
            reserved: raw.reserved(),
        })
    }
}

impl super::ValidPackDataTrait<RecordingDate> for super::ValidPack<RecordingDate> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        // The unwrapping is safe, because validation ensures that the timezone is a supported
        // value.  A panic should not actually happen.
        let tz_hours =
            self.timezone.map(|tz| u8::try_from(tz.local_minus_utc() / (60 * 60)).unwrap());
        let tz_thirty_minute = self
            .timezone
            .map(|tz| u8::try_from((tz.local_minus_utc() % (60 * 60)) / (30 * 60)).unwrap());
        RawRecordingDate::builder()
            .with_tz_hour_units(tz_hours.map_or(u4::MAX, |tz_hours| u4::new(tz_hours % 10)))
            .with_tz_hour_tens(tz_hours.map_or(u2::MAX, |tz_hours| u2::new(tz_hours / 10)))
            .with_tm(
                tz_thirty_minute.map_or(u1::MAX, |tz_thirty_minute| !u1::new(tz_thirty_minute)),
            )
            .with_ds(self.daylight_saving_time.unwrap_or(DaylightSavingTime::Normal))
            .with_day_units(
                self.date.map_or(u4::MAX, |d| u4::new(u8::try_from(d.day() % 10).unwrap())),
            )
            .with_day_tens(
                self.date.map_or(u2::MAX, |d| u2::new(u8::try_from(d.day() / 10).unwrap())),
            )
            .with_reserved(self.reserved)
            .with_month_units(
                self.date.map_or(u4::MAX, |d| u4::new(u8::try_from(d.month() % 10).unwrap())),
            )
            .with_month_tens(
                self.date.map_or(u1::MAX, |d| u1::new(u8::try_from(d.month() / 10).unwrap())),
            )
            .with_week(match self.weekday {
                Some(Weekday::Sun) => RawWeekday::Sunday,
                Some(Weekday::Mon) => RawWeekday::Monday,
                Some(Weekday::Tue) => RawWeekday::Tuesday,
                Some(Weekday::Wed) => RawWeekday::Wednesday,
                Some(Weekday::Thu) => RawWeekday::Thursday,
                Some(Weekday::Fri) => RawWeekday::Friday,
                Some(Weekday::Sat) => RawWeekday::Saturday,
                None => RawWeekday::NoInfo,
            })
            .with_year_units(
                self.date.map_or(u4::MAX, |d| u4::new(u8::try_from(d.year() % 10).unwrap())),
            )
            .with_year_tens(
                self.date.map_or(u4::MAX, |d| u4::new(u8::try_from((d.year() / 10) % 10).unwrap())),
            )
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
