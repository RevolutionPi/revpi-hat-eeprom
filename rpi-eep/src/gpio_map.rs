// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

const MAX_GPIOS: usize = 28;

use crate::ToBuffer;

/// 0=leave at default, 1-8=drive*2mA, 9-15=reserved
#[derive(Clone, Copy, Debug)]
enum GpioDrive {
    Default = 0,
    Drive2mA = 1,
    Drive4mA = 2,
    Drive6mA = 3,
    Drive8mA = 4,
    Drive10mA = 5,
    Drive12mA = 6,
    Drive14mA = 7,
    Drive16mA = 8,
}

/// 0=leave at default, 1=slew rate limiting, 2=no slew limiting, 3=reserved
#[derive(Clone, Copy, Debug)]
enum GpioSlew {
    /// leave at default
    Default = 0,
    /// slew rate limiting
    RateLimiting = 1,
    /// no slew limiting
    NoLimit = 2,
}

/// 0=leave at default, 1=hysteresis disabled, 2=hysteresis enabled, 3=reserved
#[derive(Clone, Copy, Debug)]
enum GpioHysteresis {
    /// leave at default
    Default = 0,
    /// hysteresis disabled
    Disable = 1,
    /// hysteresis enabled
    Enable = 2,
}
/// defines if the board backpowers the Pi
///
/// ```text
/// 0=board does not back power Pi
/// 1=board back powers and can supply up to 1.3A to the Pi
/// 2=board back powers and can supply up to 2A to the Pi
/// 3=reserved
/// If back_power=2 high current USB mode is automatically enabled.
/// ```
#[derive(Clone, Copy, Debug)]
enum GpioBackPower {
    /// board does not back power Pi
    None = 0,
    /// board back powers and can supply up to 1.3A to the Pi
    BackPower1A3 = 1,
    /// board back powers and can supply up to 2A to the Pi
    BackPower2A = 2,
}

/// GPIO function as per FSEL GPIO register field in BCM2835 datasheet
///
/// ```text
/// FSELn - Function Select n
/// 000 = GPIO Pin n is an input
/// 001 = GPIO Pin n is an output
/// 100 = GPIO Pin n takes alternate function 0
/// 101 = GPIO Pin n takes alternate function 1
/// 110 = GPIO Pin n takes alternate function 2
/// 111 = GPIO Pin n takes alternate function 3
/// 011 = GPIO Pin n takes alternate function 4
/// 010 = GPIO Pin n takes alternate function 5
/// ```
#[derive(Clone, Copy, Debug)]
enum GpioFsel {
    /// GPIO Pin is an input
    Input = 0,
    /// GPIO Pin is an output
    Output = 1,
    /// GPIO Pin takes alternate function 0
    Alt0 = 4,
    /// GPIO Pin takes alternate function 1
    Alt1 = 5,
    /// GPIO Pin takes alternate function 2
    Alt2 = 6,
    /// GPIO Pin takes alternate function 3
    Alt3 = 7,
    /// GPIO Pin takes alternate function 4
    Alt4 = 3,
    /// GPIO Pin takes alternate function 5
    Alt5 = 2,
}

/// 0=leave at default setting,  1=pullup, 2=pulldown, 3=no pull
#[derive(Clone, Copy, Debug)]
enum GpioPull {
    /// leave at default setting
    Default = 0,
    /// pullup
    Up = 1,
    /// pulldown
    Down = 2,
    /// no pull
    None = 3,
}

#[derive(Debug)]
struct GpioPin {
    fsel: GpioFsel,
    pull: GpioPull,
    used: bool,
}

impl GpioPin {
    fn new() -> GpioPin {
        GpioPin {
            fsel: GpioFsel::Input,
            pull: GpioPull::None,
            used: false,
        }
    }

    fn to_buffer(&self) -> u8 {
        let fsel = self.fsel as u8;
        let pull = self.pull as u8;
        (fsel & 0x07) | (pull & 0x03) << 5 | (self.used as u8) << 7
    }
}

/// This struct implements the GPIO map Atom
///
/// [GPIO map atom data](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#gpio-map-atom-data-type0x0002):
/// ```text
/// Bytes   Field
/// 1       bank_drive  bank drive strength/slew/hysteresis, BCM2835 can only set per bank, not per IO
///           Bits in byte:
///           [3:0] drive       0=leave at default, 1-8=drive*2mA, 9-15=reserved
///           [5:4] slew        0=leave at default, 1=slew rate limiting, 2=no slew limiting, 3=reserved
///           [7:6] hysteresis  0=leave at default, 1=hysteresis disabled, 2=hysteresis enabled, 3=reserved
/// 1       power
///           [1:0] back_power  0=board does not back power Pi
///                             1=board back powers and can supply up to 1.3A to the Pi
///                             2=board back powers and can supply up to 2A to the Pi
///                             3=reserved
///                             If back_power=2 high current USB mode is automatically enabled.
///           [7:2] reserved    set to 0
///28      1 byte per IO pin
///          Bits in each byte:
///           [2:0] func_sel    GPIO function as per FSEL GPIO register field in BCM2835 datasheet
///           [4:3] reserved    set to 0
///           [6:5] pulltype    0=leave at default setting,  1=pullup, 2=pulldown, 3=no pull
///           [  7] is_used     1=board uses this pin, 0=not connected and therefore not used
/// ```
#[derive(Debug)]
pub struct EEPAtomGpioMapData {
    drive: GpioDrive,
    slew: GpioSlew,
    hysteresis: GpioHysteresis,
    back_power: GpioBackPower,
    gpios: Vec<GpioPin>,
}

impl ToBuffer for EEPAtomGpioMapData {
    fn len(&self) -> usize {
        1 + 1 + 28
    }

    fn to_buffer(&self, buf: &mut Vec<u8>) -> () {
        let drive = self.drive as u8;
        let slew = self.slew as u8;
        let hyst = self.hysteresis as u8;
        let bank_drive = (drive & 0x0f) | (slew & 0x30) << 4 | (hyst & 0xc0) << 6;
        buf.push(bank_drive);

        let back_power = self.back_power as u8 & 0x3;
        buf.push(back_power);

        for gpio in &self.gpios {
            buf.push(gpio.to_buffer());
        }
    }
}

#[test]
fn test_eep_atom_gpio_map() {
    let mut gpios: Vec<GpioPin> = Vec::with_capacity(MAX_GPIOS);
    for _ in 0..MAX_GPIOS {
        gpios.push(GpioPin::new());
    }
    println!("{:?}", gpios);

    let map = EEPAtomGpioMapData {
        drive: GpioDrive::Drive8mA,
        slew: GpioSlew::Default,
        hysteresis: GpioHysteresis::Default,
        back_power: GpioBackPower::None,
        gpios,
    };
}