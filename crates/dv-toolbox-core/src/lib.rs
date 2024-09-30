//! This crate provides functions for working with and restoring videos in
//! [Digital Video (DV) format](https://en.wikipedia.org/wiki/DV_(video_format)).
//!
//! Currently supported formats:
//! - [IEC 61834-2](https://webstore.iec.ch/en/publication/5984): this format was recorded by many
//!   consumer camcorders, among other devices.

// NOTE: Dead code and unused imports are sometimes allowed while this crate is under development.
// Eventually, they should be removed.

#![allow(dead_code, unused_imports)]
mod ffutil;
mod ioutil;

#[cfg(test)]
mod testutil;
