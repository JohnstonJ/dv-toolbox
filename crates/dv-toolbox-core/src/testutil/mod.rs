use std::path::PathBuf;

/// Directory containing test-related data files.
pub(crate) fn test_resource(path: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "resources/test", path].iter().collect()
}
