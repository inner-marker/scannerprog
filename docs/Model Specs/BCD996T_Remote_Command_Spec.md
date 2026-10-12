# BCD996T Operation Specification — Remote Commands

> Markdown conversion of section 7.13 (Remote Command) and the CTCSS/DCS code list of the Uniden *BCD996T Operation Specification*, PC protocol V3.02 (pp. 169–201).
> This is an older protocol generation than the BCD396XT/BCD325P2/BCD996P2 documents: Motorola systems are split into Type 1 and Type 2 (with P25 under Type 2), sites are typed when appended, there is no DMR/MotoTRBO, no number tags, no alert-light colors, and the source has no font-data section.
> Wire formats are reproduced as written in the source. Apparent typos in the original are kept in place and listed under [Errata notes](#errata-notes-conversion) at the end.

---

## Contents

- [7.13 Remote Command](#713-remote-command)
  - [Remote communication format](#remote-communication-format)
  - [Format of this document](#format-of-this-document)
  - [General notes](#general-notes)
  - [Remote command list](#remote-command-list)
- [Command reference](#command-reference)
  - [Remote control](#remote-control) — GID, KEY, QSH, QSC, CSC, PWR, STS, GLG, JPM, MNU
  - [System information](#system-information) — MDL, VER
  - [Program control mode](#program-control-mode) — PRG, EPG
  - [System settings](#system-settings) — BKL, CLR, KBP, OMS, PRI, AGV, AGS
  - [Scan settings](#scan-settings) — SCT … MEM, MBP, MCP
  - [Location settings](#location-settings) — LIH, LIT, CLA, DLA, LIN
  - [Search / Close Call settings](#search--close-call-settings) — SCO, BBS, SHK, GLF, ULF, LOF, CLC
  - [Service search settings](#service-search-settings) — SSP
  - [Custom search settings](#custom-search-settings) — CSG, CSP
  - [Weather settings](#weather-settings) — WXS, SGP
  - [Other settings](#other-settings) — TON, CNT, DUD, SCN, VOL, SQL, P25, PWT
  - [GPS settings](#gps-settings) — GDO, GGA, RMC
  - [Test](#test) — WIN
- [CTCSS/DCS code list](#ctcssdcs-code-list)
- [Errata notes (conversion)](#errata-notes-conversion)

---

## 7.13 Remote Command

### Remote communication format

| Item | Value |
|---|---|
| BPS rate | 4800 / 9600 / 19200 / 38400 / 57600 / 115200 bps |
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
8. `[INDEX]` / `[xxx_INDEX]` is an index into the internal memory chain. The Dynamic Memory Allocation structure always uses it as a handle to access data and to traverse forward/reverse or up/down. Range: 1 to the maximum memory block (**about 15000** on this model).
9. `[FRQ]`, `[BASEx]` and `[LIMIT_x]` are in **frequency format**: an 8‑digit number with no decimal point, ordered from the 1 GHz digit down to the 100 Hz digit.
   - e.g. `08510125` = 851.0125 MHz
10. `[TGID]` is in TGID format; the format depends on the trunked system type (see the separate appendix).
11. `[NAME]` is a custom name. If the user sets only space characters, the name reverts to the default name.
12. `[LATITUDE]` — North or South latitude, DMS format `DDMMSSssL`:
    - `DD` degree (00–90, 2 digits fixed)
    - `MM` minute (00–59, 2 digits fixed)
    - `SS` second (00–59, 2 digits fixed), `ss` hundredths (00–99, 2 digits fixed)
    - `L` bearing (`N` North / `S` South)
    - e.g. North 40°42′51.12″ → `40425112N`
13. `[LONGITUDE]` — West or East longitude, DMS format `DDDMMSSssL`:
    - `DDD` degree (000–180, 3 digits fixed)
    - `MM`, `SS`, `ss` as above
    - `L` bearing (`W` West / `E` East)
    - e.g. West 74°00′23.05″ → `074002305W`

### Remote command list

"PM" = marked Program Mode only in the source list.

| No. | Category | Command | Function | PM |
|---:|---|---|---|:---:|
| 1 | Remote Control | GID | Get Current TalkGroup ID Status | |
| 2 | Remote Control | KEY | Push KEY | |
| 3 | Remote Control | QSH | Go to quick search hold mode | |
| 4 | Remote Control | QSC | Set current frequency and get reception status | |
| 5 | Remote Control | CSC | Go to Custom search and get reception status | |
| 6 | Remote Control | PWR | Get RSSI Level | |
| 7 | Remote Control | STS | Get Current Status | |
| 8 | Remote Control | GLG | Get Reception Status | |
| 9 | Remote Control | JPM | Jump Mode | |
| 10 | Remote Control | MNU | Menu Mode | |
| 11 | System Information | MDL | Get Model Info | |
| 12 | System Information | VER | Get Firmware Version | |
| 13 | Program Control Mode | PRG | Enter Program Mode | |
| 14 | Program Control Mode | EPG | Exit Program Mode | |
| 15 | System Settings | BKL | Get/Set Backlight | ✓ |
| 16 | System Settings | CLR | Clear All Memory | ✓ |
| 17 | System Settings | KBP | Get/Set Key Beep | ✓ |
| 18 | System Settings | OMS | Get/Set Opening Message | ✓ |
| 19 | System Settings | PRI | Get/Set Priority Mode | ✓ |
| 20 | System Settings | AGV | Get/Set Auto Gain Control | ✓ |
| 21 | System Settings | AGS | Get/Set Audio AGC Setting | ✓ |
| 22 | Scan Settings | SCT | Get System Count | ✓ |
| 23 | Scan Settings | SIH | Get System Index Head | ✓ |
| 24 | Scan Settings | SIT | Get System Index Tail | ✓ |
| 25 | Scan Settings | QSL | Get/Set System/Site Quick Lockout | ✓ |
| 26 | Scan Settings | QGL | Get/Set Group Quick Lockout | ✓ |
| 27 | Scan Settings | CSY | Create System | ✓ |
| 28 | Scan Settings | DSY | Delete System | ✓ |
| 29 | Scan Settings | SIN | Get/Set System Info | ✓ |
| 30 | Scan Settings | TRN | Get/Set Trunk Info | ✓ |
| 31 | Scan Settings | AST | Append Site | ✓ |
| 32 | Scan Settings | SIF | Get/Set Site Info | ✓ |
| 33 | Scan Settings | MBP | Get/Set Motorola VHF/UHF Band Plan | ✓ |
| 34 | Scan Settings | ABP | Get/Set APCO-P25 Band Plan | ✓ |
| 35 | Scan Settings | TFQ | Get/Set Trunk Frequency Info | ✓ |
| 36 | Scan Settings | AGC | Append Channel Group | ✓ |
| 37 | Scan Settings | AGT | Append TalkGroup ID Group | ✓ |
| 38 | Scan Settings | DGR | Delete Group / Site | ✓ |
| 39 | Scan Settings | GIN | Get/Set Group Info | ✓ |
| 40 | Scan Settings | ACC | Append Channel / Trunk Frequency | ✓ |
| 41 | Scan Settings | ACT | Append TalkGroup ID | ✓ |
| 42 | Scan Settings | DCH | Delete Channel | ✓ |
| 43 | Scan Settings | CIN | Get/Set Channel Info | ✓ |
| 44 | Scan Settings | TIN | Get/Set TalkGroup ID Info | ✓ |
| 45 | Scan Settings | GLI | Get Lockout TalkGroup ID (for Rvw L/O ID) | ✓ |
| 46 | Scan Settings | SLI | Get Search L/O TalkGroup ID | ✓ |
| 47 | Scan Settings | ULI | Unlock TalkGroup ID (for Rvw L/O ID) | ✓ |
| 48 | Scan Settings | LOI | Lockout ID (TalkGroup ID) | ✓ |
| 49 | Scan Settings | REV | Get Rev Index | ✓ |
| 50 | Scan Settings | FWD | Get Fwd Index | ✓ |
| 51 | Scan Settings | RMB | Get Remains of Memory Block | ✓ |
| 52 | Scan Settings | MEM | Get Memory Used | ✓ |
| 53 | Location Setting | LIH | Get Location Alert System Index Head | ✓ |
| 54 | Location Setting | LIT | Get Location Alert System Index Tail | ✓ |
| 55 | Location Setting | CLA | Create Location Alert System | ✓ |
| 56 | Location Setting | DLA | Delete Location Alert System | ✓ |
| 57 | Location Setting | LIN | Get/Set Location Alert System Info | ✓ |
| 58 | Search / Close Call Settings | SCO | Get/Set Search/Close Call Settings | ✓ |
| 59 | Search / Close Call Settings | BBS | Get/Set Broadcast Screen Band Settings | ✓ |
| 60 | Search / Close Call Settings | SHK | Get/Set Search Key Settings | ✓ |
| 61 | Search / Close Call Settings | GLF | Get Global Lockout Freq | ✓ |
| 62 | Search / Close Call Settings | ULF | Unlock Global L/O | ✓ |
| 63 | Search / Close Call Settings | LOF | Lock Out Frequency | ✓ |
| 64 | Search / Close Call Settings | CLC | Get/Set Close Call Settings | ✓ |
| 65 | Service Search Settings | SSP | Get/Set Service Search Settings | ✓ |
| 66 | Custom Search Settings | CSG | Get/Set Custom Search Group | ✓ |
| 67 | Custom Search Settings | CSP | Get/Set Custom Search Settings | ✓ |
| 68 | Weather Settings | WXS | Get/Set Weather Settings | ✓ |
| 69 | Weather Settings | SGP | Get/Set SAME Group Settings | ✓ |
| 70 | Tone-Out Setting | TON | Get/Set Tone-Out Settings | ✓ |
| 71 | LCD Contrast Settings | CNT | Get/Set LCD Contrast Settings | ✓ |
| 72 | LCD Upside-down Settings | DUD | Get/Set LCD Upside-down Settings | ✓ |
| 73 | Scanner Option Settings | SCN | Get/Set Scanner Option Settings | ✓ |
| 74 | Volume Level Settings | VOL | Get/Set Volume Level Settings | |
| 75 | Squelch Level Settings | SQL | Get/Set Squelch Level Settings | |
| 76 | APCO Data Settings | P25 | Get/Set APCO Data Settings | |
| 77 | GPS Settings | GDO | Get/Set GPS Format | ✓ |
| 78 | GPS Settings | GGA | Get GGA Data from GPS | |
| 79 | GPS Settings | RMC | Get RMC Data from GPS | |
| 80 | P25 Waiting Time | PWT | Get/Set P25 Waiting Time | |
| 81 | Motorola Custom Band Plan | MCP | Get/Set Motorola Custom Band Plan | |
| 82 | TEST | WIN | Get Window Voltage | |

---

## Command reference

Common parameter definitions reused by many commands:

| Parameter | Values |
|---|---|
| `[RSV]` | Reserve parameter — always only `,` (empty) unless stated otherwise |
| `[MOD]` | Modulation: `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` (subset per command) |
| `[ATT]` | Attenuation: `0` OFF / `1` ON |
| `[DLY]` | Delay time: `0` OFF / `1`–`5` (TON also allows `30` / `INF`) |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON (SIN: `0` OFF / `1` Marked channel / `2` All channel) |
| `[APCO]` | APCO threshold mode: `AUTO` Auto / `MAN` Manual / `DFLT` Default |
| `[THRESHOLD]` | APCO threshold: `0`–`63` |

**Search step `[STP]`** (QSH, QSC, SCO, CSP):

| Code | Step | Code | Step | Code | Step |
|---|---|---|---|---|---|
| `AUTO` | Auto | `833` | 8.33 k | `2000` | 20 k |
| `500` | 5 k | `1000` | 10 k | `2500` | 25 k |
| `625` | 6.25 k | `1250` | 12.5 k | `5000` | 50 k |
| `750` | 7.5 k | `1500` | 15 k | `10000` | 100 k |

**Broadcast Screen bitfield `[BSC]`** (QSH, QSC, SCO) — 16 digits, each `0` (OFF) or `1` (ON), leftmost first:

| Digit position (left → right) | Meaning |
|---|---|
| 1 | Pager |
| 2 | FM |
| 3 | UHF TV |
| 4 | VHF TV |
| 5 | NOAA WX |
| 6 | Reserve |
| 7 | Band 1 |
| 8 | Band 2 |
| … | … |
| 16 | Band 10 |

---

### Remote control

#### GID — Get Current TGID Status

```text
# Controller → Radio
GID\r
# Radio → Controller
GID,[SITE_TYPE],[TGID],[ID_SRCH_MODE],[NAME1],[NAME2],[NAME3]\r
```

| Parameter | Description |
|---|---|
| `[SITE_TYPE]` | Site type (values not listed in this source; see AST/CSY) |
| `[TGID]` | TGID |
| `[ID_SRCH_MODE]` | `0` ID SCAN mode / `1` ID SEARCH mode |
| `[NAME1]` | System / Site name (alpha tag) |
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
| `P` | PRI |
| `W` | WX |
| `G` | GPS |
| `M` | MENU |
| `F` | VFO push / FUNC |
| `H` | HOLD |
| `S` | SCAN / SEARCH |
| `L` | L/O |
| `0`–`9` | Digits 0–9 |
| `.` | . / no |
| `E` | E / yes |
| `Q` | SQL push |
| `V` | VOL push |
| `>` | VFO right (set `KEY_MODE` to `P`) |
| `<` | VFO left (set `KEY_MODE` to `P`) |

| `[KEY_MODE]` | Meaning |
|---|---|
| `P` | Press |
| `L` | Long press |
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

The key-hold state times out 10 seconds after a key-hold command (e.g. `KEY,F,H`) if there is no further communication.

#### QSH — Go to Quick Search Hold mode

```text
# Controller → Radio
QSH,[FRQ],[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[RECORD]\r
# Radio → Controller
QSH,OK\r   or   QSH,NG\r
```

| Parameter | Description |
|---|---|
| `[FRQ]` | Frequency |
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[CODE_SRCH]` | CTCSS/DCS search: `0` OFF / `1` ON |
| `[BSC]` | Broadcast Screen bitfield (see above) |
| `[REP]` | Repeater Find: `0` OFF / `1` ON |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

Invalid while the scanner is in Menu Mode, during Direct Entry, or during Quick Save.

**Function:** Sets an arbitrary frequency and changes to Quick Search Hold (VFO) mode. Parameters such as `STP` change the contents of the Srch/CloCall option.

#### QSC — Set current frequency and get reception status

```text
# Controller → Radio
QSC,[FRQ],[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[RECORD]\r
# Radio → Controller
QSC,[RSSI],[FRQ],[SQL]\r   or   QSC,NG\r
```

Parameters as for QSH, plus:

| Parameter | Description |
|---|---|
| `[RSSI]` | RSSI A/D value (0–1023) |
| `[SQL]` | Squelch status: `0` CLOSE / `1` OPEN |

Invalid while in Menu Mode, during Direct Entry, or during Quick Save.

**Function:** Sets an arbitrary frequency and changes to Quick Search Hold (VFO) mode. Parameters such as `STP` change the contents of the Srch/CloCall option.

#### CSC — Go to Custom Search and get reception status

```text
# Controller → Radio
CSC,ON\r     # ① start streaming
CSC,OFF\r    # ② stop streaming
# Radio → Controller
# ① repeated, one line per frequency:
CSC,[RSSI],[FRQ],[SQL]\r
CSC,[RSSI],[FRQ],[SQL]\r
...
# ②
CSC,OK\r
```

| Parameter | Description |
|---|---|
| `[RSSI]` | RSSI A/D value (0–1023) |
| `[FRQ]` | Current frequency |
| `[SQL]` | `0` CLOSE / `1` OPEN |

Outputs the custom search status of each frequency sequentially. Use `CSC,OFF` to stop the output. Invalid while in Menu Mode, during Direct Entry, or during Quick Save.

#### PWR — Get RSSI Level

```text
# Controller → Radio
PWR\r
# Radio → Controller
PWR,[RSSI],[FRQ]\r
```

| Parameter | Description |
|---|---|
| `[RSSI]` | RSSI A/D value (0–1023) |
| `[FRQ]` | Current frequency (1 GHz digit → 100 Hz digit) |

Returns the current RSSI level and its frequency.

#### STS — Get Current Status

```text
# Controller → Radio
STS\r
# Radio → Controller (single line on the wire)
STS,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE],[L3_CHAR],[L3_MODE],[L4_CHAR],[L4_MODE],...,[L8_CHAR],[L8_MODE],[SQL],[MUT],[RSV],[WAT],[LED_1],[LED_2],[SIG_LVL],[BK_COLOR],[BK_DIMMER]\r
```

| Parameter | Description |
|---|---|
| `[DSP_FORM]` | Display form, 4–8 digits; each `0` = small font, `1` = large font |
| `[Ln_CHAR]` | Line *n* characters, 16 chars (fixed length) |
| `[Ln_MODE]` | Line *n* display mode, 16 chars |
| `[SQL]` | Squelch: `0` CLOSE / `1` OPEN |
| `[MUT]` | Mute: `0` OFF / `1` ON |
| `[RSV]` | Reserve — always only `0` |
| `[WAT]` | Weather alert: `0` no alert / `1` alert / `$$$` alert SAME code |
| `[LED_1]` | CC LED: `0` OFF / `1` ON |
| `[LED_2]` | Alert LED: `0` OFF / `1` ON |
| `[SIG_LVL]` | Signal level (0–5) |
| `[BK_COLOR]` | Backlight color: `GREEN` / `RED` |
| `[BK_DIMMER]` | Backlight dimmer: `0` OFF / `1` Low / `2` Middle / `3` High |

**Display mode characters (`[Ln_MODE]`):** space = normal, `*` = reverse, `_` = underline. If all 16 characters are normal, only `,` is sent. The number of `[Lx_CHAR]`/`[Lx_MODE]` pairs depends on the display form.

**Ex.1** — Menu screen (squelch OPEN, mute OFF, no weather alert):

```text
→ STS\r
← 1111,                  # DSP_FORM: 4 lines, all large font
  -- M E N U --   ,      # L1_CHAR
  ________________,      # L1_MODE (underlined)
  Program System  ,      # L2_CHAR
  ****************,      # L2_MODE (reversed)
  Program Location,      # L3_CHAR
  ,                      # L3_MODE (all normal)
  Srch/CloCall Opt,      # L4_CHAR
  ,                      # L4_MODE (all normal)
  1,0,0,0,0,0,0,GREEN,0\r # SQL,MUT,RSV,WAT,LED_1,LED_2,SIG_LVL,BK_COLOR,BK_DIMMER
```

**Ex.2** — Hold screen (squelch CLOSE, mute ON, weather alert):

```text
→ STS\r
← 011000,                # DSP_FORM: 6 lines
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
  0,1,0,0,0,0,1,GREEN,1\r # status fields
```

Returns current scanner status.

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
| `[MOD]` | `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` OFF / `1` ON |
| `[CTCSS/DCS]` | CTCSS/DCS status (0–231) — see [code list](#ctcssdcs-code-list) |
| `[NAME1]` | System, site, or search name |
| `[NAME2]` | Group name |
| `[NAME3]` | Channel name |
| `[SQL]` | `0` CLOSE / `1` OPEN |
| `[MUT]` | `0` OFF / `1` ON |

The scanner returns `GLG,,,,,,,,,\r` until it detects a frequency or TGID.

#### JPM — Jump Mode

```text
# Controller → Radio
JPM,[JUMP_MODE],[INDEX]\r
# Radio → Controller
JPM,OK\r
```

| `[JUMP_MODE]` | Mode | `[INDEX]` values |
|---|---|---|
| `SCN_MODE` | Scan mode | Channel index |
| `SVC_MODE` | Service Search mode | `PublicSafety`, `News`, `HAM`, `Marine`, `Railroad`, `Air`, `CB`, `FRS/GMRS`, `Racing`, `TV`, `FM`, `Special`, `Military` |
| `CTM_MODE` | Custom Search mode | `RESERVE` |
| `CC_MODE` | Close Call Only mode | `RESERVE` |
| `WX_MODE` | WX Scan mode | `NORMAL`, `A_ONLY`, `SAME_1` … `SAME_5`, `ALL_FIPS` |
| `FTO_MODE` | Tone-Out mode | `RESERVE` |

The scanner returns `NG` when the mode switch cannot be performed.

#### MNU — Menu Mode

```text
# Controller → Radio
MNU,[MENU_INDEX]\r
# Radio → Controller
MNU,OK\r
```

| `[MENU_INDEX]` | Menu |
|---|---|
| `SVC_MENU` | Service Search Select menu |
| `WX_MENU` | WX Select menu |
| `CCBAND_MENU` | Close Call Band Filter menu |
| `SCR_OPT_MENU` | Broadcast Screen Band menu |
| `GL_LIST_MENU` | Search Global Lockout List Review menu |
| `SETTING_MENU` | Setting menu |

The scanner returns `NG` when the mode switch cannot be performed.

---

### System information

#### MDL — Get Model Info

```text
# Controller → Radio
MDL\r
# Radio → Controller
MDL,BCD996T\r
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

Invalid while in Menu Mode, during Direct Entry, or during Quick Save. In Program Mode the scanner displays "Remote Mode" on line 1 and "Keypad Lock" on line 2.

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

#### BKL — Get/Set Backlight *(PM)*

```text
# Controller → Radio
BKL\r                       # ① get
BKL,[DIMMER],[COLOR]\r      # ② set
# Radio → Controller
BKL,[DIMMER],[COLOR]\r      # ①
BKL,OK\r                    # ②
```

| Parameter | Values |
|---|---|
| `[DIMMER]` | `OFF` / `LOW` / `MIDL` / `HIGH` / `+POL` / `-POL` |
| `[COLOR]` | `GREEN` / `RED` |

#### CLR — Clear All Memory *(PM)*

```text
# Controller → Radio
CLR\r
# Radio → Controller
CLR,OK\r
```

Resets all memory to initial settings. Takes dozens of seconds. Only the PC Control (baud rate) setting is not reset.

#### KBP — Get/Set Key Beep *(PM)*

```text
# Controller → Radio
KBP\r              # ① get
KBP,[LEVEL]\r      # ② set
# Radio → Controller
KBP,[LEVEL]\r      # ①
KBP,OK\r           # ②
```

| Parameter | Values |
|---|---|
| `[LEVEL]` | Beep level: `0` Auto / `1`–`15` / `99` OFF |

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

`#` = priority setting: `0` OFF / `1` ON / `2` PLUS ON.

#### AGV — Get/Set Auto Gain Control *(PM)*

```text
# Controller → Radio
AGV\r                                # ① get
AGV,[AGC_ANALOG],[AGC_DIGITAL]\r     # ② set
# Radio → Controller
AGV,[AGC_ANALOG],[AGC_DIGITAL]\r     # ①
AGV,OK\r                             # ②
```

| Parameter | Values |
|---|---|
| `[AGC_ANALOG]` | AGC for analog audio: `0` OFF / `1` ON |
| `[AGC_DIGITAL]` | AGC for digital audio: `0` OFF / `1` ON |

#### AGS — Get/Set AGC Settings *(PM)*

```text
# Controller → Radio
AGS\r                                                                                 # ① get
AGS,[A_Res_Time],[A_Ref_Gain],[A_Range],[D_Res_Time],[D_Ref_Gain],[D_Range]\r         # ② set (commas missing in source)
# Radio → Controller
AGS,[A_Res_Time],[A_Ref_Gain],[A_Range],[D_Res_Time],[D_Ref_Gain],[D_Range]\r         # ①
AGS,OK\r                                                                              # ②
```

| Parameter | Values |
|---|---|
| `[A_Res_Time]` | Analog response time: −4 … +6 |
| `[A_Ref_Gain]` | Analog reference gain: −5 … +5 |
| `[A_Range]` | Analog dynamic range: 0–15 |
| `[D_Res_Time]` | Digital response time: −8 … +8 |
| `[D_Ref_Gain]` | Digital reference gain: −5 … +5 |
| `[D_Range]` | Digital dynamic range: 0–6 (reserved) |

---

### Scan settings

#### SCT — Get System Count *(PM)*

```text
# Controller → Radio
SCT\r
# Radio → Controller
SCT,###\r    # ### = 0–500
```

Returns the number of stored systems.

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

#### QSL — Get/Set System/Site Quick Lockout *(PM)*

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
| `0` | Not assigned | `-` |
| `1` | On | the number |
| `2` | Off | `*` |

Quick key order matches the LCD icons:

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

Cannot turn on/off a quick key that has no system/site.

#### QGL — Get/Set Group Quick Lockout *(PM)*

```text
# Controller → Radio
QGL,[SYS_INDEX]\r                # ① get
QGL,[SYS_INDEX],##########\r     # ② set
# Radio → Controller
QGL,##########\r                 # ①
QGL,OK\r                         # ②
```

`##########` — group quick key status of `[SYS_INDEX]`, each digit `0` not assigned (`-`) / `1` On (number) / `2` Off (`*`). Order matches the LCD icons (1–9, 0). Cannot turn on/off a quick key that has no group.

#### CSY — Create System *(PM)*

```text
# Controller → Radio
CSY,[SYS_TYPE]\r
# Radio → Controller
CSY,[SYS_INDEX]\r
```

| `[SYS_TYPE]` | System type |
|---|---|
| `CNV` | Conventional |
| `MOT1` | Motorola Type 1 |
| `MOT2` | Motorola Type 2 / P25 |
| `EDC` | EDACS Narrow / Wide |
| `EDS` | EDACS SCAT |
| `LTR` | LTR |

Creates a system and returns its index (the handle for get/set of system info). Returns `-1` if creation failed due to lack of resources. There is no protect-bit parameter on this model.

#### DSY — Delete System *(PM)*

```text
# Controller → Radio
DSY,[SYS_INDEX]\r
# Radio → Controller
DSY,OK\r
```

#### SIN — Get/Set System Info *(PM)*

```text
# Controller → Radio
SIN,[INDEX]\r     # ① get
# ② set (single line on the wire)
SIN,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[SKP],[RSV],[RSV],[APCO],[THRESHOLD],[START_KEY],[RECORD],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
SIN,[SYS_TYPE],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[SKP],[RSV],[RSV],[APCO],[THRESHOLD],[REV_INDEX],[FWD_INDEX],[CHN_GRP_HEAD],[CHN_GRP_TAIL],[SEQ_NO],[START_KEY],[RECORD],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV]\r
SIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | System index |
| `[SYS_TYPE]` | See CSY |
| `[NAME]` | Name (max. 16 chars) |
| `[QUICK_KEY]` | Quick key (0–99 / `.` = none) |
| `[HLD]` | System hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[APCO]` | APCO threshold mode: `AUTO` / `MAN` / `DFLT` |
| `[THRESHOLD]` | APCO threshold (0–63) |
| `[REV_INDEX]` | Reverse system index in scan order |
| `[FWD_INDEX]` | Forward system index in scan order |
| `[CHN_GRP_HEAD]` | Channel group index head (conventional) / site index head (trunked) |
| `[CHN_GRP_TAIL]` | Channel group index tail (conventional) / site index tail (trunked) |
| `[SEQ_NO]` | System sequence number (1–500) |
| `[START_KEY]` | Startup configuration key (0–9 / `.` = none) |
| `[RECORD]` | Tape-out: `0` OFF / `1` Marked channel / `2` All channel |
| `[LATITUDE]` / `[LONGITUDE]` | See general notes 12–13 |
| `[RANGE]` | Range (1 = 0.01 mile) |
| `[GPS_ENABLE]` | GPS location detection: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

- Parameters not applicable to the system type are returned as just `,`, and ignored in set commands.
- In set commands, `,`-only parameters are unchanged; any format error aborts the command.

#### TRN — Get/Set Trunk Info *(PM)*

```text
# Controller → Radio
TRN,[INDEX]\r     # ① get
# ② set (single line on the wire)
TRN,[INDEX],[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[I-CALL],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[D_END_CODE]\r

# Radio → Controller
# ① (single line on the wire)
TRN,[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[I-CALL],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[D_END_CODE],[TGID_GRP_HEAD],[TGID_GRP_TAIL],[ID_LOUT_GRP_HEAD],[ID_LOUT_GRP_TAIL]\r
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
| `[EMG]` | Emergency alert: `0` Ignore / `1`–`9` Alert |
| `[EMGL]` | Emergency alert level: `0` OFF / `1`–`15` |
| `[FMAP]` | Fleet map: `0`–`15` preset / `16` custom |
| `[CTM_FMAP]` | Custom fleet map, 8 digits (blocks 0–7), each `0`–`E` = size code 0–14 |
| `[D_END_CODE]` | Motorola digital end code: `0` Ignore / `1` Yes |
| `[TGID_GRP_HEAD]` / `[TGID_GRP_TAIL]` | TGID index head / tail of the system |
| `[ID_LOUT_GRP_HEAD]` / `[ID_LOUT_GRP_TAIL]` | L/O TGID group index head / tail |
| `[RSV]` | Always only `,` (9 between `[CTM_FMAP]` and `[D_END_CODE]`) |

- Parameters not applicable to the system type are returned as `,` and ignored in set commands.
- `,`-only parameters are unchanged in set; any format error aborts.

#### AST — Append Site *(PM)*

```text
# Controller → Radio
AST,[SYS_INDEX],[SITE_TYPE]\r
# Radio → Controller
AST,[SITE_INDEX]\r
```

| System | `[SITE_TYPE]` | Meaning |
|---|---|---|
| Motorola Type 2 | `M82S` | Motorola 800 MHz Standard |
| | `M82P` | Motorola 800 MHz Splinter |
| | `M92` | Motorola 900 MHz band |
| | `MV2` | Motorola VHF band |
| | `MU2` | Motorola UHF band |
| | `MP25` | Motorola P25 |
| | `M82C` | Motorola 800 MHz Custom |
| Motorola Type 1 | `M81S` | Motorola 800 MHz Standard |
| | `M81P` | Motorola 800 MHz Splinter |
| | `M81C` | Motorola 800 MHz Custom |
| EDACS | `EDN` | EDACS Narrow |
| | `EDW` | EDACS Wide |

Appends a site to the system. Returns `-1` if creation failed due to lack of resources.

#### SIF — Get/Set Site Info *(PM)*

```text
# Controller → Radio
SIF,[INDEX]\r     # ① get
# ② set (single line on the wire)
SIF,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[APCO],[THRESHOLD],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
SIF,[SITE_TYPE],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[APCO],[THRESHOLD],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV]\r
SIF,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[SITE_TYPE]` | Site type (see AST) |
| `[NAME]` | Name (max. 16 chars) |
| `[QUICK_KEY]` | 0–99 / `.` = none |
| `[HLD]` | Site hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[MOD]` | `AUTO` / `FM` / `NFM` |
| `[ATT]` | `0` OFF / `1` ON |
| `[C-CH]` | Control channel only: `0` OFF / `1` ON |
| `[APCO]` | APCO threshold mode: `AUTO` / `MAN` / `DFLT` |
| `[THRESHOLD]` | APCO threshold (0–63) |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward site index in scan order |
| `[SYS_INDEX]` | System index |
| `[CHN_HEAD]` / `[CHN_TAIL]` | Channel index head / tail of the group list |
| `[SEQ_NO]` | Site sequence number (1–256) |
| `[START_KEY]` | Startup configuration (0–9 / `.` = none) |
| `[LATITUDE]` / `[LONGITUDE]` | See general notes 12–13 |
| `[RANGE]` | Range (1 = 0.01 mile) |
| `[GPS_ENABLE]` | `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

- Non-applicable parameters are returned as `,` and ignored in set commands.
- `,`-only parameters unchanged in set; any format error aborts.

#### MBP — Get/Set Motorola VHF/UHF Band Plan *(PM)*

```text
# Controller → Radio
MBP,[INDEX]\r     # ① get
# ② set (single line on the wire)
MBP,[INDEX],[BASE1],[STEP1],[OFFSET1],[BASE2],[STEP2],[OFFSET2],[BASE3],[STEP3],[OFFSET3]\r
# Radio → Controller
MBP,[BASE1],[STEP1],[OFFSET1],[BASE2],[STEP2],[OFFSET2],[BASE3],[STEP3],[OFFSET3]\r   # ①
MBP,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[BASEn]` | Base frequency *n* (frequency format) |
| `[STEPn]` | Step *n* (codes not listed in this source) |
| `[OFFSETn]` | Offset *n* |

Gets/sets the band plan for a Motorola VHF/UHF site. `,`-only parameters unchanged in set; any format error aborts.

#### MCP — Get/Set Motorola Custom Band Plan *(PM)*

```text
# Controller → Radio
MCP,[INDEX]\r     # ① get
# ② set (single line on the wire)
MCP,[INDEX],[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r

# Radio → Controller
# ① (single line on the wire)
MCP,[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r
MCP,OK\r          # ② (printed "MCP, OK" in source)
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[LOWERn]` / `[UPPERn]` | Lower / upper frequency *n* |
| `[STEPn]` | Step *n* (see table) |
| `[OFFSETn]` | Offset *n* (−1023 … 1023) |

**Step codes:**

| Code | Step | Code | Step | Code | Step |
|---|---|---|---|---|---|
| `500` | 5.0 k | `625` | 6.25 k | `1000` | 10.0 k |
| `1250` | 12.5 k | `1500` | 15.0 k | `1875` | 18.75 k |
| `2000` | 20.0 k | `2500` | 25.0 k | `3000` | 30.0 k |
| `3125` | 31.25 k | `3500` | 35.0 k | `3750` | 37.5 k |
| `4000` | 40.0 k | `4375` | 43.75 k | `4500` | 45.0 k |
| `5000` | 50.0 k | `5500` | 55.0 k | `5625` | 56.25 k |
| `6000` | 60.0 k | `6250` | 62.5 k | `6500` | 65.0 k |
| `6875` | 68.75 k | `7000` | 70.0 k | `7500` | 75.0 k |
| `8000` | 80.0 k | `8125` | 81.25 k | `8500` | 85.0 k |
| `8750` | 87.5 k | `9000` | 90.0 k | `9375` | 93.75 k |
| `9500` | 95.0 k | `10000` | 100.0 k | | |

Gets/sets the band plan for a Motorola 800 MHz Custom site. If only `,` parameters are sent, the band plan is unchanged; any format error aborts. When the system protect bit is ON, all parameters are returned as reserve parameters. The site must be an 800 MHz Custom site (`M82C` / `M81C`) first; the source says to set this "by using SIF".

#### ABP — Get/Set APCO-P25 Band Plan *(PM)*

```text
# Controller → Radio
ABP,[INDEX]\r     # ① get
# ② set (single line on the wire)
ABP,[INDEX],[BASE_FREQ_0],[SPACING_FREQ_0],[BASE_FREQ_1],[SPACING_FREQ_1],...,[BASE_FREQ_E],[SPACING_FREQ_E],[BASE_FREQ_F],[SPACING_FREQ_F]\r

# Radio → Controller
# ① — note: this model's response includes [INDEX]
ABP,[INDEX],[BASE_FREQ_0],[SPACING_FREQ_0],...,[BASE_FREQ_F],[SPACING_FREQ_F]\r
ABP,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[BASE_FREQ_n]` | Base frequency, encoded as hex: `(base_MHz × 10^6) / 5` |
| `[SPACING_FREQ_n]` | Spacing, encoded as hex: `(spacing_kHz × 10^3) / 125` |

*n* = band plan number (`0`–`F`).

Example — base 851.00625 MHz, spacing 6.25 kHz:

```text
BASE_FREQ_n    = (851.00625 × 10^6) / 5 = A2510A2 (hex)   # 170201250 decimal
SPACING_FREQ_n = (6.25 × 10^3) / 125    = 32 (hex)        # 50 decimal
```

A band plan with no data returns `0`.

#### TFQ — Get/Set Trunk Frequency Info *(PM)*

```text
# Controller → Radio
TFQ,[CHN_INDEX]\r                             # ① get
TFQ,[CHN_INDEX],[FRQ],[LCN],[LOUT],[RECORD]\r # ② set
# Radio → Controller
TFQ,[FRQ],[LCN],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RECORD]\r   # ①
TFQ,OK\r                                      # ②
```

| Parameter | Description |
|---|---|
| `[CHN_INDEX]` | Trunk frequency index |
| `[FRQ]` | Trunk frequency |
| `[LCN]` | LCN |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward frequency index of the site |
| `[SYS_INDEX]` | System index of the frequency |
| `[GRP_INDEX]` | Index of the site |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

- `,`-only parameters unchanged in set; any format error aborts.
- For Motorola or EDACS SCAT systems, `[LCN]` is ignored.

#### AGC — Append Channel Group *(PM)*

```text
# Controller → Radio
AGC,[SYS_INDEX]\r
# Radio → Controller
AGC,[GRP_INDEX]\r    # -1 if no resources
```

Appends a channel group to the system.

#### AGT — Append TGID Group *(PM)*

```text
# Controller → Radio
AGT,[SYS_INDEX]\r
# Radio → Controller
AGT,[GRP_INDEX]\r    # -1 if no resources
```

Appends a TGID group to the system.

#### DGR — Delete Group / Site *(PM)*

```text
# Controller → Radio
DGR,[INDEX]\r        # group or site index
# Radio → Controller
DGR,OK\r
```

Deletes a channel group, TGID group, or site.

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
| `[NAME]` | Name (max. 16 chars) |
| `[QUICK_KEY]` | `1`–`9`, `0` (= 10), `.` = none |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward group index in the system |
| `[SYS_INDEX]` | System index |
| `[CHN_HEAD]` / `[CHN_TAIL]` | Channel index head / tail of the group list |
| `[SEQ_NO]` | Group sequence number in the system |

`,`-only parameters unchanged in set; any format error aborts. Groups have no GPS location fields on this model.

#### ACC — Append Channel / Trunk Frequency *(PM)*

```text
# Controller → Radio
ACC,[GRP_INDEX]\r    # channel group index, or site index
# Radio → Controller
ACC,[CHN_INDEX]\r    # appended channel / trunk frequency index; -1 if no resources
```

Appends a channel to a channel group, or a trunk frequency to a site.

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
DCH,[INDEX]\r        # channel, TGID, or trunk-frequency index
# Radio → Controller
DCH,OK\r
```

Deletes a channel or TGID; also valid for deleting a trunk frequency.

#### CIN — Get/Set Channel Info *(PM)*

```text
# Controller → Radio
CIN,[INDEX]\r     # ① get
# ② set (single line on the wire)
CIN,[INDEX],[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[RECORD],[AUDIO_TYPE]\r

# Radio → Controller
# ① (single line on the wire)
CIN,[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RECORD],[AUDIO_TYPE]\r
CIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Channel index |
| `[NAME]` | Name (max. 16 chars) |
| `[FRQ]` | Channel frequency |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[CTCSS/DCS]` | 0–231 — see [code list](#ctcssdcs-code-list) |
| `[TLOCK]` | CTCSS/DCS tone lockout: `0` OFF / `1` ON |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[PRI]` | Priority: `0` OFF / `1` ON |
| `[ATT]` | `0` OFF / `1` ON |
| `[ALT]` | Alert tone: `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | Alert tone level: `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward channel index in the channel group |
| `[SYS_INDEX]` | System index of the channel |
| `[GRP_INDEX]` | Group index of the channel |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |
| `[AUDIO_TYPE]` | `0` All / `1` Analog only / `2` Digital only |

`,`-only parameters unchanged in set; any format error aborts.

#### TIN — Get/Set TGID Info *(PM)*

```text
# Controller → Radio
TIN,[INDEX]\r     # ① get
TIN,[INDEX],[NAME],[TGID],[LOUT],[RSV],[ALT],[ALTL],[RECORD],[AUDIO_TYPE]\r   # ② set

# Radio → Controller
# ① (single line on the wire)
TIN,[NAME],[TGID],[LOUT],[RSV],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RECORD],[AUDIO_TYPE]\r
TIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | TGID index |
| `[NAME]` | Name (max. 16 chars) |
| `[TGID]` | TGID |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[RSV]` | Always only `,` (sits where later models put `[PRI]`) |
| `[ALT]` | `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward TGID index in the group |
| `[SYS_INDEX]` | System index of the TGID |
| `[GRP_INDEX]` | Group index of the TGID |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |
| `[AUDIO_TYPE]` | `0` All / `1` Analog only / `2` Digital only |

`,`-only parameters unchanged in set; any format error aborts.

#### GLI — Get Lockout TGID (for Rvw L/O ID) *(PM)*

```text
# Controller → Radio
GLI,[SYS_INDEX]\r
# Radio → Controller
GLI,[TGID]\r     # one locked-out TGID
GLI,-1\r         # no more lockout TGIDs
```

Gets the L/O TGID list of a system. Call repeatedly until `-1` is returned.

#### SLI — Get Search L/O TGID *(PM)*

```text
# Controller → Radio
SLI,[SYS_INDEX]\r
# Radio → Controller
SLI,[TGID]\r
SLI,-1\r         # no more lockout TGIDs
```

Gets the Search L/O TGID list — locked-out TGIDs that don't belong to any group in the system. Unlike GLI, does not return L/O TGIDs that belong to a group. Call repeatedly until `-1`.

#### ULI — Unlock TGID (for Rvw L/O ID) *(PM)*

```text
# Controller → Radio
ULI,[SYS_INDEX],[TGID]\r
# Radio → Controller
ULI,OK\r
```

Unlocks an L/O TGID in a system (removes it from the L/O list).

#### LOI — Lockout ID (TGID) *(PM)*

```text
# Controller → Radio
LOI,[SYS_INDEX],[TGID]\r
# Radio → Controller
LOI,OK\r
```

Locks out a TGID for the system (adds it to the L/O list).

#### REV — Get Rev Index *(PM)*

```text
# Controller → Radio
REV,[INDEX]\r
# Radio → Controller
REV,[INDEX]\r    # -1 if none
```

`[INDEX]`: index of a system, site, group, channel, TGID, or location alert system. Returns the reverse (backward) index in the memory chain, or `-1` if none.

#### FWD — Get Fwd Index *(PM)*

```text
# Controller → Radio
FWD,[INDEX]\r
# Radio → Controller
FWD,[INDEX]\r    # -1 if none
```

Returns the forward index in the memory chain, or `-1` if none.

#### RMB — Get Remains of Memory Block *(PM)*

```text
# Controller → Radio
RMB\r
# Radio → Controller
RMB,#####\r      # free blocks, not zero-padded
```

#### MEM — Get Memory Used *(PM)*

```text
# Controller → Radio
MEM\r
# Radio → Controller
MEM,[MEMORY_USED],[SYS],[SITE],[CHN],[LOC]\r
```

| Parameter | Description |
|---|---|
| `[MEMORY_USED]` | Percent of memory used (0–100) |
| `[SYS]` | Number of systems (0–500) |
| `[SITE]` | Number of sites (0–1000) |
| `[CHN]` | Number of channels (0–6000) |
| `[LOC]` | Number of location systems (0–1000) |

---

### Location settings

`[LAS_TYPE]` (location alert type): `POI` · `DROAD` (dangerous road) · `DXING` (dangerous crossing).

#### LIH — Get Location Alert System Index Head *(PM)*

```text
# Controller → Radio
LIH,[LAS_TYPE]\r
# Radio → Controller
LIH,[INDEX]\r
```

#### LIT — Get Location Alert System Index Tail *(PM)*

```text
# Controller → Radio
LIT,[LAS_TYPE]\r
# Radio → Controller
LIT,[INDEX]\r
```

#### CLA — Create Location Alert System *(PM)*

```text
# Controller → Radio
CLA,[LAS_TYPE]\r
# Radio → Controller
CLA,[INDEX]\r    # -1 if no resources
```

Creates a location alert system and returns its index (the handle for get/set).

#### DLA — Delete Location Alert System *(PM)*

```text
# Controller → Radio
DLA,[INDEX]\r
# Radio → Controller
DLA,OK\r
```

#### LIN — Get/Set Location Alert System Info *(PM)*

```text
# Controller → Radio
LIN,[INDEX]\r     # ① get
# ② set (single line on the wire)
LIN,[INDEX],[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR]\r

# Radio → Controller
# ① (single line on the wire)
LIN,[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SEQ_NO],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR]\r
LIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Location alert system index |
| `[LAS_TYPE]` | `POI` / `DROAD` / `DXING` |
| `[NAME]` | Name (max. 16 chars) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[ALT]` | Alert tone: `0` OFF / `1`–`4` |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward index |
| `[SEQ_NO]` | Sequence number |
| `[LATITUDE]` / `[LONGITUDE]` | See general notes 12–13 |
| `[RANGE]` | 1–200 (1 = 0.01 mile) |
| `[SPEED]` | Speed limit 0–200 (mph) |
| `[DIR]` | Heading `0`–`358` in 2° steps, or `360` = all directions |

Non-applicable parameters are ignored in set; `,`-only parameters unchanged; any format error aborts.

---

### Search / Close Call settings

#### SCO — Get/Set Search/Close Call Settings *(PM)*

```text
# Controller → Radio
SCO\r             # ① get
SCO,[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[APCO],[THRESHOLD],[MAX_STORE],[RECORD]\r   # ② set
# Radio → Controller
SCO,[STP],[MOD],[ATT],[DLY],[SKP],[CODE_SRCH],[BSC],[REP],[APCO],[THRESHOLD],[MAX_STORE],[RECORD]\r   # ①
SCO,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` OFF / `1` ON |
| `[CODE_SRCH]` | CTCSS/DCS search: `0` OFF / `1` ON |
| `[BSC]` | Broadcast Screen bitfield |
| `[REP]` | Repeater find: `0` OFF / `1` ON |
| `[APCO]` | APCO threshold mode: `AUTO` / `MAN` / `DFLT` |
| `[THRESHOLD]` | APCO threshold (0–63) |
| `[MAX_STORE]` | Max auto store (1–256) |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

`,`-only parameters unchanged in set; any format error aborts.

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
| `[INDEX]` | 1–9, `0` = 10 |
| `[LIMIT_L]` | Lower limit frequency (00000000–99999999) |
| `[LIMIT_H]` | Upper limit frequency (00000000–99999999) |

#### SHK — Get/Set Search Key Settings *(PM)*

```text
# Controller → Radio
SHK\r                                                                          # ① get
SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[SRCH_KEY_4],[SRCH_KEY_5],[SRCH_KEY_6]\r   # ② set
# Radio → Controller
SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[SRCH_KEY_4],[SRCH_KEY_5],[SRCH_KEY_6]\r   # ①
SHK,OK\r                                                                       # ②
```

`[SRCH_KEY_1]`–`[SRCH_KEY_6]` (search range):

| Value | Range | Value | Range |
|---|---|---|---|
| `.` | Not assigned | `Special` | Special |
| `PublicSafety` | Public Safety | `Military` | Military air |
| `News` | News | `CUSTOM_1` … `CUSTOM_10` | Custom 1–10 range |
| `HAM` | HAM radio | | |
| `Marine` | Marine | | |
| `Railroad` | Railroad | | |
| `Air` | Air | | |
| `CB` | CB radio | | |
| `FRS/GMRS` | FRS/GMRS | | |
| `Racing` | Racing | | |
| `TV` | TV broadcast | | |
| `FM` | FM broadcast | | |

#### GLF — Get Global Lockout Freq *(PM)*

```text
# Controller → Radio
GLF\r
# Radio → Controller
GLF,[FRQ]\r      # one locked-out frequency (250000–13000000)
GLF,-1\r         # no more
```

Call repeatedly until `-1` to get the whole global L/O list.

#### ULF — Unlock Global L/O *(PM)*

```text
# Controller → Radio
ULF,[FRQ]\r      # 250000–13000000
# Radio → Controller
ULF,OK\r
```

Unlocks an L/O frequency (removes it from the list).

#### LOF — Lock Out Frequency *(PM)*

```text
# Controller → Radio
LOF,[FRQ]\r      # 250000–13000000
# Radio → Controller
LOF,OK\r
```

Locks out a frequency (adds it to the L/O list).

#### CLC — Get/Set Close Call Settings *(PM)*

```text
# Controller → Radio
CLC\r             # ① get
CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY]\r   # ② set
# Radio → Controller
CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY]\r   # ①
CLC,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[CC_MODE]` | `0` OFF / `1` CC PRI / `2` CC DND |
| `[CC_OVERRIDE]` | `1` ON / `0` OFF |
| `[ALTB]` | Alert beep: `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[ALTP]` | Close Call pause: `3`, `5`, `10`, `15`, `30`, `45`, `60` s / `INF` |
| `[CC_BAND]` | 7-digit bitfield (see below) |
| `[LOUT]` | Lockout for CC hits with scan: `0` unlocked / `1` lockout |
| `[HLD]` | System hold time for CC hits with scan (0–255) |
| `[QUICK_KEY]` | Quick key for CC hits with scan (0–99 / `.` = none) |
| `[RSV]` | Always only `,` |

`[CC_BAND]` — each digit `0` OFF / `1` ON, left → right:

| Position | Band |
|---|---|
| 1 | VHF LOW1 |
| 2 | VHF LOW2 |
| 3 | AIR BAND |
| 4 | VHF HIGH1 |
| 5 | VHF HIGH2 |
| 6 | UHF |
| 7 | 800 MHz+ |

`,`-only parameters unchanged in set; any format error aborts.

---

### Service search settings

#### SSP — Get/Set Service Search Settings *(PM)*

```text
# Controller → Radio
SSP,[SRCH_INDEX]\r                                                                  # ① get
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RECORD]\r        # ② set
# Radio → Controller
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RECORD]\r        # ①
SSP,OK\r                                                                            # ②
```

| `[SRCH_INDEX]` | Range |
|---|---|
| `1` | Public Safety |
| `2` | News |
| `3` | HAM Radio |
| `4` | Marine |
| `5` | Railroad |
| `6` | Air |
| `7` | CB Radio |
| `8` | FRS/GMRS |
| `9` | Racing |
| `10` | TV Broadcast |
| `11` | FM Broadcast |
| `12` | Special |
| `15` | Military Air |

| Parameter | Description |
|---|---|
| `[DLY]` | `0` OFF / `1`–`5` |
| `[ATT]` | `0` / `1` |
| `[HLD]` | System hold time for search with scan (0–255) |
| `[LOUT]` | Lockout for search with scan: `0` / `1` |
| `[QUICK_KEY]` | 0–99 / `.` |
| `[START_KEY]` | 0–9 / `.` |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

Any format error aborts the set command.

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

Each `#` is `0` (valid) or `1` (invalid) for custom search ranges 1–10, in LCD icon order. **Cannot set all custom search ranges to `0`** (as written in source; see errata).

#### CSP — Get/Set Custom Search Settings *(PM)*

```text
# Controller → Radio
CSP,[SRCH_INDEX]\r   # ① get
# ② set (single line on the wire)
CSP,[SRCH_INDEX],[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[SKP],[HLD],[LOUT],[C-CH],[APCO],[THRESHOLD],[QUICK_KEY],[START_KEY],[RECORD]\r

# Radio → Controller
# ① (single line on the wire; comma between START_KEY and RECORD missing in source)
CSP,[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[SKP],[HLD],[LOUT],[C-CH],[APCO],[THRESHOLD],[QUICK_KEY],[START_KEY],[RECORD]\r
CSP,OK\r             # ②
```

| Parameter | Description |
|---|---|
| `[SRCH_INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[LIMIT_L]` / `[LIMIT_H]` | Lower / upper limit (250000–13000000) |
| `[STP]` | Search step (see table above) |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | `0` OFF / `1`–`5` |
| `[SKP]` | Data skip: `0` / `1` |
| `[HLD]` | System hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[C-CH]` | Control channel only: `0` OFF / `1` ON |
| `[APCO]` | APCO threshold mode: `AUTO` / `MAN` / `DFLT` |
| `[THRESHOLD]` | APCO threshold (0–63) |
| `[QUICK_KEY]` | 0–99 / `.` |
| `[START_KEY]` | 0–9 / `.` |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

`,`-only parameters unchanged in set; any format error aborts.

---

### Weather settings

#### WXS — Get/Set Weather Settings *(PM)*

```text
# Controller → Radio
WXS\r                                  # ① get
WXS,[DLY],[ATT],[ALT_PRI],[RECORD]\r   # ② set
# Radio → Controller
WXS,[DLY],[ATT],[ALT_PRI],[RECORD]\r   # ①
WXS,OK\r                               # ②
```

| Parameter | Description |
|---|---|
| `[DLY]` | `0` OFF / `1`–`5` |
| `[ATT]` | `0` / `1` |
| `[ALT_PRI]` | Weather alert priority: `0` OFF / `1` ON |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |

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
| `[FIPS1-8]` | FIPS code, 6 digits (`000000`–`999999`), or `------` = none |

`,`-only parameters unchanged in set; any format error aborts.

---

### Other settings

#### TON — Get/Set Tone-Out Settings *(PM)*

```text
# Controller → Radio
TON,[INDEX]\r     # ① get (printed "TON[INDEX]" in source)
# ② set (single line on the wire)
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RECORD]\r
# Radio → Controller
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RECORD]\r   # ①
TON,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[FRQ]` | Channel frequency |
| `[MOD]` | `AUTO` / `FM` / `NFM` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | `0` OFF / `1`–`5` / `30` / `INF` |
| `[ALT]` | `0` OFF / `1`–`9` |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[TONE_A]` / `[TONE_B]` | Tone frequency in 0.1 Hz units (e.g. `10000` = 1000.0 Hz, `00000` = 0.0 Hz) |
| `[RECORD]` | Tape-out: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

#### CNT — Get/Set LCD Contrast Settings *(PM)*

```text
# Controller → Radio
CNT\r
# Radio → Controller
CNT,[CONTRAST]\r      # 1–15
```

Described as Get/Set, but the source shows only the get form.

#### DUD — Get/Set LCD Upside-down Settings *(PM)*

```text
# Controller → Radio
DUD\r
# Radio → Controller
DUD,[UPSIDEDW]\r
```

| Parameter | Description |
|---|---|
| `[UPSIDEDW]` | LCD upside-down: `0` Off / `1` On |

Described as Get/Set, but the source shows only the get form.

#### SCN — Get/Set Scanner Option Settings *(PM)*

```text
# Controller → Radio (only this form is given)
SCN,[DISP_MODE],[ACT_CHAN],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r
# Radio → Controller
SCN,[DISP_MODE],[ACT_CHAN],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r
```

| Parameter | Description |
|---|---|
| `[DISP_MODE]` | Display mode: `1` / `2` |
| `[ACT_CHAN]` | Active channel display mode: `0` Off / `1` On |
| `[RSV]` | Always only `,` (18 reserved fields) |

#### VOL — Get/Set Volume Level Settings

```text
# Controller → Radio
VOL\r              # ① get
VOL,[LEVEL]\r      # ② set — 0–29
# Radio → Controller
VOL,[LEVEL]\r      # ①
VOL,OK\r           # ②
```

#### SQL — Get/Set Squelch Level Settings

```text
# Controller → Radio
SQL\r              # ① get
SQL,[LEVEL]\r      # ② set — 0 OPEN / 1–18 / 19 CLOSE
# Radio → Controller
SQL,[LEVEL]\r      # ①
SQL,OK\r           # ②
```

#### P25 — Get/Set APCO Data Settings

```text
# Controller → Radio
P25\r                              # ① get
P25,[THRESHOLD]\r                  # ② set
# Radio → Controller
P25,[APCO],[THRESHOLD],[ERR_RATE]\r   # ①
P25,OK\r                           # ② when [APCO] is not DFLT
P25,NG\r                           # ② when [APCO] is DFLT
```

| Parameter | Description |
|---|---|
| `[APCO]` | APCO threshold mode: `AUTO` / `MAN` / `DFLT` / `NONE` |
| `[THRESHOLD]` | APCO threshold (0–63) |
| `[ERR_RATE]` | Error rate (0–99) |

#### PWT — Get/Set P25 Waiting Time *(PM)*

```text
# Controller → Radio
PWT\r              # ① get
PWT,[TIME]\r       # ② set
# Radio → Controller
PWT,[TIME]\r       # ①
PWT,OK\r           # ②
```

| Parameter | Description |
|---|---|
| `[TIME]` | P25 wait time in ms: `0`, `100`, `200`, … `900`, `1000` |

On later models this setting moved into per-system/per-search `[P25WAITING]` fields.

---

### GPS settings

#### GDO — Get/Set GPS Display Option *(PM)*

```text
# Controller → Radio
GDO\r                                                              # ① get
GDO,[DISP_MODE],[UNIT],[TIME_FORMAT],[TIME_ZONE],[POS_FORMAT]\r    # ② set
# Radio → Controller
GDO,[DISP_MODE],[UNIT],[TIME_FORMAT],[TIME_ZONE],[POS_FORMAT]\r    # ①
GDO,OK\r                                                           # ②
```

| Parameter | Description |
|---|---|
| `[DISP_MODE]` | `0` ETA / `1` Clock / `2` Elevation / `3` Speed / `4` Location |
| `[UNIT]` | `0` mile / `1` km |
| `[TIME_FORMAT]` | `0` 12 H / `1` 24 H |
| `[TIME_ZONE]` | `-14.0` … `14.0` in 0.5 steps (e.g. `-14.0` = −14.0 h) |
| `[POS_FORMAT]` | `DMS` / `DEG` |

#### GGA — Get GGA Data from GPS

```text
# Controller → Radio
GGA\r
# Radio → Controller
GGA,[GGA_DATA]\r
```

`[GGA_DATA]` is the latest GGA sentence received from the GPS (`$GPGGA` … up to the checksum). If no GGA data has been received, only `,` is returned.

#### RMC — Get RMC Data from GPS

```text
# Controller → Radio
RMC\r
# Radio → Controller
RMC,[RMC_DATA]\r
```

`[RMC_DATA]` is the latest RMC sentence received from the GPS (`$GPRMC` … up to the checksum). If no RMC data has been received, only `,` is returned.

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

---

## CTCSS/DCS code list

### None / Search

| Mode | Code |
|---|---:|
| NONE / All | 0 |
| SEARCH | 127 |

### CTCSS

| Tone | Code | Tone | Code | Tone | Code |
|---|---:|---|---:|---|---:|
| 67.0 Hz | 64 | 107.2 Hz | 78 | 167.9 Hz | 93 |
| 69.3 Hz | 65 | 110.9 Hz | 79 | 171.3 Hz | 94 |
| 71.9 Hz | 66 | 114.8 Hz | 80 | 173.8 Hz | 95 |
| 74.4 Hz | 67 | 118.8 Hz | 81 | 177.3 Hz | 96 |
| 77.0 Hz | 68 | 123.0 Hz | 82 | 179.9 Hz | 97 |
| 79.7 Hz | 69 | 127.3 Hz | 83 | 183.5 Hz | 98 |
| 82.5 Hz | 70 | 131.8 Hz | 84 | 186.2 Hz | 99 |
| 85.4 Hz | 71 | 136.5 Hz | 85 | 189.9 Hz | 100 |
| 88.5 Hz | 72 | 141.3 Hz | 86 | 192.8 Hz | 101 |
| 91.5 Hz | 73 | 146.2 Hz | 87 | 196.6 Hz | 102 |
| 94.8 Hz | 74 | 151.4 Hz | 88 | 199.5 Hz | 103 |
| 97.4 Hz | 75 | 156.7 Hz | 89 | 203.5 Hz | 104 |
| 100.0 Hz | 76 | 159.8 Hz | 90 | 206.5 Hz | 105 |
| 103.5 Hz | 77 | 162.2 Hz | 91 | 210.7 Hz | 106 |
| | | 165.5 Hz | 92 | 218.1 Hz | 107 |
| | | | | 225.7 Hz | 108 |
| | | | | 229.1 Hz | 109 |
| | | | | 233.6 Hz | 110 |
| | | | | 241.8 Hz | 111 |
| | | | | 250.3 Hz | 112 |
| | | | | 254.1 Hz | 113 |

### DCS

| DCS | Code | DCS | Code | DCS | Code | DCS | Code |
|---|---:|---|---:|---|---:|---|---:|
| 023 | 128 | 152 | 154 | 306 | 179 | 506 | 208 |
| 025 | 129 | 155 | 155 | 311 | 180 | 516 | 209 |
| 026 | 130 | 156 | 156 | 315 | 181 | 523 | 210 |
| 031 | 131 | 162 | 157 | 325 | 182 | 526 | 211 |
| 032 | 132 | 165 | 158 | 331 | 183 | 532 | 212 |
| 036 | 133 | 172 | 159 | 332 | 184 | 546 | 213 |
| 043 | 134 | 174 | 160 | 343 | 185 | 565 | 214 |
| 047 | 135 | 205 | 161 | 346 | 186 | 606 | 215 |
| 051 | 136 | 212 | 162 | 351 | 187 | 612 | 216 |
| 053 | 137 | 223 | 163 | 356 | 188 | 624 | 217 |
| 054 | 138 | 225 | 164 | 364 | 189 | 627 | 218 |
| 065 | 139 | 226 | 165 | 365 | 190 | 631 | 219 |
| 071 | 140 | 243 | 166 | 371 | 191 | 632 | 220 |
| 072 | 141 | 244 | 167 | 411 | 192 | 654 | 221 |
| 073 | 142 | 245 | 168 | 412 | 193 | 662 | 222 |
| 074 | 143 | 246 | 169 | 413 | 194 | 664 | 223 |
| 114 | 144 | 251 | 170 | 423 | 195 | 703 | 224 |
| 115 | 145 | 252 | 171 | 431 | 196 | 712 | 225 |
| 116 | 146 | 255 | 172 | 432 | 197 | 723 | 226 |
| 122 | 147 | 261 | 173 | 445 | 198 | 731 | 227 |
| 125 | 148 | 263 | 174 | 446 | 199 | 732 | 228 |
| 131 | 149 | 265 | 175 | 452 | 200 | 734 | 229 |
| 132 | 150 | 266 | 176 | 454 | 201 | 743 | 230 |
| 134 | 151 | 271 | 177 | 455 | 202 | 754 | 231 |
| 143 | 152 | 274 | 178 | 462 | 203 | | |
| 145 | 153 | | | 464 | 204 | | |
| | | | | 465 | 205 | | |
| | | | | 466 | 206 | | |
| | | | | 503 | 207 | | |

---

---

## Errata notes (conversion)

These are observations about the source text, not part of the original spec:

- **KEY** — response is given as `KEY,OK\r`, but the examples show bare `OK\r`; Ex.2's last line shows `→ OK` (direction reversed). Verify against hardware.
- **Command list** — PWT and MCP aren't marked Program-Mode-only in the list, but MCP's own text says it is, and PWT's says the same.
- **GID** — `[SITE_TYPE]` values are not listed.
- **QSC/QSH** — FUNCTION text mentions "UASD", which appears nowhere else.
- **STS** — `[BK_DIMMER]` is printed with a doubled opening parenthesis.
- **ACT** — response shows `[INDEX]` but the definition names it `[TGID_INDEX]`.
- **TIN** — `[RSV]` occupies the slot where later models have `[PRI]`.
- **SIN** — set form uses `[GPS_ENABLE]`, response uses `[GPS ENABLE]` (with a space).
- **MBP** — the step and offset value ranges are not given.
- **MCP** — says to set the band plan type to "800MHz Custom" with SIF, but SIF has no such field on this model; the site type is chosen in AST (`M82C` / `M81C`). Also mentions a "system protect bit", which this model's CSY/SIN don't have. The OK reply is printed `MCP, OK` with a space.
- **CSP** — response is missing a comma between `[START_KEY]` and `[RECORD]`.
- **CSG** — says `0` = valid, `1` = invalid, then says all ranges can't be set to `0`; one of these is likely inverted.
- **TON** — get form is printed `TON[INDEX]` with no comma.
- **CNT / DUD** — titled Get/Set, but only the get form is given.
- **SCN** — only a full set-style line is shown for the command and response; no bare `SCN\r` get or `SCN,OK` reply.
- **AGS** — fields are printed with no commas between them; the `D_*` fields are labelled "Analog" in the source (shown here as Digital, matching their names).
- **MEM** — `[SYS]` range is missing its closing parenthesis.
