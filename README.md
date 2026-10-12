# README

Scanner Programmer for Uniden Scanners

## AI Disclosure

Generative AI was used in the development of this project. Do with that information as you see fit.

## Features

- [x] Maintain the Database of Scanning Presets
- [ ] Control the Scanner Functions Remotely (usually USB)
- [ ] Record Scanned Audio & Metadata

### Supported Scanners

Supported Scanner Models:

- Uniden BCD325P2
- Uniden BCD396XT ‼️
- Uniden BC346XT ‼️
- Uniden BCD996XT ‼️
- Uniden BCD996P2 ‼️
- Uniden BR330T ‼️ (live control; database transfer pending)
- Uniden BCD996T ‼️ (live control; database transfer pending)
- Uniden BCT15 ‼️ (live control; database transfer pending)
- Uniden BCT15X ‼️ (live control; database transfer pending)

‼️ = Untested with a Physical Device

Future Scanner Models (USB interface):

- Uniden BC245XLT
- Uniden BC250D
- Uniden BC780XLT
- Uniden BC785XLT
- Uniden BC895XLT
- Uniden BCD436HP
- Uniden BCD536HP
- Uniden BCT8
- Uniden HomePatrol-1
- Uniden SDS100
- Uniden SDS150
- Uniden SDS200

Unknown if they can be Inplimented:

- Uniden BC125AT (no remote commamd specification available)
- Uniden BC355N (no remote commamd specification available)
- Uniden BC365CRS (no remote commamd specification available)
- Uniden BC75XLT (no remote commamd specification available)
- Uniden BCD260DN (no remote command specification available)
- Uniden HomePatrol-2 (no remote commamd specification available)
- Uniden SR30C (no remote commamd specification available)

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
