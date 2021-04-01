
# OpenVehicleDiag (OVD) ECU JSON Specification

Version 1.0 (31/03/2021)

This document outlines the JSON Specification which OpenVehicleDiag uses
for ECU diagnostics. It is designed to be a simple, easy to understand replacement for ODX, and proprietary data formats such as Daimler' CBF and SMR-D data format.

## Table of contents

* [JSON Root](#JSON-Root)
* [ECU Variant](#ECU-Variant)
  * [Pattern](#Pattern)
  * [Errors](#Error)
  * [Services](#Service)
    * [Parameter](#Parameter)
* [Connection](#Connection)
  * [Connection type](#Connection-Type)

## JSON Root

Example

```json
{
  "name": "Awesome ECU",
  "description": "My awesome engine ECU!",
  "variants": [ ... ],
  "connections": [ ... ]
}
```

`root` **Properties**
|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**name**|String|Name of the ECU|Yes|
|**description**|String|A brief description of the ECU|Yes|
|**variants**|Array|A list of ECU Variants. See [ECU Variant](#ECU-Variant)|Yes|
|**connections**|Array|A list of connection methods for communicating with the ECU .See [Connection](#Connection)|Yes|


## ECU Variant

An ECU Variant is used to identify a particular software version of an ECU. Since an ECU can get updates over time, this is necessary as with certain software updates, an ECU can change/modify error code descriptions and also add/remove diagnostic routines that can be executed.

Example of a single ECU Variant entry

```json
{
  "name": "SW_V_01",
  "description": "My Awesome ECU software version 0.1",
  "patterns": [ ... ],
  "errors": [ ... ],
  "adjustments": [ ... ],
  "actuations": [ ... ],
  "functions": [ ... ],
  "downloads": [ ... ]

}
```
|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**name**|String|A short version string of the ECU Software version|Yes|
|**description**|String|Description of the ECU Software version|Yes|
|**patterns**|Array|A list of [Pattern](#Pattern) objects that are used to identify a particular vendor of an ECU's software version|Yes|
|**errors**|Array|A list of [error](#Error) objects that this ECU Software version can potentially throw|Yes|
|**adjustments**|Array|A list of [service](#Service) objects that can be executed on this ECU variant in order to modify certain functions of the ECU, such as specifying a new engine idle RPM|No|
|**actuations**|Array|A list of [service](#Service) objects that can be executed on this ECU variant in order to manipulate components the ECU controls temporarily during the diagnostic session|No|
|**functions**|Array|A list of [service](#Service) objects that can be executed on this ECU variant in order to modify the ECUs current state, such as soft rebooting an ECU|No|
|**downloads**|Array|A list of [service](#Service) objects that can be executed on this ECU variant in order to read data from the ECU|No|

### Pattern

An ECU pattern is used to identify which hardware vendor is responsible for implementing the parent variant's software version, since its possible for 1 ECU software implementation to be implemented by multiple hardware vendors such as Bosch, Siemens and Delphi.

Example ECU Pattern

```json
{
  "vendor": "rnd-ash@github.com",
  "vendor_id": 12345
}
```

|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**vendor**|String|Who makes the physical ECU|Yes|
|**vendor_id**|Integer|The vendor ID of the pattern. Every vendor must have a unique vendor_id for each software version of the ECU. This is a 2 byte value that is retrieved with [`read_dcs_id`](https://github.com/rnd-ash/OpenVehicleDiag/blob/9413eb20f15f54f8c822ac10db7b70b2845358c6/app_rust/src/commapi/protocols/kwp2000/read_ecu_identification.rs#L55) or [`read_dcx_mcc_id`](https://github.com/rnd-ash/OpenVehicleDiag/blob/9413eb20f15f54f8c822ac10db7b70b2845358c6/app_rust/src/commapi/protocols/kwp2000/read_ecu_identification.rs#L81) under KWP2000 |Yes|

### Error

An error is used to describe a throwable DTC (Diagnostic trouble code) that an ECU can throw under certain circumstances.

Example error

```json
{
  "error_name": "P2082-002",
  "summary": "MAF implausible",
  "description": "Mass airflow sensor is producing inconsistent readings",
  "envs": [ ... ]
}

```

|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**error_name**|String|The shorthand error code|Yes|
|**summary**|String|The summary of the error|Yes|
|**description**|String|A more detailed description of the error|Yes|
|**envs**|Array|A list of status data that can be queried about the DTC. This can usually give some useful insight into various performance metrics and sensor readings when the DTC was triggered. This can be done over KWP2000 using [`get_status_dtc`](https://github.com/rnd-ash/OpenVehicleDiag/blob/9413eb20f15f54f8c822ac10db7b70b2845358c6/app_rust/src/commapi/protocols/kwp2000/read_status_dtc.rs#L6) Each entry is a [Parameter](#Parameter)|No|

### Service

A service is used to describe an IO operation that can be executed on the ECU.

Example

```json
{
  "name": "Read injector status",
  "description": "Retrieves the injector quantity per stroke for all cylinders",
  "payload": "22FB",
  "input_params": [ ... ],
  "output_params": [ ... ],
}
```



|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**name**|String|Name of the service|Yes|
|**description**|String|Description of what the service does|Yes|
|**payload**|Hex String|The payload to send to the ECU|Yes|
|**payload**|Array|A list of [parameters](#Parameter) that can be added to the end of the existing content in the `payload` field. These will be inputted by the user before service execution|No|
|**input_params**|Array|A list of [parameter](#Parameter) objects that will be used to format the users input into the ECU Request payload|No|
|**output_params**|Array|A list of [parameter](#Parameter) objects that will be used to interpret the ECU's positive response message.|No|


#### Parameter

A parameter is used to define a data format used for either input or output, as well as defining the position in the bit stream in either the ECU payload or ECU Response message


Example JSON

```json
{
  "name": "Supply voltage",
  "description": "Supply voltage being measured by the ECU",
  "unit": "V",
  "start_bit": 32,
  "length_bits": 8,
  "byte_order": "BigEndian",
  "data_format": "Identical",
  "valid_bounds": {
    "upper": 100.0,
    "lower": 0.0
  }
}
```
|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**name**|String|Name of the parameter|Yes|
|**description**|String|Description of the parameter|Yes|
|**unit**|String|Optional unit string, which will be appended to the output value when being displayed as a string|No|
|**start_bit**|String|Start bit in the ECU Payload or ECU response message where this parameter is stored|Yes|
|**length_bits**|Integer|The number of bits long the parameter is|Yes|
|**byte_order**|String|The byte order of the parameter (See below)|Yes|
|**data_format**|Enum|Data format of the parameter. See [here](#A-list-of-valid-data-formats) for a full list of accepted data format entries|Yes|
|**valid_bounds**|JSON|Multi use. If the parameter is in the parent services' `input_parameters` section, this field demotes the upper and lower
bound for user input. If the parameter is in parent services' `output_parameters` section, it is used for graphing, to mark the upper and lower bounds of the graph's range|No|


* **Allowed values (`data_format`)**
  * `BigEndian` - Byte order is Big Endian
  * `LittleEndian` - Byte order is Little Endian


#### A list of valid data formats

* **Binary**

The output value is formatted as a binary string.

Example JSON:

```json
"data_format": "Binary,
```

Example outputs:

```
INPUT: [0x20]
OUTPUT: "0b00100000"
```

---

* **Hexdump**


The output value is formatted as a Hex array string.

Example JSON:

```json
"data_format": "HexDump,
```

Example outputs:

```
INPUT: [0x20, 0xFF, 0x00]
OUTPUT: "[0x20 0xFF 0x00]"
```

---

* **String**

The input value is decoded as a String using a specified String encoding option

Example JSON:

```json
"data_format": {
  "String": "Utf8"
},
```


Example outputs:
```
INPUT: [0x54, 0x65, 0x73, 0x74, 0x20, 0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]
OUTPUT: "Test message"
```

##### Allowed values (`String`)

* `ASCII` - The String is encoded as ASCII (1 byte per character)
* `Utf8` - The String is encoded as UTF-8 (1 byte per character)
* `Utf16` - The String is encoded as UTF-16 (2 bytes per character)


---

* **Bool**

The output value is formatted as a boolean. The value `0` is interpreted as `False`, and any other value is interpreted as `True`. If the fields `pos_name` and `neg_name` as specified (See below), then `pos_name` is used in place of `True`, and `neg_name` is used in place of `False`.

Example JSON:

```json
"data_format": {
  "Bool": {
    "pos_name": "This is positive",
    "neg_name": "This is negative"
  }
},
```

Example outputs:
```
INPUT: [0x01]
OUTPUT: "This is positive"

INPUT: [0xFF]
OUTPUT: "This is positive"

INPUT: [0x00]
OUTPUT: "This is negative"
```

---

* **Table**

The output value is formatted as a String based on an enum table. Each enum entry (Table) can have a defined start and end value, in case the enum's definition covers a wide range of numbers. If no match was found in the table, `UNDEFINED` is returned.

Example JSON:

```json
"data_format": {
  "Table": [
    {
      "name": "This value is between 0 and 10",
      "start": 0.0,
      "end": 10.0
    },
    {
      "name": "This value is only 11",
      "start": 11.0,
      "end": 11.0
    },
    {
      "name": "This value is only 100",
      "start": 100.0,
      "end": 100.0
    }
  ]
},
```

Example outputs:

```
INPUT: [0x00]
OUTPUT: "This value is between 0 and 10"

INPUT: [0x05]
OUTPUT: "This value is between 0 and 10"

INPUT: [0x64]
OUTPUT: "This value is only 100"

INPUT: [0xFF]
OUTPUT: "UNDEFINED(0xFF)"
```

---

* **Identical**

The output value is formatted as number based on the raw input.

Example JSON:

```json
"data_format": "Identical"
```

Example outputs:

```
INPUT: [0x00]
OUTPUT: "0"

INPUT: [0x05]
OUTPUT: "5"

INPUT: [0x64]
OUTPUT: "100"

INPUT: [0xFF]
OUTPUT: "255"
```

---

* **Linear**

The output value is calculated by a simple `y=mx+c` equation, where the `multiplier` field is `m`, and the `offset` field is `c`

Example JSON:

```json
"data_format": {
  "Linear": {
    "multiplier": 0.125,
    "offset": -40.0
  }
},
```

Example outputs:

```
INPUT: [0x00]
OUTPUT: "-40.0"

INPUT: [0x10]
OUTPUT: "-38.0"

INPUT: [0xFF]
OUTPUT: "-8.125"
```

---

* **ScaleLinear**

> :warning: **This is not implemented in 1.0**

The output value is calculated using a table of linear functions

---

* **RatFunc**

> :warning: **This is not implemented in 1.0**

The output value is calculated using a rational function

---

* **ScaleRatFunc**

> :warning: **This is not implemented in 1.0**

The output value is calculated using table of rational functions

---

* **TableInterpretation**

> :warning: **This is not implemented in 1.0**

The output value is calculated using defined interpolation

---

* **Compucode**

> :warning: **This is not implemented in 1.0**

The output value is calculated using a Java virtual machine that runs bytecode that implementes the `I_CompuCode()` interface

---

## Connection

A connection entry is used in order to allow OpenVehicleDiag to identify automatically how to configure the OBD-II ports interfaces in order to communicate with the ECU in the vehicle.

Example

```json
{
  "baud": 500000,
  "send_id": 2016,
  "global_send_id": 2016,
  "connection_type": {
    "ISOTP": {
      "blocksize": 8,
      "st_min": 20
    }
  },
  "server_type": "KWP2000",
  "recv_id": 2024
}
```

|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**baud**|Integer|The baud speed (Bus speed) of the connection|Yes|
|**send_id**|Integer|The diagnostic tester ID|Yes|
|**recv_id**|Integer|The diagnostic receiver ID|Yes|
|**global_send_id**|Integer|The global tester present diagnostic ID|No|
|**connection_type**|Enum|The physical connection method to the ECU. See [Connection Type](#Connection-Type)|Yes|
|**server_type**|Enum|The diagnostic server type|Yes|

#### server_type

Specifies the diagnostic server type that the ECU uses

* **Allowed values**
  * `KWP2000` - The ECU requires a KWP2000 diagnostic server
  * `UDS` - The ECU requires a UDS diagnostic server


### Connection Type

Example (LIN Connection method)

```json
...
"connection_type": {
  "LIN": {
    "max_segment_size": 254,
    "wake_up_method": "FiveBaudInit"
  }
}
...

```

|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**max_segment_size**|Integer|The maximum frame size allowed to be transmitted over K-Line|Yes|
|**wake_up_method**|Enum|Specifies the wake up method for K-Line|Yes|

#### LIN.wake_up_method

Specifies the wake up method for the K-Line (LIN) network on the OBD-II Port

* **Allowed values**
  * `FastInit` - Utilize the Fast init wake up method
  * `FiveBaudInit` - Utilize the five baud initialization wake up method

----

Example (ISO-TP Connection method)

```json
...
"connection_type": {
  "ISOTP": {
    "blocksize": 8,
    "st_min": 20,
    "ext_can_addr": false,
    "ext_isotp_addr": false,
  }
}
...
```

|   |Type|Description|Required|
|:--:|:--:|:--|:--:|
|**blocksize**|Integer|The maximum number of CAN Frames allowed to be transmitted over ISO-TP before the ECU must send another flow control message back to the tester|Yes|
|**st_min**|Integer|The minimum delay in milliseconds before sending consecutive CAN Frames to the ECU|Yes|
|**ext_can_addr**|Boolean|Indicates if CAN ID shall be 29bit (Extended - True) or 11bit (Standard - False)|Yes|
|**ext_isotp_addr**|Boolean|Indicates if the ISO-TP layer shall use extended addressing or not|Yes|