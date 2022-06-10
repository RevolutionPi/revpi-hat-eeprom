use clap::{Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The UUID for the device, will be calculated from the pid, pver, prev
    /// and serial if omitted.
    #[clap(long)]
    uuid: Option<String>,
    /// The product ID of the device.
    #[clap(long)]
    pid: u16,
    /// The product version of device.
    #[clap(long)]
    pver: u16,
    /// The product revision of the device.
    #[clap(long)]
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
    #[clap(long)]
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
