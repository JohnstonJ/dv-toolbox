use std::ffi;
use std::ptr;

use ffmpeg_next as ffmpeg;
use ffmpeg_next::codec;
use ffmpeg_next::format::format;

#[cfg(test)]
mod tests;

/// Locates an input container file format given its short name.
pub fn find_input_format(short_name: &str) -> Option<format::Input> {
    let short_name = ffi::CString::new(short_name).ok()?;
    let fmt_ptr = unsafe { ffmpeg::ffi::av_find_input_format(short_name.as_ptr()) };
    let fmt_ref = unsafe { fmt_ptr.cast_mut().as_mut() }?;
    Some(unsafe { format::Input::wrap(fmt_ref) })
}

/// Guess an output container file format given some possibilities.
pub fn guess_output_format(
    short_name: Option<&str>,
    filename: Option<&str>,
    mime_type: Option<&str>,
) -> Option<format::Output> {
    // silently convert strings with null to None
    let cvt_str = |s: Option<&str>| s.map(|v| ffi::CString::new(v).ok()).flatten();
    // map &Option<CString> to a pointer that may be null
    let map_ptr = |s: &Option<ffi::CString>| s.as_deref().map_or(ptr::null(), |v| v.as_ptr());

    let short_name: Option<ffi::CString> = cvt_str(short_name);
    let filename: Option<ffi::CString> = cvt_str(filename);
    let mime_type: Option<ffi::CString> = cvt_str(mime_type);
    let fmt_ptr = unsafe {
        ffmpeg::ffi::av_guess_format(map_ptr(&short_name), map_ptr(&filename), map_ptr(&mime_type))
    };
    let fmt_ref = unsafe { fmt_ptr.cast_mut().as_mut() }?;
    Some(unsafe { format::Output::wrap(fmt_ref) })
}

/// Test if an output container format can store a codec.
pub fn format_supports_codec(
    format: &format::Output,
    codec: &ffmpeg::Codec,
    compliance: codec::Compliance,
) -> Result<bool, ffmpeg::Error> {
    match unsafe {
        ffmpeg::ffi::avformat_query_codec(format.as_ptr(), codec.id().into(), compliance.into())
    } {
        ffmpeg::ffi::AVERROR_PATCHWELCOME => Ok(false),
        err @ ..0 => Err(ffmpeg::Error::from(err)),
        0 => Ok(false),
        1.. => Ok(true),
    }
}
