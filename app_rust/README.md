# Open vehicle diagnostics app



## Features (Current)
* CAN Tracer
* OBD Toolbox
* Automated UDS/KWP2000 scanner
* Read and clear error codes

## Screenshots
[See the screenshots folder](screenshots/)

## Diagnostic adapter API Support
* SAE J2534 (Passthru)

## Platform support
| OS      | Adapter support | Note |
|---------|-----------------|------|
| Windows | ALL J2534       |      |
| Linux   | Macchina only   | Use Macchina M2 and [this](github.com/rnd-ash/MacchinaM2-J2534-Rust) driver |
| Mac OSX | Macchina only   | Use Macchina M2 and [this](github.com/rnd-ash/MacchinaM2-J2534-Rust) driver |

## Youtube video playlist
App progress updates and demos are [posted here](https://youtube.com/playlist?list=PLxrw-4Vt7xtty50LmMoLXN2iKiUknbMng)

## Launch args
* `-debug_ui` - Enables debugging of the user interface showing all layout constraints and boundaries


## Questions and answers

### Question
Why must I use Macchina's M2 adapter on Linux and Mac OSX?

### Answer
The J2534 diagnostic API only officially supports Windows. Therefore, all commercial J2534 devices will only have drivers for Windows.
This is why I designed a custom cross-platform J2534 driver for Macchina's M2 module so that the API can be used on UNIX based operating systems.


### Question
Can you add xxx feature to this app?

### Answer
Submit a github issue to request new features 😃