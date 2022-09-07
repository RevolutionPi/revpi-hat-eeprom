// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use chrono::NaiveDate;
use clap::Parser;
use eui48::MacAddress;
use revpi_hat_eep::error::RevPiError;
use revpi_hat_eep::RevPiHatEeprom;
use rpi_hat_eep::{gpio_map, EepAtom, EepAtomCustomData, ToBytes, Eep};
use std::error::Error;
use std::fs::File;
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

/// Parse and validate a string for a date of the format YYYY-MM-DD (ISO8601/RFC3339).
///
/// Parse a string of the form YYYY-MM-DD (ISO8601/RFC3339) and return a
/// chrono::NaiveDate struct.
///
/// # EXAMPLES
/// ```
/// assert_eq!(parse_date_iso8601("2022-03-15"), Ok(chrono::NaiveDate::from_ymd(2022, 3, 15)));
/// ```
fn parse_date_iso8601(src: &str) -> Result<chrono::NaiveDate, String> {
    let date = NaiveDate::parse_from_str(src, "%F");
    match date {
        Ok(date) => Ok(date),
        Err(e) => Err(format!("{e}")),
    }
}

#[test]
fn test_parse_date_rfc3339() {
    assert_eq!(
        parse_date_iso8601("2022-03-15"),
        Ok(NaiveDate::from_ymd(2022, 3, 15))
    );
    assert_eq!(
        parse_date_iso8601("2022-3-15"),
        Ok(NaiveDate::from_ymd(2022, 3, 15))
    );
    assert_eq!(
        parse_date_iso8601("2O22-03-15"),
        Err("input contains invalid characters".to_string())
    );
    assert_eq!(
        parse_date_iso8601("2022-030-15"),
        Err("input contains invalid characters".to_string())
    );
    assert_eq!(
        parse_date_iso8601("2022-13-15"),
        Err("input is out of range".to_string())
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

fn create_rpi_eep(
    config: RevPiHatEeprom,
    serial: u32,
    edate: NaiveDate,
    mac: MacAddress,
) -> Result<rpi_hat_eep::Eep, RevPiError> {
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
    /// The serial number for the device.
    #[clap(long, parse(try_from_str = parse_prefixed_int))]
    pub serial: u32,
    /// The end test date for the device. In the format YYYY-MM-DD (ISO8601/RFC3339). If omitted the current date is used.
    #[clap(long, parse(try_from_str = parse_date_iso8601))]
    pub edate: Option<chrono::NaiveDate>,
    /// The (first) mac address of the device.
    #[clap(long)]
    pub mac: MacAddress,
    /// Configuration file in JSON format
    #[clap(value_parser, value_name = "CONFIG")]
    pub config: PathBuf,
    /// Output file name
    #[clap(value_parser, value_name = "OUTPUT", default_value = "out.eep")]
    pub outfile: PathBuf,
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

    let config = match revpi_hat_eep::parse_config(&config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "ERROR: Invalid config file `{}': {e}: {}",
                cli.config.to_string_lossy(),
                e.source().unwrap()
            );
            process::exit(1);
        }
    };

    match config.validate() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("ERROR: Invalid config: {e}: {}", e);
            process::exit(1);
        }
    }

    let _outfile = match File::create(&cli.outfile) {
        Ok(outfile) => outfile,
        Err(e) => {
            eprintln!(
                "ERROR: Can't create file `{}`: {e}",
                cli.outfile.to_string_lossy()
            );
            process::exit(1)
        }
    };

    let edate = match cli.edate {
        Some(edate) => edate,
        None => chrono::Local::today().naive_local(),
    };

    let eep = create_rpi_eep(config, cli.serial, edate, cli.mac).unwrap();
}
