// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use crc::{Crc, CRC_16_ARC, Digest};
use std::io::Write;

mod gpio_map;

pub trait ToBuffer {
    fn len(&self) -> usize;
    fn to_buffer(&self, buf: &mut Vec<u8>);
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
#[derive(Debug)]
pub struct EEP {
    //pub header: EEPHeader,
    pub atoms: Vec<EEPAtom>,
}

impl EEP {}

/// This struct implements the EEPROM Header Structure
///
/// [EEPROM Header Structure](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md#eeprom-header-structure):
/// ```text
/// Bytes   Field
/// 4       signature   signature: 0x52, 0x2D, 0x50, 0x69 ("R-Pi" in ASCII)
/// 1       version     EEPROM data format version (0x00 reserved, 0x01 = first version)
/// 1       reserved    set to 0
/// 2       numatoms    total atoms in EEPROM
/// 4       eeplen      total length in bytes of all eeprom data (including this header)
/// ```

#[derive(Debug)]
pub struct EEPHeader {
    /// 0x52, 0x2D, 0x50, 0x69 ("R-Pi" in ASCII)
    signature: u32,
    /// EEPROM data format version (0x00 reserved, 0x01 = first version)
    version: u8,
    /// set to 0
    _reseved: u8,
    // total atoms in EEPROM
    //numatoms: u16,
    // total length in bytes of all eeprom data (including this header)
    //eeplen: u32,
}

#[derive(Debug)]
pub enum EEPAtomData {
    /// vendor info (0x0001)
    VendorInfo(EEPAtomVendorData),
    /// GPIO map (0x0002)
    GpioMap(gpio_map::EEPAtomGpioMapData),
    /// Linux device tree blob (0x0003)
    LinuxDTB(EEPAtomLinuxDTBData),
    /// manufacturer custom data (0x0004)
    ManufCustomData(EEPAtomCustomData),
}

impl ToBuffer for EEPAtomData {
    fn len(&self) -> usize {
        match self {
            EEPAtomData::VendorInfo(data) => data.len(),
            EEPAtomData::GpioMap(data) => data.len(),
            EEPAtomData::LinuxDTB(data) => data.len(),
            EEPAtomData::ManufCustomData(data) => data.len(),
        }
    }
    fn to_buffer(&self, buf: &mut Vec<u8>) {
        match self {
            EEPAtomData::VendorInfo(data) => data.to_buffer(buf),
            EEPAtomData::GpioMap(data) => data.to_buffer(buf),
            EEPAtomData::LinuxDTB(data) => data.to_buffer(buf),
            EEPAtomData::ManufCustomData(data) => data.to_buffer(buf),
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
#[derive(Clone, Copy, Debug)]
pub enum EEPAtomType
{
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
#[derive(Debug)]
pub struct EEPAtom {
    atype: EEPAtomType,
    /// incrementing atom count
    count: u16,
    // length in bytes of data+CRC
    //dlen: u32,
    /// N bytes, N = dlen-2
    data: EEPAtomData,
    // CRC-16 of entire atom (type, count, dlen, data)
    //crc16: u16,
}

fn crc_write<T: Write>(f: &mut dyn Write, buf: &[u8], digest: &mut Digest<u16>) {
    digest.update(buf);
    f.write(buf).unwrap();
}

/// The Atom crc16 algorithem
const ATOM_CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_ARC);

impl EEPAtom {
    fn get_data_len<T: ToBuffer>(atom: &dyn ToBuffer) -> usize {
        atom.len()
    }

    fn get_data<T: ToBuffer>(atom: &dyn ToBuffer, buf: &mut Vec<u8>) -> () {
        atom.to_buffer(buf)
    }

    fn to_buffer(&self, buf: &mut Vec<u8>) -> () {
        let mut digest = ATOM_CRC16.digest();
        let atype = self.atype as u8;
        crc_write::<Vec<u8>>(buf, &[atype], &mut digest);
        buf.push(self.atype as u8);
        buf.extend_from_slice(&self.count.to_le_bytes());
        let dlen = self.data.len() as u32 + 2;
        buf.extend_from_slice(&dlen.to_le_bytes());
        self.data.to_buffer(buf);
        let crc16 = ATOM_CRC16.checksum(buf);
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
#[derive(Debug)]
pub struct EEPAtomVendorData {
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

impl ToBuffer for EEPAtomVendorData {
    fn len(&self) -> usize {
        16 + 2 + 2 + 1 + 1 + self.vstr.len() + self.pstr.len()
    }

    fn to_buffer(&self, buf: &mut Vec<u8>) -> () {
        // The UUID is stored in reverse order in the EEPROM
        for b in self.uuid.as_bytes().iter().rev() {
            buf.push(*b)
        }
        buf.extend_from_slice(&self.pid.to_le_bytes());
        buf.extend_from_slice(&self.pver.to_le_bytes());
        buf.push(u8::try_from(self.vstr.len()).unwrap());
        buf.push(u8::try_from(self.pstr.len()).unwrap());
        buf.extend_from_slice(self.vstr.as_bytes());
        buf.extend_from_slice(self.pstr.as_bytes());
    }
}

#[test]
fn test_eep_atom_vendor_data() {
    let data = EEPAtomVendorData {
        uuid: uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
        pid: 123u16,
        pver: 3u16,
        vstr: "ACME Technology Company".to_string(),
        pstr: "Special Sensor Board".to_string(),
    };
    let mut buf: Vec<u8> = Vec::new();
    data.to_buffer(&mut buf);
    assert_eq!(data.len(), buf.len())
}

#[derive(Debug)]
enum LinuxDTB {
    Blob(Vec<u8>),
    Name(String),
}

#[derive(Debug)]
pub struct EEPAtomLinuxDTBData {
    data: LinuxDTB,
}

impl ToBuffer for EEPAtomLinuxDTBData {
    fn len(&self) -> usize {
        match &self.data {
            LinuxDTB::Blob(data) => data.len(),
            LinuxDTB::Name(data) => data.len(),
        }
    }

    fn to_buffer(&self, buf: &mut Vec<u8>) -> () {
        match &self.data {
            LinuxDTB::Blob(data) => buf.extend(data),
            LinuxDTB::Name(data) => buf.extend(data.as_bytes()),
        }
    }
}

#[derive(Debug)]
pub struct EEPAtomCustomData {
    data: Vec<u8>,
}

impl ToBuffer for EEPAtomCustomData {
    fn len(&self) -> usize {
        self.data.len()
    }

    fn to_buffer(&self, buf: &mut Vec<u8>) -> () {
        buf.extend(&self.data)
    }
}
