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
    #[clap(long, default_value = "Kunbus GmbH")]
    vstr: String,
    /// The product string for the device.
    #[clap(long)]
    pstr: String,
    /// The device tree overlay name for the device.
    #[clap(long)]
    dtstr: String,
    /// The serial number for the device.
    #[clap(long, parse(try_from_str = parse_u32))]
    serial: u32,
    /// The end test date for the device. In the format YYYY-MM-DD (ISO8601/RFC3339). If omitted the current date is used.
    #[clap(long)]
    edate: Option<String>,
    /// The (first) mac address of the device.
    #[clap(long)]
    mac: String,
}

fn main() {
    let cli = Cli::parse();
}
