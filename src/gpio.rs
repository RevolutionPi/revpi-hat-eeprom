// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use serde::{Serialize, Deserialize};

use crate::RevPiError;

const MAX_GPIOS: usize = 28;

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
    Drive16mA
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankSlew {
    Default,
    RateLimiting,
    NoLimit,
}

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GpioPin {
    gpio: u8,
    fsel: GpioFsel,
    pull: GpioPull,
}

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
                return Err(
                    RevPiError::ValidationError(
                        format!("gpio# mustn't be 0 or 1 (they are used for the HAT EEPROM): {}", gpio.gpio)
                    ).into()
                )
            }
            if gpio.gpio as usize >= MAX_GPIOS {
                return Err(
                    RevPiError::ValidationError(
                        format!("gpio#: {} >= {}", gpio.gpio, MAX_GPIOS)
                    ).into()
                )
            }
            if configured_gpios[gpio.gpio as usize] {
                return Err(
                    RevPiError::ValidationError(
                        format!("gpio#: {} defined more then once", gpio.gpio)
                    ).into()
                )
            }
            configured_gpios[gpio.gpio as usize] = true;
        }
        Ok(())
    }
}
