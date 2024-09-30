use std::os::raw;

use rsmpeg::{avcodec::AVCodec, avformat::AVOutputFormat};

#[cfg(test)]
mod tests;

/// Standards compliance level of a codec in a container.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Compliance {
    /// Strictly conform to an older more strict version of the spec or reference software.
    VeryStrict,

    /// Strictly conform to all the things in the spec no matter what consequences.
    Strict,

    /// Normal compliance level.
    Normal,

    /// Allow unofficial extensions.
    Unofficial,

    /// Allow nonstandardized experimental things.
    Experimental,
}

impl From<Compliance> for raw::c_int {
    fn from(value: Compliance) -> raw::c_int {
        match value {
            Compliance::VeryStrict => rsmpeg::ffi::FF_COMPLIANCE_VERY_STRICT.try_into().unwrap(),
            Compliance::Strict => rsmpeg::ffi::FF_COMPLIANCE_STRICT.try_into().unwrap(),
            Compliance::Normal => rsmpeg::ffi::FF_COMPLIANCE_NORMAL.try_into().unwrap(),
            Compliance::Unofficial => rsmpeg::ffi::FF_COMPLIANCE_UNOFFICIAL,
            Compliance::Experimental => rsmpeg::ffi::FF_COMPLIANCE_EXPERIMENTAL,
        }
    }
}

/// Test if an output container format can store a codec.
pub(crate) fn format_supports_codec(
    format: &AVOutputFormat,
    codec: &AVCodec,
    compliance: Compliance,
) -> rsmpeg::error::Result<bool> {
    match unsafe { rsmpeg::ffi::avformat_query_codec(format.as_ptr(), codec.id, compliance.into()) }
    {
        rsmpeg::ffi::AVERROR_PATCHWELCOME => Ok(false),
        err @ ..0 => Err(err.into()),
        0 => Ok(false),
        1.. => Ok(true),
    }
}
