// SPDX-FileCopyrightText: 2022-2025 KUNBUS GmbH <support@kunbus.com>
//
// SPDX-License-Identifier: GPL-2.0-or-later

pub const BANK0_GPIOS: usize = 28;
pub const BANK1_GPIOS: usize = 18;

use crate::ToBytes;
use num_derive::FromPrimitive;

#[derive(Debug, PartialEq)]
enum GpioErrorType {
    OutOfBound,
}

impl std::fmt::Display for GpioErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OutOfBound => "GPIO index out of bound",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub struct GpioError {
    gpio_no: usize,
    bank: GpioBank,
    etype: GpioErrorType,
}

impl std::error::Error for GpioError {}

impl std::fmt::Display for GpioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gpio: {} ({}): {}", self.gpio_no, self.bank, self.etype)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpioBank {
    Bank0,
    Bank1,
}

impl std::fmt::Display for GpioBank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bank = match self {
            Self::Bank0 => "Bank0",
            Self::Bank1 => "Bank1",
        };
        write!(f, "{}", bank)
    }
}

/// 0=leave at default, 1-8=drive*2mA, 9-15=reserved
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum GpioDrive {
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
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum GpioSlew {
    /// leave at default
    Default = 0,
    /// slew rate limiting
    RateLimiting = 1,
    /// no slew limiting
    NoLimit = 2,
}

/// 0=leave at default, 1=hysteresis disabled, 2=hysteresis enabled, 3=reserved
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum GpioHysteresis {
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
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum GpioBackPower {
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
#[derive(Clone, Copy, Debug, Default)]
pub enum GpioFsel {
    /// GPIO Pin is an input
    #[default]
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
#[derive(Clone, Copy, Debug, Default)]
pub enum GpioPull {
    /// leave at default setting
    #[default]
    Default = 0,
    /// pullup
    Up = 1,
    /// pulldown
    Down = 2,
    /// no pull
    NoPull = 3,
}

#[derive(Debug, Default, Clone)]
pub struct GpioPin {
    fsel: GpioFsel,
    pull: GpioPull,
    used: bool,
}

impl GpioPin {
    pub const fn new(fsel: GpioFsel, pull: GpioPull, used: bool) -> Self {
        Self { fsel, pull, used }
    }

    const fn to_u8(&self) -> u8 {
        let fsel = self.fsel as u8;
        let pull = self.pull as u8;
        (fsel & 0x07) | (pull & 0x03) << 5 | (self.used as u8) << 7
    }
}

#[test]
fn test_gpio_pin() {
    assert_eq!(
        GpioPin::new(GpioFsel::Input, GpioPull::Default, false).to_u8(),
        0x00_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Output, GpioPull::Default, false).to_u8(),
        0x01_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt0, GpioPull::Default, false).to_u8(),
        0x04_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt1, GpioPull::Default, false).to_u8(),
        0x05_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt2, GpioPull::Default, false).to_u8(),
        0x06_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt3, GpioPull::Default, false).to_u8(),
        0x07_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt4, GpioPull::Default, false).to_u8(),
        0x03_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt5, GpioPull::Default, false).to_u8(),
        0x02_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Input, GpioPull::Up, false).to_u8(),
        0x20_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Input, GpioPull::Down, false).to_u8(),
        0x40_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Input, GpioPull::NoPull, false).to_u8(),
        0x60_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Input, GpioPull::Default, true).to_u8(),
        0x80_u8
    );
    assert_eq!(
        GpioPin::new(GpioFsel::Alt3, GpioPull::NoPull, true).to_u8(),
        0xe7_u8
    );
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
pub struct EepAtomGpioMapData {
    bank: GpioBank,
    drive: GpioDrive,
    slew: GpioSlew,
    hysteresis: GpioHysteresis,
    back_power: GpioBackPower,
    gpios: Vec<GpioPin>,
}

impl EepAtomGpioMapData {
    pub fn new(
        bank: GpioBank,
        drive: GpioDrive,
        slew: GpioSlew,
        hysteresis: GpioHysteresis,
        back_power: GpioBackPower,
    ) -> Self {
        let gpios = match bank {
            GpioBank::Bank0 => vec![GpioPin::default(); BANK0_GPIOS],
            GpioBank::Bank1 => vec![GpioPin::default(); BANK1_GPIOS],
        };
        Self {
            bank,
            drive,
            slew,
            hysteresis,
            back_power,
            gpios,
        }
    }

    pub fn set(&mut self, n: usize, gpio: GpioPin) -> Result<(), GpioError> {
        let n = match self.bank {
            GpioBank::Bank0 => n,
            GpioBank::Bank1 => match n.checked_sub(BANK0_GPIOS) {
                Some(n) => n,
                None => {
                    return Err(GpioError {
                        gpio_no: n,
                        bank: self.bank,
                        etype: GpioErrorType::OutOfBound,
                    })
                }
            },
        };
        if n >= self.gpios.len() {
            return Err(GpioError {
                gpio_no: n,
                bank: self.bank,
                etype: GpioErrorType::OutOfBound,
            });
        }
        self.gpios[n] = gpio;
        Ok(())
    }
}

impl ToBytes for EepAtomGpioMapData {
    fn len(&self) -> usize {
        // 1 byte drive_bank; 1 byte power; 28 bytes gpio pins configuration
        1 + 1 + self.gpios.len()
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let drive = self.drive as u8;
        let slew = self.slew as u8;
        let hyst = self.hysteresis as u8;
        let bank_drive = (drive & 0x0f) | (slew & 0x03) << 4 | (hyst & 0x03) << 6;
        buf.push(bank_drive);

        let back_power = self.back_power as u8 & 0x3;
        buf.push(back_power);

        for gpio in &self.gpios {
            buf.push(gpio.to_u8());
        }
    }
}

#[test]
fn test_eep_atom_gpio_map() {
    let mut gpio_map = EepAtomGpioMapData::new(
        GpioBank::Bank0,
        GpioDrive::Default,
        GpioSlew::Default,
        GpioHysteresis::Default,
        GpioBackPower::None,
    );

    let mut buf: Vec<u8> = Vec::new();
    gpio_map.to_bytes(&mut buf);
    /*
     * check if the buffer has the expected size of 30 bytes
     * 1 byte drive_bank; 1 byte power; 28 bytes gpio pins configuration
     */
    assert_eq!(buf.len(), 1 + 1 + 28);
    for b in buf {
        assert_eq!(b, 0);
    }

    /* chack that set() only allows setting gpios in the range of 0-27 */
    assert_eq!(
        gpio_map.set(
            0,
            GpioPin {
                fsel: GpioFsel::Alt4,
                pull: GpioPull::Down,
                used: true
            }
        ),
        Ok(())
    );
    assert_eq!(
        gpio_map.set(
            BANK0_GPIOS - 1,
            GpioPin {
                fsel: GpioFsel::Alt5,
                pull: GpioPull::NoPull,
                used: true
            }
        ),
        Ok(())
    );
    assert_eq!(
        gpio_map.set(
            BANK0_GPIOS,
            GpioPin {
                fsel: GpioFsel::Alt0,
                pull: GpioPull::Up,
                used: true
            }
        ),
        Err(GpioError {
            gpio_no: BANK0_GPIOS,
            bank: GpioBank::Bank0,
            etype: GpioErrorType::OutOfBound
        })
    );
}
