# README

Scanner Programmer for Uniden Scanners

## AI Disclosure

Generative AI was used in the development of this project. Do with that information as you see fit.

## Features

- [x] Maintain the Database of Scanning Presets
- [ ] Control the Scanner Functions Remotely (usually USB)
- [ ] Record Scanned Audio & Metadata

### Supported Scanners

At this time, the program only interfaces with the **Uniden BCD325P2** scanner.

Supported Scanner Models:

- Uniden BCD325P2
- Uniden BCD396XT ‼️
- Uniden BC346XT ‼️
- Uniden BCD996XT ‼️
- Uniden BCD996P2 ‼️

‼️ = Untested with a Physical Device

Future Scanner Models (USB interface):

- Uniden BC245XLT
- Uniden BC250D
- Uniden BC346XT
- Uniden BC780XLT
- Uniden BC785XLT
- Uniden BC895XLT
- Uniden BC346XT (new; frequency ranges unverified)
- Uniden BCD396XT
- Uniden BCD436HP
- Uniden BCD536HP
- Uniden BCD996T
- Uniden BCD996XT
- Uniden BCT8
- Uniden BCT15
- Uniden BCT15X
- Uniden BR330T
- Uniden SDS100
- Uniden SDS150
- Uniden SDS200

Unknown if they can be Inplimented:

- BC125AT (no remote commamd specification available)
- BC355DN (no remote commamd specification available)

## Supported Operating Systems

See the [Releases](https://github.com/inner-marker/scannerprog/releases) section on the right for the supported operating systems and available binaries.

Note: there may be additional dependencies required for Linux distributions when installing via `.deb` or `.rpm` packages.

Note: Not all of the features have been tested on all operating systems.

- Linux
  - `scannerprog-*.deb` 
    - (install via `sudo dpkg -i scannerprog-*.deb`)
    - Ubuntu, Mint, Debian
  - `scannerprog-*.rpm` 
    - (install via `sudo rpm -i scannerprog-*.rpm`)
    - Fedora, CentOS, openSUSE
  - `scannerprog-*.AppImage` 
    - (Stand-alone, no installation required)
    - Most Linux distributions
- macOS
  - `scannerprog-*.dmg` (Unsigned, may require bypassing Gatekeeper)
- Windows
  - `scannerprog-*-setup.exe`
