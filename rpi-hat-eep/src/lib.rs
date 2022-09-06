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
    pub fn new() -> Eep {
        let atoms: Vec<EepAtom> = Vec::new();
        Eep { atoms }
    }

    pub fn push(&mut self, mut atom: EepAtom) -> Result<(), String> {
        match atom.atype {
            EepAtomType::VendorInfo => {
                if !self.atoms.is_empty() {
                    return Err("Wrong order: vendor info".to_string());
                }
            }
            EepAtomType::GpioMap => {
                if self.atoms.len() != 1 {
                    return Err("Wrong order: gpio map".to_string());
                }
            }
            EepAtomType::LinuxDTB => {
                if self.atoms.len() != 2 {
                    return Err("Wrong order: dtb".to_string());
                }
            }
            EepAtomType::ManufCustomData => {
                if self.atoms.len() < 2 {
                    return Err("Wrong order: custom".to_string());
                }
            }
        }
        atom.count = self.atoms.len() as u16;
        self.atoms.push(atom);
        Ok(())
    }
}

impl ToBytes for Eep {
    fn len(&self) -> usize {
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

impl Default for Eep {
    fn default() -> Self {
        Eep::new()
    }
}

#[derive(Debug)]
pub enum EepAtomData {
    /// vendor info (0x0001, [`EepAtomType::VendorInfo`])
    VendorInfo(EepAtomVendorData),
    /// GPIO map (0x0002, [`EepAtomType::GpioMap`])
    GpioMap(gpio_map::EepAtomGpioMapData),
    /// Linux device tree blob (0x0003, [`EepAtomType::LinuxDTB`])
    LinuxDTB(EepAtomLinuxDTBData),
    /// manufacturer custom data (0x0004, [`EepAtomType::ManufCustomData`])
    ManufCustomData(EepAtomCustomData),
}

impl ToBytes for EepAtomData {
    fn len(&self) -> usize {
        match self {
            EepAtomData::VendorInfo(data) => data.len(),
            EepAtomData::GpioMap(data) => data.len(),
            EepAtomData::LinuxDTB(data) => data.len(),
            EepAtomData::ManufCustomData(data) => data.len(),
        }
    }
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            EepAtomData::VendorInfo(data) => data.to_bytes(buf),
            EepAtomData::GpioMap(data) => data.to_bytes(buf),
            EepAtomData::LinuxDTB(data) => data.to_bytes(buf),
            EepAtomData::ManufCustomData(data) => data.to_bytes(buf),
        };
    }
}

/// This enum implements the Atom Types
///
/// [Atom Types](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#atom-types):
/// ```text
/// 0x0000 = invalid
/// 0x0001 = vendor info
/// 0x0002 = GPIO map
/// 0x0003 = Linux device tree blob
/// 0x0004 = manufacturer custom data
/// 0x0005-0xfffe = reserved for future use
/// 0xffff = invalid
/// ```
/// The enume does not define any value for invalid or reserved types. Any value not defined by this
/// enum is treated as an invalid/error.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum EepAtomType {
    VendorInfo = 0x0001,
    GpioMap = 0x0002,
    LinuxDTB = 0x0003,
    ManufCustomData = 0x0004,
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
    /// The atom count (ATOM1...ATOMn). It is the same as the index of the Atom in the [`Eep`] atoms vector +1
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
            count: 0xffff,
            data: EepAtomData::VendorInfo(data),
        }
    }

    pub fn new_gpio_map(data: EepAtomGpioMapData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::GpioMap,
            count: 0xffff,
            data: EepAtomData::GpioMap(data),
        }
    }

    pub fn new_linux_dtb(data: EepAtomLinuxDTBData) -> EepAtom {
        EepAtom {
            atype: EepAtomType::LinuxDTB,
            count: 0xffff,
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
}

impl ToBytes for EepAtom {
    fn len(&self) -> usize {
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
    pub uuid: uuid::Uuid,
    /// product ID
    pub pid: u16,
    /// product version
    pub pver: u16,
    /// ASCII vendor string e.g. "ACME Technology Company"
    pub vstr: String,
    /// ASCII product string e.g. "Special Sensor Board"
    pub pstr: String,
}

impl EepAtomVendorData {
    pub fn new(
        uuid: uuid::Uuid,
        pid: u16,
        pver: u16,
        vstr: String,
        pstr: String,
    ) -> Result<EepAtomVendorData, String> {
        if vstr.len() > u8::MAX.into() {
            return Err(format!(
                "Vendor string to long: {} (max: {} bytes)",
                vstr.len(),
                u8::MAX
            ));
        }
        if pstr.len() > u8::MAX.into() {
            return Err(format!(
                "Product string to long: {} (max: {} bytes)",
                vstr.len(),
                u8::MAX
            ));
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
    let data = EepAtomVendorData {
        uuid: uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
        pid: 123u16,
        pver: 3u16,
        vstr: "ACME Technology Company".to_string(),
        pstr: "Special Sensor Board".to_string(),
    };
    let mut buf: Vec<u8> = Vec::new();
    data.to_bytes(&mut buf);
    assert_eq!(data.len(), buf.len())
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