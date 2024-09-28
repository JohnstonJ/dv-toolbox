use ffmpeg_next as ffmpeg;
use std::{path::PathBuf, sync::Once};

static INIT_FFMPEG: Once = Once::new();

/// Initialize FFmpeg only once.
///
/// Tests run in parallel, so we need to call this at the start of any tests that work with
/// FFmpeg to ensure no race conditions / undefined behavior.
pub(crate) fn init_ffmpeg() {
    INIT_FFMPEG.call_once(|| ffmpeg::init().unwrap());
}

/// Directory containing test-related data files.
pub(crate) fn test_resource(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources/test", path].iter().collect()
}
