// SPDX-FileCopyrightText: 2022-2025 KUNBUS GmbH <support@kunbus.com>
//
// SPDX-License-Identifier: GPL-2.0-or-later

pub mod gpio;

use std::path::{Path, PathBuf};

use self::gpio::GpioBank;
use chrono::NaiveDate;
use macaddr::MacAddr6;
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
/// The definition of a template used to compute a [`RevPiHatEeprom`] from a [`RawRevPiHatEeprom`]
/// if the field [`RawRevPiHatEeprom::include`] is given.
///
/// The template defines only fields that may be overridden. Additionally, to be a valid template
/// and be allowed to be included in a [`RawRevPiHatEeprom`] in the first place, the fields
/// [`TemplateDefinition::version`] and [`TemplateDefinition::eeprom_data_version`] must match the
/// fields [`RawRevPiHatEeprom::version`] and [`RawRevPiHatEeprom::eeprom_data_version`]
/// respectively, otherwise it's an invalid template inclusion and should produce an error.
pub struct TemplateDefinition {
    pub version: u16,
    pub eeprom_data_version: u16,
    pub gpiobanks: Vec<GpioBank>,
}

impl TemplateDefinition {
    pub fn from_file(template_dir: &Path, name: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::fs::read_to_string(template_dir.join(name))?;
        let template: Self = serde_json::from_str(&s)?;
        Ok(template)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
/// Definition of how to include a [`TemplateDefinition`].
///
/// The template may be included either as a [`TemplateInclude::Filename`] or as a
/// [`TemplateInclude::Object`]. The former is the name of a file in the template dir that is
/// specified elsewhere, the latter is an inline [`TemplateDefinition`] which should only be used
/// for testing.
pub enum TemplateInclude {
    Filename(PathBuf),
    Object(TemplateDefinition),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
/// The raw form of a [`RevPiHatEeprom`] which allows inclusion of a [`TemplateDefinition`].
///
/// The [`RawRevPiHatEeprom`] may define a [`TemplateDefinition`] to include. The
/// [`TemplateDefinition`] acts as the base of the definition. If any fields are given in both
/// the [`RawRevPiHatEeprom`] and the [`TemplateDefinition`], the [`RawRevPiHatEeprom`] definition
/// will be used.
///
/// An error occurs if the [`RawRevPiHatEeprom`] overrides all the fields in the included
/// [`TemplateDefinition`] as the [`TemplateDefinition`] is unnecessary in this case.
pub struct RawRevPiHatEeprom {
    pub version: u16,
    pub eeprom_data_version: u16,
    pub vstr: String,
    pub pstr: String,
    pub pid: u16,
    pub prev: u16,
    pub pver: u16,
    pub dtstr: String,
    pub serial: Option<u32>,
    pub edate: Option<NaiveDate>,
    pub mac: Option<MacAddr6>,
    pub gpiobanks: Option<Vec<GpioBank>>,
    pub include: Option<TemplateInclude>,
}

impl TryFrom<&str> for RawRevPiHatEeprom {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let raw_eep: Self = serde_json::from_str(value)?;
        Ok(raw_eep)
    }
}

/// This struct describes the RevPi HAT EEPROM configuration.
///
/// This describe the [RevPi HAT
/// EEPROM](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md)
/// configuration. The struct is used to parse the JSON configuration with
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    pub mac: Option<MacAddr6>,
    /// The configuration of the first gpiobank, see [GPIO map atom data](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/docs/RevPi-HAT-EEPROM-Format.md#gpio-map-atom-data-type0x0002)
    pub gpiobanks: Vec<GpioBank>,
}

impl RevPiHatEeprom {
    /// Create a [`RevPiHatEeprom`] from a configuration string.
    ///
    /// The configuration may be a [`RawRevPiHatEeprom`] which might include a template. If this is
    /// the case, the `template_dir` needs to be known, which is the second parameter of this
    /// function.
    ///
    /// The argument `template_dir` is lazily evaluated. This means that checking if the directory
    /// exists is only done if the `include` keyword is used in the [`RawRevPiHatEeprom`].
    pub fn from_config_str(
        template_dir: &Path,
        config: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let raw_eep: RawRevPiHatEeprom = serde_json::from_str(config)?;
        Self::from_raw_definition(template_dir, raw_eep)
    }

    /// Create a [`RevPiHatEeprom`] from a [`RawRevPiHatEeprom`].
    ///
    /// The argument `template_dir` is lazily evaluated. This means that checking if the directory
    /// exists is only done if the `include` keyword is used in the [`RawRevPiHatEeprom`].
    pub fn from_raw_definition(
        template_dir: &Path,
        raw_definition: RawRevPiHatEeprom,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let Some(include) = raw_definition.include else {
            if let Some(gpiobanks) = raw_definition.gpiobanks {
                return Ok(Self {
                    version: raw_definition.version,
                    eeprom_data_version: raw_definition.eeprom_data_version,
                    vstr: raw_definition.vstr,
                    pstr: raw_definition.pstr,
                    pid: raw_definition.pid,
                    prev: raw_definition.prev,
                    pver: raw_definition.pver,
                    dtstr: raw_definition.dtstr,
                    serial: raw_definition.serial,
                    edate: raw_definition.edate,
                    mac: raw_definition.mac,
                    gpiobanks,
                });
            }

            return Err(Box::new(ValidationError(
                "Definition requires \"gpiobanks\" attribute".to_string(),
            )));
        };

        // check if all fields in the template are overridden
        if raw_definition.gpiobanks.is_some() {
            return Err(Box::new(ValidationError(
                "All fields of the template are overridden, template is useless".to_string(),
            )));
        }

        let def = match include {
            TemplateInclude::Filename(name) => TemplateDefinition::from_file(template_dir, &name)?,
            TemplateInclude::Object(def) => def,
        };

        if raw_definition.version != def.version
            || raw_definition.eeprom_data_version != def.eeprom_data_version
        {
            return Err(Box::new(ValidationError(
                "Version fields of definition and template have to match".to_string(),
            )));
        }

        let definition = Self {
            version: raw_definition.version,
            eeprom_data_version: raw_definition.eeprom_data_version,
            vstr: raw_definition.vstr,
            pstr: raw_definition.pstr,
            pid: raw_definition.pid,
            prev: raw_definition.prev,
            pver: raw_definition.pver,
            dtstr: raw_definition.dtstr,
            serial: raw_definition.serial,
            edate: raw_definition.edate,
            mac: raw_definition.mac,
            gpiobanks: def.gpiobanks,
        };
        definition.validate()?;

        Ok(definition)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        if self.version != 1 {
            return Err(ValidationError(format!(
                "invalid value: `{}`: Unsupported format version",
                self.version
            )));
        }
        if self.pstr.len() >= 256 {
            return Err(ValidationError(format!(
                "invalid value: `{}`: Product string too long {} (max: {}) bytes",
                self.pstr,
                self.pstr.len(),
                u8::MAX
            )));
        }
        if self.vstr.len() >= 256 {
            return Err(ValidationError(format!(
                "invalid value: `{}`: Vendor string too long: {} (max: {}) bytes",
                self.vstr,
                self.vstr.len(),
                u8::MAX
            )));
        }
        if self.dtstr.len() >= u32::MAX as usize {
            return Err(ValidationError(format!(
                "invalid value: `{}`: Device tree string too long: {} (max: {}) bytes",
                self.dtstr,
                self.dtstr.len(),
                u32::MAX
            )));
        }
        if self.gpiobanks.is_empty() || self.gpiobanks.len() > 2 {
            return Err(ValidationError(format!(
                "unsupported number of gpio banks: {} (min: 1; max: 2)",
                self.gpiobanks.len()
            )));
        }
        self.gpiobanks[0].validate(gpio_map::GpioBank::Bank0)?;
        if self.gpiobanks.len() > 1 {
            self.gpiobanks[1].validate(gpio_map::GpioBank::Bank1)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, create_dir};

    use super::*;
    use sealed_test::prelude::*;

    #[test]
    fn test_same_versions_definition_template() -> Result<(), Box<dyn std::error::Error>> {
        let template = TemplateDefinition {
            version: 1,
            eeprom_data_version: 1,
            gpiobanks: vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )],
        };

        let raw_definition = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 1,
            vstr: String::new(),
            pstr: String::new(),
            pid: 1,
            prev: 1,
            pver: 1,
            dtstr: String::new(),
            serial: None,
            edate: None,
            mac: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Object(template)),
        };

        RevPiHatEeprom::from_raw_definition(&PathBuf::new(), raw_definition)?;

        Ok(())
    }

    #[test]
    fn test_different_versions_definition_template() -> Result<(), Box<dyn std::error::Error>> {
        let template = TemplateDefinition {
            version: 2,
            eeprom_data_version: 1,
            gpiobanks: vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )],
        };

        let raw_definition = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 1,
            vstr: String::new(),
            pstr: String::new(),
            pid: 1,
            prev: 1,
            pver: 1,
            dtstr: String::new(),
            serial: None,
            edate: None,
            mac: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Object(template)),
        };

        RevPiHatEeprom::from_raw_definition(&PathBuf::new(), raw_definition).unwrap_err();

        Ok(())
    }

    #[test]
    fn test_redundant_template() -> Result<(), Box<dyn std::error::Error>> {
        let template = TemplateDefinition {
            version: 1,
            eeprom_data_version: 1,
            gpiobanks: vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )],
        };

        let raw_definition = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 1,
            vstr: String::new(),
            pstr: String::new(),
            pid: 1,
            prev: 1,
            pver: 1,
            dtstr: String::new(),
            serial: None,
            edate: None,
            mac: None,
            gpiobanks: Some(vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )]),
            include: Some(TemplateInclude::Object(template)),
        };

        RevPiHatEeprom::from_raw_definition(&PathBuf::new(), raw_definition).unwrap_err();

        Ok(())
    }

    #[sealed_test]
    fn test_templates_folder() -> Result<(), Box<dyn std::error::Error>> {
        let expected = RevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )],
        };
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Filename("test.json".into())),
        };
        let template = r#"
        {
            "version": 1,
            "eeprom_data_version": 3,
            "gpiobanks": [
                {
                    "drive": "8mA",
                    "slew": "default",
                    "hysteresis": "default",
                    "gpios": []
                }
            ]
        }
        "#;
        create_dir("templates")?;
        fs::write("templates/test.json", template)?;

        let eep = RevPiHatEeprom::from_raw_definition(
            &std::env::current_dir()?.join("templates"),
            raw_config,
        )?;

        assert_eq!(eep, expected);

        Ok(())
    }

    #[test]
    fn test_raw_without_template() -> Result<(), Box<dyn std::error::Error>> {
        let expected = RevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )],
        };
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: Some(vec![GpioBank::new(
                gpio::GpioBankDrive::Drive8mA,
                gpio::GpioBankSlew::Default,
                gpio::GpioBankHysteresis::Default,
                vec![],
            )]),
            include: None,
        };
        assert_eq!(
            RevPiHatEeprom::from_raw_definition(&PathBuf::new(), raw_config)?,
            expected
        );

        Ok(())
    }

    #[test]
    fn test_raw_no_gpiobanks() -> Result<(), Box<dyn std::error::Error>> {
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: None,
            include: None,
        };
        RevPiHatEeprom::from_raw_definition(&PathBuf::new(), raw_config).unwrap_err();

        Ok(())
    }

    #[sealed_test]
    fn test_template_dir_not_present() -> Result<(), Box<dyn std::error::Error>> {
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Filename("nonexistent.json".into())),
        };
        RevPiHatEeprom::from_raw_definition(
            &std::env::current_dir()?.join("non-existent"),
            raw_config,
        )
        .unwrap_err();

        Ok(())
    }

    #[sealed_test]
    fn test_template_not_present() -> Result<(), Box<dyn std::error::Error>> {
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Filename("nonexistent.json".into())),
        };
        let templates_path = std::env::current_dir()?.join("templates");
        create_dir(&templates_path)?;
        RevPiHatEeprom::from_raw_definition(&templates_path, raw_config).unwrap_err();

        Ok(())
    }

    #[sealed_test]
    fn test_empty_template() -> Result<(), Box<dyn std::error::Error>> {
        let raw_config = RawRevPiHatEeprom {
            version: 1,
            eeprom_data_version: 3,
            vstr: "KUNBUS GmbH".to_string(),
            pstr: "RevPi Test".to_string(),
            pid: 666,
            prev: 3,
            pver: 333,
            dtstr: "revpi-test".to_string(),
            edate: None,
            mac: None,
            serial: None,
            gpiobanks: None,
            include: Some(TemplateInclude::Filename("empty.json".into())),
        };
        let templates_path = std::env::current_dir()?.join("templates");
        create_dir(&templates_path)?;
        std::fs::write(templates_path.join("empty.json"), "")?;
        RevPiHatEeprom::from_raw_definition(&templates_path, raw_config).unwrap_err();

        Ok(())
    }
}
