use garde::Valid;
use garde::Validate;
use itertools::ExactlyOneError;
use itertools::Itertools;
use snafu::{ResultExt, Snafu};

use crate::ffutil;
use crate::ioutil;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::media;
use num::Rational32;
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

#[cfg(test)]
mod tests;

/// Top-level metadata about a DV file.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Validate)]
pub struct Info {
    /// Size of the DV file in bytes.
    #[garde(skip)]
    pub file_size: u64,
    // Frame rate of the video stream.
    #[garde(custom(is_supported_video_frame_rate))]
    pub video_frame_rate: Rational32,
}

pub type ValidInfo = Valid<Info>;

fn is_supported_video_frame_rate(value: &Rational32, _context: &()) -> garde::Result {
    // Make sure we got exact NTSC or PAL/SECAM frame rate, which are currently the only
    // possible frame rates that we support and know about in this crate.
    match *value {
        v if v == Rational32::new(1, 2) || v == Rational32::new(1, 3) => Ok(()),
        _ => Err(garde::Error::new("video frame rate is not a supported NTSC/PAL/SECAM rate")),
    }
}

impl Info {
    /// Reads a limited amount of metadata from a DV file.
    ///
    /// The current position in the file will be ignored.  The function will always seek to the
    /// start.  The final position in the file is unspecified.
    pub fn read<R: io::Read + io::Seek + 'static>(reader: Rc<RefCell<R>>) -> InfoResult<Info> {
        // Get file size by seeking to end, then seek back to start
        let mut borrowed = reader.borrow_mut();
        let file_size = ioutil::retry_if_interrupted(|| borrowed.seek(io::SeekFrom::End(0)))
            .context(FileSizeSnafu)?;
        ioutil::retry_if_interrupted(|| borrowed.seek(io::SeekFrom::Start(0)))
            .context(FileSizeSnafu)?;
        std::mem::drop(borrowed); // so that open_seekable_input can use it

        // Open the file with FFmpeg
        let format_context =
            ffutil::open_seekable_input(reader, Some(&ffutil::find_input_format("dv").unwrap()))
                .context(OpenSnafu)?;

        // Get the one and only video stream
        let video_stream = format_context
            .streams()
            .filter(|s| s.parameters().medium() == media::Type::Video)
            .exactly_one()
            .context(VideoExactlyOneSnafu)?;

        //TODO
        panic!(
            "avg rate {:?}, rate {:?}, frames {:?}, duration {:?}, timebase {:?}, medium {:?}",
            format_context.stream(0).unwrap().avg_frame_rate(), // junk
            format_context.stream(0).unwrap().rate(),           // good
            format_context.stream(0).unwrap().frames(),         // junk
            format_context.stream(0).unwrap().duration(),       // great!
            format_context.stream(0).unwrap().time_base(),      // great! (use with duration)
            format_context.stream(0).unwrap().parameters().medium()
        );

        Ok(Info { file_size })
    }
}

/// Result type for calls related to reading Info structures.
pub type InfoResult<T, E = InfoError> = std::result::Result<T, E>;

/// Error type for calls related to reading Info structures.
#[derive(Debug, Snafu)]
pub struct InfoError(InnerInfoError);

#[derive(Debug, Snafu)]
enum InnerInfoError {
    #[snafu(display("Could not read file size by seeking within the file"))]
    FileSize { source: io::Error, backtrace: snafu::Backtrace },

    #[snafu(display("Could not open file using FFmpeg"))]
    Open { source: ffmpeg::Error, backtrace: snafu::Backtrace },

    #[snafu(display("Could not find exactly one video stream"))]
    VideoExactlyOne { source: ExactlyOneError<StreamIter>, backtrace: snafu::Backtrace },
}
