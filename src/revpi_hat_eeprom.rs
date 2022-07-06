// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use serde::{Deserialize, Serialize};

use crate::RevPiError;
use crate::gpio::GpioBank;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RevPiHatEeprom {
    pub vstr: String,
    pub pstr: String,
    pub pid: u16,
    pub prev: u16,
    pub pver: u16,
    pub dtstr: String,
    pub gpiobank: GpioBank,
}

pub fn parse_config(s: &str) -> Result<RevPiHatEeprom, RevPiError> {
    serde_json::from_str(s).map_err(|error| { RevPiError::from(error) })
}
