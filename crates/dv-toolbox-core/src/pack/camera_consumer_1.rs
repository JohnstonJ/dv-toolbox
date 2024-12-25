use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use arbitrary_int::{u2, u4, u6, u7, Number};
use bitbybit::{bitenum, bitfield};
use garde::Validate;
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use super::RawFocusMode;

#[cfg(test)]
mod tests;

super::util::optional_enum! {
    /// Auto exposure mode of the consumer camera
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum AutoExposureMode {
        /// Full automatic mode
        FullAutomatic = 0x0,

        /// Gain priority mode
        GainPriority = 0x1,

        /// Shutter priority mode
        ShutterPriority = 0x2,

        /// Iris priority mode
        IrisPriority = 0x3,

        /// Manual mode
        Manual = 0x4,

        Reserved5 = 0x5,
        Reserved6 = 0x6,
        Reserved7 = 0x7,
        Reserved8 = 0x8,
        Reserved9 = 0x9,
        Reserved10 = 0xA,
        Reserved11 = 0xB,
        Reserved12 = 0xC,
        Reserved13 = 0xD,
        Reserved14 = 0xE,
    }

    #[bitenum(u4, exhaustive = true)]
    pub(crate) enum RawAutoExposureMode {
        NoInfo = 0xF,
    }
}

super::util::optional_enum! {
    /// White balance mode of the consumer camera
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum WhiteBalanceMode {
        /// Automatic mode
        Automatic = 0x0,

        /// Hold mode
        Hold = 0x1,

        /// One-push mode
        OnePush = 0x2,

        /// Preset mode
        Preset = 0x3,

        Reserved4 = 0x4,
        Reserved5 = 0x5,
        Reserved6 = 0x6,
    }

    #[bitenum(u3, exhaustive = true)]
    pub(crate) enum RawWhiteBalanceMode {
        NoInfo = 0x7,
    }
}

super::util::optional_enum! {
    /// Selected white balance value of the consumer camera
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
    #[allow(missing_docs)]
    pub enum WhiteBalance {
        /// Candle
        Candle = 0x00,

        /// Incandescent lamp
        IncandescentLamp= 0x01,

        /// Fluorescent lamp of low color temperature
        FluorescentLampLowColorTemperature = 0x02,

        /// Fluorescent lamp of high color temperature
        FluorescentLampHighColorTemperature = 0x03,

        /// Sunlight
        Sunlight = 0x04,

        /// Cloudiness
        Cloudiness = 0x05,

        /// Others
        Others = 0x06,

        Reserved7 = 0x07,
        Reserved8 = 0x08,
        Reserved9 = 0x09,
        Reserved10 = 0x0A,
        Reserved11 = 0x0B,
        Reserved12 = 0x0C,
        Reserved13 = 0x0D,
        Reserved14 = 0x0E,
        Reserved15 = 0x0F,
        Reserved16 = 0x10,
        Reserved17 = 0x11,
        Reserved18 = 0x12,
        Reserved19 = 0x13,
        Reserved20 = 0x14,
        Reserved21 = 0x15,
        Reserved22 = 0x16,
        Reserved23 = 0x17,
        Reserved24 = 0x18,
        Reserved25 = 0x19,
        Reserved26 = 0x1A,
        Reserved27 = 0x1B,
        Reserved28 = 0x1C,
        Reserved29 = 0x1D,
        Reserved30 = 0x1E,
    }

    #[bitenum(u5, exhaustive = true)]
    pub(crate) enum RawWhiteBalance {
        NoInfo = 0x1F,
    }
}

/// Provides some of the settings used by a consumer camera to record the video.
///
/// DV standards:
///
/// - IEC 61834-4:1998 Section 10.1 - Consumer Camera 1 (CAMERA)
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Validate, Serialize, Deserialize)]
#[garde(context(super::PackContext))]
pub struct CameraConsumer1 {
    // Exposure and gain settings
    //
    /// Auto exposure mode
    #[garde(skip)]
    pub auto_exposure_mode: Option<AutoExposureMode>,

    /// Iris is F number rounded to exactly 1 decimal place
    ///
    /// Special values:
    /// - `0.0`: under F1.0
    /// - `999.9`: iris is closed
    #[garde(custom(check_iris))]
    pub iris: Option<Decimal>,

    /// Value of the automatic gain control
    #[garde(custom(check_auto_gain_control))]
    pub auto_gain_control: Option<u4>,

    // White balance settings
    //
    /// White balance mode
    #[garde(skip)]
    pub white_balance_mode: Option<WhiteBalanceMode>,

    /// Selected white balance value
    #[garde(skip)]
    pub white_balance: Option<WhiteBalance>,

    // Focus settings
    //
    /// Focus mode
    #[garde(skip)]
    pub focus_mode: super::FocusMode,

    /// Focus position in terms of length, in centimeters
    #[garde(custom(super::check_focus_position))]
    pub focus_position: Option<u16>,

    /// Reserved bits; should normally be set to `0x3`.
    #[garde(skip)]
    pub reserved: u2,
}

static IRIS_BITS_TO_DECIMAL: LazyLock<[Option<Decimal>; 64]> = LazyLock::new(|| {
    let mut iris_values = [None; 64];

    // Special defined iris values
    iris_values[0x3D] = Some(dec!(0.0)); // under F1.0
    iris_values[0x3E] = Some(dec!(999.9)); // closed iris
    iris_values[0x3F] = None; // no information

    // Remaining iris values follow a simple exponential formula
    for (iris_bits, iris_value) in iris_values.iter_mut().enumerate().take(0x3C + 1) {
        // calculate value using floating point math
        let v = 2.0_f64.powf(f64::from(u8::try_from(iris_bits).unwrap()) / 8.0);
        // convert to a decimal and then round to one decimal point
        let v = Decimal::try_from(v).unwrap();
        let v = v.round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
        *iris_value = Some(v);
    }
    iris_values
});

static IRIS_DECIMAL_TO_IRIS_BITS: LazyLock<HashMap<Option<Decimal>, u6>> = LazyLock::new(|| {
    HashMap::<Option<Decimal>, u6>::from_iter(
        IRIS_BITS_TO_DECIMAL
            .iter()
            .enumerate()
            .map(|(iris_bits, d)| (*d, u6::new(u8::try_from(iris_bits).unwrap()))),
    )
});

static VALID_IRIS_VALUES: LazyLock<HashSet<Decimal>> = LazyLock::new(|| {
    HashSet::<Decimal>::from_iter(IRIS_DECIMAL_TO_IRIS_BITS.clone().into_keys().flatten())
});

fn check_iris(iris: &Option<Decimal>, _ctx: &super::PackContext) -> garde::Result {
    match iris {
        Some(iris) => {
            CameraConsumer1::valid_iris_values().contains(iris).then_some(()).ok_or_else(|| {
                garde::Error::new(format!(
                    "iris value {iris} not supported: only iris values returned by the \
                    valid_iris_values function are supported",
                ))
            })
        }
        None => Ok(()),
    }
}

fn check_auto_gain_control(
    auto_gain_control: &Option<u4>,
    _ctx: &super::PackContext,
) -> garde::Result {
    if *auto_gain_control == Some(u4::MAX) {
        Err(garde::Error::new(
            "instead of specifying Some(0xF), use None to indicate no information",
        ))
    } else {
        Ok(())
    }
}

#[bitfield(u32)]
struct RawCameraConsumer1 {
    // PC1
    #[bits(0..=5, rw)]
    iris: u6,
    #[bits(6..=7, rw)]
    reserved: u2,

    // PC2
    #[bits(8..=11, rw)]
    agc: u4,
    #[bits(12..=15, rw)]
    ae_mode: RawAutoExposureMode,

    // PC3
    #[bits(16..=20, rw)]
    white_balance: RawWhiteBalance,
    #[bits(21..=23, rw)]
    wb_mode: RawWhiteBalanceMode,

    // PC4
    #[bits(24..=30, rw)]
    focus: u7,
    #[bit(31, rw)]
    fcm: RawFocusMode,
}

impl CameraConsumer1 {
    /// Returns the list of valid iris values that are recognized by
    /// [`CameraConsumer1::iris`].
    pub fn valid_iris_values() -> &'static HashSet<Decimal> {
        &VALID_IRIS_VALUES
    }

    /// Returns the list of valid focus positions that are recognized by
    /// [`CameraConsumer1::focus_position`].
    pub fn valid_focus_positions() -> &'static HashSet<u16> {
        &super::VALID_FOCUS_POSITIONS
    }
}

impl super::PackData for CameraConsumer1 {
    fn try_from_raw(
        raw: &super::RawPackData,
        _ctx: &super::PackContext,
    ) -> Result<Self, super::RawError> {
        let raw = RawCameraConsumer1::new_with_raw_value(u32::from_le_bytes(*raw));
        Ok(Self {
            auto_exposure_mode: raw.ae_mode().into(),
            iris: IRIS_BITS_TO_DECIMAL[usize::from(raw.iris().value())],
            auto_gain_control: if raw.agc() == u4::MAX { None } else { Some(raw.agc()) },
            white_balance_mode: raw.wb_mode().into(),
            white_balance: raw.white_balance().into(),
            focus_mode: raw.fcm().into(),
            focus_position: super::FOCUS_POSITION_BITS_TO_CENTIMETERS
                [usize::from(raw.focus().value())],
            reserved: raw.reserved(),
        })
    }
}

impl super::ValidPackDataTrait<CameraConsumer1> for super::ValidPack<CameraConsumer1> {
    fn to_raw(&self, _ctx: &super::PackContext) -> super::RawPackData {
        RawCameraConsumer1::builder()
            .with_iris(IRIS_DECIMAL_TO_IRIS_BITS[&self.iris])
            .with_reserved(self.reserved)
            .with_agc(self.auto_gain_control.unwrap_or(u4::MAX))
            .with_ae_mode(self.auto_exposure_mode.into())
            .with_white_balance(self.white_balance.into())
            .with_wb_mode(self.white_balance_mode.into())
            .with_focus(super::FOCUS_POSITION_CENTIMETERS_TO_BITS[&self.focus_position])
            .with_fcm(self.focus_mode.into())
            .build()
            .raw_value()
            .to_le_bytes()
    }
}
