// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

pub mod gpio;

use self::gpio::GpioBank;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ValidationError(String);

impl std::error::Error for ValidationError {}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// This struct describs the RevPi HAT EEPROM configuration
///
/// This describe the [RevPi HAT EEPROM](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md)
/// configuration. The struct is used to parse the JSON confirguation with
/// [serde_json](https://docs.serde.rs/serde_json/).
///
/// # Example JSON configuration
/// ```json
/// {
///     "version": 1,
///     "vstr": "KUNBUS GmbH",
///     "pstr": "RevPi ExampleDevice 8GB",
///     "pid": 666,
///     "prev": 3,
///     "pver": 333,
///     "dtstr": "revpi-example-2022",
///     "gpiobanks": [
///         {
///             "drive": "8mA",
///             "slew": "default",
///             "hysteresis": "enable",
///             "gpios": [
///                 {
///                     "gpio": 2,
///                     "fsel": "input",
///                     "pull": "default"
///                 },
///                 {
///                     "gpio": 3,
///                     "fsel": "output",
///                     "pull": "none"
///                 },
///                 {
///                     "gpio": 4,
///                     "fsel": "alt1",
///                     "pull": "up"
///                 }
///             ]
///         }
///     ]
/// }
/// ```
///
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RevPiHatEeprom {
    /// The version of the used [RevPi HAT EEPROM Format](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#0-format-version)
    pub version: u16,
    /// The vendor string (max. 255 chars (bytes)), see [Vendor String](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#vendor-string-vstr)
    pub vstr: String,
    /// The product string (max. 255 chars (bytes)), see [Product String](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#product-string-pstr)
    pub pstr: String,
    /// The product ID (16 bits), see [Product ID](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#product-id-pid)
    pub pid: u16,
    /// The product revision (16 bits), see [Product Revision](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#2-product-revision-prev)
    pub prev: u16,
    /// The customer visible product version multiplied with 100 (16 bits), see [Product Version](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#product-version-pver)
    pub pver: u16,
    /// The device tree overlay name, see [Linux Device Tree (Blob) Atom](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#linux-device-tree-blob-atom)
    pub dtstr: String,
    /// The configuration of the first gpiobank, see [GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
    pub gpiobanks: Vec<GpioBank>,
}

pub fn parse_config(s: &str) -> Result<RevPiHatEeprom, Box<dyn std::error::Error>> {
    let eep: RevPiHatEeprom = serde_json::from_str(s)?;
    validate(&eep)?;
    Ok(eep)
}

fn validate(eep: &RevPiHatEeprom) -> Result<(), ValidationError> {
    if eep.version != 1 {
        return Err(ValidationError(format!(
            "invalid value: `{}`: Unsupported format version",
            eep.version
        )));
    }
    if eep.pstr.len() >= 256 {
        return Err(ValidationError(format!(
            "invalid value: `{}`: Product string to long {} (max: {}) bytes",
            eep.pstr,
            eep.pstr.len(),
            u8::MAX
        )));
    }
    if eep.vstr.len() >= 256 {
        return Err(ValidationError(format!(
            "invalid value: `{}`: Vendor string to long: {} (max: {}) bytes",
            eep.vstr,
            eep.vstr.len(),
            u8::MAX
        )));
    }
    if eep.dtstr.len() >= u32::MAX as usize {
        return Err(ValidationError(format!(
            "invalid value: `{}`: Device tree string to long: {} (max: {}) bytes",
            eep.dtstr,
            eep.dtstr.len(),
            u32::MAX
        )));
    }
    for bank in &eep.gpiobanks {
        bank.validate()?;
    }
    Ok(())
}
