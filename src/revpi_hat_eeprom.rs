// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use serde::{Deserialize, Serialize};

use crate::RevPiError;
use crate::gpio::GpioBank;
use crate::validate;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RevPiHatEeprom {
    pub vstr: String,
    pub pstr: String,
    pub pid: u16,
    pub prev: u16,
    pub pver: u16,
    pub dtstr: String,
    pub gpiobanks: Vec<GpioBank>,
}

pub fn parse_config(s: &str) -> Result<RevPiHatEeprom, RevPiError> {
    serde_json::from_str(s).map_err(|error| { RevPiError::from(error) })
}

impl RevPiHatEeprom {
    pub fn validate(&self) -> Result<(), RevPiError> {
        if validate::validate_string_max255(&self.vstr).is_err() {
            eprintln!("ERROR: Config contains invalid vstr `{}': string to long",
                      self.vstr);
        }
        if validate::validate_string_max255(&self.pstr).is_err() {
            eprintln!("ERROR: Config contains invalid pstr `{}': string to long",
                      self.pstr);
        }
        for bank in &self.gpiobanks {
            bank.validate()?;
        }
        Ok(())
    }
}
