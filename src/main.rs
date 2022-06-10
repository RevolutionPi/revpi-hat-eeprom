use clap::{Parser};

/// Convert a string slice to an integer, the base is determind from the prefix.
///
/// The string may contain 0b (for binary), 0o (for octal), 0x (for hex) or no
/// prefix (for decimal) values.
/// # Examples
///
/// ```
/// assert_eq!(parse_u16("0xA"), Ok(10));
/// ```
fn parse_u16(src: &str) -> Result<u16, String> {
    let val = if src.starts_with("0b") {
        u16::from_str_radix(&src[2..], 2)
    } else if src.starts_with("0o") {
        u16::from_str_radix(&src[2..], 8)
    } else if src.starts_with("0x") {
        u16::from_str_radix(&src[2..], 16)
    } else {
        src.parse()
    };
    match val {
        Ok(val) => Ok(val),
        Err(e) => Err(format!("{e}"))
    }
}

#[test]
fn test_parse_u16() {
    assert_eq!(parse_u16("0xA"), Ok(10));
    assert_eq!(parse_u16("0b1010"), Ok(10));
    assert_eq!(parse_u16("0o12"), Ok(10));
    assert_eq!(parse_u16("10"), Ok(10));
    assert_eq!(parse_u16("0"), Ok(0));
    assert_eq!(parse_u16("010"), Ok(10));
    assert_eq!(parse_u16("0xffff"), Ok(u16::MAX));
    assert_eq!(parse_u16("0x10000"), Err("number too large to fit in target type".to_string()));
    assert_eq!(parse_u16("-1"), Err("invalid digit found in string".to_string()));
}

/// Convert a string slice to an integer, the base is determind from the prefix.
///
/// The string may contain 0b (for binary), 0o (for octal), 0x (for hex) or no
/// prefix (for decimal) values.
/// # Examples
///
/// ```
/// assert_eq!(parse_u16("0xA"), Ok(10));
/// ```
fn parse_u32(src: &str) -> Result<u32, String> {
    let val = if src.starts_with("0b") {
        u32::from_str_radix(&src[2..], 2)
    } else if src.starts_with("0o") {
        u32::from_str_radix(&src[2..], 8)
    } else if src.starts_with("0x") {
        u32::from_str_radix(&src[2..], 16)
    } else {
        src.parse()
    };
    match val {
        Ok(val) => Ok(val),
        Err(e) => Err(format!("{e}"))
    }
}

#[test]
fn test_parse_u32() {
    assert_eq!(parse_u32("0xA"), Ok(10));
    assert_eq!(parse_u32("0b1010"), Ok(10));
    assert_eq!(parse_u32("0o12"), Ok(10));
    assert_eq!(parse_u32("10"), Ok(10));
    assert_eq!(parse_u32("0"), Ok(0));
    assert_eq!(parse_u32("010"), Ok(10));
    assert_eq!(parse_u32("0xffffffff"), Ok(u32::MAX));
    assert_eq!(parse_u32("0x100000000"), Err("number too large to fit in target type".to_string()));
    assert_eq!(parse_u32("-1"), Err("invalid digit found in string".to_string()));
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
    uuid: Option<String>,
    /// The product ID of the device.
    #[clap(long, parse(try_from_str = parse_u16))]
    pid: u16,
    /// The product version of device.
    #[clap(long, parse(try_from_str = parse_u16))]
    pver: u16,
    /// The product revision of the device.
    #[clap(long, parse(try_from_str = parse_u16))]
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
    #[clap(long, parse(try_from_str = parse_u32))]
    serial: u32,
    /// The end test date for the device. In the format YYYY-MM-DD (ISO8601/RFC3339). If omitted the current date is used.
    #[clap(long, parse(try_from_str = parse_date_iso8601))]
    edate: Option<chrono::NaiveDate>,
    /// The (first) mac address of the device.
    #[clap(long)]
    mac: String,
}

fn main() {
    let cli = Cli::parse();

    let edate = match cli.edate {
        Some(edate) => edate,
        None => chrono::Local::today().naive_local()
    };
}
