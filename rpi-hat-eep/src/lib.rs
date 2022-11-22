// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use crc::{Crc, CRC_16_ARC};

use self::gpio_map::EepAtomGpioMapData;

pub mod gpio_map;

#[allow(clippy::len_without_is_empty)]

/// This trait is used to write the object into a byte vector
///
/// All objects which implement this trait can be written to a Vec<u8>. How the object is written to
/// the Vec<u8> is decided by the object itself. This trait is defined by the following two methods
/// [len](ToBytes::len()) and [to_bytes](ToBytes::to_bytes()):
/// * The [len](ToBytes::len()) method returns the size the object will use when it is written into
///   the vector.
/// * The [to_bytes](ToBytes::to_bytes()) appends the object to a [Vec<u8>].
pub trait ToBytes {
    /// Return the size the object will use when it is written into the vector.
    ///
    /// This method will calculate the size of the object when it is converted into a [Vec<u8>].
    fn len(&self) -> usize;
    /// This method writes the object to a given vector.
    ///
    /// The function appends the object to a given vector. The size of the vector will be increased
    /// by [ToBytes::len()] bytes.
    fn to_bytes(&self, buf: &mut Vec<u8>);
}

#[derive(Debug)]
pub struct EepError(String);

impl std::fmt::Display for EepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EepError {}

#[derive(Debug)]
pub enum EepPushError {
    MaxAtomCountExceeded,
    WrongAtomOrder {
        atype: EepAtomType,
        prev: Option<EepAtomType>,
        expected: Vec<EepAtomType>,
    },
}

impl std::fmt::Display for EepPushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EepPushError::MaxAtomCountExceeded => {
                write!(f, "The maximum Atom count {} was exceeded", u16::MAX)
            }
            EepPushError::WrongAtomOrder {
                atype,
                prev,
                expected,
            } => {
                write!(f, "Wrong Atom type order: Got {};", atype)?;
                if let Some(prev) = prev {
                    write!(f, "previous Atom was {};", prev)?;
                } else {
                    write!(f, "this was the first Atom;")?;
                }
                if expected.is_empty() {
                    write!(f, "not further Atoms were expected.")
                } else {
                    write!(f, "expected Atom types:")?;
                    for t in expected {
                        write!(f, " {}", t)?;
                    }
                    write!(f, ".")
                }
            }
        }
    }
}

impl std::error::Error for EepPushError {}

/// This struct implemnts the EEPROM Structure
///
/// [EEPROM Structure](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#eeprom-structure):
/// ```text
/// HEADER  <- EEPROM header (Required)
/// ATOM1   <- Vendor info atom (Required)
/// ATOM2   <- GPIO map atom (Required)
/// ATOM3   <- DT blob atom (Required for compliance with the HAT specification)
/// ...
/// ATOMn
/// ```
/// The HEADER is not part of this struct as it is generated on demand.
#[derive(Debug)]
pub struct Eep {
    /// This vector contains the ATOMs (ATOM1...ATOMn)
    atoms: Vec<EepAtom>,
}

impl Eep {
    pub fn new(
        vendor_data: EepAtomVendorData,
        gpio_map_data: EepAtomGpioMapData,
    ) -> Eep {
        let atoms: Vec<EepAtom> = vec![
            EepAtom::new_vendor_info(vendor_data),
            EepAtom::new_gpio_bank0_map(gpio_map_data),
        ];
        Eep { atoms }
    }

    pub fn push(&mut self, mut atom: EepAtom) -> Result<(), EepPushError> {
        if self.atoms.len() > u16::MAX as usize + 1 {
            return Err(EepPushError::MaxAtomCountExceeded);
        }

        if self.atoms.is_empty() && atom.atype != EepAtomType::VendorInfo {
            return Err(EepPushError::WrongAtomOrder {
                atype: atom.atype,
                prev: None,
                expected: vec![EepAtomType::VendorInfo],
            });
        };

        let last = self
            .atoms
            .last()
            .expect("BUG: The Atoms vector should not be empty at this point.");

        match last.atype {
            EepAtomType::VendorInfo => match atom.atype {
                EepAtomType::GpioBank0Map => (),
                _ => {
                    return Err(EepPushError::WrongAtomOrder {
                        atype: atom.atype,
                        prev: Some(last.atype),
                        expected: vec![EepAtomType::GpioBank0Map],
                    });
                }
            },
            EepAtomType::GpioBank0Map => match atom.atype {
                EepAtomType::LinuxDTB
                | EepAtomType::ManufCustomData
                | EepAtomType::GpioBank1Map => (),
                _ => {
                    return Err(EepPushError::WrongAtomOrder {
                        atype: atom.atype,
                        prev: Some(last.atype),
                        expected: vec![
                            EepAtomType::LinuxDTB,
                            EepAtomType::ManufCustomData,
                            EepAtomType::GpioBank1Map,
                        ],
                    });
                }
            },
            EepAtomType::LinuxDTB => match atom.atype {
                EepAtomType::ManufCustomData | EepAtomType::GpioBank1Map => (),
                _ => {
                    return Err(EepPushError::WrongAtomOrder {
                        atype: atom.atype,
                        prev: Some(last.atype),
                        expected: vec![EepAtomType::ManufCustomData, EepAtomType::GpioBank1Map],
                    });
                }
            },
            EepAtomType::ManufCustomData => match atom.atype {
                EepAtomType::ManufCustomData | EepAtomType::GpioBank1Map => (),
                _ => {
                    return Err(EepPushError::WrongAtomOrder {
                        atype: atom.atype,
                        prev: Some(last.atype),
                        expected: vec![EepAtomType::ManufCustomData, EepAtomType::GpioBank1Map],
                    });
                }
            },
            EepAtomType::GpioBank1Map => {
                return Err(EepPushError::WrongAtomOrder {
                    atype: atom.atype,
                    prev: Some(last.atype),
                    expected: Vec::new(),
                });
            }
        }

        atom.count = self.atoms.len() as u16;
        self.atoms.push(atom);
        Ok(())
    }
}

impl ToBytes for Eep {
    fn len(&self) -> usize {
        /*
         * Bytes   Field
         * 4       signature
         * 1       version
         * 1       reserved
         * 2       numatoms
         * 4       eeplen
         */
        let mut len = 4 + 1 + 1 + 2 + 4;
        for atom in &self.atoms {
            len += atom.len();
        }
        len
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let signature = 0x6950_2d52u32;
        buf.extend(signature.to_le_bytes());
        // version
        buf.push(1);
        // reserved
        buf.push(0);
        // numatoms
        buf.extend((self.atoms.len() as u16).to_le_bytes());
        // eeplen
        buf.extend((self.len() as u32).to_le_bytes());
        for atom in &self.atoms {
            atom.to_bytes(buf);
        }
    }
}

#[derive(Debug)]
pub enum EepAtomData {
    /// vendor info (0x0001, [`EepAtomType::VendorInfo`])
    VendorInfo(EepAtomVendorData),
    /// GPIO (bank 0) map (0x0002, [`EepAtomType::GpioBank0Map`])
    GpioBank0Map(gpio_map::EepAtomGpioMapData),
    /// Linux device tree blob (0x0003, [`EepAtomType::LinuxDTB`])
    LinuxDTB(EepAtomLinuxDTBData),
    /// manufacturer custom data (0x0004, [`EepAtomType::ManufCustomData`])
    ManufCustomData(EepAtomCustomData),
    /// GPIO (bank 1) map (0x0005, [`EepAtomType::GpioBank1Map`])
    GpioBank1Map(gpio_map::EepAtomGpioMapData),
}

impl ToBytes for EepAtomData {
    fn len(&self) -> usize {
        match self {
            EepAtomData::VendorInfo(data) => data.len(),
            EepAtomData::GpioBank0Map(data) => data.len(),
            EepAtomData::LinuxDTB(data) => data.len(),
            EepAtomData::ManufCustomData(data) => data.len(),
            EepAtomData::GpioBank1Map(data) => data.len(),
        }
    }
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            EepAtomData::VendorInfo(data) => data.to_bytes(buf),
            EepAtomData::GpioBank0Map(data) => data.to_bytes(buf),
            EepAtomData::LinuxDTB(data) => data.to_bytes(buf),
            EepAtomData::ManufCustomData(data) => data.to_bytes(buf),
            EepAtomData::GpioBank1Map(data) => data.to_bytes(buf),
        };
    }
}

/// This enum implements the Atom Types
///
/// [Atom Types](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#atom-types):
/// ```text
/// 0x0000 = invalid
/// 0x0001 = vendor info
/// 0x0002 = GPIO (bank 0) map
/// 0x0003 = Linux device tree blob
/// 0x0004 = manufacturer custom data
/// 0x0005 = GPIO (bank 1) map
/// 0x0006-0xfffe = reserved for future use
/// 0xffff = invalid
/// ```
/// The enume does not define any value for invalid or reserved types. Any value not defined by this
/// enum is treated as an invalid/error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EepAtomType {
    VendorInfo = 0x0001,
    GpioBank0Map = 0x0002,
    LinuxDTB = 0x0003,
    ManufCustomData = 0x0004,
    GpioBank1Map = 0x0005,
}

impl std::fmt::Display for EepAtomType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EepAtomType::VendorInfo => "vendor info",
                EepAtomType::GpioBank0Map => "GPIO (bank 0) map",
                EepAtomType::LinuxDTB => "Linux device tree blob",
                EepAtomType::ManufCustomData => "manufacturer custom data",
                EepAtomType::GpioBank1Map => "GPIO (bank 1) map",
            }
        )
    }
}

/// This struct implements the Atom Structure
///
/// [Atom Structure](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#atom-structure):
/// ```text
/// Bytes   Field
/// 2       type        atom type
/// 2       count       incrementing atom count
/// 4       dlen        length in bytes of data+CRC
/// N       data        N bytes, N = dlen-2
/// 2       crc16       CRC-16 of entire atom (type, count, dlen, data)
/// ```
/// The dlen and crc16 are not stored in this struct as they are generated on demand.
#[derive(Debug)]
pub struct EepAtom {
    /// The Atom Type as defined by [`EepAtomType`]
    atype: EepAtomType,
    /// The atom count (ATOM1...ATOMn). It is the same as the index of the Atom in the [`Eep`] atoms
    /// vector. So ATOM1 has count = 0, ATOM2 has count = 1, ..., ATOMn has count = n - 1
    count: u16,
    /// The actual Atom data
    data: EepAtomData,
}

/// This defines the CRC16 algorithm used to calculate the checksum of the Atoms
const ATOM_CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_ARC);

impl EepAtom {
    pub fn new_vendor_info(data: EepAtomVendorData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::VendorInfo,
            count: 0,
            data: EepAtomData::VendorInfo(data),
        }
    }

    pub fn new_gpio_bank0_map(data: EepAtomGpioMapData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::GpioBank0Map,
            count: 1,
            data: EepAtomData::GpioBank0Map(data),
        }
    }

    pub fn new_linux_dtb(data: EepAtomLinuxDTBData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::LinuxDTB,
            count: 2,
            data: EepAtomData::LinuxDTB(data),
        }
    }

    pub fn new_custom(data: EepAtomCustomData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::ManufCustomData,
            count: 0xffff,
            data: EepAtomData::ManufCustomData(data),
        }
    }

    pub fn new_gpio_bank1_map(data: EepAtomGpioMapData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::GpioBank1Map,
            count: 1,
            data: EepAtomData::GpioBank1Map(data),
        }
    }
}

impl ToBytes for EepAtom {
    fn len(&self) -> usize {
        /*
         * Bytes   Field
         * 2       type
         * 2       count
         * 4       dlen
         * N       data
         * 2       crc16
         */
        2 + 2 + 4 + self.data.len() + 2
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let atype = self.atype as u16;
        buf.extend_from_slice(&atype.to_le_bytes());
        buf.extend_from_slice(&self.count.to_le_bytes());
        let dlen = self.data.len() as u32 + 2;
        buf.extend_from_slice(&dlen.to_le_bytes());
        self.data.to_bytes(buf);

        let crc_len = self.len() - 2;
        let crc16 = ATOM_CRC16.checksum(&buf[(buf.len() - crc_len)..]);
        buf.extend_from_slice(&crc16.to_le_bytes());
    }
}

/// This struct implements the Vendor info Atom
///
/// [Vendor info atom data](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#vendor-info-atom-data-type0x0001):
/// ```text
/// Bytes   Field
/// 16      uuid        UUID (unique for every single board ever made)
/// 2       pid         product ID
/// 2       pver        product version
/// 1       vslen       vendor string length (bytes)
/// 1       pslen       product string length (bytes)
/// X       vstr        ASCII vendor string e.g. "ACME Technology Company"
/// Y       pstr        ASCII product string e.g. "Special Sensor Board"
/// ```
/// The vslen and the pslen are implicitly given by the [`String`] type.
#[derive(Debug)]
pub struct EepAtomVendorData {
    /// UUID (unique for every single board ever made)
    uuid: uuid::Uuid,
    /// product ID
    pid: u16,
    /// product version
    pver: u16,
    /// ASCII vendor string e.g. "ACME Technology Company"
    vstr: String,
    /// ASCII product string e.g. "Special Sensor Board"
    pstr: String,
}

impl EepAtomVendorData {
    pub fn new(
        uuid: uuid::Uuid,
        pid: u16,
        pver: u16,
        vstr: String,
        pstr: String,
    ) -> Result<EepAtomVendorData, EepError> {
        if vstr.len() > u8::MAX as usize {
            return Err(EepError(format!(
                "Vendor string to long: {} (max: {} bytes)",
                vstr.len(),
                u8::MAX
            )));
        }
        if pstr.len() > u8::MAX as usize {
            return Err(EepError(format!(
                "Product string to long: {} (max: {} bytes)",
                pstr.len(),
                u8::MAX
            )));
        }
        Ok(EepAtomVendorData {
            uuid,
            pid,
            pver,
            vstr,
            pstr,
        })
    }
}

impl ToBytes for EepAtomVendorData {
    fn len(&self) -> usize {
        /*
         * Bytes   Field
         * 16      uuid
         * 2       pid
         * 2       pver
         * 1       vslen
         * 1       pslen
         * X       vstr
         * Y       pstr
         */
        16 + 2 + 2 + 1 + 1 + self.vstr.len() + self.pstr.len()
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        // The UUID is stored in reverse order in the EEPROM
        for b in self.uuid.as_bytes().iter().rev() {
            buf.push(*b)
        }
        buf.extend_from_slice(&self.pid.to_le_bytes());
        buf.extend_from_slice(&self.pver.to_le_bytes());
        // vstr.len() can't be > u8::MAX (see: EepAtomVendorData::new()
        buf.push(u8::try_from(self.vstr.len()).unwrap());
        // pstr.len() can't be > u8::MAX (see: EepAtomVendorData::new())
        buf.push(u8::try_from(self.pstr.len()).unwrap());
        buf.extend_from_slice(self.vstr.as_bytes());
        buf.extend_from_slice(self.pstr.as_bytes());
    }
}

#[test]
fn test_eep_atom_vendor_data() {
    let uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
    let data = EepAtomVendorData::new(
        uuid,
        123u16,
        3u16,
        "ACME Technology Company".to_string(),
        "Special Sensor Board".to_string(),
    ).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    data.to_bytes(&mut buf);
    assert_eq!(data.len(), buf.len());

    let long_string: String = vec!['a'; 256].into_iter().collect();
    let data = EepAtomVendorData::new(
        uuid,
        123u16,
        3u16,
        long_string.clone(),
        "Special Sensor Board".to_string(),
    );
    assert!(data.is_err());

    let data = EepAtomVendorData::new(
        uuid,
        123u16,
        3u16,
        "ACME Technology Company".to_string(),
        long_string,
    );
    assert!(data.is_err());
}

#[derive(Debug)]
pub enum LinuxDTB {
    Blob(Vec<u8>),
    Name(String),
}

#[derive(Debug)]
pub struct EepAtomLinuxDTBData {
    data: LinuxDTB,
}

impl EepAtomLinuxDTBData {
    pub fn new(data: LinuxDTB) -> EepAtomLinuxDTBData {
        EepAtomLinuxDTBData { data }
    }
}

impl ToBytes for EepAtomLinuxDTBData {
    fn len(&self) -> usize {
        match &self.data {
            LinuxDTB::Blob(data) => data.len(),
            LinuxDTB::Name(data) => data.len(),
        }
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match &self.data {
            LinuxDTB::Blob(data) => buf.extend(data),
            LinuxDTB::Name(data) => buf.extend(data.as_bytes()),
        }
    }
}

#[derive(Debug)]
pub struct EepAtomCustomData {
    data: Vec<u8>,
}

impl EepAtomCustomData {
    pub fn new(data: Vec<u8>) -> EepAtomCustomData {
        EepAtomCustomData { data }
    }
}

impl ToBytes for EepAtomCustomData {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend(&self.data);
    }
}
