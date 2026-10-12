# Progress


## To-Do

- [x] Detect a Uniden scanner (USB vid `0x1964`)
- [x] Read the basic scanner information

### Database Management

- [x] Read the database from the scanner
- [x] Save/Load database to/from a `.ron` file
- [x] Edit the database locally
- [x] Validate edits to the database
  - Names
  - Quick keys
- [x] Write the database back to the scanner

Additional Work:

- Validation
  - [~] Memory limits: Up to 500 systems, 1,000 total sites (max 256 per system), 20 groups per system, and 25,000 channels (500 max IDs or 1,000 frequencies per system) (per-model system/site/channel counts, memory, and frequency ranges are validated; per-system limits are not yet)
  - [~] Frequency limits: 25-512 MHz, 758-824 MHz, 849-869 MHz, and 894-960 MHz. 

- [x] (v0.1.3) Refactor the database. The database should be a single common database shared across all supported scanner models. Then, when a scanner is connected, only the portions of the database usable by a specific scanner model will able to be loaded. The full database will remain accessable and editable when an incompattable scanner is connected, but a UI element and a corresponding warning message should indicate the incompatibility.

### Scanning

- [ ] (v0.2.0) Remote control of the scanner functions
  - The plan here is to slightly re-think the way scanning is controlled. On the radio, the UI is somewhat clunky and is limited by the available buttons and knobs. I want to create a more intuitive and flexible interface for controlling the scanner functions. The goal is to make it easier for users to manage scanning without being constrained by the physical interface of the radio. The underlying scanner functions will be the same, but the control UI on the computer will be difrent.
  - [ ] unable and disable scan systems and groups
  - [ ] listen to a single frequency
  - [ ] listen to a single system or group

### Audio

- [ ] (v0.3.0) Stream audio from 3.5mm input to a specified output device
- [ ] (v0.3.1) Record audio from the scanner (when audio breaks a threshold level)

**Maybe sometime, if it can be done for free**

- [ ] (no current plan) Speech-to-text conversion for recorded audio