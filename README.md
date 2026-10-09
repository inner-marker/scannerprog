# README

Scanner Programmer for Uniden Scanners

## AI Disclosure

Generative AI was used in the development of this project. Do with that information as you see fit.

## Features

- [x] Maintain the Database of Scanning Presets
- [ ] Control the Scanner Remotely (via network or USB, depending on the model)
- [ ] Record Scanned Audio (via 3.5mm audio jack from the scanner into the computer)

### Supported Scanners

At this time, the program only interfaces with the **Uniden BCD325P2** scanner.

Supported Scanner Models:

- Uniden BCD325P2
- Uniden BCD396XT ‼️
- Uniden BC346XT ‼️

‼️ = Untested with a Physical Device

Future Scanner Models (USB interface):

- Uniden BC245XLT
- Uniden BC250D
- Uniden BC346XT
- Uniden BC780XLT
- Uniden BC785XLT
- Uniden BC895XLT
- Uniden BCD396XT
- Uniden BC346XT (new; frequency ranges unverified)
- Uniden BCD996P2
- Uniden BCD996T
- Uniden BCD996XT
- Uniden BCD436HP(UB376Z)
- Uniden BCD536HP(UB375Z)
- Uniden BCT8
- Uniden BCT15
- Uniden BCT15X
- Uniden BR330T

Future Scanner Models (Other interface):

- Uniden SDS200

## Supported Operating Systems

See the [Releases](https://github.com/inner-marker/scannerprog/releases) section on the right for the supported operating systems and available binaries.

Note: there may be additional dependencies required for Linux distributions when installing via `.deb` or `.rpm` packages.

Note: Not all of the features have been tested on all operating systems.

- Linux
  - `scannerprog-*.deb` 
    - (install via `sudo dpkg -i scannerprog-*.deb`)
  - `scannerprog-*.rpm` 
    - (install via `sudo rpm -i scannerprog-*.rpm`)
  - `scannerprog-*.AppImage` 
    - (Stand-alone, no installation required)
- macOS
  - `scannerprog-*.dmg`
- Windows
  - `scannerprog-*-setup.exe`
