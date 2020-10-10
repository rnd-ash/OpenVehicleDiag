# OpenVehicleDiag
A Rust based open source vehicle ECU diagnostic platform, using the J2534-2 protocol

This is for my University FYP for my degree at the University of Reading

## Project goals
* JSON Schema creation - This means I will create a custom JSON specification outlying field names that the program will use to understand what packets are sent to the ECU, over which protocol, and how to interpret the ECUs response

* J2534-2 support - The diagnostics application will utilize the J2534-2 protocol for easy use with existing diagnostic hardware

* Cross OEM support - Using the JSON Schema created, the tool will be able to work regardless of the ECU / Vehicle manufacture

* Cross platform support - Using GTK-Rs, the diagnostic application can work on any operation system

* Reverse engineering framework - Build up a framework to help others with reverse engineering OEM database files (Mercedes CBF/SMR-D etc...)

* Rebuild the Macchina-M2 UTD J2534 driver in Rust

## Feature checklists - Current status (0%)
### Reverse engineering framework
- [ ] Define a schema in JSON for others to follow
- [ ] User guide on how to write a custom parser for an OEM's Database file
- [ ] Reverse engineering note on MB CBF
### Diagnostic application
- [ ] J2534-2 API
- [ ] Packet tracing support
- [ ] Support K-Line 
- [ ] Support CAN
- [ ] Send and receive custom UDS Commands
- [ ] Realtime performance data viewer for certain views
- [ ] DTC View and clearer
### Macchina J2534 driver
- [ ] Support J2534-2 API Passthru functions
- [ ] packet Logger
- [ ] ISO9141
- [ ] ISO14230 (KWP2000)
- [ ] J1850
- [ ] CAN 
- [ ] ISO15765
- [ ] ~~SAE J2610~~
- [ ] ~~J1939~~

*NOTE* SAE J2610 and J1939 are not going to be implimented due to their niche use cases  (I don't have enough time)

## IMPORTANT
**Run `git submodule init` on first clone!**

## Repository structure
### CBFParser
Parses Mercedes CBF Files into JSON

### Common
Common library for both parser and GUI Application

### MacchinaM2-J2534-Rust
Contains Common J2534 API references and Driver code for Macchina M2 Under the dash
