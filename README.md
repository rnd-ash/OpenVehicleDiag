![](app_rust/img/logo.png)

Open Vehicle Diagnostics (OVD) is a Rust-based open source vehicle ECU diagnostic platform that makes use of the J2534-2 protocol.

The idea is to make diagnosing and exploring your cars diagnostics functions possible, via an API, thus allowing the ability for you to reprogram ECU’s without the need for a special dealer-only tool.

This is for my University FYP for my degree at the University of Reading

## IMPORTANT
If you are just here for the application and not the entire framework, check the [application folder](app_rust/)

## Youtube video playlist
Videos about OpenVehicleDiag and its development progress can be found [here](https://youtube.com/playlist?list=PLxrw-4Vt7xtty50LmMoLXN2iKiUknbMng)

## Feature checklists - Current status (33%)
Features marked with '(**WIP**)' are actively being developed!
### Reverse engineering framework
- [ ] Define a schema in JSON for others to follow (**WIP**)
- [ ] User guide on how to write a custom parser for an OEM's Database file (**WIP**)
- [ ] Reverse engineering note on MB CBF (**WIP**)
### Diagnostic application
- [x] J2534-2 API
- [x] Packet tracing support
- [ ] Support K-Line 
- [x] Support CAN 
- [x] Send and receive custom UDS Commands
- [ ] Realtime performance data viewer for certain views
- [x] DTC View and clearer
### Macchina J2534 driver
- [x] Support J2534-2 API Passthru functions
- [x] packet Logger
- [ ] ISO9141
- [ ] ISO14230 (KWP2000)
- [x] CAN
- [x] ISO15765 (ISO-TP)

## IMPORTANT
**Run `git submodule update --init --recursive` on first clone!**

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
