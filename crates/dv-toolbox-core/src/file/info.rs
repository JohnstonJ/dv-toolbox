use std::{cell::RefCell, io, rc::Rc};

use garde::{Unvalidated, Valid, Validate};
use itertools::Itertools;
use num::{rational::Ratio, CheckedMul, ToPrimitive};
use rsmpeg::{avformat::AVInputFormat, avutil};
use snafu::{prelude::*, FromString};

use crate::{ffutil, ioutil};

#[cfg(test)]
mod tests;

/// DV system as defined in IEC 61834.
///
/// The choice of system determines the entire layout of a DV file, as well as how many parts of
/// the DV file are interpreted.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum System {
    /// IEC 61834-2: SD format for 525-60 system
    ///
    /// 525 lines (480 active lines) with frame frequency of 29.97 Hz
    ///
    /// Also defined in SMPTE 306M-2002
    ///
    /// Commonly known as: Standard Definition NTSC, 720x480 resolution
    Sys525_60,

    /// IEC 61834-2: SD format for 625-50 system
    ///
    /// 625 lines (576 active lines) with frame frequency of 25.00 Hz
    ///
    /// Also defined in SMPTE 306M-2002
    ///
    /// Commonly known as: Standard Definition PAL/SECAM, 720x576 resolution
    Sys625_50,
}

/// Top-level metadata about a DV file.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Validate)]
pub struct Info {
    /// Size of the DV file in bytes.
    #[garde(skip)]
    pub file_size: u64,

    /// Frame rate of the video stream, in frames per second.
    #[garde(custom(is_supported_video_frame_rate))]
    pub video_frame_rate: Ratio<u32>,

    /// Duration of the video stream, in seconds.
    ///
    /// Up to 96 bits may be used in the numerator, and 32 bits in the denominator.
    #[garde(custom(video_duration_validations(&self)))]
    pub video_duration: Ratio<u128>,

    /// Number of stereo audio streams, as detected by FFmpeg.
    ///
    /// The DV format supports a variety of channel layouts, including mono, combinations of mono
    /// and stereo sound, etc.  FFmpeg doesn't interpret the channel layout subcode data, and
    /// always assumes stereo, so that's what we'll do as well.
    //
    // NOTE: DVCPRO50 at https://archive.org/details/SMPTEColorBarsBadTracking has no audio...
    // So zero audio channels is apparently a thing.
    #[garde(range(min = 0, max = 2))]
    pub audio_stereo_stream_count: u8,

    /// Audio sample rate in Hz.
    ///
    /// The field is present if and only if there is at least one audio stream.
    ///
    /// The value must be one of the ones supported by the DV standard.  See:
    ///
    /// - IEC 61834-2:1998 Section 6.4.1 - Encoding mode (Audio)
    /// - IEC 61834-2:1998 Table 13 - Audio encoding mode in an audio block
    ///
    /// Currently this is limited to 32000, 44100, and 48000 Hz.
    #[garde(custom(audio_sample_rate_valid(&self)))]
    pub audio_sample_rate: Option<u32>,
}

impl Info {
    fn try_video_frame_count(&self) -> Result<u64, garde::Error> {
        // multiply: video_frame_rate * video_duration
        let (fnum, fden): (u32, u32) = self.video_frame_rate.into();
        // frame_count is up to 128 bits in the numerator, and 64 bits in the denominator.
        let frame_count = Ratio::<u128>::new(fnum.into(), fden.into()) * self.video_duration;
        if !frame_count.is_integer() {
            return Err(garde::Error::new(format!(
                "Total video frame count {frame_count} is not an integer; it resulted from \
                multiplying video frame rate {} by video duration {}",
                self.video_frame_rate, self.video_duration
            )));
        }
        frame_count.to_u64().ok_or_else(|| {
            garde::Error::new(format!("Frame count {frame_count} does not fit in a 64-bit integer"))
        })
    }

    fn try_video_frame_size(&self) -> Result<u32, garde::Error> {
        // divide: file_size / video_frame_count
        let video_frame_count = self.try_video_frame_count()?;
        let video_frame_size: u64 =
            self.file_size.checked_div(video_frame_count).ok_or_else(|| {
                garde::Error::new("Video frame count is zero, so cannot calculate the frame size")
            })?;
        if self.file_size % video_frame_count != 0 {
            return Err(garde::Error::new(format!(
                "File size {} is not evenly divisible by video frame count {}",
                self.file_size, video_frame_count
            )));
        }
        video_frame_size.try_into().map_err(|_| {
            garde::Error::new(format!(
                "Video frame size count {video_frame_size} does not fit in a 64-bit integer"
            ))
        })
    }

    /// Return info about the DV video frame: (channel count, DIF sequence/track count per channel)
    fn try_video_frame_info(&self) -> Result<(u8, u8), garde::Error> {
        #[allow(clippy::identity_op)]
        match self.try_video_frame_size()? {
            s if s == 1 * 10 * 150 * 80 => Ok((1, 10)),
            s if s == 1 * 12 * 150 * 80 => Ok((1, 12)),
            s if s == 2 * 10 * 150 * 80 => Ok((2, 10)),
            s if s == 2 * 12 * 150 * 80 => Ok((2, 12)),
            s => Err(garde::Error::new(format!("Unsupported frame size {s}"))),
        }
    }

    fn try_system(&self) -> Result<System, garde::Error> {
        match self.try_video_frame_info()? {
            (1, 10) => Ok(System::Sys525_60),
            (1, 12) => Ok(System::Sys625_50),
            (2, 10) => Ok(System::Sys525_60),
            (2, 12) => Ok(System::Sys625_50),
            (channel_count, dif_sequence_count) => Err(garde::Error::new(format!(
                "Unable to determine the DV system in use from channel count of \
                {channel_count} and DIF sequences per channel of {dif_sequence_count}"
            ))),
        }
    }
}

/// Top-level metadata about a DV file.  The metadata has been validated.
pub type ValidInfo = Valid<Info>;

/// Informational functions on a validated [`Info`] structure.
pub trait ValidInfoMethods {
    /// Reads a limited amount of metadata from a DV file.
    ///
    /// The current position in the file will be ignored.  The function will always seek to the
    /// start.  The final position in the file is unspecified.
    fn read<R: io::Read + io::Seek + 'static>(reader: Rc<RefCell<R>>) -> InfoResult<ValidInfo>;

    /// Total number of frames in the video stream.
    fn video_frame_count(&self) -> u64;

    /// Size of a single DV frame in bytes.  Every frame in a DV file is exactly the same size.
    fn video_frame_size(&self) -> u32;

    /// Number of channels on the videotape / in the DV file.  1 for 25 mbps, 2 for 50 mbps.
    fn video_frame_channel_count(&self) -> u8;

    /// Number of DIF sequences per frame per channel.  10 for NTSC (30 fps), 12 for PAL/SECAM
    /// (25 fps).
    fn video_frame_dif_sequence_count(&self) -> u8;

    /// The DV system that is in use within this file.
    fn system(&self) -> System;

    /// Ideal number of audio samples per video frame.
    ///
    /// This number is an ideal number.  In many cases, the number is not an integer.  If and
    /// only if the file has no audio streams, then None is returned.
    ///
    /// For files with correctly locked audio - e.g. recorded by professional equipment with
    /// locked audio, the number will be strictly followed.  For files with unlocked audio, such
    /// as most videos recorded with consumer equipment, the actual number can be expected to
    /// vary and drift away from this ideal/average number.  In the latter case, it is beneficial
    /// to resample the audio in the file so that it follows this ideal.
    ///
    /// Learn more about locked/unlocked audio at
    /// [Adam Wilt's website](https://www.adamwilt.com/DV-FAQ-tech.html#LockedAudio).
    ///
    /// For example, NTSC is at 30000/1001 video frame rate, and might have 32 kHz audio.
    /// Every 15 video frames will have 16016 audio samples; we can't have an integer
    /// number of audio samples for any fewer amount of video frames.  However, if the video
    /// was recorded with _unlocked_ audio, then every group of 15 video frames will sometimes
    /// have more or less than 16016 audio samples.
    fn ideal_audio_samples_per_frame(&self) -> Option<Ratio<u32>>;

    /// Compares the current info with another one to see if it is of a similar format.  The file
    /// size may vary, but everything else should be the same.
    ///
    /// Useful to help detect if the format changes mid-stream.
    fn check_similar(&self, other: &Self) -> Result<(), DissimilarError>;
}

impl ValidInfoMethods for ValidInfo {
    fn read<R: io::Read + io::Seek + 'static>(reader: Rc<RefCell<R>>) -> InfoResult<ValidInfo> {
        // Get file size by seeking to end, then seek back to start
        let mut borrowed = reader.borrow_mut();
        let seek_err = "Could not read file size by seeking within the file";
        let file_size = ioutil::retry_if_interrupted(|| borrowed.seek(io::SeekFrom::End(0)))
            .whatever_context(seek_err)?;
        ioutil::retry_if_interrupted(|| borrowed.seek(io::SeekFrom::Start(0)))
            .whatever_context(seek_err)?;
        std::mem::drop(borrowed); // so that open_seekable_input can use it

        // Open the file with FFmpeg
        let format_context =
            ffutil::open_seekable_input(reader, Some(&AVInputFormat::find(c"dv").unwrap()))
                .whatever_context("Could not open file using FFmpeg")?;

        // Get the one and only video stream
        let video_stream = format_context
            .streams()
            .iter()
            .filter(|s| s.codecpar().codec_type().is_video())
            .exactly_one()
            .ok()
            .whatever_context("Could not find exactly one video stream")?;

        // Get video information

        let video_frame_rate: Ratio<u32> = ffutil::AVRationalConverter(video_stream.r_frame_rate)
            .try_into_other_ratio()
            .whatever_context("Video stream was missing a frame rate, or it was invalid")?;

        let dur_err = "Video stream was missing a duration, or it was invalid";
        let video_duration = video_stream.duration;
        if video_duration == rsmpeg::ffi::AV_NOPTS_VALUE {
            return Err(FromString::without_source(dur_err.into()));
        }
        let video_duration: u128 = u128::try_from(video_duration).ok().whatever_context(dur_err)?;
        let video_time_base: Ratio<u128> = ffutil::AVRationalConverter(video_stream.time_base)
            .try_into_other_ratio()
            .ok()
            .whatever_context(dur_err)?;
        let video_duration =
            video_time_base.checked_mul(&video_duration.into()).whatever_context(dur_err)?;

        // Get audio information

        let audio_streams: Vec<_> = format_context
            .streams()
            .iter()
            .filter(|s| s.codecpar().codec_type().is_audio())
            .collect();
        let audio_stereo_stream_count: u8 = audio_streams.len().try_into().unwrap();
        let audio_sample_rate: Option<u32> = if audio_stereo_stream_count > 0 {
            Some(audio_streams[0].codecpar().sample_rate.try_into().unwrap())
        } else {
            None
        };
        // Make assertions that all audio streams are the same and are of a supported format
        for audio_stream in audio_streams {
            let codecpar = audio_stream.codecpar();
            ensure_whatever!(
                u32::try_from(codecpar.sample_rate).unwrap() == audio_sample_rate.unwrap(),
                "All audio streams must have the same sample rate"
            );
            ensure_whatever!(
                codecpar.format == rsmpeg::ffi::AV_SAMPLE_FMT_S16,
                "All audio streams must have 16-bit signed PCM samples"
            );
            ensure_whatever!(
                codecpar
                    .ch_layout()
                    .equal(&avutil::AVChannelLayout::from_string(c"stereo").unwrap())
                    .whatever_context("Error comparing channel layouts")?,
                "All audio streams must have a stereo channel layout"
            );
        }

        UnvalidatedInfo::new(Info {
            file_size,
            video_frame_rate,
            video_duration,
            audio_stereo_stream_count,
            audio_sample_rate,
        })
        .validate()
        .whatever_context("Validation failures on the video file metadata")
    }

    fn video_frame_count(&self) -> u64 {
        self.try_video_frame_count().unwrap()
    }

    fn video_frame_size(&self) -> u32 {
        self.try_video_frame_size().unwrap()
    }

    fn video_frame_channel_count(&self) -> u8 {
        self.try_video_frame_info().unwrap().0
    }

    fn video_frame_dif_sequence_count(&self) -> u8 {
        self.try_video_frame_info().unwrap().1
    }

    fn system(&self) -> System {
        self.try_system().unwrap()
    }

    fn ideal_audio_samples_per_frame(&self) -> Option<Ratio<u32>> {
        // Guaranteed not to panic for supported video frame rates and audio sample rates
        Some(Ratio::<u32>::from(self.audio_sample_rate?) / self.video_frame_rate)
    }

    fn check_similar(&self, other: &Self) -> Result<(), DissimilarError> {
        ensure_whatever!(
            self.video_frame_rate == other.video_frame_rate,
            "Video frame rate {} does not match {}",
            other.video_frame_rate,
            self.video_frame_rate
        );
        ensure_whatever!(
            self.video_frame_size() == other.video_frame_size(),
            "Video frame size {} does not match {}",
            other.video_frame_size(),
            self.video_frame_size()
        );
        ensure_whatever!(
            self.video_frame_channel_count() == other.video_frame_channel_count(),
            "Video frame channel count {} does not match {}",
            other.video_frame_channel_count(),
            self.video_frame_channel_count()
        );
        ensure_whatever!(
            self.video_frame_dif_sequence_count() == other.video_frame_dif_sequence_count(),
            "Video DIF sequence count {} does not match {}",
            other.video_frame_dif_sequence_count(),
            self.video_frame_dif_sequence_count()
        );
        ensure_whatever!(
            self.system() == other.system(),
            "DV system {:?} does not match {:?}",
            other.system(),
            self.system()
        );
        ensure_whatever!(
            self.audio_stereo_stream_count == other.audio_stereo_stream_count,
            "Audio stereo stream count {} does not match {}",
            other.audio_stereo_stream_count,
            self.audio_stereo_stream_count
        );
        ensure_whatever!(
            self.audio_sample_rate == other.audio_sample_rate,
            "Audio sample rate {:?} does not match {:?}",
            other.audio_sample_rate,
            self.audio_sample_rate
        );
        Ok(())
    }
}

/// Top-level metadata about a DV file.  The metadata has not been validated.
pub type UnvalidatedInfo = Unvalidated<Info>;

fn is_supported_video_frame_rate(value: &Ratio<u32>, _context: &()) -> garde::Result {
    // Make sure we got exact NTSC or PAL/SECAM frame rate, which are currently the only
    // possible frame rates that we support and know about in this crate.
    match *value {
        v if v == Ratio::<u32>::new(30000, 1001) || v == Ratio::<u32>::from(25) => Ok(()),
        v => Err(garde::Error::new(format!(
            "Video frame rate {v} is not a supported NTSC/PAL/SECAM rate"
        ))),
    }
}

// Struct verifications from https://github.com/jprochazk/garde/issues/61#issuecomment-2180323501

fn video_duration_validations<T>(info: &Info) -> impl FnOnce(&T, &()) -> garde::Result + '_ {
    move |_, _| {
        info.try_video_frame_count()
            .map(|_| ())
            .and(info.try_video_frame_size().map(|_| ()))
            .and(info.try_video_frame_info().map(|_| ()))
            .and(info.try_system().map(|_| ()))
    }
}

fn audio_sample_rate_valid(info: &Info) -> impl FnOnce(&Option<u32>, &()) -> garde::Result + '_ {
    move |sample_rate, _| {
        // Sample rate should be present if and only if there are some audio streams
        match (sample_rate, info.audio_stereo_stream_count) {
            (None, 0) => Ok(()),
            (None, 1..) => Err(garde::Error::new("Could not detect sample rate for audio streams")),
            (Some(_), 0) => Err(garde::Error::new(
                "Audio sample rate cannot be provided if there are no audio streams",
            )),
            (Some(_), 1..) => Ok(()),
        }
        // Sample rate must be one of the values that are supported by the DV standard
        .and(match sample_rate {
            None | Some(32_000 | 44_100 | 48_000) => Ok(()),
            Some(s) => Err(garde::Error::new(format!("Unsupported audio sample rate {s}"))),
        })
    }
}

/// Result type for calls related to reading Info structures.
pub type InfoResult<T, E = InfoError> = std::result::Result<T, E>;

/// Error type for calls related to reading Info structures.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum InfoError {
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
        backtrace: snafu::Backtrace,
    },
}

/// Error type to indicate why one file info is not similar to another.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum DissimilarError {
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
        backtrace: snafu::Backtrace,
    },
}
