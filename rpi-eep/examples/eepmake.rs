// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

extern crate rpi_eep;

use rpi_eep::{gpio_map, ToBytes};
use rpi_eep::{gpio_map::GpioPin, EEPAtom, LinuxDTB, EEP};
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn vendor_atom(config: &EEPConfig) -> EEPAtom {
    let uuid = config.uuid.unwrap_or_else(uuid::Uuid::new_v4);
    let pid = match config.pid {
        Some(pid) => pid,
        None => panic!("ERROR: product_id is missing!"),
    };
    let pver = match config.pver {
        Some(pver) => pver,
        None => panic!("ERROR: product_ver is missing!"),
    };
    let vstr = match &config.vstr {
        Some(vstr) => vstr.clone(),
        None => panic!("ERROR: vendor string missing!"),
    };
    let pstr = match &config.pstr {
        Some(pstr) => pstr.clone(),
        None => panic!("ERROR: product string missing!"),
    };
    let data = rpi_eep::EEPAtomVendorData {
        uuid,
        pid,
        pver,
        vstr,
        pstr,
    };

    EEPAtom::new_vendor_info(data)
}

fn gpio_map_atom(config: &EEPConfig) -> EEPAtom {
    let drive = match config.gpio_drive {
        Some(drive) => drive,
        None => {
            eprintln!("WARNING: required field `gpio_drive' missing, using default value");
            gpio_map::GpioDrive::Default
        }
    };
    let slew = match config.gpio_slew {
        Some(slew) => slew,
        None => {
            eprintln!("WARNING: required field `gpio_slew' missing, using default value");
            gpio_map::GpioSlew::Default
        }
    };
    let hyst = match config.gpio_hyst {
        Some(hyst) => hyst,
        None => {
            eprintln!("WARNING: required field `gpio_hysteresis' missing, using default value");
            gpio_map::GpioHysteresis::Default
        }
    };
    let power = match config.back_power {
        Some(power) => power,
        None => {
            eprintln!("WARNING: required field `back_power' missing, using default value");
            gpio_map::GpioBackPower::None
        }
    };
    let mut gpio_map = gpio_map::EEPAtomGpioMapData::new(drive, slew, hyst, power);

    for gpio in &config.gpios {
        gpio_map.push(gpio.clone());
    }
    EEPAtom::new_gpio_map(gpio_map)
}

struct EEPConfig {
    uuid: Option<uuid::Uuid>,
    pid: Option<u16>,
    pver: Option<u16>,
    vstr: Option<String>,
    pstr: Option<String>,
    gpio_drive: Option<gpio_map::GpioDrive>,
    gpio_slew: Option<gpio_map::GpioSlew>,
    gpio_hyst: Option<gpio_map::GpioHysteresis>,
    back_power: Option<gpio_map::GpioBackPower>,
    gpios: Vec<gpio_map::GpioPin>,
    dtb: Option<rpi_eep::LinuxDTB>,
    custom: Vec<Vec<u8>>,
}

impl Default for EEPConfig {
    fn default() -> Self {
        EEPConfig {
            uuid: None,
            pid: None,
            pver: None,
            vstr: None,
            pstr: None,
            gpio_drive: None,
            gpio_slew: None,
            gpio_hyst: None,
            back_power: None,
            gpios: vec![GpioPin::default(); 28],
            dtb: None,
            custom: Vec::new(),
        }
    }
}

fn usage(code: i32) {
    println!(
        "USAGE: {} input_file output_file [dt_file] [-c  custom_file_1 ... custom_file_n]",
        env::args().next().unwrap()
    );
    exit(code)
}

fn parse_line_string(line: &str) -> String {
    let idx = line.find(|c: char| c.is_whitespace()).unwrap();
    let tmp = &line[idx..].trim_start();
    let vstr = tmp.trim_start_matches('"').trim_end_matches('"');
    vstr.to_string()
}

fn parse_line_dec_u8(line: &str) -> u8 {
    let mut iter = line.split_whitespace();
    iter.next();
    iter.next().unwrap().parse::<u8>().unwrap()
}

fn parse_line_hex_u16(line: &str) -> u16 {
    let mut iter = line.split_whitespace();
    iter.next();
    u16::from_str_radix(iter.next().unwrap().trim_start_matches("0x"), 16).unwrap()
}

fn parse_config(eep_config: &mut EEPConfig, config_str: &str) {
    let mut custom_data_str: Option<String> = None;
    for mut line in config_str.lines() {
        line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if custom_data_str.is_some() {
            if line.starts_with("end") {
                eep_config
                    .custom
                    .extend(hex::decode(custom_data_str.unwrap()));
                custom_data_str = None;
                continue;
            }
            let mut data = custom_data_str.unwrap();
            for c in line.chars() {
                if c.is_ascii_whitespace() {
                    continue;
                }
                data.push(c);
            }
            custom_data_str = Some(data);
            continue;
        }
        if line.starts_with("custom_data") {
            let mut data = String::new();
            let arg = line.trim_start_matches("custom_data").trim_start();
            if !arg.is_empty() {
                data.push_str(arg);
            }
            custom_data_str = Some(data);
        } else if line.starts_with("product_uuid") {
            let arg = line.trim_start_matches("product_uuid").trim_start();
            let uuid = match uuid::Uuid::parse_str(arg) {
                Ok(uuid) => {
                    if uuid == uuid::uuid!("00000000-0000-0000-0000-000000000000") {
                        None
                    } else {
                        Some(uuid)
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Can't parse uuid: {e}");
                    None
                }
            };
            eep_config.uuid = uuid;
        } else if line.starts_with("product_id") {
            eep_config.pid = Some(parse_line_hex_u16(line));
        } else if line.starts_with("product_ver") {
            eep_config.pver = Some(parse_line_hex_u16(line));
        } else if line.starts_with("vendor") {
            eep_config.vstr = Some(parse_line_string(line));
        } else if line.starts_with("product") {
            eep_config.pstr = Some(parse_line_string(line));
        } else if line.starts_with("gpio_drive") {
            eep_config.gpio_drive = num::FromPrimitive::from_u8(parse_line_dec_u8(line));
        } else if line.starts_with("gpio_slew") {
            eep_config.gpio_slew = num::FromPrimitive::from_u8(parse_line_dec_u8(line));
        } else if line.starts_with("gpio_hysteresis") {
            eep_config.gpio_hyst = num::FromPrimitive::from_u8(parse_line_dec_u8(line));
        } else if line.starts_with("back_power") {
            eep_config.back_power = num::FromPrimitive::from_u8(parse_line_dec_u8(line));
        } else if line.starts_with("setgpio") {
            let arg = line.trim_start_matches("setgpio").trim_start();
            let chunks: Vec<&str> = arg.split_ascii_whitespace().collect();
            let gpio: usize = chunks[0].parse().expect("Bad GPIO pin number!");
            let func = match chunks[1] {
                "INPUT" => Some(gpio_map::GpioFsel::Input),
                "OUTPUT" => Some(gpio_map::GpioFsel::Output),
                "ALT0" => Some(gpio_map::GpioFsel::Alt0),
                "ALT1" => Some(gpio_map::GpioFsel::Alt1),
                "ALT2" => Some(gpio_map::GpioFsel::Alt2),
                "ALT3" => Some(gpio_map::GpioFsel::Alt3),
                "ALT4" => Some(gpio_map::GpioFsel::Alt4),
                "ALT5" => Some(gpio_map::GpioFsel::Alt5),
                _ => None,
            }
            .unwrap();
            let pull = match chunks[2] {
                "DEFAULT" => Some(gpio_map::GpioPull::Default),
                "UP" => Some(gpio_map::GpioPull::Up),
                "DOWN" => Some(gpio_map::GpioPull::Down),
                "NONE" => Some(gpio_map::GpioPull::NoPull),
                _ => None,
            }
            .unwrap();
            eep_config.gpios[gpio] = gpio_map::GpioPin::new(func, pull, false);
            println!("SETGPIO: {} {:?}", gpio, eep_config.gpios[gpio]);
        } else {
            eprintln!("UNKNOWN");
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        usage(-1);
    }

    let input_file_name = PathBuf::from(&args[1]);
    let output_file_name = PathBuf::from(&args[2]);

    let mut input_file = match OpenOptions::new()
        .read(true)
        .write(false)
        .truncate(false)
        .create(false)
        .open(&input_file_name)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "ERROR: Can't open input file: `{}': {e}",
                input_file_name.to_str().unwrap()
            );
            exit(-1);
        }
    };
    let mut config_string = String::new();
    let _ = input_file.read_to_string(&mut config_string);

    let mut eep_config = EEPConfig::default();
    parse_config(&mut eep_config, &config_string);

    if args.len() > 3 {
        if args[3].ne("-c") {
            let dt_file_name = PathBuf::from(&args[3]);
            let mut dt_file = match OpenOptions::new()
                .read(true)
                .write(false)
                .open(&dt_file_name)
            {
                Ok(file) => file,
                Err(e) => {
                    eprintln!(
                        "Error: Can't open dt_file: `{}'': {e}",
                        dt_file_name.to_str().unwrap()
                    );
                    exit(-1);
                }
            };
            let mut buf = Vec::new();
            let _ = dt_file.read_to_end(&mut buf);
            eep_config.dtb = Some(LinuxDTB::Blob(buf));
        } else {
            eep_config.dtb = None;
        };

        if args.len() > 4 {
            for f in &args[4..] {
                if f.eq("-c") {
                    continue;
                }
                let path = PathBuf::from(f);
                let mut file = match File::open(path) {
                    Ok(file) => file,
                    Err(e) => {
                        eprintln!("ERROR: Can't open file: `{}': {e}", f);
                        exit(-1);
                    }
                };
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf);
                eep_config.custom.push(buf);
            }
        }
    }

    let mut eep = EEP::new();
    eep.push(vendor_atom(&eep_config));
    eep.push(gpio_map_atom(&eep_config));

    if eep_config.dtb.is_some() {
        let data = rpi_eep::EEPAtomLinuxDTBData::new(eep_config.dtb.unwrap());
        eep.push(EEPAtom::new_linux_dtb(data));
    }

    for data in eep_config.custom {
        let data = rpi_eep::EEPAtomCustomData::new(data);
        eep.push(EEPAtom::new_custom(data));
    }

    //println!("eeplen: {}", eep.len());
    let mut buf: Vec<u8> = Vec::with_capacity(eep.len());
    eep.to_bytes(&mut buf);

    let mut output_file = match OpenOptions::new()
        .read(false)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&output_file_name)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "ERROR: Can't open output file: `{}': {e}",
                output_file_name.to_str().unwrap()
            );
            exit(-1);
        }
    };

    let _ = output_file.write(&buf);
}
