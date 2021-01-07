# OpenVehicleDiag
A Rust based open source vehicle ECU diagnostic platform, using the J2534-2 protocol.

This is for my University FYP for my degree at the University of Reading

## Project goals
* JSON Schema creation - Create a custom JSON specification outlying field names that the program will use to understand what packets are sent to the ECU, over which protocol, and how to interpret the ECUs response (See schema drafting documentation)[SCHEMA.md]

* J2534-2 support - The diagnostics application will utilize the J2534-2 protocol for easy use with existing diagnostic hardware from various vendors **DONE**

* Cross OEM support - Using the JSON Schema created, the tool will be able to work regardless of the ECU / Vehicle manufacture, just has to have valid JSON as input

* Cross platform support - Using Iced, the diagnostic application can work on any operation system - **DONE**

* Reverse engineering framework - Build up a framework to help others with reverse engineering OEM database files (Mercedes CBF/SMR-D etc...)

* Create a rust J2534 driver for Macchina's M2 UTD ODB-II module in Rust

## Feature checklists - Current status (5%)
Features marked with '(**WIP**)' are actively being developed!
### Reverse engineering framework
- [ ] Define a schema in JSON for others to follow (**WIP**)
- [ ] User guide on how to write a custom parser for an OEM's Database file 
- [ ] Reverse engineering note on MB CBF (**WIP**)
### Diagnostic application
- [x] J2534-2 API
- [x] Packet tracing support
- [ ] Support K-Line 
- [x] Support CAN 
- [ ] Send and receive custom UDS Commands
- [ ] Realtime performance data viewer for certain views
- [ ] DTC View and clearer
### Macchina J2534 driver
- [x] Support J2534-2 API Passthru functions
- [x] packet Logger
- [ ] ISO9141
- [ ] ISO14230 (KWP2000)
- [ ] J1850 (VPW + PWM)
- [x] CAN
- [x] ISO15765 (ISO-TP)
- [ ] ~~SAE J2610~~
- [ ] ~~J1939~~

*NOTE* SAE J2610 and J1939 are not going to be implemented due to their niche use cases  (I don't have enough time and they are mainly for HGVs)

## IMPORTANT
**Run `git submodule init --recursive` on first clone!**

## Repository structure

### app_rust
Directory of the OVD app (See contained README)

### CBFParser
Parses Mercedes CBF Files into JSON

### SMRParser
Parses Mercedes SMR Files into JSON

### Common
Common library for both parser and GUI Application

### MacchinaM2-J2534-Rust
Contains Common J2534 API references and Driver code for Macchina M2 Under the dash
