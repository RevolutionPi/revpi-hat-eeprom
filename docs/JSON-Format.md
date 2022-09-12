# RevPi HAT EEPROM JSON format

The RevPi HAT EEPROM format is an extension to the RPi HAT EEPROM format¹ and describes additional data fields that are used by a RevPi.

To be able to generate an image for the RevPi HAT EEPROM with extensions to the official format, a tool was developed. This tool get's some of the information over cmdline parameters, others can be defined in a JSON file which is parsed by the tool. This document describes the format of the JSON file.

## JSON format

The JSON for the Revolution Pi HAT EEPROM tool contains one main section with data like product type and revision and two JSON objects each of which is added as array.

_gpiobanks_ describes settings for a gpio bank of the SoC. At the moment only bank 0 is supported. See [The Raspberry Pi HAT ID EEPROM FORMAT SPECIFICATION](https://github.com/RevolutionPi/revpi-hat-eeprom/blob/master/RevPi-HAT-EEPROM-Format.md#the-raspberry-pi-hat-id-eeprom-format-specification) for more information. It contains an array of gpio objects with settings for separate gpios. Add only gpios to this array whose settings should be changed.

### Example JSON

```json
{
    "version": 1,
    "vstr": "Kunbus GmbH",
    "pstr": "RevPi MiniXL",
    "pid": 642,
    "prev": 3,
    "pver": 120,
    "dtstr": "revpi-example-2022",
    "gpiobanks": [
        {
            "drive": "8mA",
            "slew": "default",
            "hysteresis": "enable",
            "gpios": [
                {
                    "gpio": 2,
                    "fsel": "input",
                    "pull": "default"
                },
                {
                    "gpio": 3,
                    "fsel": "output",
                    "pull": "none"
                },
                {
                    "gpio": 4,
                    "fsel": "alt1",
                    "pull": "up"
                }
            ]
        }
    ]
}
```

### Main properties

All fields are mandatory.

| Field     | JSON Datatype             | Range       | Description | Example  |
|:----------|:--------------------------|:------------|:------------|:---------|
| version   | number                    | u16         | Version of the EEPROM format | 1 |
| vstr      | string                    | 255&#160;chars | Vendor of the device | Kunbus&#160;GmbH  |
| pstr      | string                    | 255&#160;chars | Product name         | RevPi&#160;MiniXL |
| pid       | number                    | u16         | Product identification number | 42 |
| prev      | number                    | u16         | Product revision     | 3 |
| pver      | number                    | u16         | Product version      | 21 |
| dtstr     | string                    | 255&#160;chars | Name of devicetree blob for this device | revpi-example-2022 |
| gpiobanks | array of gpiobank objects |             | List of gpiobanks to configure (only bank0 supported at the moment) | |

### GPIOBanks object

> #### INFO
> Please have a look into the datasheet of the specific Broadcom SoC for more information about bank and gpio functions and settings.

> #### INFO
> Only GPIOs that are listed in the JSON will be modified. All other GPIOs will keep their settings.

| Field      | Field Type | JSON Datatype         | Range     | Description                              |
|:-----------|:-----------|:----------------------|:----------|:-----------------------------------------|
| drive      | mandatory  | string (enum)         | see below | Set drive strength of gpio bank          |
| slew       | mandatory  | string (enum)         | see below | Set slew rate of gpio bank               |
| hysteresis | mandatory  | string (enum)         | see below | Set hysteresis of gpio bank              |
| gpios      | mandatory  | array of gpio objects |           | List of gpios, that should be configured |

#### Enum _drive_ property

Allowed values for the enum **drive** from the GPIO banks object.
Controls the drive strength of all gpios on a bank.

| Value     | Description                                   |
|:----------|:----------------------------------------------|
| "default" | Keeps the drive strength setting of this bank |
| "2mA"     | Set drive strength to 2 mA                    |
| "4mA"     | Set drive strength to 4 mA                    |
| "6mA"     | Set drive strength to 6 mA                    |
| "8mA"     | Set drive strength to 8 mA                    |
| "10mA"    | Set drive strength to 10 mA                   |
| "12mA"    | Set drive strength to 12 mA                   |
| "14mA"    | Set drive strength to 14 mA                   |
| "16mA"    | Set drive strength to 16 mA                   |

#### Enum _slew_ property

Allowed values for the enum **slew** from the GPIO banks object.
Controls the slew setting of the gpio bank.

| Value          | Description                        |
|:---------------|:-----------------------------------|
| "default"      | Keeps the slew setting of the bank |
| "ratelimiting" | Sets slew to rate limiting         |
| "nolimit"      | slew is not limited                |

#### Enum _hysteresis_ property

Allowed values for the enum **hysteresis** from the GPIO banks object.
Controls the different hysteresis settings on the gpio bank.

| Value     | Description                              |
|:----------|:-----------------------------------------|
| "default" | Keeps the hysteresis setting of the bank |
| "disable" | Disable the hysteresis on the bank       |
| "enable"  | Enable the hysteresis on the bank        |

### GPIO object

| Field   | Field Type | JSON Datatype | Range             | Description           |
|:--------|:-----------|:--------------|:------------------|:----------------------|
| "gpio"  | mandatory  | number        | 2 - 27 for bank 0 | Number of the GPIO    |
| "fsel"  | mandatory  | string (enum) | see below         | Function select       |
| "pull"  | mandatory  | string (enum) | see below         | Pull resistor setting |

#### Enum _fsel_ property

Allowed values for the enum **fsel** from the GPIO object.
Controls direction and alternative function settings of a gpio.

| Value    | Description                     |
|:---------|:--------------------------------|
| "input"  | Set gpio direction to input     |
| "output" | Set gpio direction to output    |
| "alt0 "  | Activate alternative function 0 |
| "alt1 "  | Activate alternative function 1 |
| "alt2 "  | Activate alternative function 2 |
| "alt3 "  | Activate alternative function 3 |
| "alt4 "  | Activate alternative function 4 |
| "alt5 "  | Activate alternative function 5 |

#### Enum _pull_ property

Allowed values for the enum **pull** from the GPIO object.
Controls the pull resistors on each gpio.

| Value     | Description                        |
|:----------|:-----------------------------------|
| "default" | Keep setting of pin                |
| "up"      | Activate pull up resistor on pin   |
| "down"    | Activate pull down resistor on pin |
| "none"    | No pull resitor active on pin      |

## Validate own JSON files

Own EEPROM definitions in JSON can be validated either by using it directly with the Revolution Pi HAT EEPROM image generator tool or by validating it beforehand with the provided JSON schema file `eep.schema`.

To validate own JSON files for the HAT EEPROM tool, the python tool and library `jsonschema` can be used together with the provided schema file for validation.
Install `jsonschema` from the command line with pip (python installation required):

```
$ pip install jsonschema
```

There are also libraries for other languages than python, check out https://json-schema.org/implementations.html for more.

After jsonschema is installed, it can be called from the commandline with your json and the schema file:

```
jsonschema --instance own.json eep.schema
```


¹ https://github.com/raspberrypi/hats/blob/master/eeprom-format.md
