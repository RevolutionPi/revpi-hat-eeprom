// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use serde::{Serialize, Deserialize};

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
