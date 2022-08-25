// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use crate::RevPiError;
use serde::{Deserialize, Serialize};

const MAX_GPIOS: usize = 28;

/// This defines possible values for the pin drive strength
///
/// The drive strength can only be set per bank. So this will apply for all pins
/// of this bank. The drive strength can be set to 2, 4, 6, 8, 10, 12, 14 and
/// 16 mA. It can also be left at default. Then the actual drive strength
/// depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankDrive {
    Default,
    #[serde(rename = "2mA")]
    Drive2mA,
    #[serde(rename = "4mA")]
    Drive4mA,
    #[serde(rename = "6mA")]
    Drive6mA,
    #[serde(rename = "8mA")]
    Drive8mA,
    #[serde(rename = "10mA")]
    Drive10mA,
    #[serde(rename = "12mA")]
    Drive12mA,
    #[serde(rename = "14mA")]
    Drive14mA,
    #[serde(rename = "16mA")]
    Drive16mA,
}

/// This defines possible values for the pin drive slew rate
///
/// The slew rate can only be set per bank. So this will apply for all pins
/// of this bank. The slew rate can be set to slew rate limiting and no
/// slew limiting. It can also be left at default. Then the actual slew rate
/// depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankSlew {
    Default,
    RateLimiting,
    NoLimit,
}

/// This defines possible values for the pin drive hysteresis
///
/// The hysteresis can only be set per bank. So this will apply for all pins
/// of this bank. The hysteresis can be set to hysteresis disabled and
/// hysteresis enabled. It can also be left at default. Then the actual
/// hysteresis depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankHysteresis {
    Default,
    Disable,
    Enable,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioFsel {
    Input,
    Output,
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioPull {
    Default,
    Up,
    Down,
    None,
}

/// This struct represents a single gpio pin
///
/// Every gpio pin has a pin number, a function configuration and a pull
/// configuration. The function configuration can be one of the alternate
/// functions ([Alt0](GpioFsel::Alt0) - [Alt5](GpioFsel::Alt5)) or a gpio in
/// [input](GpioFsel::Input) or [output](GpioFsel::Output) mode.
/// For the alternate pin functions see the reference manual of your SoC.
/// The pull confirguation can be set to [pullup](GpioPull::Up),
/// [pulldown](GpioPull::Down), [no pull](GpioPull::None) and to leave it at
/// [default](GpioPull::Default).
///
/// Currently only the first gpio bank is supported by the HAT EEPROM. Thus
/// leavs only the first 28 gpios. The gpios 0 and 1 are used for the HAT EEPROM
/// and should not be changed. The gpio bank validation will not allow to modify
/// the gpios 0 and 1 also the gpios higher then 27.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GpioPin {
    gpio: u8,
    fsel: GpioFsel,
    pull: GpioPull,
}

/// This struct represents the GPIO configuration the the HAT EEPROM
///
/// This struct is used to deserialize the GPIO configuration from a RevPi HAT
/// EEPROM configuration in json format. See [RevPi HAT EEPROM Format: GPIO map
/// atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
/// for details about the meaning of the values in this struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GpioBank {
    drive: GpioBankDrive,
    slew: GpioBankSlew,
    hysteresis: GpioBankHysteresis,
    gpios: Vec<GpioPin>,
}

impl GpioBank {
    pub fn validate(&self) -> Result<(), RevPiError> {
        let mut configured_gpios: Vec<bool> = vec![false; MAX_GPIOS];
        for gpio in &self.gpios {
            if gpio.gpio == 0 || gpio.gpio == 1 {
                return Err(RevPiError::ValidationError(format!(
                    "gpio# mustn't be 0 or 1 (they are used for the HAT EEPROM): {}",
                    gpio.gpio
                )));
            }
            if gpio.gpio as usize >= MAX_GPIOS {
                return Err(RevPiError::ValidationError(format!(
                    "gpio#: {} >= {}",
                    gpio.gpio, MAX_GPIOS
                )));
            }
            if configured_gpios[gpio.gpio as usize] {
                return Err(RevPiError::ValidationError(format!(
                    "gpio#: {} defined more then once",
                    gpio.gpio
                )));
            }
            configured_gpios[gpio.gpio as usize] = true;
        }
        Ok(())
    }
}
