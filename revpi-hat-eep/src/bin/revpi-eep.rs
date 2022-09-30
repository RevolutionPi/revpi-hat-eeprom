// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use chrono::NaiveDate;
use clap::Parser;
use eui48::MacAddress;
use revpi_hat_eep::RevPiHatEeprom;
use rpi_hat_eep::{gpio_map, Eep, EepAtom, EepAtomCustomData, ToBytes};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process;

// Disable manual_strip Clippy warning.
// In parse_prefixed_int() it is not possible to do it the suggested way.
#[allow(clippy::manual_strip)]

/// Convert a string slice to an integer, the base is determind from the prefix.
///
/// The string may contain 0b (for binary), 0o (for octal), 0x (for hex) or no
/// prefix (for decimal) values.
/// # Examples
///
/// ```
/// assert_eq!(parse_prefixed_int("0xA"), Ok(10));
/// ```
fn parse_prefixed_int<T>(src: &str) -> Result<T, String>
where
    T: num::Unsigned + num::Num<FromStrRadixErr = std::num::ParseIntError>,
{
    let val = if src.starts_with("0b") {
        T::from_str_radix(&src[2..], 2)
    } else if src.starts_with("0o") {
        T::from_str_radix(&src[2..], 8)
    } else if src.starts_with("0x") {
        T::from_str_radix(&src[2..], 16)
    } else {
        T::from_str_radix(src, 10)
    };
    match val {
        Ok(val) => Ok(val),
        Err(e) => Err(format!("{e}")),
    }
}

#[test]
fn test_parse_prefixed_int() {
    assert_eq!(parse_prefixed_int::<u8>("0xA"), Ok(10));
    assert_eq!(parse_prefixed_int::<u16>("0xA"), Ok(10));
    assert_eq!(parse_prefixed_int::<u32>("0xA"), Ok(10));
    assert_eq!(parse_prefixed_int::<u64>("0xA"), Ok(10));
    assert_eq!(parse_prefixed_int("0b1010"), Ok(10u16));
    assert_eq!(parse_prefixed_int("0o12"), Ok(10u16));
    assert_eq!(parse_prefixed_int("10"), Ok(10u16));
    assert_eq!(parse_prefixed_int("0"), Ok(0u16));
    assert_eq!(parse_prefixed_int("010"), Ok(10u16));
    assert_eq!(parse_prefixed_int("0xffff"), Ok(u16::MAX));
    assert_eq!(parse_prefixed_int("0xffffffff"), Ok(u32::MAX));
    assert_eq!(parse_prefixed_int("0xffffffffffffffff"), Ok(u64::MAX));
    assert_eq!(
        parse_prefixed_int::<u16>("0x10000"),
        Err("number too large to fit in target type".to_string())
    );
    assert_eq!(
        parse_prefixed_int::<u16>("-1"),
        Err("invalid digit found in string".to_string())
    );
}

fn calc_uuid(pid: u16, pver: u16, prev: u16, serial: u32) -> uuid::Uuid {
    let mut bytes: Vec<u8> = Vec::with_capacity(10);
    bytes.extend_from_slice(&u16::to_le_bytes(pid));
    bytes.extend_from_slice(&u16::to_le_bytes(pver));
    bytes.extend_from_slice(&u16::to_le_bytes(prev));
    bytes.extend_from_slice(&u32::to_le_bytes(serial));
    let digest = md5::compute(&bytes);
    uuid::Builder::from_md5_bytes(*digest).into_uuid()
}

fn create_rpi_eep(config: RevPiHatEeprom) -> Result<rpi_hat_eep::Eep, Box<dyn std::error::Error>> {
    let serial = config
        .serial
        .expect("BUG: Missing serial in RevPiHatEeprom configuration");
    let edate = config
        .edate
        .expect("BUG: Missing end test date in RevPiHatEeprom configuration");
    let mac = config
        .mac
        .expect("BUG: Missing mac address in RevPiHatEeprom confirguration");

    let uuid = calc_uuid(config.pid, config.pver, config.prev, serial);
    let vendor_data = rpi_hat_eep::EepAtomVendorData::new(
        uuid,
        config.pid,
        config.pver,
        config.vstr,
        config.pstr,
    )?;

    let gpio_map: gpio_map::EepAtomGpioMapData = config.gpiobanks[0].clone().try_into()?;

    let mut eep = Eep::new(vendor_data, gpio_map);

    let dtb = rpi_hat_eep::EepAtomLinuxDTBData::new(rpi_hat_eep::LinuxDTB::Name(config.dtstr));
    eep.push(EepAtom::new_linux_dtb(dtb))?;

    let data = EepAtomCustomData::new(config.version.to_string().into_bytes());
    eep.push(EepAtom::new_custom(data))?;

    let data = EepAtomCustomData::new(serial.to_string().into_bytes());
    eep.push(EepAtom::new_custom(data))?;

    let data = EepAtomCustomData::new(config.prev.to_string().into_bytes());
    eep.push(EepAtom::new_custom(data))?;

    let data = EepAtomCustomData::new(edate.to_string().into_bytes());
    eep.push(EepAtom::new_custom(data))?;

    let data = EepAtomCustomData::new("0".as_bytes().to_vec());
    eep.push(EepAtom::new_custom(data))?;

    let data = EepAtomCustomData::new(
        mac.to_string(eui48::MacAddressFormat::HexString)
            .into_bytes(),
    );
    eep.push(EepAtom::new_custom(data))?;

    Ok(eep)
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The serial number for the device. It is mandatory if the serial is not included in the
    /// config file. This option will override the serial from the config file.
    #[clap(long, value_parser = parse_prefixed_int::<u32>)]
    pub serial: Option<u32>,
    /// The end test date for the device. In the format YYYY-MM-DD (ISO8601/RFC3339). If omitted the
    /// current date is used. This option will override a given edate attribute from the config file.
    #[clap(long)]
    pub edate: Option<NaiveDate>,
    /// The (first) mac address of the device. It is mandatory if the mac is not included in the
    /// config file. This option will override the mac from the config file.
    #[clap(long)]
    pub mac: Option<MacAddress>,
    /// Full json configuration export file name. The full json configuration includes also the
    /// serial, edate and mac.
    #[clap(long, value_parser, value_name = "EXPORT_CONFIG")]
    pub export: Option<PathBuf>,
    /// Configuration file in JSON format
    #[clap(value_parser, value_name = "CONFIG")]
    pub config: PathBuf,
    /// Output file name
    #[clap(value_parser, value_name = "OUTPUT", default_value = "out.eep")]
    pub outfile_name: PathBuf,
}

fn export_config(config: &RevPiHatEeprom, export_path: PathBuf) {
    let json = serde_json::to_string(config)
        .expect("BUG: Can't create (full) json from RevPiHatEeprom config");
    let mut export_file = match OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&export_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "ERROR: Can't open json export file: `{}': {e}",
                export_path.to_string_lossy()
            );
            process::exit(1);
        }
    };
    if let Err(e) = export_file.write_all(json.as_bytes()) {
        eprintln!(
            "ERROR: Can't write to file `{}`: {e}",
            export_path.to_string_lossy()
        );
        process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();

    let config = match std::fs::read_to_string(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "ERROR: Can't read config file `{}': {e}",
                cli.config.to_string_lossy()
            );
            process::exit(1)
        }
    };

    let mut config = match revpi_hat_eep::parse_config(&config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "ERROR: Invalid config file `{}': {e}",
                cli.config.to_string_lossy(),
            );
            process::exit(1);
        }
    };

    let serial = if let Some(serial_cli) = cli.serial {
        if let Some(serial_config) = config.serial {
            eprintln!(
                "WARNING: Overriding serial from the config file (`{}`) \
                with the serial from the program arguments (`{}`).",
                serial_config,
                serial_cli
            );
        }
        serial_cli
    } else if let Some(serial_config) = config.serial {
        serial_config
    } else {
        eprintln!("ERROR: The `serial` was neither specified as argument nor in the config file.");
        process::exit(1);
    };

    let edate = if let Some(edate_cli) = cli.edate {
        if let Some(edate_config) = config.edate {
            eprintln!(
                "WARNING: Overriding edate from the config file (`{}`) \
                with the edate from the program arguments (`{}`).",
                edate_config,
                edate_cli
            )
        }
        edate_cli
    } else if let Some(edate_config) = config.edate {
        edate_config
    } else {
        chrono::Local::today().naive_local()
    };

    let mac = if let Some(mac_cli) =  cli.mac {
        if let Some(mac_config) = config.mac {
            eprintln!(
                "WARNING: Overriding mac from the config file (`{}`) \
                with the mac from the program arguments (`{}`).",
                mac_config.to_hex_string(),
                mac_cli.to_hex_string()
            );
        }
        mac_cli
    } else if let Some(mac_config) = config.mac {
        mac_config
    } else {
        eprintln!("ERROR: The `mac` was neither specified as argument nor in the config file.");
        process::exit(1);
    };

    config.serial = Some(serial);
    config.edate = Some(edate);
    config.mac = Some(mac);

    if let Some(export_path) = cli.export {
        export_config(&config, export_path)
    };

    let eep = match create_rpi_eep(config) {
        Ok(eep) => eep,
        Err(e) => {
            eprintln!("Error: Can't create EEP: {e}");
            process::exit(1);
        }
    };
    let mut buf: Vec<u8> = Vec::new();
    eep.to_bytes(&mut buf);

    let mut output_file = match OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&cli.outfile_name)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "ERROR: Can't open output file: `{}': {e}",
                cli.outfile_name.to_string_lossy()
            );
            process::exit(1);
        }
    };

    if let Err(e) = output_file.write_all(&buf) {
        eprintln!(
            "ERROR: Can't write data to the output file: `{}': {e}",
            cli.outfile_name.to_string_lossy()
        );
        process::exit(1);
    }
}
