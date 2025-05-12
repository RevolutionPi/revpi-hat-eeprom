// SPDX-FileCopyrightText: 2022-2025 KUNBUS GmbH <support@kunbus.com>
//
// SPDX-License-Identifier: GPL-2.0-or-later

use crate::ValidationError;
use rpi_hat_eep::gpio_map;
use rpi_hat_eep::gpio_map::{BANK0_GPIOS, BANK1_GPIOS};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const MAX_GPIOS: usize = BANK0_GPIOS + BANK1_GPIOS;

/// This defines possible values for the pin drive strength
///
/// The drive strength can only be set per bank. So this will apply for all pins
/// of this bank. The drive strength can be set to 2, 4, 6, 8, 10, 12, 14 and
/// 16 mA. It can also be left at default. Then the actual drive strength
/// depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<gpio_map::GpioDrive> for GpioBankDrive {
    fn from(drive: gpio_map::GpioDrive) -> Self {
        match drive {
            gpio_map::GpioDrive::Default => GpioBankDrive::Default,
            gpio_map::GpioDrive::Drive2mA => GpioBankDrive::Drive2mA,
            gpio_map::GpioDrive::Drive4mA => GpioBankDrive::Drive4mA,
            gpio_map::GpioDrive::Drive6mA => GpioBankDrive::Drive6mA,
            gpio_map::GpioDrive::Drive8mA => GpioBankDrive::Drive8mA,
            gpio_map::GpioDrive::Drive10mA => GpioBankDrive::Drive10mA,
            gpio_map::GpioDrive::Drive12mA => GpioBankDrive::Drive12mA,
            gpio_map::GpioDrive::Drive14mA => GpioBankDrive::Drive14mA,
            gpio_map::GpioDrive::Drive16mA => GpioBankDrive::Drive16mA,
        }
    }
}

impl From<GpioBankDrive> for gpio_map::GpioDrive {
    fn from(drive: GpioBankDrive) -> Self {
        match drive {
            GpioBankDrive::Default => gpio_map::GpioDrive::Default,
            GpioBankDrive::Drive2mA => gpio_map::GpioDrive::Drive2mA,
            GpioBankDrive::Drive4mA => gpio_map::GpioDrive::Drive4mA,
            GpioBankDrive::Drive6mA => gpio_map::GpioDrive::Drive6mA,
            GpioBankDrive::Drive8mA => gpio_map::GpioDrive::Drive8mA,
            GpioBankDrive::Drive10mA => gpio_map::GpioDrive::Drive10mA,
            GpioBankDrive::Drive12mA => gpio_map::GpioDrive::Drive12mA,
            GpioBankDrive::Drive14mA => gpio_map::GpioDrive::Drive14mA,
            GpioBankDrive::Drive16mA => gpio_map::GpioDrive::Drive16mA,
        }
    }
}

/// This defines possible values for the pin drive slew rate
///
/// The slew rate can only be set per bank. So this will apply for all pins
/// of this bank. The slew rate can be set to slew rate limiting and no
/// slew limiting. It can also be left at default. Then the actual slew rate
/// depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankSlew {
    Default,
    RateLimiting,
    NoLimit,
}

impl From<gpio_map::GpioSlew> for GpioBankSlew {
    fn from(slew: gpio_map::GpioSlew) -> Self {
        match slew {
            gpio_map::GpioSlew::Default => GpioBankSlew::Default,
            gpio_map::GpioSlew::RateLimiting => GpioBankSlew::RateLimiting,
            gpio_map::GpioSlew::NoLimit => GpioBankSlew::NoLimit,
        }
    }
}

impl From<GpioBankSlew> for gpio_map::GpioSlew {
    fn from(slew: GpioBankSlew) -> Self {
        match slew {
            GpioBankSlew::Default => gpio_map::GpioSlew::Default,
            GpioBankSlew::RateLimiting => gpio_map::GpioSlew::RateLimiting,
            GpioBankSlew::NoLimit => gpio_map::GpioSlew::NoLimit,
        }
    }
}

/// This defines possible values for the pin drive hysteresis
///
/// The hysteresis can only be set per bank. So this will apply for all pins
/// of this bank. The hysteresis can be set to hysteresis disabled and
/// hysteresis enabled. It can also be left at default. Then the actual
/// hysteresis depends not on this configuration.
///
/// For details see: [RevPi HAT EEPROM Format: GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GpioBankHysteresis {
    Default,
    Disable,
    Enable,
}

impl From<gpio_map::GpioHysteresis> for GpioBankHysteresis {
    fn from(hyst: gpio_map::GpioHysteresis) -> Self {
        match hyst {
            gpio_map::GpioHysteresis::Default => GpioBankHysteresis::Default,
            gpio_map::GpioHysteresis::Disable => GpioBankHysteresis::Disable,
            gpio_map::GpioHysteresis::Enable => GpioBankHysteresis::Enable,
        }
    }
}

impl From<GpioBankHysteresis> for gpio_map::GpioHysteresis {
    fn from(hyst: GpioBankHysteresis) -> Self {
        match hyst {
            GpioBankHysteresis::Default => gpio_map::GpioHysteresis::Default,
            GpioBankHysteresis::Disable => gpio_map::GpioHysteresis::Disable,
            GpioBankHysteresis::Enable => gpio_map::GpioHysteresis::Enable,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<gpio_map::GpioFsel> for GpioFsel {
    fn from(fsel: gpio_map::GpioFsel) -> Self {
        match fsel {
            gpio_map::GpioFsel::Input => GpioFsel::Input,
            gpio_map::GpioFsel::Output => GpioFsel::Output,
            gpio_map::GpioFsel::Alt0 => GpioFsel::Alt0,
            gpio_map::GpioFsel::Alt1 => GpioFsel::Alt1,
            gpio_map::GpioFsel::Alt2 => GpioFsel::Alt2,
            gpio_map::GpioFsel::Alt3 => GpioFsel::Alt3,
            gpio_map::GpioFsel::Alt4 => GpioFsel::Alt4,
            gpio_map::GpioFsel::Alt5 => GpioFsel::Alt5,
        }
    }
}

impl From<GpioFsel> for gpio_map::GpioFsel {
    fn from(fsel: GpioFsel) -> Self {
        match fsel {
            GpioFsel::Input => gpio_map::GpioFsel::Input,
            GpioFsel::Output => gpio_map::GpioFsel::Output,
            GpioFsel::Alt0 => gpio_map::GpioFsel::Alt0,
            GpioFsel::Alt1 => gpio_map::GpioFsel::Alt1,
            GpioFsel::Alt2 => gpio_map::GpioFsel::Alt2,
            GpioFsel::Alt3 => gpio_map::GpioFsel::Alt3,
            GpioFsel::Alt4 => gpio_map::GpioFsel::Alt4,
            GpioFsel::Alt5 => gpio_map::GpioFsel::Alt5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GpioPull {
    Default,
    Up,
    Down,
    None,
}

impl From<gpio_map::GpioPull> for GpioPull {
    fn from(pull: gpio_map::GpioPull) -> Self {
        match pull {
            gpio_map::GpioPull::Default => GpioPull::Default,
            gpio_map::GpioPull::Up => GpioPull::Up,
            gpio_map::GpioPull::Down => GpioPull::Down,
            gpio_map::GpioPull::NoPull => GpioPull::None,
        }
    }
}

impl From<GpioPull> for gpio_map::GpioPull {
    fn from(pull: GpioPull) -> Self {
        match pull {
            GpioPull::Default => gpio_map::GpioPull::Default,
            GpioPull::Up => gpio_map::GpioPull::Up,
            GpioPull::Down => gpio_map::GpioPull::Down,
            GpioPull::None => gpio_map::GpioPull::NoPull,
        }
    }
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GpioPin {
    gpio: u8,
    fsel: GpioFsel,
    pull: GpioPull,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<Vec<String>>,
}

/// This struct represents the GPIO configuration of the HAT EEPROM
///
/// This struct is used to deserialize the GPIO configuration from a RevPi HAT
/// EEPROM configuration in json format. See [RevPi HAT EEPROM Format: GPIO map
/// atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
/// for details about the meaning of the values in this struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GpioBank {
    drive: GpioBankDrive,
    slew: GpioBankSlew,
    hysteresis: GpioBankHysteresis,
    gpios: Vec<GpioPin>,
}

impl GpioBank {
    #[must_use]
    pub const fn new(
        drive: GpioBankDrive,
        slew: GpioBankSlew,
        hysteresis: GpioBankHysteresis,
        gpios: Vec<GpioPin>,
    ) -> Self {
        Self {
            drive,
            slew,
            hysteresis,
            gpios,
        }
    }

    pub fn validate(&self, bank_no: gpio_map::GpioBank) -> Result<(), ValidationError> {
        let mut configured_gpios: Vec<bool> = vec![false; MAX_GPIOS];
        for gpio in &self.gpios {
            if gpio.gpio == 0 || gpio.gpio == 1 {
                return Err(ValidationError(format!(
                    "gpio# mustn't be 0 or 1 (they are used for the HAT EEPROM): {}",
                    gpio.gpio
                )));
            }
            match bank_no {
                gpio_map::GpioBank::Bank0 => {
                    if gpio.gpio as usize >= BANK0_GPIOS {
                        return Err(ValidationError(format!(
                            "gpio# (bank0): {} (MIN: {}, MAX: {})",
                            gpio.gpio,
                            2,
                            BANK0_GPIOS - 1
                        )));
                    }
                }
                gpio_map::GpioBank::Bank1 => {
                    if (gpio.gpio as usize) >= BANK0_GPIOS + BANK1_GPIOS
                        || (gpio.gpio as usize) < BANK0_GPIOS
                    {
                        return Err(ValidationError(format!(
                            "gpio# (bank1): {} (MIN: {}, MAX: {})",
                            gpio.gpio,
                            BANK0_GPIOS,
                            MAX_GPIOS - 1
                        )));
                    }
                }
            }
            if configured_gpios[gpio.gpio as usize] {
                return Err(ValidationError(format!(
                    "gpio#: {} defined more then once",
                    gpio.gpio
                )));
            }
            configured_gpios[gpio.gpio as usize] = true;
        }
        Ok(())
    }
}

impl Display for GpioBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl GpioBank {
    pub fn into_gpio_map(
        self,
        bank: gpio_map::GpioBank,
    ) -> Result<gpio_map::EepAtomGpioMapData, gpio_map::GpioError> {
        let mut gpio_map = gpio_map::EepAtomGpioMapData::new(
            bank,
            self.drive.into(),
            self.slew.into(),
            self.hysteresis.into(),
            gpio_map::GpioBackPower::None,
        );

        for gpio in self.gpios {
            gpio_map.set(
                gpio.gpio as usize,
                gpio_map::GpioPin::new(gpio.fsel.into(), gpio.pull.into(), true),
            )?;
        }
        Ok(gpio_map)
    }
}
