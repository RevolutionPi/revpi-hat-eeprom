// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

pub mod gpio;

use self::gpio::GpioBank;
use chrono::NaiveDate;
use eui48::MacAddress;
use rpi_hat_eep::gpio_map;
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
///     "eeprom_data_version": 3,
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
///                     "pull": "up",
///                     "comment": [
///                         "This configures the I2C1 SCL",
///                         "external pull-up missing"
///                     ]
///                 }
///             ]
///         },
///         {
///             "drive": "16mA",
///             "slew": "default",
///             "hysteresis": "default",
///             "gpios": [
///                 {
///                     "gpio": 31,
///                     "fsel": "input",
///                     "pull": "none",
///                     "comment": [
///                         "LAN9514 nRESET (USB_CM.RUN)",
///                         "external pull-up"
///                     ]
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
    /// The version of the HAT EEPROM content (16 bits) see [EEPROM Data Version](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#6-eeprom-data-version)
    pub eeprom_data_version: u16,
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
    /// The serial number which is also printed on the casing of the RevPi, see [Serial](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#1-serial)
    pub serial: Option<u32>,
    /// The end test date represents the current date as of when the end of line test is/was done, see [Endtest Date](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#3-endtest-date)
    pub edate: Option<NaiveDate>,
    /// The first mac address of the device, see [MAC Address](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#5-mac-address)
    pub mac: Option<MacAddress>,
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
    if eep.gpiobanks.is_empty() || eep.gpiobanks.len() > 2 {
        return Err(ValidationError(format!(
            "unsupported number of gpio banks: {} (min: 1; max: 2)",
            eep.gpiobanks.len()
        )));
    }
    eep.gpiobanks[0].validate(gpio_map::GpioBank::Bank0)?;
    if eep.gpiobanks.len() > 1 {
        eep.gpiobanks[1].validate(gpio_map::GpioBank::Bank1)?;
    }
    Ok(())
}
