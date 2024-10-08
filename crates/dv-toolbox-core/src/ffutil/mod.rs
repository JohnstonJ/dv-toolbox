//! Contains functions for wrapping FFmpeg.
//!
//! This fills in some of the gaps in functionality that [`rsmpeg`] does not provide.

mod avio_context;
mod format;
mod rational;

pub(crate) use avio_context::*;
#[allow(unused_imports)]
pub(crate) use format::*;
pub(crate) use rational::*;
