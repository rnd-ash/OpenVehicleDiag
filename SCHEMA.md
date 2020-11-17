# Currently a brainstorm
Structs are stored (here)[common/src/schema.rs]

## Basic meta-data
* Ignition required to communicate with ECU?
* ECU Vendor

## Communication parameters needed
This can be one or more object per ECU (Some ECU's have multiple physical protocols for communication with diagnostic applications)

* Protocol
    * ISO15765
    * CAN
    * ISO9141
    * ISO41320
    * J1850

* Protocol parameters
    * Bus speed
    * Send ID / Receive ID
    * Max block size (ISO15765)
    * Min Separation time (ISO15765)
    * Wake up method (K-Line based protocols)

* Tester present required?
    This would indicate the diagnostic application has to send a constant message to the ECU every *n* milliseconds in order to keep the diagnostic session alive. The payload would be listed in this field.

## ECU Variant detection
how can we detect different variations of the ECU? And what data would be different depending on variation (Example. Some variations of the same ECU may have a slightly different function list or way to interpret function responses)

* Vendor
* Software version
* Hardware version

## DTC Table
* DTC Name (EG: P2000)
* DTC Desc (String)

## Function Table
This is to list all the functions or commands that can be ran on the ECU

* Function name
* Function description
* Function preparation (Some functions require others to be ran first)
* Security level - OVD will, for now, only run functions that don't require seed-key access to the ECU
* Presentation list for interpreting the ECU's response
* Function payload to send to ECU
* Some way of telling OVD what a negative response looks like vs a positive response

### Presentations
* Reference to parent function?
* Unit (rpm / C / deg)
* Upper limit (Optional)
* Lower limit (Optional)
* bit range within ECU response to look for
* Conversion formula for converting raw to human readable form (Optional)