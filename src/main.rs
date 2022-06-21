use clap::{Parser};
use eui48::MacAddress;
use uuid::{Uuid, Builder};

/// Convert a string slice to an integer, the base is determind from the prefix.
///
/// The string may contain 0b (for binary), 0o (for octal), 0x (for hex) or no
/// prefix (for decimal) values.
/// # Examples
///
/// ```
/// assert_eq!(parse_prefixed_int("0xA"), Ok(10));
/// ```
fn parse_prefixed_int<T: num::Unsigned>(src: &str) -> Result<T, String>
    where T: num::Num<FromStrRadixErr = std::num::ParseIntError>,
{
    let val = if src.starts_with("0b") {
        T::from_str_radix(&src[2..], 2)
    } else if src.starts_with("0o") {
        T::from_str_radix(&src[2..], 8)
    } else if src.starts_with("0x") {
        T::from_str_radix(&src[2..], 16)
    } else {
        T::from_str_radix(&src, 10)
    };
    match val {
        Ok(val) => Ok(val),
        Err(e) => Err(format!("{e}"))
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
    assert_eq!(parse_prefixed_int::<u16>("0x10000"), Err("number too large to fit in target type".to_string()));
    assert_eq!(parse_prefixed_int::<u16>("-1"), Err("invalid digit found in string".to_string()));
}

/// Validate a string to be max 255 bytes long.
///
/// Check if a given string can fit into a 255 byte buffer.
///
/// # Examples
/// ```
/// assert_eq!(parse_string_max255("foo bar"), Ok("foo bar".to_string()));
/// ```
fn parse_string_max255(src: &str) -> Result<String, String> {
    if src.as_bytes().len() >= 256 {
        Err("string to long to fit into target memory (max 255 chars/bytes)".to_string())
    } else {
        Ok(src.to_string())
    }
}

#[test]
fn test_parse_string_max255() {
    assert_eq!(parse_string_max255("foo bar"), Ok("foo bar".to_string()));
    assert_eq!(parse_string_max255("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Duis ut diam quam nulla porttitor massa id neque. Facilisis volutpat est velit egestas dui id ornare arcu. Hac habitasse platea dict"),
               Err("string to long to fit into target memory (max 255 chars/bytes)".to_string()));
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
fn parse_date_iso8601(src: &str) -> Result<chrono::NaiveDate, String>
{
    let parse_from_str = chrono::NaiveDate::parse_from_str;
    let date = parse_from_str(src, "%F");
    match date {
        Ok(date) => Ok(date),
        Err(e) => Err(format!("{e}"))
    }
}

#[test]
fn test_parse_date_rfc3339() {
    assert_eq!(parse_date_iso8601("2022-03-15"), Ok(chrono::NaiveDate::from_ymd(2022, 3, 15)));
    assert_eq!(parse_date_iso8601("2022-3-15"), Ok(chrono::NaiveDate::from_ymd(2022, 3, 15)));
    assert_eq!(parse_date_iso8601("2O22-03-15"), Err("input contains invalid characters".to_string()));
    assert_eq!(parse_date_iso8601("2022-030-15"), Err("input contains invalid characters".to_string()));
    assert_eq!(parse_date_iso8601("2022-13-15"), Err("input is out of range".to_string()));
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The UUID for the device, will be calculated from the pid, pver, prev
    /// and serial if omitted.
    #[clap(long)]
    uuid: Option<Uuid>,
    /// The product ID of the device.
    #[clap(long, parse(try_from_str = parse_prefixed_int))]
    pid: u16,
    /// The product version of device.
    #[clap(long, parse(try_from_str = parse_prefixed_int))]
    pver: u16,
    /// The product revision of the device.
    #[clap(long, parse(try_from_str = parse_prefixed_int))]
    prev: u16,
    /// The vendor string for the device.
    #[clap(long, default_value = "Kunbus GmbH", parse(try_from_str = parse_string_max255))]
    vstr: String,
    /// The product string for the device.
    #[clap(long, parse(try_from_str = parse_string_max255))]
    pstr: String,
    /// The device tree overlay name for the device.
    #[clap(long)]
    dtstr: String,
    /// The serial number for the device.
    #[clap(long, parse(try_from_str = parse_prefixed_int))]
    serial: u32,
    /// The end test date for the device. In the format YYYY-MM-DD (ISO8601/RFC3339). If omitted the current date is used.
    #[clap(long, parse(try_from_str = parse_date_iso8601))]
    edate: Option<chrono::NaiveDate>,
    /// The (first) mac address of the device.
    #[clap(long)]
    mac: MacAddress,
}

fn main() {
    let cli = Cli::parse();

    let edate = match cli.edate {
        Some(edate) => edate,
        None => chrono::Local::today().naive_local()
    };

    let uuid = match cli.uuid {
        Some(uuid) => uuid,
        None => {
            let mut bytes: Vec<u8> = Vec::with_capacity(10);
            bytes.extend_from_slice(&u16::to_le_bytes(cli.pid));
            bytes.extend_from_slice(&u16::to_le_bytes(cli.pver));
            bytes.extend_from_slice(&u16::to_le_bytes(cli.prev));
            bytes.extend_from_slice(&u32::to_le_bytes(cli.serial));
            let digest = md5::compute(&bytes);
            Builder::from_md5_bytes(*digest).into_uuid()
        }
    };

    println!("PID:    {:#04x}", cli.pid);
    println!("PVER:   {:#04x}", cli.pver);
    println!("PREV:   {:02}", cli.prev);
    println!("VSTR:   {}", cli.vstr);
    println!("PSTR:   {}", cli.pstr);
    println!("DTSTR:  {}", cli.dtstr);
    println!("SERIAL: {}", cli.serial);
    println!("EDATE:  {}", edate);
    println!("MAC:    {}", cli.mac);
    println!("UUID:   {}", uuid);
}
