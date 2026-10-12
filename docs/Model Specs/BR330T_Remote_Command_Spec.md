# BR330T Remote Command Specification

> Markdown conversion of Uniden's *BR330T Remote Command* document (section 7.1, preceded by the "Uniden Programming Control Codes" end-user license agreement), 34 pages, © 2003–2004 Uniden America Corporation.
> The BR330T (BearTracker/racing scanner) uses an early form of the comma-separated protocol later used by the BCD996T/BCT15: no sites (each Motorola/EDACS band variant is its own system type, and the band plan lives in `TRN`), racing systems, an on-air clone channel (`AIR`) and a bar-antenna setting (`BAR`). The document refers to a CTCSS/DCS code list (codes 0–231) but doesn't include one.
> Wire formats are reproduced as written in the source. Apparent typos in the original are listed under [Errata notes](#errata-notes-conversion) at the end.

---

## Contents

- [License terms (summary)](#license-terms-summary)
- [7.1 Remote Command](#71-remote-command)
  - [Remote communication format](#remote-communication-format)
  - [Format of this document](#format-of-this-document)
  - [General notes](#general-notes)
  - [Remote command list](#remote-command-list)
- [Command reference](#command-reference)
  - [Remote control](#remote-control) — GID, KEY, POF, QSH, STS, GLG
  - [System information](#system-information) — MDL, VER
  - [Program control mode](#program-control-mode) — PRG, EPG
  - [System settings](#system-settings) — BLT, BSV, CLR, KBP, OMS, PRI, AGV, BAR
  - [Scan settings](#scan-settings) — SCT … MEM
  - [Search / Close Call settings](#search--close-call-settings) — SCO, BBS, GLF, ULF, LOF, CLC
  - [Service search settings](#service-search-settings) — SSP
  - [Custom search settings](#custom-search-settings) — CSG, CSP
  - [Weather settings](#weather-settings) — WXS, SGP
  - [Other settings](#other-settings) — TON, AIR, CNT, VOL, SQL
  - [Test](#test) — WIN, BAV
- [Errata notes (conversion)](#errata-notes-conversion)

---

## License terms (summary)

The PDF opens with Uniden America Corporation's *End User License Agreement* for the "Uniden Programming Control Codes". It is summarized here rather than reproduced; refer to the PDF for the binding text. The main terms:

- **Grant (2A):** you may use the codes "as is", royalty-free, personal, non-assignable and non-exclusive, **solely to create software or firmware that extends Uniden scanners or makes them work with a PC or other control device.**
- **Restrictions (2B–2C, 3):** don't use the codes to build a competing scanner, or for anything other than controlling the Uniden models they apply to. Don't distribute, sell, sublicense, modify or create derivative works of the codes beyond what section 2 allows.
- **Confidentiality (2G):** the codes and documentation are Uniden confidential and proprietary information.
- **Required notice (2H):** software that contains the codes, and the user materials that come with it, must be marked: *"This product contains Uniden proprietary and/or copyright control codes. Used with permission."*
- **No support, no warranty (2D–2E, 5):** the documentation may be inaccurate (it went through translation), and Uniden has no obligation to correct it or provide support.
- **No endorsement (2L):** permission to use the codes isn't an endorsement of your product.
- **Other terms:** Uniden may audit your use (2N); you indemnify Uniden for claims arising from your products (2P); liability is limited to the greater of $10 or what you paid (6B); Texas law governs (8).

---

## 7.1 Remote Command

### Remote communication format

| Item | Value |
|---|---|
| BPS rate | 9600 / 19200 / 38400 / 57600 / 115200 bps |
| Start / Stop bit | 1 bit / 1 bit |
| Data length | 8 bit |
| Parity check | None |
| Code | ASCII |
| Flow control | None |
| Return code | Carriage Return (`\r`) only |

### Format of this document

Each command is described as:

- **`<COMMAND NAME>`** — summary of the command's function
- **Controller → Radio** — command format
- **Radio → Controller** — response format

### General notes

1. Every command must wait for a response from the scanner before the next command is accepted.
2. All memory access commands are accepted only in **Program Mode**. Use `PRG` to enter Program Mode and `EPG` to exit.
3. Error messages are not described per command; the scanner returns:
   1. Command format error / value error: `ERR\r`
   2. Command is invalid at this time: `NG\r`
   3. Framing error: `FER\r`
   4. Overrun error: `ORER\r`
4. `[\r]` means "hit the Enter key" / "send the Return code".
5. Some long commands/responses are shown across multiple lines for page width, but on the wire they are **a single line**.
6. In a set command, parameters sent as only `,` (empty) are not changed.
7. A set command is aborted if any format error is detected.
8. `[INDEX]` / `[xxx_INDEX]` is an index into the internal memory chain. The Dynamic Memory Allocation structure always uses it as a handle to access data and to traverse forward/reverse or up/down. Range: 1 to the maximum memory block (**about 7600**).
9. `[FRQ]`, `[BASEx]` and `[LIMIT_x]` are in **frequency format**: an 8‑digit number with no decimal point, ordered from the 1 GHz digit down to the 100 Hz digit.
   - e.g. `08510125` = 851.0125 MHz
10. `[TGID]` is in TGID format; the format depends on the trunked system type (see the separate appendix, not included).
11. `[NAME]` is a custom name. If the user sets only space characters, the name reverts to the default name.

### Remote command list

"PM" = accepted in Program Mode only.

| No. | Category | Command | Function | PM |
|---:|---|---|---|:---:|
| 1 | Remote Control | GID | Get Current Talkgroup ID Status | |
| 2 | Remote Control | KEY | Push KEY | |
| 3 | Remote Control | POF | Power OFF | |
| 4 | Remote Control | QSH | Go to quick search hold mode | |
| 5 | Remote Control | STS | Get Status | |
| 6 | Remote Control | GLG | Get Reception Status | |
| 7 | System Information | MDL | Get Model Info | |
| 8 | System Information | VER | Get Firmware Version | |
| 9 | Programming Mode Control | PRG | Enter Program Mode | |
| 10 | Programming Mode Control | EPG | Exit Program Mode | |
| 11 | System Setting | BLT | Get/Set Backlight | ✓ |
| 12 | System Setting | BSV | Get/Set Battery Save | ✓ |
| 13 | System Setting | CLR | Clear All Memory | ✓ |
| 14 | System Setting | KBP | Get/Set Key Beep | ✓ |
| 15 | System Setting | OMS | Get/Set Opening Message | ✓ |
| 16 | System Setting | PRI | Get/Set Priority Mode | ✓ |
| 17 | System Setting | AGV | Get/Set Auto Gain Control | ✓ |
| 18 | System Setting | BAR | Get/Set Bar Antenna | ✓ |
| 19 | Scan Settings | SCT | Get System Count | ✓ |
| 20 | Scan Settings | SIH | Get System Index Head | ✓ |
| 21 | Scan Settings | SIT | Get System Index Tail | ✓ |
| 22 | Scan Settings | QSL | Get/Set System Quick Lockout | ✓ |
| 23 | Scan Settings | QGL | Get/Set Group Quick Lockout | ✓ |
| 24 | Scan Settings | CSY | Create System | ✓ |
| 25 | Scan Settings | DSY | Delete System | ✓ |
| 26 | Scan Settings | CPS | Copy System | ✓ |
| 27 | Scan Settings | SIN | Get/Set System Info | ✓ |
| 28 | Scan Settings | TRN | Get/Set Trunk Info | ✓ |
| 29 | Scan Settings | TFQ | Get/Set Trunk Frequency Info | ✓ |
| 30 | Scan Settings | AGC | Append Channel Group | ✓ |
| 31 | Scan Settings | AGT | Append TGID Group | ✓ |
| 32 | Scan Settings | DGR | Delete Group | ✓ |
| 33 | Scan Settings | GIN | Get/Set Group Info | ✓ |
| 34 | Scan Settings | ACC | Append Channel | ✓ |
| 35 | Scan Settings | ACT | Append TGID | ✓ |
| 36 | Scan Settings | DCH | Delete Channel | ✓ |
| 37 | Scan Settings | CIN | Get/Set Channel Info | ✓ |
| 38 | Scan Settings | TIN | Get/Set TGID Info | ✓ |
| 39 | Scan Settings | GLI | Get Lockout TGID (for RVW L/O ID) | ✓ |
| 40 | Scan Settings | SLI | Get Search L/O TGID | ✓ |
| 41 | Scan Settings | ULI | Unlock TGID (for RVW L/O ID) | ✓ |
| 42 | Scan Settings | LOI | Lockout ID (TGID) | ✓ |
| 43 | Scan Settings | REV | Get Rev Index | ✓ |
| 44 | Scan Settings | FWD | Get Fwd Index | ✓ |
| 45 | Scan Settings | RMB | Get Remains of Memory Block | ✓ |
| 46 | Scan Settings | MEM | Get Memory Used | ✓ |
| 47 | Search / Close Call Settings | SCO | Get/Set Search/Close Call Settings | ✓ |
| 48 | Search / Close Call Settings | BBS | Get/Set Broadcast Screen Band Settings | ✓ |
| 49 | Search / Close Call Settings | GLF | Get Global Lockout Frequency | ✓ |
| 50 | Search / Close Call Settings | ULF | Unlock Global L/O | ✓ |
| 51 | Search / Close Call Settings | LOF | Lockout Frequency | ✓ |
| 52 | Search / Close Call Settings | CLC | Get/Set Close Call Settings | ✓ |
| 53 | Service Search Settings | SSP | Get/Set Service Search Settings | ✓ |
| 54 | Custom Search Settings | CSG | Get/Set Custom Search Group | ✓ |
| 55 | Custom Search Settings | CSP | Get/Set Custom Search Settings | ✓ |
| 56 | Weather Settings | WXS | Get/Set Weather Setting | ✓ |
| 57 | Weather Settings | SGP | Get/Set SAME Group Settings | ✓ |
| 58 | Tone-Out Settings | TON | Get/Set Tone-Out Settings | ✓ |
| 59 | On-Air Clone Settings | AIR | Get/Set On-Air Clone Settings | ✓ |
| 60 | LCD Contrast Setting | CNT | Get/Set LCD Contrast Settings | ✓ |
| 61 | Volume Level Settings | VOL | Get/Set Volume Level Settings | |
| 62 | Squelch Level Settings | SQL | Get/Set Squelch Level Settings | |
| 63 | Test | WIN | *Get Window Voltage | |
| 64 | Test | BAV | *Get Battery Voltage | |

---

## Command reference

Common parameter definitions reused by several commands:

| Parameter | Values |
|---|---|
| `[RSV]` | Reserve parameter — always only `,` (empty) unless stated otherwise |
| `[MOD]` | Modulation: `AUTO` / `AM` / `FM` / `NFM` / `WFM` (subset per command; no `FMB` on this model) |
| `[ATT]` | Attenuation: `0` OFF / `1` ON |
| `[DLY]` | Delay time: `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |

**Search step `[STP]`** (QSH, SCO, CSP):

| Code | Step | Code | Step |
|---|---|---|---|
| `AUTO` | Auto | `1250` | 12.5 k |
| `500` | 5 k | `1500` | 15 k |
| `625` | 6.25 k | `2000` | 20 k |
| `750` | 7.5 k | `2500` | 25 k |
| `833` | 8.33 k | `5000` | 50 k |
| `900` | 9 k | `10000` | 100 k |
| `1000` | 10 k | | |

**Broadcast Screen bitfield `[BSC]`** (QSH, SCO) — 16 digits, each `0` (OFF) or `1` (ON), leftmost first:

| Digit position (left → right) | Meaning |
|---|---|
| 1 | Pager |
| 2 | FM |
| 3 | UHF TV |
| 4 | VHF TV |
| 5 | NOAA WX |
| 6 | AM |
| 7 | Band 1 |
| 8 | Band 2 |
| … | … |
| 16 | Band 10 |

---

### Remote control

#### GID — Get Current Talkgroup ID Status

```text
# Controller → Radio
GID\r
# Radio → Controller
GID,[SYS_TYPE],[TGID],[ID_SRCH_MODE],[NAME1],[NAME2],[NAME3]\r
```

| Parameter | Description |
|---|---|
| `[SYS_TYPE]` | System type (see CSY) |
| `[TGID]` | TGID |
| `[ID_SRCH_MODE]` | `0` ID SCAN mode / `1` ID SEARCH mode |
| `[NAME1]` | System name (alpha tag) |
| `[NAME2]` | Group name (alpha tag) |
| `[NAME3]` | TGID name (alpha tag) |

**Function:** Returns the TGID currently displayed on the LCD. After the TGID has been read once, the scanner returns `,,,,,\r` until the next reception.
**Note:** Returns `,,,,,\r` when no TGID is displayed.

#### KEY — Push KEY

```text
# Controller → Radio
KEY,[KEY_CODE],[KEY_MODE]\r
# Radio → Controller
KEY,OK\r
```

| `[KEY_CODE]` | Key |
|---|---|
| `M` | MENU |
| `F` | F |
| `H` | HOLD |
| `S` | SCAN/SEARCH |
| `L` | L/O |
| `1` | 1 / PRI |
| `2` | 2 / WX |
| `3`–`6`, `8`, `9` | Digits |
| `7` | 7 / RCL |
| `0` | 0 / CAR |
| `.` | . / NO / REV |
| `E` | E / YES / ATT |
| `>` | VFO right (set `KEY_MODE` to `P`) |
| `<` | VFO left (set `KEY_MODE` to `P`) |
| `^` | VFO push |
| `P` | POWER / LIGHT / LOCK |

| `[KEY_MODE]` | Meaning |
|---|---|
| `P` | Press (one push) |
| `L` | Long press (press and hold a few seconds) |
| `H` | Hold (press and hold until Release received) |
| `R` | Release (cancel Hold state) |

Examples:

```text
# Ex.1 — Press MENU key
→ KEY,M,P\r
← OK\r

# Ex.2 — Press F + SCAN
→ KEY,F,H\r    # hold F key
← OK\r
→ KEY,S,P\r    # press SCAN key (F + SCAN operation)
← OK\r
→ KEY,F,R\r    # release F key
→ OK\r         # (direction arrow as printed in source)

# Ex.3 — Press and hold L/O key
→ KEY,L,L\r
← OK\r
```

The scanner can't be turned off with this command (use POF).

#### POF — Power OFF

```text
# Controller → Radio
POF\r
# Radio → Controller
POF,OK\r
```

Turns off the scanner. After this command the scanner accepts no further commands.

#### QSH — Go to quick search hold mode

```text
# Controller → Radio
QSH,[FRQ],[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP]\r
# Radio → Controller
QSH,OK\r      # ①
QSH,NG\r      # ② invalid in Menu Mode, during Direct Entry or Quick Save
```

| Parameter | Description |
|---|---|
| `[FRQ]` | Frequency |
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[CODE_SRCH]` | CTCSS/DCS search: `0` OFF / `1` ON |
| `[BSC]` | Broadcast Screen bitfield (see above) |
| `[REP]` | Repeater Find: `0` OFF / `1` ON |

**Function:** Sets an arbitrary frequency and changes to Quick Search Hold (VFO) mode. Parameters such as `STP` change the contents of the Srch/CloCall option.

#### STS — Get Current Status

```text
# Controller → Radio
STS\r
# Radio → Controller (single line on the wire)
STS,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE],[L3_CHAR],[L3_MODE],[L4_CHAR],[L4_MODE],...,[L8_CHAR],[L8_MODE],[SQL],[MUT],[BAT],[WAT]\r
```

| Parameter | Description |
|---|---|
| `[DSP_FORM]` | Display form, 4–8 digits; each `0` = small font, `1` = large font |
| `[Ln_CHAR]` | Line *n* characters, 16 chars (fixed length) |
| `[Ln_MODE]` | Line *n* display mode, 16 chars |
| `[SQL]` | Squelch: `0` CLOSE / `1` OPEN |
| `[MUT]` | Mute: `0` OFF / `1` ON |
| `[BAT]` | Battery low: `0` no alert / `1` alert |
| `[WAT]` | Weather alert: `0` no alert / `1` alert / `$$$` alert SAME event code |

**Display mode characters (`[Ln_MODE]`):** space = normal, `*` = reverse, `_` = underline. If all 16 characters are normal, only `,` is sent. The number of `[Lx_CHAR]`/`[Lx_MODE]` pairs depends on the display form.

**Ex.1** — Menu screen (squelch OPEN, mute OFF, no battery alert, no weather alert):

```text
→ STS\r
← 1111,                  # DSP_FORM: 4 lines, all large font
  -- M E N U --   ,      # L1_CHAR
  ________________,      # L1_MODE (underlined)
  Program System  ,      # L2_CHAR
  ****************,      # L2_MODE (reversed)
  Srch/CloCall Opt,      # L3_CHAR
  ,                      # L3_MODE
  Search for. . . ,      # L4_CHAR
  ,                      # L4_MODE
  1,0,0,0,\r             # SQL,MUT,BAT,WAT (trailing comma as printed)
```

**Ex.2** — Hold screen (squelch CLOSE, mute ON, no battery alert, weather alert):

```text
→ STS\r
← 01100,                 # DSP_FORM (printed with 5 digits for 6 lines)
  HOLD    L/O     ,      # L1_CHAR
  ,                      # L1_MODE
  SYSTEM 1        ,      # L2_CHAR
  ,                      # L2_MODE
  851.0125MHz     ,      # L3_CHAR
  ,                      # L3_MODE
  P NFM ATT       ,      # L4_CHAR
  ,                      # L4_MODE
  S1: 5           ,      # L5_CHAR
  ,                      # L5_MODE
  GRP 2       WX,        # L6_CHAR
  ,                      # L6_MODE
  0,1,0,1,\r             # SQL,MUT,BAT,WAT
```

#### GLG — Get Reception Status

```text
# Controller → Radio
GLG\r
# Radio → Controller
GLG,[FRQ/TGID],[MOD],[ATT],[CTCSS/DCS],[NAME1],[NAME2],[NAME3],[SQL],[MUT]\r
GLG,,,,,,,,,\r      # nothing received yet
```

| Parameter | Description |
|---|---|
| `[FRQ/TGID]` | Frequency or TGID |
| `[MOD]` | `AM` / `FM` / `NFM` / `WFM` |
| `[ATT]` | `0` OFF / `1` ON |
| `[CTCSS/DCS]` | CTCSS/DCS status (0–231; code list not included in this document) |
| `[NAME1]` | System or search name |
| `[NAME2]` | Group name |
| `[NAME3]` | Channel name |
| `[SQL]` | `0` CLOSE / `1` OPEN |
| `[MUT]` | `0` OFF / `1` ON |

The scanner returns `GLG,,,,,,,,,\r` until it detects a frequency or TGID.

---

### System information

#### MDL — Get Model Info

```text
# Controller → Radio
MDL\r
# Radio → Controller
MDL,BR330T\r
```

#### VER — Get Firmware Version

```text
# Controller → Radio
VER\r
# Radio → Controller
VER,Version 1.00.00\r
```

---

### Program control mode

#### PRG — Enter Program Mode

```text
# Controller → Radio
PRG\r
# Radio → Controller
PRG,OK\r   # ① success
PRG,NG\r   # ② refused
```

Invalid while in Menu Mode, during Direct Entry, or during Quick Save. In Program Mode the scanner displays "Remote Mode" on line 1 and "Keypad Lock" on line 2; the POWER key and Function key still work.

#### EPG — Exit Program Mode

```text
# Controller → Radio
EPG\r
# Radio → Controller
EPG,OK\r
```

The scanner exits Program Mode and goes to Scan Hold Mode.

---

### System settings

#### BLT — Get/Set Backlight *(PM)*

```text
# Controller → Radio
BLT\r           # ① get
BLT,##\r        # ② set
# Radio → Controller
BLT,##\r        # ①
BLT,OK\r        # ②
```

`##`: `IF` infinite · `10` 10 s · `30` 30 s · `KY` keypress · `SQ` squelch.

#### BSV — Get/Set Battery Save *(PM)*

```text
# Controller → Radio
BSV\r           # ① get
BSV,#\r         # ② set — 0 OFF / 1 ON
# Radio → Controller
BSV,#\r         # ①
BSV,OK\r        # ②
```

#### CLR — Clear All Memory *(PM)*

```text
# Controller → Radio
CLR\r
# Radio → Controller
CLR,OK\r
```

Resets all memory to initial settings. Takes about 20 seconds. Only the PC Control (baud rate) setting is not reset.

#### KBP — Get/Set Key Beep *(PM)*

```text
# Controller → Radio
KBP\r              # ① get
KBP,[LEVEL]\r      # ② set
# Radio → Controller
KBP,[LEVEL]\r      # ①
KBP,OK\r           # ②
```

`[LEVEL]`: beep level `0` Auto / `1`–`15` / `99` OFF.

#### OMS — Get/Set Opening Message *(PM)*

```text
# Controller → Radio
OMS\r                                              # ① get
OMS,[L1_CHAR],[L2_CHAR],[L3_CHAR],[L4_CHAR]\r      # ② set
# Radio → Controller
OMS,[L1_CHAR],[L2_CHAR],[L3_CHAR],[L4_CHAR]\r      # ①
OMS,OK\r                                           # ②
```

Each `[Ln_CHAR]` is max. 16 characters. If only spaces are set, the line reverts to the default message.

#### PRI — Get/Set Priority Mode *(PM)*

```text
# Controller → Radio
PRI\r        # ① get
PRI,#\r      # ② set
# Radio → Controller
PRI,#\r      # ①
PRI,OK\r     # ②
```

`#`: `0` OFF / `1` ON / `2` PLUS ON.

#### AGV — Get/Set Auto Gain Control *(PM)*

```text
# Controller → Radio
AGV\r                          # ① get
AGV,[AGC_ANALOG],[RSV]\r       # ② set
# Radio → Controller
AGV,[AGC_ANALOG],[RSV]\r       # ①
AGV,OK\r                       # ②
```

| Parameter | Values |
|---|---|
| `[AGC_ANALOG]` | AGC for analog audio: `0` OFF / `1` ON |
| `[RSV]` | Reserved — the source says it is always `0` here |

#### BAR — Get/Set Bar Antenna *(PM)*

```text
# Controller → Radio
BAR\r           # ① get (printed "BAR [\r]" with a space)
BAR,#\r         # ② set — 0 OFF / 1 ON
# Radio → Controller
BAR,#\r         # ①
BAR,OK\r        # ②
```

---

### Scan settings

#### SCT — Get System Count *(PM)*

```text
# Controller → Radio
SCT\r
# Radio → Controller
SCT,###\r    # ### = 0–200
```

#### SIH — Get System Index Head *(PM)*

```text
# Controller → Radio
SIH\r
# Radio → Controller
SIH,[SYS_INDEX]\r
```

Returns the first index of the stored system list.

#### SIT — Get System Index Tail *(PM)*

```text
# Controller → Radio
SIT\r
# Radio → Controller
SIT,[SYS_INDEX]\r
```

Returns the last index of the stored system list.

#### QSL — Get/Set System Quick Lockout *(PM)*

```text
# Controller → Radio
QSL\r                                                                                     # ① get
QSL,[PAGE0],[PAGE1],[PAGE2],[PAGE3],[PAGE4],[PAGE5],[PAGE6],[PAGE7],[PAGE8],[PAGE9]\r     # ② set
# Radio → Controller
QSL,[PAGE0],[PAGE1],[PAGE2],[PAGE3],[PAGE4],[PAGE5],[PAGE6],[PAGE7],[PAGE8],[PAGE9]\r     # ①
QSL,OK\r                                                                                  # ②
```

Each `[PAGEn]` is 10 digits, each `0`–`2`:

| Digit | Meaning | Display |
|---|---|---|
| `0` | Quick key not assigned | `-` |
| `1` | On | the number |
| `2` | Off | `*` |

| Page | Quick keys |
|---|---|
| `PAGE0` | 1–9, 0 |
| `PAGE1` | 11–19, 10 |
| `PAGE2` | 21–29, 20 |
| `PAGE3` | 31–39, 30 |
| `PAGE4` | 41–49, 40 |
| `PAGE5` | 51–59, 50 |
| `PAGE6` | 61–69, 60 |
| `PAGE7` | 71–79, 70 |
| `PAGE8` | 81–89, 80 |
| `PAGE9` | 91–99, 90 |

Order matches the LCD icons. Cannot turn on/off a quick key that has no system.

#### QGL — Get/Set Group Quick Lockout *(PM)*

```text
# Controller → Radio
QGL,[SYS_INDEX]\r                # ① get
QGL,[SYS_INDEX],##########\r     # ② set (printed with a space before the digits)
# Radio → Controller
QGL,##########\r                 # ①
QGL,OK\r                         # ②
```

`##########` — group quick key status of the system: `0` not assigned (`-`) / `1` On / `2` Off (`*`), in LCD icon order. Cannot turn on/off a quick key that has no group.

#### CSY — Create System *(PM)*

```text
# Controller → Radio
CSY,[SYS_TYPE]\r
# Radio → Controller
CSY,[SYS_INDEX]\r     # -1 if no resources
```

| `[SYS_TYPE]` | System type |
|---|---|
| `RACE` | Racing |
| `CNV` | Conventional |
| `M82S` | Motorola 800 MHz Type 2 Standard |
| `M82P` | Motorola 800 MHz Type 2 Splinter |
| `M92` | Motorola 900 MHz Type 2 |
| `MV2` | Motorola VHF Type 2 |
| `MU2` | Motorola UHF Type 2 |
| `M81S` | Motorola 800 MHz Type 1 Standard |
| `M81P` | Motorola 800 MHz Type 1 Splinter |
| `EDN` | EDACS Narrow |
| `EDW` | EDACS Wide |
| `EDS` | EDACS SCAT |
| `LTR` | LTR |

The index returned is the handle for get/set of system info.

#### DSY — Delete System *(PM)*

```text
# Controller → Radio
DSY,[SYS_INDEX]\r
# Radio → Controller
DSY,OK\r
```

#### CPS — Copy System *(PM)*

```text
# Controller → Radio
CPS,[SYS_INDEX1],[NAME]\r
# Radio → Controller
CPS,[SYS_INDEX2]\r     # -1 if no resources
```

| Parameter | Description |
|---|---|
| `[SYS_INDEX1]` | Index of the source system |
| `[NAME]` | Name of the copy |
| `[SYS_INDEX2]` | Index of the copy |

#### SIN — Get/Set System Info *(PM)*

```text
# Controller → Radio
SIN,[INDEX]\r     # ① get
SIN,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[SKP],[MOD],[ATT],[RSV],[RSV]\r   # ② set
# Radio → Controller
# ① (single line on the wire)
SIN,[SYS_TYPE],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[SKP],[MOD],[ATT],[RSV],[RSV],[REV_INDEX],[FWD_INDEX],[CHN_GRP_HEAD],[CHN_GRP_TAIL],[SEQ_NO]\r
SIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | System index |
| `[SYS_TYPE]` | See CSY |
| `[NAME]` | Name (max. 16 chars) |
| `[QUICK_KEY]` | Quick key (1–99 / `.` = none); racing systems always return `.` |
| `[HLD]` | System hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[MOD]` | Modulation (trunked systems only): `AUTO` / `FM` / `NFM` |
| `[ATT]` | Attenuation (trunked systems only): `0` / `1` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward system index in scan order |
| `[CHN_GRP_HEAD]` / `[CHN_GRP_TAIL]` | Channel group index head / tail of the system |
| `[SEQ_NO]` | System sequence number (1–200) |
| `[RSV]` | Always only `,` |

- Parameters not applicable to the system type are returned as `,` and ignored in set commands.
- `,`-only parameters are unchanged in set; any format error aborts.

#### TRN — Get/Set Trunk Info *(PM)*

```text
# Controller → Radio
TRN,[INDEX]\r     # ① get
# ② set (single line on the wire)
TRN,[INDEX],[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[I-CALL],[C-CH],[EMG],[EMGL],[FMAP],[CTM_FMAP],[BASE1],[STEP1],[OFFSET1],[BASE2],[STEP2],[OFFSET2],[BASE3],[STEP3],[OFFSET3],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
TRN,[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[I-CALL],[C-CH],[EMG],[EMGL],[FMAP],[CTM_FMAP],[BASE1],[STEP1],[OFFSET1],[BASE2],[STEP2],[OFFSET2],[BASE3],[STEP3],[OFFSET3],[RSV],[TGID_GRP_HEAD],[TGID_GRP_TAIL],[ID_LOUT_GRP_HEAD],[ID_LOUT_GRP_TAIL]\r
TRN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | System index |
| `[ID_SEARCH]` | `0` ID Scan mode / `1` Search mode |
| `[S_BIT]` | Motorola status bit: `0` Ignore / `1` Yes |
| `[END_CODE]` | Motorola end code: `0` Ignore / `1` Yes |
| `[AFS]` | EDACS ID format: `0` Decimal / `1` AFS |
| `[I-CALL]` | I-Call: `0` OFF / `1` ON |
| `[C-CH]` | Control channel only: `0` OFF / `1` ON |
| `[EMG]` | Emergency alert: `0` Ignore / `1`–`9` Alert |
| `[EMGL]` | Emergency alert level: `0` OFF / `1`–`15` |
| `[FMAP]` | Fleet map: `0`–`15` preset / `16` custom |
| `[CTM_FMAP]` | Custom fleet map, 8 digits (blocks 0–7), each `0`–`E` = size code 0–14 |
| `[BASEn]` / `[STEPn]` / `[OFFSETn]` | Band plan *n* (1–3): base frequency, step, offset. `[STEP3]`/`[OFFSET3]` are for Motorola UHF/VHF systems only |
| `[TGID_GRP_HEAD]` / `[TGID_GRP_TAIL]` | TGID index head / tail of the system |
| `[ID_LOUT_GRP_HEAD]` / `[ID_LOUT_GRP_TAIL]` | L/O TGID group index head / tail |
| `[RSV]` | Always only `,` |

- Non-applicable parameters are returned as `,` and ignored in set commands.
- `,`-only parameters unchanged in set; any format error aborts.

#### TFQ — Get/Set Trunk Frequency Info *(PM)*

```text
# Controller → Radio
TFQ,[CHN_INDEX]\r                         # ① get
TFQ,[CHN_INDEX],[FRQ],[LCN],[LOUT]\r      # ② set
# Radio → Controller
TFQ,[FRQ],[LCN],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX]\r   # ①
TFQ,OK\r                                  # ②
```

| Parameter | Description |
|---|---|
| `[CHN_INDEX]` | Channel (frequency) index |
| `[FRQ]` | Frequency of the trunked system |
| `[LCN]` | LCN |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward index in the system frequency group |
| `[SYS_INDEX]` | System index of the frequency |
| `[GRP_INDEX]` | Index of the system frequency group |

- `,`-only parameters unchanged in set; any format error aborts.
- For Motorola or EDACS SCAT systems, `[LCN]` is ignored.

#### AGC — Append Channel Group *(PM)*

```text
# Controller → Radio
AGC,[SYS_INDEX]\r
# Radio → Controller
AGC,[GRP_INDEX]\r    # -1 if no resources
```

#### AGT — Append TGID Group *(PM)*

```text
# Controller → Radio
AGT,[SYS_INDEX]\r
# Radio → Controller
AGT,[GRP_INDEX]\r    # -1 if no resources
```

#### DGR — Delete Group *(PM)*

```text
# Controller → Radio
DGR,[GRP_INDEX]\r
# Radio → Controller
DGR,OK\r
```

Deletes a channel group or TGID group.

#### GIN — Get/Set Group Info *(PM)*

```text
# Controller → Radio
GIN,[GRP_INDEX]\r                                 # ① get
GIN,[GRP_INDEX],[NAME],[QUICK_KEY],[LOUT]\r       # ② set
# Radio → Controller
GIN,[GRP_TYPE],[NAME],[QUICK_KEY],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO]\r   # ①
GIN,OK\r                                          # ②
```

| Parameter | Description |
|---|---|
| `[GRP_INDEX]` | Group index |
| `[GRP_TYPE]` | `C` channel group / `T` TGID group |
| `[NAME]` | Name (max. 16 chars). For a **racing group**, the format is the car number (3 characters, left-aligned and padded with blanks if shorter), a blank, then the driver name, with the rest filled with blanks |
| `[QUICK_KEY]` | `1`–`9`, `0` (= 10), `.` = none |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward group index in the system |
| `[SYS_INDEX]` | System index |
| `[CHN_HEAD]` / `[CHN_TAIL]` | Channel index head / tail of the group list |
| `[SEQ_NO]` | Group sequence number in the system |

#### ACC — Append Channel *(PM)*

```text
# Controller → Radio
ACC,[GRP_INDEX]\r    # channel group index
# Radio → Controller
ACC,[CHN_INDEX]\r    # -1 if no resources
```

#### ACT — Append TGID *(PM)*

```text
# Controller → Radio
ACT,[GRP_INDEX]\r    # TGID group index
# Radio → Controller
ACT,[INDEX]\r        # appended TGID index ([TGID_INDEX]); -1 if no resources
```

#### DCH — Delete Channel *(PM)*

```text
# Controller → Radio
DCH,[INDEX]\r        # channel, TGID, or trunked-system frequency index
# Radio → Controller
DCH,OK\r
```

#### CIN — Get/Set Channel Info *(PM)*

```text
# Controller → Radio
CIN,[INDEX]\r     # ① get
CIN,[INDEX],[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL]\r   # ② set
# Radio → Controller
# ① (single line on the wire)
CIN,[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX]\r
CIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Channel index |
| `[NAME]` | Name (max. 16 chars) |
| `[FRQ]` | Channel frequency |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` |
| `[CTCSS/DCS]` | CTCSS/DCS mode (0–231) |
| `[TLOCK]` | CTCSS/DCS tone lockout: `0` OFF / `1` ON |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[PRI]` | Priority: `0` OFF / `1` ON |
| `[ATT]` | `0` OFF / `1` ON |
| `[ALT]` | Alert tone: `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | Alert tone level: `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward channel index in the channel group |
| `[SYS_INDEX]` | System index of the channel |
| `[GRP_INDEX]` | Group index of the channel |

#### TIN — Get/Set TGID Info *(PM)*

```text
# Controller → Radio
TIN,[INDEX]\r                                          # ① get
TIN,[INDEX],[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL]\r  # ② set
# Radio → Controller
TIN,[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX]\r   # ①
TIN,OK\r                                               # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | TGID index |
| `[NAME]` | Name (max. 16 chars) |
| `[TGID]` | TGID |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[PRI]` | Priority: `0` OFF / `1` ON |
| `[ALT]` | `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward TGID index in the TGID group |
| `[SYS_INDEX]` | System index of the TGID |
| `[GRP_INDEX]` | Group index of the TGID |

#### GLI — Get Lockout TGID (for Rvw L/O ID) *(PM)*

```text
# Controller → Radio
GLI,[SYS_INDEX]\r
# Radio → Controller
GLI,[TGID]\r     # one locked-out TGID
GLI,-1\r         # no more lockout TGIDs
```

Call repeatedly until `-1` is returned.

#### SLI — Get Search L/O TGID *(PM)*

```text
# Controller → Radio
SLI,[SYS_INDEX]\r
# Radio → Controller
SLI,[TGID]\r
SLI,-1\r         # no more lockout TGIDs
```

Gets the Search L/O TGIDs — locked-out TGIDs that don't belong to any group in the system. Call repeatedly until `-1`.

#### ULI — Unlock TGID (for Rvw L/O ID) *(PM)*

```text
# Controller → Radio
ULI,[SYS_INDEX],[TGID]\r
# Radio → Controller
ULI,OK\r
```

#### LOI — Lockout ID (TGID) *(PM)*

```text
# Controller → Radio
LOI,[SYS_INDEX],[TGID]\r
# Radio → Controller
LOI,OK\r
```

#### REV — Get Rev Index *(PM)*

```text
# Controller → Radio
REV,[INDEX]\r
# Radio → Controller
REV,[INDEX]\r    # -1 if none
```

#### FWD — Get Fwd Index *(PM)*

```text
# Controller → Radio
FWD,[INDEX]\r
# Radio → Controller
FWD,[INDEX]\r    # -1 if none
```

#### RMB — Get Remains of Memory Block *(PM)*

```text
# Controller → Radio
RMB\r
# Radio → Controller
RMB,####\r       # free blocks, 0–9999
```

#### MEM — Get Memory Used *(PM)*

```text
# Controller → Radio
MEM\r
# Radio → Controller
MEM,###\r        # percent of memory used, 0–100
```

---

### Search / Close Call settings

#### SCO — Get/Set Search/Close Call Settings *(PM)*

```text
# Controller → Radio
SCO\r             # ① get
SCO,[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE]\r   # ② set
# Radio → Controller
SCO,[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE]\r   # ①
SCO,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[CODE_SRCH]` | CTCSS/DCS search: `0` OFF / `1` ON |
| `[BSC]` | Broadcast Screen bitfield |
| `[REP]` | Repeater find: `0` OFF / `1` ON |
| `[MAX_STORE]` | Max auto store (1–256) |
| `[RSV]` | Always only `,` |

#### BBS — Get/Set Broadcast Screen Band Settings *(PM)*

```text
# Controller → Radio
BBS,[INDEX]\r                         # ① get
BBS,[INDEX],[LIMIT_L],[LIMIT_H]\r     # ② set
# Radio → Controller
BBS,[LIMIT_L],[LIMIT_H]\r             # ①
BBS,OK\r                              # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | 1–9, `0` = 10 (called `[SCR_INDEX]` in the definitions) |
| `[LIMIT_L]` | Lower limit frequency (00000000–99999999) |
| `[LIMIT_H]` | Upper limit frequency (00000000–99999999) |

#### GLF — Get Global Lockout Freq *(PM)*

```text
# Controller → Radio
GLF\r
# Radio → Controller
GLF,[FRQ]\r      # one locked-out frequency (1000–13000000)
GLF,-1\r         # no more
```

Call repeatedly until `-1` to get the whole global L/O list.

#### ULF — Unlock Global L/O *(PM)*

```text
# Controller → Radio
ULF,[FRQ]\r      # 1000–13000000
# Radio → Controller
ULF,OK\r
```

#### LOF — Lock Out Frequency *(PM)*

```text
# Controller → Radio
LOF,[FRQ]\r      # 1000–13000000
# Radio → Controller
LOF,OK\r
```

#### CLC — Get/Set Close Call Settings *(PM)*

```text
# Controller → Radio
CLC\r                                                                   # ① get
CLC,[CC_MODE],[CC_OVERRIDE],[ALTM],[ALTB],[ALTL],[ALTP],[CC_BAND]\r     # ② set
# Radio → Controller
CLC,[CC_MODE],[CC_OVERRIDE],[ALTM],[ALTB],[ALTL],[ALTP],[CC_BAND]\r     # ①
CLC,OK\r                                                                # ②
```

| Parameter | Description |
|---|---|
| `[CC_MODE]` | `1` ON / `0` OFF |
| `[CC_OVERRIDE]` | `1` ON / `0` OFF |
| `[ALTM]` | Alert mode: `N` none / `B` beep / `L` light / `A` beep + light |
| `[ALTB]` | Alert beep: `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[ALTP]` | Close Call pause: `3`, `5`, `10`, `15`, `30`, `45`, `60` s / `INF` |
| `[CC_BAND]` | 7-digit bitfield (see below) |

`[CC_BAND]` — each digit `0` OFF / `1` ON, left → right:

| Position | Band |
|---|---|
| 1 | Reserve (always `0`) |
| 2 | VHF |
| 3 | AIR BAND |
| 4 | VHF HIGH |
| 5 | UHF 1 |
| 6 | UHF 2 |
| 7 | 800 MHz+ |

---

### Service search settings

#### SSP — Get/Set Service Search Settings *(PM)*

```text
# Controller → Radio
SSP,[SRCH_INDEX]\r                                  # ① get
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT]\r         # ② set
# Radio → Controller
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT]\r         # ①
SSP,OK\r                                            # ②
```

| `[SRCH_INDEX]` | Range | `[SRCH_INDEX]` | Range |
|---|---|---|---|
| `1` | Public Safety | `8` | FRS/GMRS |
| `2` | News | `9` | Racing |
| `3` | HAM Radio | `10` | TV Broadcast |
| `4` | Marine | `11` | FM Broadcast |
| `5` | Railroad | `12` | Special |
| `6` | Air | `13` | AM Broadcast |
| `7` | CB Radio | | |

| Parameter | Description |
|---|---|
| `[DLY]` | `0` OFF / `1`–`5` |
| `[ATT]` | `0` / `1` |
| `[HLD]` | System hold time for search with scan (0–255) |
| `[LOUT]` | Lockout for search with scan: `0` / `1` |

---

### Custom search settings

#### CSG — Get/Set Custom Search Group *(PM)*

```text
# Controller → Radio
CSG\r               # ① get
CSG,##########\r    # ② set
# Radio → Controller
CSG,##########\r    # ①
CSG,OK\r            # ②
```

Each `#` is `0` (valid) or `1` (invalid) for custom search ranges 1–10, in LCD icon order.

#### CSP — Get/Set Custom Search Settings *(PM)*

```text
# Controller → Radio
CSP,[SRCH_INDEX]\r   # ① get
CSP,[SRCH_INDEX],[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[SKP],[HLD],[LOUT],[C-CH],[RSV],[RSV]\r   # ② set
# Radio → Controller
CSP,[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[SKP],[HLD],[LOUT],[C-CH],[RSV],[RSV]\r   # ①
CSP,OK\r             # ②
```

| Parameter | Description |
|---|---|
| `[SRCH_INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[LIMIT_L]` / `[LIMIT_H]` | Lower / upper limit (1000–13000000) |
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` / `1` |
| `[HLD]` | System hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[C-CH]` | Control channel only: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

---

### Weather settings

#### WXS — Get/Set Weather Settings *(PM)*

```text
# Controller → Radio
WXS\r                          # ① get
WXS,[DLY],[ATT],[ALT_PRI]\r    # ② set
# Radio → Controller
WXS,[DLY],[ATT],[ALT_PRI]\r    # ① (printed with a space after "WXS,")
WXS,OK\r                       # ②
```

| Parameter | Description |
|---|---|
| `[DLY]` | `0` OFF / `1`–`5` |
| `[ATT]` | `0` / `1` |
| `[ALT_PRI]` | Weather alert priority: `0` OFF / `1` ON |

#### SGP — Get/Set SAME Group Settings *(PM)*

```text
# Controller → Radio
SGP,[SAME_INDEX]\r     # ① get
SGP,[SAME_INDEX],[NAME],[FIPS1],[FIPS2],[FIPS3],[FIPS4],[FIPS5],[FIPS6],[FIPS7],[FIPS8]\r   # ② set
# Radio → Controller
SGP,[NAME],[FIPS1],[FIPS2],[FIPS3],[FIPS4],[FIPS5],[FIPS6],[FIPS7],[FIPS8]\r                # ①
SGP,OK\r               # ②
```

| Parameter | Description |
|---|---|
| `[SAME_INDEX]` | 1–5 |
| `[NAME]` | SAME group name (max. 16 chars) |
| `[FIPS1-8]` | FIPS code, 6 digits (`000000`–`999999`), or dashes = none |

---

### Other settings

#### TON — Get/Set Tone-Out Settings *(PM)*

```text
# Controller → Radio
TON,[INDEX]\r     # ① get (printed "TON[INDEX]" in source)
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV]\r   # ② set
# Radio → Controller
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV]\r   # ①
TON,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[FRQ]` | Channel frequency |
| `[MOD]` | `AUTO` / `FM` / `NFM` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | `0` OFF / `1`–`5` / `INF` |
| `[ALT]` | `0` OFF / `1`–`9` |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[TONE_A]` / `[TONE_B]` | Tone frequency in 0.1 Hz units (e.g. `10000` = 1000.0 Hz) |
| `[RSV]` | Always only `,` |

#### AIR — Get/Set On-Air Clone Settings *(PM)*

```text
# Controller → Radio
AIR\r               # ① get
AIR,[FRQ],[MOD]\r   # ② set
# Radio → Controller
AIR,[FRQ],[MOD]\r   # ①
AIR,OK\r            # ②
```

| Parameter | Description |
|---|---|
| `[FRQ]` | Frequency |
| `[MOD]` | `AUTO` / `FM` / `NFM` |

#### CNT — Get/Set LCD Contrast Settings *(PM)*

```text
# Controller → Radio
CNT\r
# Radio → Controller
CNT,[CONTRAST]\r      # 1–15
```

Described as Get/Set, but the source shows only the get form.

#### VOL — Get/Set Volume Level Settings

```text
# Controller → Radio
VOL\r              # ① get
VOL,[LEVEL]\r      # ② set — 0 OFF / 1–15
# Radio → Controller
VOL,[LEVEL]\r      # ①
VOL,OK\r           # ②
```

#### SQL — Get/Set Squelch Level Settings

```text
# Controller → Radio
SQL\r              # ① get
SQL,[LEVEL]\r      # ② set — 0 OPEN / 1–14 / 15 CLOSE
# Radio → Controller
SQL,[LEVEL]\r      # ①
SQL,OK\r           # ②
```

---

### Test

#### WIN — Get Window Voltage

```text
# Controller → Radio
WIN\r
# Radio → Controller
WIN,###,[FRQ]\r  # ### = A/D value 0–255
```

Returns current window voltage and its frequency (1 GHz digit → 100 Hz digit). Test-mode command.

#### BAV — Get Battery Voltage

```text
# Controller → Radio
BAV\r
# Radio → Controller
BAV,####\r       # A/D value 0–1023
```

Battery level [V] = (3.2 V × `####` × 2) / 1023. Test-mode command.

---

## Errata notes (conversion)

These are observations about the source text, not part of the original spec:

- **CTCSS/DCS** — GLG and CIN refer to a "CTCSS/DCS Code List" (codes 0–231) that isn't included in this document.
- **TGID formats** — general note 10 refers to "another Appendix" for TGID formats, which isn't included.
- **KEY** — response is given as `KEY,OK\r`, but the examples show bare `OK\r`; Ex.2's last line shows `→ OK` (direction reversed). Verify against hardware.
- **QSH** — FUNCTION text begins "SS specifies arbitrary frequency…" (garbled; other models say "UASD").
- **STS** — both example responses end with a trailing comma after `[WAT]` (`1,0,0,0,` / `0,1,0,1,`), and Ex.2's display form `01100` has 5 digits for a 6-line screen.
- **AGV** — `[RSV]` is described as "always 0", whereas `[RSV]` elsewhere is always empty.
- **BAR** — get form printed `BAR [\r]` with a space.
- **QGL** — set form printed with a space before the digits: `QGL,[SYS_INDEX], ##########`.
- **CSY** — "The Index if Created System" (of).
- **ACT** — response shows `[INDEX]` but the definition names it `[TGID_INDEX]`; the description says "Append Channel to the group".
- **BBS** — the command uses `[INDEX]` but the definition names it `[SCR_INDEX]`.
- **LOF** — header printed `<COMMAND LOF` (missing `>`).
- **WXS** — response printed `WXS, [DLY]` with a space.
- **TON** — get form is printed `TON[INDEX]` with no comma.
- **SGP** — "none" is shown as 8 dashes (`--------`) though FIPS codes are 6 digits.
- **CNT** — titled Get/Set, but only the get form is given.
- **Command list** — WIN is listed as "*Get Windows Voltage".
