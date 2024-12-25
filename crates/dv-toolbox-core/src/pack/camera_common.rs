//! Common data types shared between multiple camera packs.

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use arbitrary_int::u7;
use bitbybit::bitenum;
use serde::{Deserialize, Serialize};

super::util::required_enum! {
    /// Focus mode of the camera
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    pub enum FocusMode {
        /// Automatic focus
        Automatic = 0x0,

        /// Manual focus
        Manual = 0x1,
    }

    #[bitenum(u1, exhaustive = true)]
    pub(crate) enum RawFocusMode;
}

pub(crate) static FOCUS_POSITION_BITS_TO_CENTIMETERS: LazyLock<[Option<u16>; 128]> =
    LazyLock::new(|| {
        let mut positions = [None; 128];

        // Special defined focus positions
        positions[0x7F] = None; // no information

        // Focus positions follow a simple exponential formula
        for position_bits in 0x0_u16..=0x7E_u16 {
            let msb = position_bits >> 2;
            let lsb = position_bits & 0x03;
            let position = msb * 10_u16.checked_pow(lsb.into()).unwrap();
            positions[usize::from(position_bits)] = Some(position);
        }
        positions
    });

pub(crate) static FOCUS_POSITION_CENTIMETERS_TO_BITS: LazyLock<HashMap<Option<u16>, u7>> =
    LazyLock::new(|| {
        HashMap::<Option<u16>, u7>::from_iter(
            FOCUS_POSITION_BITS_TO_CENTIMETERS.iter().enumerate().rev().map(
                |(focus_bits, position)| (*position, u7::new(u8::try_from(focus_bits).unwrap())),
            ),
        )
    });

pub(crate) static VALID_FOCUS_POSITIONS: LazyLock<HashSet<u16>> = LazyLock::new(|| {
    HashSet::<u16>::from_iter(FOCUS_POSITION_CENTIMETERS_TO_BITS.clone().into_keys().flatten())
});

pub(crate) fn check_focus_position(
    focus_position: &Option<u16>,
    _ctx: &super::PackContext,
) -> garde::Result {
    match focus_position {
        Some(position) => VALID_FOCUS_POSITIONS.contains(position).then_some(()).ok_or_else(|| {
            garde::Error::new(format!(
                "focus position {position} not supported: only focus positions returned by the \
                    valid_focus_positions function are supported",
            ))
        }),
        None => Ok(()),
    }
}
