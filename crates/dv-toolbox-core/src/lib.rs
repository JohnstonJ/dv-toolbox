//! This crate provides functions for working with and restoring videos in
//! [Digital Video (DV) format](https://en.wikipedia.org/wiki/DV_(video_format)).
//!
//! Currently supported formats:
//! - [IEC 61834-2](https://webstore.iec.ch/en/publication/5984): this format was recorded by many
//!   consumer camcorders, among other devices.
//!
//! **IMPORTANT**: Before calling any code in this crate, you must first initialize FFmpeg global
//! variables by calling the [`ffmpeg_next::init`] function.  This should be done once at the
//! beginning of your main function before starting any threads.

// TODO make ffutil and ioutil private
pub mod ffutil;
pub mod file;
pub mod ioutil;

#[cfg(test)]
mod testutil;
