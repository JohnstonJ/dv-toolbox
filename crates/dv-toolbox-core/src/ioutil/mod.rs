//! Contains general-purpose I/O utility functions.

use std::io;
use std::io::ErrorKind;

/// Retry function for as long as we are interrupted
pub fn retry_if_interrupted<F, O>(mut f: F) -> io::Result<O>
where
    F: FnMut() -> io::Result<O>,
{
    loop {
        match f() {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            result => break result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[googletest::test]
    fn test_retry_if_interrupted() {
        let mut call_count = 0;
        retry_if_interrupted(|| {
            call_count += 1;
            if call_count < 3 {
                Err(io::Error::new(ErrorKind::Interrupted, "interrupted"))
            } else {
                Ok(())
            }
        })
        .unwrap();

        expect_that!(call_count, eq(3));
    }
}
