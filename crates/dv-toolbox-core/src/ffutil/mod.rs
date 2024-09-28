//! Contains functions for wrapping FFmpeg.
//!
//! This fills in some of the gaps in functionality that [`ffmpeg_next`] does not provide.

mod avio_context;
mod format;
mod rational;

pub use avio_context::{open_seekable_input, open_seekable_output, CustomFormatContextWrapper};
pub use format::find_input_format;
pub use format::format_supports_codec;
pub use format::guess_output_format;
pub use rational::AVRationalConverter;
