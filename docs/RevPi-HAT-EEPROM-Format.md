# RevPi HAT EEPROM Format (v1)  <!-- omit in toc -->

- [Revisions of this Document](#revisions-of-this-document)
- [RevPi Hat EEPROM Format Specification (v1)](#revpi-hat-eeprom-format-specification-v1)
  - [Device Tree Attributes](#device-tree-attributes)
  - [Vendor Info Atom](#vendor-info-atom)
    - [UUID](#uuid)
    - [Product ID (pid)](#product-id-pid)
    - [Product Version (pver)](#product-version-pver)
    - [Vendor String (vstr)](#vendor-string-vstr)
    - [Product String (pstr)](#product-string-pstr)
  - [GPIO Map Atom](#gpio-map-atom)
  - [Linux Device Tree (Blob) Atom](#linux-device-tree-blob-atom)
  - [Custom Atoms](#custom-atoms)
    - [(0) Format Version](#0-format-version)
    - [(1) Serial](#1-serial)
    - [(2) Product Revision (prev)](#2-product-revision-prev)
    - [(3) Endtest Date](#3-endtest-date)
    - [(4) LOT/Batch Number](#4-lotbatch-number)
    - [(5) MAC Address](#5-mac-address)
    - [(6) EEPROM Data Version](#6-eeprom-data-version)
- [Appendix](#appendix)
  - [The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](#the-raspberry-pi-hat-id-eeprom-format-specification)
    - [EEPROM Structure](#eeprom-structure)
    - [EEPROM Header Structure](#eeprom-header-structure)
    - [Atom Structure](#atom-structure)
    - [Atom Types](#atom-types)
      - [Vendor info atom data (type=0x0001):](#vendor-info-atom-data-type0x0001)
      - [GPIO map atom data (type=0x0002):](#gpio-map-atom-data-type0x0002)
      - [Device Tree atom data (type=0x0003):](#device-tree-atom-data-type0x0003)

## Revisions of this Document

| Version | Date       | Changes |
|---------|------------|---------|
| 1.0     | 2022-06-10 | Initial release |
| 1.1     | 2022-09-06 | On strings no `\0` termination is needed. Restructure the document. Move the _Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION_ to the Appendix. |
| 1.2     | 2022-10-04 | Update the description of the product version |
| 1.3     | 2022-11-22 | Add EEPROM Data Version field |

## RevPi Hat EEPROM Format Specification (v1)

The RevPi HAT EEPROM format is based on the [Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](https://github.com/raspberrypi/hats/blob/9616b5cd2bdf3e1d2d0330611387d639c1916100/eeprom-format.md) (RPi Hat Spec.). The data supplied this way is added to the device tree by the bootloader. A `hat` node is created below the root of the device tree. It can be accessed at runtime through the procfs: `/proc/device-tree/hat/`. The `hat` node can contain various attributes.

The mandatory atoms of the _Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION_ are also mandatory for this format. For details see [The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](#the-raspberry-pi-hat-id-eeprom-format-specification).

### Device Tree Attributes

| Field           | ProcFS                              | Type         | Example                                |
|-----------------|-------------------------------------|--------------|----------------------------------------|
| UUID            | `/proc/device-tree/hat/uuid`        | ASCII string | `9362d4cc-c3d8-4de6-0000-55d08e938e60` |
| product ID      | `/proc/device-tree/hat/product_id`  | ASCII string | `0x0001`                               |
| product version | `/proc/device-tree/hat/product_ver` | ASCII string | `0x0078`                               |
| vendor string   | `/proc/device-tree/hat/vendor`      | ASCII string | `KUNBUS GmbH`                          |
| product string  | `/proc/device-tree/hat/product`     | ASCII string | `RevPi Core`                           |

The RevPi Hat EEPROM Format uses the custom atoms of the RPi Hat Spec. As the custom atoms can’t be renamed the index of the custom atom is important and can’t be changed without a new version of this specification. The table below shows the custom attributes defined for the RevPi HAT EEPROM Format:

| Index | Description         | Data Type    | Attribute Type | Example             |
|-------|---------------------|--------------|----------------|---------------------|
| 0     | Format Version      | u16          | ASCII String   | `1`                 |
| 1     | Serial              | u32          | ASCII String   | `21389`             |
| 2     | Product Revision    | u16          | ASCII String   | `2`                 |
| 3     | Endtest Date        | u32          | ASCII String   | `20220419`          |
| 4     | LOT/Batch Number    | TBD          | TBD            | TBD                 |
| 5     | (first) MAC Address | ASCII String | ASCII String   | `C8:3E:A7:01:32:5E` |
| 6     | EEPROM Data Version | u16          | ASCII String   | `3`                 |

The _Data Type_ is a hint. A valid value of the attribute will never exceed the size of the data type. _Attribute Type_ is the actual type used to represent the value in the attribute.

### Vendor Info Atom

| Bytes | Field | Description
|-------|-------|----------------------------------------------------|
| 16    | uuid  | UUID (unique for every single board ever made)     |
| 2     | pid   | product ID                                         |
| 2     | pver  | product version                                    |
| 1     | vslen | vendor string length (bytes)                       |
| 1     | pslen | product string length (bytes)                      |
| X     | vstr  | ASCII vendor string e.g. "ACME Technology Company" |
| Y     | pstr  | ASCII product string e.g. "Special Sensor Board"   |

#### UUID

The UUID must be [RFC 4122](https://datatracker.ietf.org/doc/html/rfc4122) compliant. Which describes a 128-bit unsigned integer, represented as a hexadecimal string split into five groups with dashes.
The UUID for the RevPi devices is based on section [4.4](https://datatracker.ietf.org/doc/html/rfc4122#section-4.4) of https://datatracker.ietf.org/doc/html/rfc4122. Instead of a random number we use the MD5 sum of the [Product-ID (pid)](#product-id-pid), [Product-Version (pver)](#product-version-pver), [Product-Revision](#2-product-revision-prev) and [Serial](#1-serial). All these values are concatenated and a MD5 sum is calculated. The MD5 sum is then used to generate the UUID.

```text
+--------------+---------------+--------------+-----------------+
| pid (16 bit) | pver (16 bit) | prev (16bit) | serial (32 bit) |
+--------------+---------------+--------------+-----------------+
|                      MD5 sum (128 bit)                        |
+---------------------------------------------------------------+
|                        UUID (128 bit)                         |
+---------------------------------------------------------------+
```

The algorithm to calculate the UUID is as follows:

> - Set the two most significant bits (bits 6 and 7) of the clock_seq_hi_and_reserved to zero and one, respectively.
> - Set the four most significant bits (bits 12 through 15) of the time_hi_and_version field to the 4-bit version number from Section [4.1.3](https://datatracker.ietf.org/doc/html/rfc4122#section-4.1.3).
> - Set all the other bits to randomly (or pseudo-randomly) chosen values.

As the MD5 sum is already 128 bit in length. We us this value and just make the modifications to the bits described above.

##### Data Type <!-- omit in toc -->
128-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/uuid`

##### Attribute Representation <!-- omit in toc -->
Base 16 (hex) representation with dashes as 36 character ASCII string.

##### Example(s) <!-- omit in toc -->
`ebb5c735-0308-4e3c-9aea-8a270aebfe15`

#### Product ID (pid)

The product ID is a 16-bit unsigned integer. Every product has it’s own ID. The ID is the same as the PR number without the leading 1.

##### Data Type <!-- omit in toc -->
16-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/product_id`

##### Attribute Representation <!-- omit in toc -->
Base 16 (hex) representation as ASCII string.

##### Example(s) <!-- omit in toc -->
PR100302 RevPi Connect+ - 8GB → 302 → `0x012e`

#### Product Version (pver)

The product version is a 16-bit unsigned integer. It reflects the customer visible version which is lasered to the front of the device. The product version consists of the major number before and a minor number after the point. In the 16-bit unsigned integer the last two base 10 digits are used for the minor number and the rest is used for the major.

##### Data Type <!-- omit in toc -->
16-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/product_ver`

##### Attribute Representation <!-- omit in toc -->
Base 16 (hex) representation as ASCII string.

##### Example(s) <!-- omit in toc -->
1.2 → major(1), minor(2) → major * 100 + minor → 102 → `0x0066`

#### Vendor String (vstr)

The vendor string is a character array with variable length (vslen). The vendor string for all Revolution Pi devices will be `KUNBUS GmbH`.

##### Data Type <!-- omit in toc -->
ASCII string

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/vendor`

##### Attribute Representation <!-- omit in toc -->
ASCII string

##### Example(s) <!-- omit in toc -->
`KUNBUS GmbH`

#### Product String (pstr)

The vendor string is a character array with variable length (pslen). The product string is the human readable name of the Revolution Pi product. It is the same as written on the device (front). If the string on the device contains line brakes these will be converted to spaces.

##### Data Type <!-- omit in toc -->
ASCII string

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/product`

##### Attribute Representation <!-- omit in toc -->
ASCII string

##### Example(s) <!-- omit in toc -->
`RevPi Connect+ 8GB`
`RevPi Core 3GB`

### GPIO Map Atom

The GPIO map can be used to configure GPIOs before the kernel is booted.

| :warning: Only the first 28 GPIOs (the first bank) can be configured this way. GPIOs 28 and above need to be configured in the devicetree/kernel. |
|---------------------------------------------------------------------------------------------------------------------------------------------------|

| :warning: Some properties like the drive strength, slew rate and hysteresis will be applied for the whole bank (GPIOs 0-27). |
|------------------------------------------------------------------------------------------------------------------------------|

See [GPIO map atom data (type=0x0002):](#gpio-map-atom-data-type0x0002) of the [The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](#the-raspberry-pi-hat-id-eeprom-format-specification) for more details.

### Linux Device Tree (Blob) Atom

The Linux device tree blob atom can contain a compiled device tree overlay blob (not used for Revolution Pi). Or a file which contains the name of an overlay in ASCII format. Every Revolution Pi device has it’s own device tree overlay. This atom contains the name of the overlay.

##### Example(s) <!-- omit in toc -->
For the Revolution Pi Connect (the original Connect, not any variant) the string would be:
`revpi-connect`

See [Device Tree atom data (type=0x0003):](#device-tree-atom-data-type0x0003) of the [The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](#the-raspberry-pi-hat-id-eeprom-format-specification) for more details.

### Custom Atoms

It is possible to add further custom atoms. These custom atoms can contain any data in ASCII or binary form. Every custom data atom gets its own own entry in the device tree. The entry is called `custom_N` where `N` is the count and starts with `0` (zero) for the first custom data entry.

#### (0) Format Version

This is the version of the _RevPi Hat EEPROM Format Specification_. The version is represented as an  unsigned integer. For every new version of format the version is incremented.

##### Data Type <!-- omit in toc -->
16-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_0`

##### Attribute Representation <!-- omit in toc -->
Base 10 representation as ASCII string

##### Example(s) <!-- omit in toc -->
`1`

#### (1) Serial

The serial number which is also printed on the casing of the RevPi. This number is used to generate the [UUID](#uuid).

| :information_source: This field is directly programmed by the endtester. There is currently no other central place which defines the serial number. |
|-----------------------------------------------------------------------------------------------------------------------------------------------------|

##### Data Type <!-- omit in toc -->
32-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_1`

##### Attribute Representation <!-- omit in toc -->
Base 10 representation as ASCII string

##### Example(s) <!-- omit in toc -->
`21389`, `41020`

#### (2) Product Revision (prev)

The Product revision. This is the R**xx** part of the PR. For the _RevPi Connect_ it might look like this _PR100274R**03**_. In this case the product revision is _3_.

##### Data Type <!-- omit in toc -->
16-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_2`

##### Attribute Representation <!-- omit in toc -->
Base 10 representation as ASCII string

##### Example(s) <!-- omit in toc -->
`3`

#### (3) Endtest Date

This attribute will be written by the endtester. It represents the current date as of the endtest is done. The format conforms to ISO 8601: YYYY-MM-DD.

##### Data Type <!-- omit in toc -->
32-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_3`

##### Attribute Representation <!-- omit in toc -->
10 character ASCII String

##### Example(s) <!-- omit in toc -->
`2022-03-07`, `2023-12-22`

#### (4) LOT/Batch Number

A LOT or Batch number to identify all components from which the device was assembled.

| :information_source: There is currently no LOT/Batch Number this field was introduced for future use. Currently this field should be written with an ASCII `0`. |
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|

##### Data Type <!-- omit in toc -->
TBD

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_4`

##### Attribute Representation <!-- omit in toc -->
TBD

##### Example(s) <!-- omit in toc -->
TBD

#### (5) MAC Address

This file represents the (first) MAC address for this device. If the device has more MAC addresses assigned they are derived from the first (this) MAC address. The MAC addresses are just incremented by 1 for every assigned MAC address.

##### Data Type <!-- omit in toc -->
17 character ASCII String

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_5`

##### Attribute Representation <!-- omit in toc -->
17 character ASCII String

##### Example(s) <!-- omit in toc -->
`C8:3E:A7:01:32:5E`

#### (6) EEPROM Data Version

This attribute represents the version of the content of the EEPROM. The version must be incremented every time a new version of the revpi-hat-data for the product is released. So the EEPROM Data Version is independent for each product.
A version below 1 is invalid and must be treated as development version.
The version must be incremented for every released change to the content of the EEPROM (with exception to Endtest Date and MAC Address).

##### Data Type <!-- omit in toc -->
16-bit unsigned integer

##### Attribute Path <!-- omit in toc -->
`/proc/device-tree/hat/custom_6`

##### Attribute Representation <!-- omit in toc -->
Base 10 representation as ASCII string

##### Example(s) <!-- omit in toc -->
`3`

## Appendix

### The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION

https://github.com/raspberrypi/hats/blob/master/eeprom-format.md

#### EEPROM Structure

```text
  HEADER  <- EEPROM header (Required)
  ATOM1   <- Vendor info atom (Required)
  ATOM2   <- GPIO map atom (Required)
  ATOM3   <- DT blob atom (Required for compliance with the HAT specification)
  ...
  ATOMn
```

#### EEPROM Header Structure

```text
  Bytes   Field
  4       signature   signature: 0x52, 0x2D, 0x50, 0x69 ("R-Pi" in ASCII)
  1       version     EEPROM data format version (0x00 reserved, 0x01 = first version)
  1       reserved    set to 0
  2       numatoms    total atoms in EEPROM
  4       eeplen      total length in bytes of all eeprom data (including this header)
```

#### Atom Structure

```text
  Bytes   Field
  2       type        atom type
  2       count       incrementing atom count
  4       dlen        length in bytes of data+CRC
  N       data        N bytes, N = dlen-2
  2       crc16       CRC-16 of entire atom (type, count, dlen, data)
```

#### Atom Types

```text
  0x0000 = invalid
  0x0001 = vendor info
  0x0002 = GPIO map
  0x0003 = Linux device tree blob
  0x0004 = manufacturer custom data
  0x0005-0xfffe = reserved for future use
  0xffff = invalid
```

##### Vendor info atom data (type=0x0001):

Note that the UUID is mandatory and must be filled in correctly according to RFC 4122
(every HAT can then be uniquely identified). It protects against the case where a user
accidentally stacks 2 identical HATs on top of each other - this error case is only
detectable if the EEPROM data in each is different. The UUID is also useful for
manufacturers as a per-board 'serial number'.

```text
  Bytes   Field
  16      uuid        UUID (unique for every single board ever made)
  2       pid         product ID
  2       pver        product version
  1       vslen       vendor string length (bytes)
  1       pslen       product string length (bytes)
  X       vstr        ASCII vendor string e.g. "ACME Technology Company"
  Y       pstr        ASCII product string e.g. "Special Sensor Board"
```

##### GPIO map atom data (type=0x0002):

  GPIO map for bank 0 GPIO on 40W B+ header.

  **NOTE** GPIO number refers to BCM2835 GPIO number and **NOT** J8 pin number!

```text
  Bytes   Field
  1       bank_drive  bank drive strength/slew/hysteresis, BCM2835 can only set per bank, not per IO
            Bits in byte:
            [3:0] drive       0=leave at default, 1-8=drive*2mA, 9-15=reserved
            [5:4] slew        0=leave at default, 1=slew rate limiting, 2=no slew limiting, 3=reserved
            [7:6] hysteresis  0=leave at default, 1=hysteresis disabled, 2=hysteresis enabled, 3=reserved
  1       power
            [1:0] back_power  0=board does not back power Pi
                              1=board back powers and can supply up to 1.3A to the Pi
                              2=board back powers and can supply up to 2A to the Pi
                              3=reserved
                              If back_power=2 high current USB mode is automatically enabled.
            [7:2] reserved    set to 0
  28      1 byte per IO pin
            Bits in each byte:
            [2:0] func_sel    GPIO function as per FSEL GPIO register field in BCM2835 datasheet
            [4:3] reserved    set to 0
            [6:5] pulltype    0=leave at default setting,  1=pullup, 2=pulldown, 3=no pull
            [  7] is_used     1=board uses this pin, 0=not connected and therefore not used
```

##### Device Tree atom data (type=0x0003):

Binary data (the name or contents of a `.dtbo` overlay, for board hardware).

For more information on the Device Tree atom contents, see the [Device Tree Guide](devicetree-guide.md).
