# BC346XT Operation Specification — Remote Commands

> Markdown conversion of section 7.13 (Remote Command), the CTCSS/DCS code list, and section 7.14 (Font Data) of the Uniden *BC346XT Operation Specification* (pp. 193–238).
> Same command layout as the BCD396XT, but with no P25/digital support: AGC, P25, color and NAC fields are reserved (sent as empty `,`), and only CNV/MOT/EDC/EDS/LTR systems exist.
> Wire formats are reproduced as written in the source. Apparent typos in the original are kept in place and listed under [Errata notes](#errata-notes-conversion) at the end.

---

## Contents

- [7.13 Remote Command](#713-remote-command)
  - [Remote communication format](#remote-communication-format)
  - [Format of this document](#format-of-this-document)
  - [General notes](#general-notes)
  - [Remote command list](#remote-command-list)
- [Command reference](#command-reference)
  - [Remote control](#remote-control) — GID, KEY, POF, QSH, QSC, CSC, PWR, STS, GLG, JPM, MNU, JNT
  - [System information](#system-information) — MDL, VER
  - [Program control mode](#program-control-mode) — PRG, EPG
  - [System settings](#system-settings) — BLT, BSV, COM, CLR, KBP, OMS, PRI, AGV
  - [Scan settings](#scan-settings) — SCT … MEM
  - [Location settings](#location-settings) — LIH, LIT, CLA, DLA, LIN
  - [Search / Close Call settings](#search--close-call-settings) — SCO, BBS, SHK, GLF, ULF, LOF, CLC
  - [Service search settings](#service-search-settings) — SSP
  - [Custom search settings](#custom-search-settings) — CSG, CBP, CSP
  - [Weather settings](#weather-settings) — WXS, SGP
  - [Other settings](#other-settings) — TON, CNT, SCN, VOL, SQL, P25, DBC, GDO, BSP
  - [IF exchange list settings](#if-exchange-list-settings) — GIE, CIE, RIE
  - [Test](#test) — BAV, WIN
- [CTCSS/DCS code list](#ctcssdcs-code-list)
- [7.14 Font Data](#714-font-data)
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
8. `[INDEX]` / `[xxx_INDEX]` is an index into the internal memory chain. The Dynamic Memory Allocation structure always uses it as a handle to access data and to traverse forward/reverse or up/down. Range: 1 to the maximum memory block (about 45000).
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

"PM" = accepted in Program Mode only.

| No. | Category | Command | Function | PM |
|---:|---|---|---|:---:|
| 1 | Remote Control | GID | Get Current TalkGroup ID Status | |
| 2 | Remote Control | KEY | Push KEY | |
| 3 | Remote Control | POF | Power Off | |
| 4 | Remote Control | QSH | Go to quick search hold mode | |
| 5 | Remote Control | QSC | Set current frequency and get reception status | |
| 6 | Remote Control | CSC | Go to Custom search and get reception status | |
| 7 | Remote Control | PWR | Get RSSI Level | |
| 8 | Remote Control | STS | Get Current Status | |
| 9 | Remote Control | GLG | Get Reception Status | |
| 10 | Remote Control | JPM | Jump Mode | |
| 11 | Remote Control | JNT | Jump to Number Tag | |
| 12 | Remote Control | MNU | Menu Mode | |
| 13 | System Information | MDL | Get Model Info | |
| 14 | System Information | VER | Get Firmware Version | |
| 15 | Program Control Mode | PRG | Enter Program Mode | |
| 16 | Program Control Mode | EPG | Exit Program Mode | |
| 17 | System Settings | BLT | Get/Set Backlight | ✓ |
| 18 | System Settings | BSV | Get/Set Battery Info | ✓ |
| 19 | System Settings | COM | Get/Set COM port setting | ✓ |
| 20 | System Settings | CLR | Clear All Memory | ✓ |
| 21 | System Settings | KBP | Get/Set Key Beep and setting | ✓ |
| 22 | System Settings | OMS | Get/Set Opening Message | ✓ |
| 23 | System Settings | PRI | Get/Set Priority Mode | ✓ |
| 24 | System Settings | AGV | Get/Set Auto Gain Control | ✓ |
| 25 | Scan Settings | SCT | Get System Count | ✓ |
| 26 | Scan Settings | SIH | Get System Index Head | ✓ |
| 27 | Scan Settings | SIT | Get System Index Tail | ✓ |
| 28 | Scan Settings | QSL | Get/Set System/Site Quick Lockout | ✓ |
| 29 | Scan Settings | QGL | Get/Set Group Quick Lockout | ✓ |
| 30 | Scan Settings | CSY | Create System | ✓ |
| 31 | Scan Settings | DSY | Delete System | ✓ |
| 32 | Scan Settings | SIN | Get/Set System Info | ✓ |
| 33 | Scan Settings | TRN | Get/Set Trunk Info | ✓ |
| 34 | Scan Settings | AST | Append Site | ✓ |
| 35 | Scan Settings | SIF | Get/Set Site Info | ✓ |
| 36 | Scan Settings | MCP | Get/Set Motorola Custom Band Plan | ✓ |
| 37 | Scan Settings | ABP | Get/Set APCO-P25 Band Plan | ✓ |
| 38 | Scan Settings | TFQ | Get/Set Trunk Frequency Info | ✓ |
| 39 | Scan Settings | AGC | Append Channel Group | ✓ |
| 40 | Scan Settings | AGT | Append TalkGroup ID Group | ✓ |
| 41 | Scan Settings | DGR | Delete Group / Site | ✓ |
| 42 | Scan Settings | GIN | Get/Set Group Info | ✓ |
| 43 | Scan Settings | ACC | Append Channel / Trunk Frequency | ✓ |
| 44 | Scan Settings | ACT | Append TalkGroup ID | ✓ |
| 45 | Scan Settings | DCH | Delete Channel | ✓ |
| 46 | Scan Settings | CIN | Get/Set Channel Info | ✓ |
| 47 | Scan Settings | TIN | Get/Set TalkGroup ID Info | ✓ |
| 48 | Scan Settings | GLI | Get Lockout TalkGroup ID (for Rvw L/O ID) | ✓ |
| 49 | Scan Settings | SLI | Get Search L/O TalkGroup ID | ✓ |
| 50 | Scan Settings | ULI | Unlock TalkGroup ID (for Rvw L/O ID) | ✓ |
| 51 | Scan Settings | LOI | Lockout ID (TalkGroup ID) | ✓ |
| 52 | Scan Settings | REV | Get Rev Index | ✓ |
| 53 | Scan Settings | FWD | Get Fwd Index | ✓ |
| 54 | Scan Settings | RMB | Get Remains of Memory Block | ✓ |
| 55 | Scan Settings | MEM | Get Memory Used | ✓ |
| 56 | Location Setting | LIH | Get Location Alert System Index Head | ✓ |
| 57 | Location Setting | LIT | Get Location Alert System Index Tail | ✓ |
| 58 | Location Setting | CLA | Create Location Alert System | ✓ |
| 59 | Location Setting | DLA | Delete Location Alert System | ✓ |
| 60 | Location Setting | LIN | Get/Set Location Alert System Info | ✓ |
| 61 | Search / Close Call Settings | SCO | Get/Set Search/Close Call Settings | ✓ |
| 62 | Search / Close Call Settings | BBS | Get/Set Broadcast Screen Band Settings | ✓ |
| 63 | Search / Close Call Settings | SHK | Get/Set Search Key Settings | ✓ |
| 64 | Search / Close Call Settings | GLF | Get Global Lockout Freq | ✓ |
| 65 | Search / Close Call Settings | ULF | Unlock Global L/O | ✓ |
| 66 | Search / Close Call Settings | LOF | Lock Out Frequency | ✓ |
| 67 | Search / Close Call Settings | CLC | Get/Set Close Call Settings | ✓ |
| 68 | Service Search Settings | SSP | Get/Set Service Search Settings | ✓ |
| 69 | Custom Search Settings | CSG | Get/Set Custom Search Group | ✓ |
| 70 | Custom Search Settings | CBP | Get/Set C-Ch Only Custom search MOT Band Plan | ✓ |
| 71 | Custom Search Settings | CSP | Get/Set Custom Search Settings | ✓ |
| 72 | Weather Settings | WXS | Get/Set Weather Settings | ✓ |
| 73 | Weather Settings | SGP | Get/Set SAME Group Settings | ✓ |
| 74 | Tone-Out Setting | TON | Get/Set Tone-Out Settings | ✓ |
| 75 | LCD Contrast Settings | CNT | Get/Set LCD Contrast Settings | ✓ |
| 76 | Scanner Option Settings | SCN | Get/Set Scanner Option Settings | ✓ |
| 77 | Volume Level Settings | VOL | Get/Set Volume Level Settings | |
| 78 | Squelch Level Settings | SQL | Get/Set Squelch Level Settings | |
| 79 | APCO Data Settings | P25 | Get/Set APCO Data Settings | |
| 80 | Default Band Coverage Settings | DBC | Get/Set Default Band Coverage Settings | ✓ |
| 81 | GPS Settings | GDO | Get/Set GPS Format | ✓ |
| 82 | Band Scope Settings | BSP | Get/Set Band Scope Settings | ✓ |
| 83 | IF exchange list Settings | GIE | Get Global IF exchange Frequency | ✓ |
| 84 | IF exchange list Settings | CIE | Clear IF exchange Frequency | ✓ |
| 85 | IF exchange list Settings | RIE | Register IF exchange Frequency | ✓ |
| 86 | TEST | BAV | Get Battery Voltage | |
| 87 | TEST | WIN | Get Window Voltage | |

---

## Command reference

Common parameter definitions reused by many commands:

| Parameter | Values |
|---|---|
| `[RSV]` | Reserve parameter — always only `,` (empty) unless stated otherwise |
| `[MOD]` | Modulation: `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` (subset per command) |
| `[ATT]` | Attenuation: `0` OFF / `1` ON |
| `[DLY]` | Delay time: `-10`, `-5`, `-2`, `0`, `1`, `2`, `5`, `10`, `30` |
| `[NUMBER_TAG]` | Number tag: `0`–`999` / `NONE` |
| `[ALT_PATTERN]` | Alert light pattern: `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[VOL_OFFSET]` | Volume offset: `-3` … `+3` |

**Broadcast Screen bitfield `[BSC]`** (used by QSH, QSC, SCO) — 16 digits, each `0` (OFF) or `1` (ON), leftmost first:

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
| `[SITE_TYPE]` | `CNV` Conventional · `MOT` Motorola · `EDC` EDACS Narrow/Wide · `EDS` EDACS SCAT · `LTR` LTR |
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
| `M` | Menu |
| `F` | Func |
| `H` | Hold |
| `S` | Scan/Srch |
| `L` | L/O |
| `0`–`9` | Digits 0–9 |
| `.` | ./NO |
| `E` | E/YES |
| `>` | VFO right (set `KEY_MODE` to `P`) |
| `<` | VFO left (set `KEY_MODE` to `P`) |
| `^` | VFO push |
| `P` | Power/Light/Lock |

| `[KEY_MODE]` | Meaning |
|---|---|
| `P` | Press |
| `L` | Long press |
| `H` | Hold (press and hold until Release received) |
| `R` | Release (cancel Hold state) |

Examples:

```text
# Ex.1 — Press Menu key
→ KEY,M,P\r
← OK\r

# Ex.2 — Press F + Scan
→ KEY,F,P\r    # hold F key
← OK\r
→ KEY,S,P\r    # press Scan key (F + Scan operation)
← OK\r
→ KEY,F,P\r    # release F key
→ OK\r         # (direction arrow as printed in source)

# Ex.3 — Press and hold L/O key
→ KEY,L,L\r
← OK\r
```

The key-hold state times out 10 seconds after a key-hold command (e.g. `KEY,F,H`) if there is no further communication.

#### POF — Power Off

```text
# Controller → Radio
POF\r
# Radio → Controller
POF,OK\r
```

Turns off the scanner. After this command the scanner accepts no further commands.

#### QSH — Go to Quick Search Hold mode

```text
# Controller → Radio (single line on the wire)
QSH,[FRQ],[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[RSV],[RSV]\r
# Radio → Controller
QSH,OK\r   or   QSH,NG\r
```

| Parameter | Description |
|---|---|
| `[FRQ]` | Frequency |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | `-10,-5,-2,0,1,2,5,10,30` |
| `[CODE_SRCH]` | `0` OFF / `1` CTCSS/DCS |
| `[BSC]` | Broadcast Screen bitfield (see above) |
| `[REP]` | Repeater Find: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

Invalid while the scanner is in Menu Mode, during Direct Entry, or during Quick Save.

**Function:** Sets an arbitrary frequency and changes to Quick Search Hold (VFO) mode. Parameters such as STP change the contents of the Srch/CloCall option.
**Note:** The command works even if only `[FRQ]` is set.

#### QSC — Set current frequency and get reception status

```text
# Controller → Radio (single line on the wire)
QSC,[FRQ],[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[RSV],[RSV]\r
# Radio → Controller
QSC,[RSSI],[FRQ],[SQL]\r   or   QSC,NG\r
```

Parameters as for QSH, plus:

| Parameter | Description |
|---|---|
| `[CODE_SRCH]` | `0` OFF / `1` CTCSS/DCS search |
| `[RSSI]` | RSSI A/D value (0–1023) |
| `[SQL]` | Squelch status: `0` CLOSE / `1` OPEN |

Invalid while in Menu Mode, during Direct Entry, or during Quick Save.

**Function:** Sets an arbitrary frequency and changes to Quick Search Hold (VFO) mode. Parameters such as STP change the contents of the Srch/CloCall option.

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
STS,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE],[L3_CHAR],[L3_MODE],[L4_CHAR],[L4_MODE],...,[L8_CHAR],[L8_MODE],[SQL],[MUT],[BAT],[WAT],[RSV],[RSV],[SIG_LVL],[RSV],[BK_DIMMER]\r
```

| Parameter | Description |
|---|---|
| `[DSP_FORM]` | Display form, 4–8 digits; each `0` = small font, `1` = large font |
| `[Ln_CHAR]` | Line *n* characters, 16 chars (fixed length) |
| `[Ln_MODE]` | Line *n* display mode, 16 chars |
| `[SQL]` | Squelch: `0` CLOSE / `1` OPEN |
| `[MUT]` | Mute: `0` OFF / `1` ON |
| `[BAT]` | Battery low: `0` no alert / `1` alert |
| `[WAT]` | Weather alert: `0` no alert / `1` alert / `$$$` alert SAME code |
| `[RSV]` | Reserve — always only `0` |
| `[SIG_LVL]` | Signal level (0–5) |
| `[BK_DIMMER]` | Backlight dimmer: `0` OFF / `1` Low / `2` Middle / `3` High |

**Display mode characters (`[Ln_MODE]`):** space = normal, `*` = reverse, `_` = underline. If all 16 characters are normal, only `,` is sent. The number of `[Lx_CHAR]`/`[Lx_MODE]` pairs depends on the display form.

**Ex.1** — Menu screen (squelch OPEN, mute OFF, no battery alert, no weather alert):

```text
→ STS\r
← 1111,                 # DSP_FORM: 4 lines, all large font
  -- M E N U --   ,     # L1_CHAR
  ________________,     # L1_MODE (underlined)
  Program System  ,     # L2_CHAR
  ****************,     # L2_MODE (reversed)
  Program Location,     # L3_CHAR
  ,                     # L3_MODE (all normal)
  Srch/CloCall Opt,     # L4_CHAR
  ,                     # L4_MODE (all normal)
  1,0,0,0,,,0,,,\r      # SQL,MUT,BAT,WAT,RSV,RSV,SIG_LVL,RSV,BK_DIMMER
```

**Ex.2** — Hold screen (squelch CLOSE, mute ON, no battery alert, weather alert):

```text
→ STS\r
← 011000,               # DSP_FORM: 6 lines
  HOLD    L/O     ,     # L1_CHAR
  ,                     # L1_MODE
  SYSTEM 1        ,     # L2_CHAR
  ,                     # L2_MODE
  851.0125MHz     ,     # L3_CHAR
  ,                     # L3_MODE
  P NFM ATT       ,     # L4_CHAR
  ,                     # L4_MODE
  S1: 5           ,     # L5_CHAR
  ,                     # L5_MODE
  GRP 2       WX,       # L6_CHAR
  ,                     # L6_MODE
  0,1,0,0,,,1,,,\r # status fields
```

Returns current scanner status.

#### GLG — Get Reception Status

```text
# Controller → Radio
GLG\r
# Radio → Controller (single line on the wire)
GLG,[FRQ/TGID],[MOD],[ATT],[CTCSS/DCS],[NAME1],[NAME2],[NAME3],[SQL],[MUT],[SYS_TAG],[CHAN_TAG],[RSV]\r
GLG,,,,,,,,,,\r      # nothing received yet
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
| `[SYS_TAG]` | Current system number tag (0–999 / `NONE`) |
| `[CHAN_TAG]` | Current channel number tag (0–999 / `NONE`) |

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
| `SVC_MODE` | Service Search mode | `PublicSafety`, `News`, `HAM`, `Marine`, `Railroad`, `Air`, `CB`, `FRS/GMRS/MURS`, `Racing`, `FM`, `Special` |
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

#### JNT — Jump to Number Tag

```text
# Controller → Radio
JNT,[SYS_TAG],[CHAN_TAG]\r
# Radio → Controller
JNT,OK\r
```

| Parameter | Description |
|---|---|
| `[SYS_TAG]` | System number tag (0–999 / `NONE`) |
| `[CHAN_TAG]` | Channel number tag (0–999 / `NONE`) |

- Both blank → error.
- `[SYS_TAG]` blank, `[CHAN_TAG]` set → jump to that channel tag in the current system.
- `[SYS_TAG]` set, `[CHAN_TAG]` blank → jump to the first channel of that system tag.

---

### System information

#### MDL — Get Model Info

```text
# Controller → Radio
MDL\r
# Radio → Controller
MDL,BC346XT\r
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

#### BLT — Get/Set Backlight *(PM)*

```text
# Controller → Radio
BLT\r                              # ① get
BLT,[EVNT],[RSV],[DIMMER]\r      # ② set
# Radio → Controller
BLT,[EVNT],[RSV],[DIMMER]\r      # ①
BLT,OK\r                           # ②
```

| Parameter | Values |
|---|---|
| `[EVNT]` | `IF` infinite · `10` 10 s · `30` 30 s · `KY` keypress · `SQ` squelch |
| `[RSV]` | Reserved (no backlight color) |
| `[DIMMER]` | `1` Low / `2` Middle / `3` High |

#### BSV — Get/Set Battery Info *(PM)*

```text
# Controller → Radio
BSV\r                              # ① get
BSV,[BAT_SAVE],[CHARGE_TIME]\r     # ② set
# Radio → Controller
BSV,[BAT_SAVE],[CHARGE_TIME]\r     # ①
BSV,OK\r                           # ②
```

| Parameter | Values |
|---|---|
| `[BAT_SAVE]` | `0` OFF / `1` ON |
| `[CHARGE_TIME]` | Battery charge time (1–16) |

#### COM — Get/Set COM port setting *(PM)*

```text
# Controller → Radio
COM,\r                       # ① get (written "COM,[/r]" in source)
COM,[BAUDRATE],[RSV]\r       # ② set
# Radio → Controller
COM,[BAUDRATE],[RSV]\r       # ①
COM,OK\r                     # ②
```

| `[BAUDRATE]` | Meaning |
|---|---|
| `OFF` | Off |
| `4800` / `9600` / `19200` / `38400` / `57600` / `115200` | bps |

**Notes:** After receiving `COM,OK`, do not send the next command for 2 seconds. Only the PC Control (baud rate) setting is not reset to an initial value.

#### CLR — Clear All Memory *(PM)*

```text
# Controller → Radio
CLR\r
# Radio → Controller
CLR,OK\r
```

Resets all memory to initial settings. Takes dozens of seconds. Only the PC Control (baud rate) setting is not reset.

#### KBP — Get/Set Key Beep and setting *(PM)*

```text
# Controller → Radio
KBP\r                          # ① get
KBP,[LEVEL],[LOCK],[SAFE]\r    # ② set
# Radio → Controller
KBP,[LEVEL],[LOCK],[SAFE]\r    # ①
KBP,OK\r                       # ②
```

| Parameter | Values |
|---|---|
| `[LEVEL]` | Beep level: `0` Auto / `1`–`15` / `99` OFF |
| `[LOCK]` | Key lock: `0` OFF / `1` ON |
| `[SAFE]` | Key safe: `0` OFF / `1` ON |

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
PRI\r                                   # ① get
PRI,[PRI_MODE],[MAX_CHAN],[INTERVAL]\r  # ② set
# Radio → Controller
PRI,[PRI_MODE],[MAX_CHAN],[INTERVAL]\r  # ①
PRI,OK\r                                # ②
```

| Parameter | Values |
|---|---|
| `[PRI_MODE]` | `0` OFF / `1` ON / `2` PLUS ON |
| `[MAX_CHAN]` | Priority scan max channels at once (1–100) |
| `[INTERVAL]` | Priority scan interval time (1–10) |

#### AGV — Get/Set Auto Gain Control *(PM)*

Present only for compatibility with the BCD396XT; every field is reserved.

```text
# Controller → Radio
AGV\r                                              # ① get
AGV,[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r    # ② set
# Radio → Controller
AGV,[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r    # ①
AGV,OK\r                                           # ②
```

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
CSY,[SYS_TYPE],[PROTECT]\r
# Radio → Controller
CSY,[SYS_INDEX]\r
```

| Parameter | Values |
|---|---|
| `[SYS_TYPE]` | `CNV` Conventional · `MOT` Motorola · `EDC` EDACS Narrow/Wide · `EDS` EDACS SCAT · `LTR` LTR |
| `[PROTECT]` | Protect bit: `0` OFF / `1` ON |
| `[SYS_INDEX]` | Index of the created system |

Creates a system and returns its index (the handle for get/set of system info). Returns `-1` if creation failed due to lack of resources.

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
SIN,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[RSV],[RSV],[RSV],[RSV],[RSV],[START_KEY],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
SIN,[SYS_TYPE],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[RSV],[RSV],[RSV],[RSV],[RSV],[REV_INDEX],[FWD_INDEX],[CHN_GRP_HEAD],[CHN_GRP_TAIL],[SEQ_NO],[START_KEY],[RSV],[RSV],[RSV],[RSV],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV],[PROTECT],[RSV]\r
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
| `[DLY]` | Delay time |
| `[REV_INDEX]` | Reverse system index in scan order |
| `[FWD_INDEX]` | Forward system index in scan order |
| `[CHN_GRP_HEAD]` | Channel group index head (conventional) / site index head (trunked) |
| `[CHN_GRP_TAIL]` | Channel group index tail (conventional) / site index tail (trunked) |
| `[SEQ_NO]` | System sequence number (1–500) |
| `[START_KEY]` | Startup configuration key (0–9 / `.` = none) |
| `[NUMBER_TAG]` | 0–999 / `NONE` |
| `[PROTECT]` | Protect bit: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

- Parameters not applicable to the system type are returned as just `,`, and ignored in set commands.
- In set commands, `,`-only parameters are unchanged; any format error aborts the command.
- When the protect bit is ON, all parameters except `[SYS_TYPE]`, `[NAME]`, `[REV_INDEX]`, `[FWD_INDEX]`, `[CHN_GRP_HEAD]`, `[CHN_GRP_TAIL]` are returned as reserve parameters.

#### TRN — Get/Set Trunk Info *(PM)*

```text
# Controller → Radio
TRN,[INDEX]\r     # ① get
# ② set (single line on the wire)
TRN,[INDEX],[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[RSV],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[MOT_ID],[RSV],[EMG_PATTERN],[RSV],[PRI_ID_SCAN]\r

# Radio → Controller
# ① (single line on the wire)
TRN,[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[RSV],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[TGID_GRP_HEAD],[TGID_GRP_TAIL],[ID_LOUT_GRP_HEAD],[ID_LOUT_GRP_TAIL],[MOT_ID],[RSV],[EMG_PATTERN],[RSV],[PRI_ID_SCAN]\r
TRN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | System index |
| `[ID_SEARCH]` | `0` ID Scan mode / `1` Search mode |
| `[S_BIT]` | Motorola status bit: `0` Ignore / `1` Yes |
| `[END_CODE]` | Motorola end code: `0` Ignore / `1` Yes |
| `[AFS]` | EDACS ID format: `0` Decimal / `1` AFS |
| `[EMG]` | Emergency alert: `0` Ignore / `1`–`9` Alert |
| `[EMGL]` | Emergency alert level: `0` OFF / `1`–`15` |
| `[FMAP]` | Fleet map: `0`–`15` preset / `16` custom |
| `[CTM_FMAP]` | Custom fleet map, 8 digits (blocks 0–7), each `0`–`E` = size code 0–14 |
| `[TGID_GRP_HEAD]` / `[TGID_GRP_TAIL]` | TGID group index head / tail of the system |
| `[ID_LOUT_GRP_HEAD]` / `[ID_LOUT_GRP_TAIL]` | L/O TGID group index head / tail |
| `[MOT_ID]` | Motorola ID format: `0` Decimal / `1` HEX |
| `[EMG_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[PRI_ID_SCAN]` | Priority ID scan: `0` OFF / `1` ON |
| `[RSV]` | Always only `,` |

- Parameters not applicable to the system type are returned as `,` and ignored in set commands.
- `,`-only parameters are unchanged in set; any format error aborts.
- When the protect bit is ON, all parameters except `[TGID_GRP_HEAD]`, `[TGID_GRP_TAIL]`, `[ID_LOUT_GRP_HEAD]`, `[ID_LOUT_GRP_TAIL]` are returned as reserve parameters.

#### AST — Append Site *(PM)*

```text
# Controller → Radio
AST,[SYS_INDEX],[RSV]\r
# Radio → Controller
AST,[SITE_INDEX]\r
```

Appends a site to the system. Returns `-1` if creation failed due to lack of resources.

#### SIF — Get/Set Site Info *(PM)*

```text
# Controller → Radio
SIF,[INDEX]\r     # ① get
# ② set (single line on the wire)
SIF,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[RSV],[RSV],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV],[MOT_TYPE],[EDACS_TYPE],[RSV],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
SIF,[RSV],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[RSV],[RSV],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV],[MOT_TYPE],[EDACS_TYPE],[RSV],[RSV]\r
SIF,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[NAME]` | Name (max. 16 chars) |
| `[QUICK_KEY]` | 0–99 / `.` = none |
| `[HLD]` | Site hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[MOD]` | `AUTO` / `FM` / `NFM` |
| `[ATT]` | `0` OFF / `1` ON |
| `[C-CH]` | Control channel only — always `1` (ON) |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward site index in scan order |
| `[SYS_INDEX]` | System index |
| `[CHN_HEAD]` / `[CHN_TAIL]` | Channel index head / tail of the group list |
| `[SEQ_NO]` | Site sequence number (1–256) |
| `[START_KEY]` | Startup configuration (0–9 / `.` = none) |
| `[LATITUDE]` / `[LONGITUDE]` | See general notes 12–13 |
| `[RANGE]` | 1–250 (1 = 0.5 mile or km) |
| `[GPS_ENABLE]` | `0` OFF / `1` ON |
| `[MOT_TYPE]` | Band type for MOT/EDACS: `STD` / `SPL` / `CUSTOM` |
| `[EDACS_TYPE]` | `WIDE` / `NARROW` |
| `[RSV]` | Always only `,` |

- Non-applicable parameters are returned as `,` and ignored in set commands.
- `,`-only parameters unchanged in set; any format error aborts.
- When the protect bit is ON, all except `[REV_INDEX]`, `[FWD_INDEX]`, `[SYS_INDEX]`, `[CHN_HEAD]`, `[CHN_TAIL]` are returned as reserve parameters.

#### MCP — Get/Set Motorola Custom Band Plan *(PM)*

```text
# Controller → Radio
MCP,[INDEX]\r     # ① get
# ② set (single line on the wire)
MCP,[INDEX],[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r

# Radio → Controller
# ① (single line on the wire)
MCP,[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r
MCP,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |
| `[LOWERn]` / `[UPPERn]` | Lower / upper frequency *n* |
| `[STEPn]` | Step *n* (see table) |
| `[OFFSETn]` | Offset *n* (−1023 … 1023) |

**Step codes** (shared with CBP):

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

Gets/sets the band plan for MOT 800 custom / VHF / UHF sites. If only `,` parameters are sent, the band plan is unchanged; any format error aborts. When the protect bit is ON, all parameters are returned as reserve parameters. **Before using this command, set the band plan type to `CUSTOM` with SIF.**

#### ABP — Get/Set APCO-P25 Band Plan *(PM)*

Present only for compatibility with the BCD396XT; all 32 band-plan fields are reserved.

```text
# Controller → Radio
ABP,[INDEX]\r                                  # ① get
ABP,[INDEX],[RSV],[RSV],...,[RSV],[RSV]\r      # ② set — 32 [RSV] fields
# Radio → Controller
ABP,[RSV],[RSV],...,[RSV],[RSV]\r              # ① 32 [RSV] fields
ABP,OK\r                                       # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | Site index |

A band plan with no data returns `0`. When the protect bit is ON, all parameters are returned as reserve parameters.

#### TFQ — Get/Set Trunk Frequency Info *(PM)*

```text
# Controller → Radio
TFQ,[CHN_INDEX]\r                                                               # ① get
TFQ,[CHN_INDEX],[FRQ],[LCN],[LOUT],[RSV],[NUMBER_TAG],[VOL_OFFSET],[RSV]\r      # ② set
# Radio → Controller
TFQ,[FRQ],[LCN],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[NUMBER_TAG],[VOL_OFFSET],[RSV]\r   # ①
TFQ,OK\r                                                                        # ②
```

| Parameter | Description |
|---|---|
| `[CHN_INDEX]` | Trunk frequency index |
| `[FRQ]` | Trunk frequency |
| `[LCN]` | LCN — EDACS Wide/Narrow: 1–30; LTR: 1–20 |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward frequency index of the site |
| `[SYS_INDEX]` | System index of the frequency |
| `[GRP_INDEX]` | Index of the site |
| `[NUMBER_TAG]` | 0–999 / `NONE` |
| `[VOL_OFFSET]` | −3 … +3 |

- `,`-only parameters unchanged in set; any format error aborts.
- For Motorola or EDACS SCAT systems, `[LCN]` is ignored.
- When the protect bit is ON, all except `[REV_INDEX]`, `[FWD_INDEX]`, `[SYS_INDEX]`, `[GRP_INDEX]` are returned as reserve parameters.
- `[NUMBER_TAG]` and `[VOL_OFFSET]` are used only for SCAT systems.

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
GIN,[GRP_INDEX]\r                                                                          # ① get
GIN,[GRP_INDEX],[NAME],[QUICK_KEY],[LOUT],[LATITUDE],[LONGITUDE],[RANGE],[GPS ENABLE]\r    # ② set
# Radio → Controller (single line on the wire)
GIN,[GRP_TYPE],[NAME],[QUICK_KEY],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO],[LATITUDE],[LONGITUDE],[RANGE],[GPS ENABLE]\r   # ①
GIN,OK\r                                                                                   # ②
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
| `[LATITUDE]` / `[LONGITUDE]` | See general notes 12–13 |
| `[RANGE]` | 1–250 (1 = 0.5 mile or km) |
| `[GPS ENABLE]` | `0` OFF / `1` ON |

- `,`-only parameters unchanged in set; any format error aborts.
- When the protect bit is ON, all except `[NAME]`, `[REV_INDEX]`, `[FWD_INDEX]`, `[SYS_INDEX]`, `[CHN_HEAD]`, `[CHN_TAIL]` are returned as reserve parameters.

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
CIN,[INDEX],[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[RSV],[RSV],[RSV],[NUMBER_TAG],[RSV],[ALT_PATTERN],[VOL_OFFSET]\r

# Radio → Controller
# ① (single line on the wire)
CIN,[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[AUDIO_TYPE],[RSV],[NUMBER_TAG],[RSV],[ALT_PATTERN],[VOL_OFFSET]\r
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
| `[AUDIO_TYPE]` | Response only; not defined in this spec (BCD396XT: `0` All / `1` Analog only / `2` Digital only) |
| `[NUMBER_TAG]` | 0–999 / `NONE` |
| `[ALT_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[VOL_OFFSET]` | −3 … +3 |

- `,`-only parameters unchanged in set; any format error aborts.
- When the protect bit is ON, all except `[REV_INDEX]`, `[FWD_INDEX]`, `[SYS_INDEX]`, `[GRP_INDEX]` are returned as reserve parameters.

#### TIN — Get/Set TGID Info *(PM)*

```text
# Controller → Radio
TIN,[INDEX]\r     # ① get
# ② set (single line on the wire)
TIN,[INDEX],[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL],[RSV],[RSV],[NUMBER_TAG],[RSV],[ALT_PATTERN],[VOL_OFFSET]\r

# Radio → Controller
# ① (single line on the wire)
TIN,[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[AUDIO_TYPE],[NUMBER_TAG],[RSV],[ALT_PATTERN],[VOL_OFFSET]\r
TIN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | TGID index |
| `[NAME]` | Name (max. 16 chars) |
| `[TGID]` | TGID |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[PRI]` | `0` OFF / `1` ON |
| `[ALT]` | `0` OFF / `1`–`9` tone no. |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Reverse / forward TGID index in the group |
| `[SYS_INDEX]` | System index of the TGID |
| `[GRP_INDEX]` | Group index of the TGID |
| `[AUDIO_TYPE]` | Response only; not defined in this spec (BCD396XT: `0` All / `1` Analog only / `2` Digital only) |
| `[NUMBER_TAG]` | 0–999 / `NONE` |
| `[ALT_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[VOL_OFFSET]` | −3 … +3 |

- `,`-only parameters unchanged in set; any format error aborts.
- When the protect bit is ON, all except `[REV_INDEX]`, `[FWD_INDEX]`, `[SYS_INDEX]`, `[GRP_INDEX]` are returned as reserve parameters.

#### GLI — Get Lockout TGID (for Rvw L/O ID) *(PM)*

```text
# Controller → Radio
GLI,[SYS_INDEX]\r
# Radio → Controller
GLI,[TGID]\r     # one locked-out TGID
GLI,-1\r         # no more lockout TGIDs
```

Gets the L/O TGID list of a system. Call repeatedly until `-1` is returned. When the protect bit is ON, only `-1` is returned.

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
| `[CHN]` | Number of channels (0–9000) |
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
LIN,[INDEX],[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR],[RSV],[ALT_PATTERN]\r

# Radio → Controller
# ① (single line on the wire)
LIN,[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SEQ_NO],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR],[RSV],[ALT_PATTERN]\r
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
| `[RANGE]` | 1–80 (1 = 0.05 mile or km) |
| `[SPEED]` | Speed limit 0–200 (mph or km/h) |
| `[DIR]` | Heading: `360` all · `0` N · `44` NE · `90` E · `134` SE · `180` S · `224` SW · `270` W · `314` NW |
| `[ALT_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |

Non-applicable parameters are ignored in set; `,`-only parameters unchanged; any format error aborts.

---

### Search / Close Call settings

#### SCO — Get/Set Search/Close Call Settings *(PM)*

```text
# Controller → Radio
SCO\r             # ① get
# ② set (single line on the wire)
SCO,[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE],[RSV],[RSV],[RSV],[RSV]\r
# Radio → Controller
SCO,[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE],[RSV],[RSV],[RSV],[RSV]\r   # ①
SCO,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` OFF / `1` ON |
| `[DLY]` | Delay time |
| `[CODE_SRCH]` | `0` OFF / `1` CTCSS/DCS |
| `[BSC]` | Broadcast Screen bitfield |
| `[REP]` | Repeater find: `0` OFF / `1` ON |
| `[MAX_STORE]` | Max auto store (1–256) |

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
SHK\r                                                          # ① get
SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[RSV],[RSV],[RSV]\r # ② set
# Radio → Controller
SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[RSV],[RSV],[RSV]\r # ①
SHK,OK\r                                                       # ②
```

`[SRCH_KEY_1]`–`[SRCH_KEY_3]` (search range):

| Value | Range | Value | Range |
|---|---|---|---|
| `.` | Not assigned | `CUSTOM_1` … `CUSTOM_10` | Custom 1–10 range |
| `PublicSafety` | Public Safety | `TONE_OUT` | Tone Out mode |
| `News` | News | `B_SCOPE` | Band Scope |
| `HAM` | HAM radio | | |
| `Marine` | Marine | | |
| `Railroad` | Railroad | | |
| `Air` | Air | | |
| `CB` | CB radio | | |
| `FRS/GMRS/MURS` | FRS/GMRS/MURS | | |
| `Racing` | Racing | | |
| `FM` | FM broadcast | | |
| `Special` | Special | | |

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
# ② set (single line on the wire)
CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY],[NUMBER_TAG],[RSV],[ALT_PATTERN]\r
# Radio → Controller
CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY],[NUMBER_TAG],[RSV],[ALT_PATTERN]\r   # ①
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
| `[NUMBER_TAG]` | 0–999 / `NONE` |
| `[ALT_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[RSV]` | Always only `,` |

`[CC_BAND]` — each digit `0` OFF / `1` ON, left → right:

| Position | Band |
|---|---|
| 1 | VHF LOW1 |
| 2 | VHF LOW2 |
| 3 | AIR BAND |
| 4 | VHF HIGH |
| 5 | Reserve |
| 6 | UHF |
| 7 | 800 MHz+ |

`,`-only parameters unchanged in set; any format error aborts.

---

### Service search settings

#### SSP — Get/Set Service Search Settings *(PM)*

```text
# Controller → Radio
SSP,[SRCH_INDEX]\r   # ① get
# ② set (single line on the wire)
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV]\r
# Radio → Controller
SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV]\r   # ①
SSP,OK\r             # ②
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
| `8` | FRS/GMRS/MURS |
| `9` | Racing |
| `11` | FM Broadcast |
| `12` | Special |

| Parameter | Description |
|---|---|
| `[DLY]` | Delay time |
| `[ATT]` | `0` OFF / `1` ON |
| `[HLD]` | System hold time for search with scan (0–255) |
| `[LOUT]` | Lockout for search with scan: `0` / `1` |
| `[QUICK_KEY]` | 0–99 / `.` |
| `[START_KEY]` | 0–9 / `.` |
| `[NUMBER_TAG]` | 0–999 / `NONE` |

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

#### CBP — Get/Set C-Ch Only Custom Search MOT Band Plan *(PM)*

```text
# Controller → Radio
CBP,[SRCH_INDEX]\r   # ① get
# ② set (single line on the wire)
CBP,[SRCH_INDEX],[MOT_TYPE],[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r

# Radio → Controller
# ① (single line on the wire)
CBP,[MOT_TYPE],[LOWER1],[UPPER1],[STEP1],[OFFSET1],[LOWER2],[UPPER2],[STEP2],[OFFSET2],[LOWER3],[UPPER3],[STEP3],[OFFSET3],[LOWER4],[UPPER4],[STEP4],[OFFSET4],[LOWER5],[UPPER5],[STEP5],[OFFSET5],[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r
CBP,OK\r             # ②
```

| Parameter | Description |
|---|---|
| `[SRCH_INDEX]` | 1–9, `0` = 10 |
| `[MOT_TYPE]` | `STD` / `SPL` / `CUSTOM` |
| `[LOWERn]` / `[UPPERn]` | Lower / upper frequency *n* |
| `[STEPn]` | Step code — same table as [MCP](#mcp--getset-motorola-custom-band-plan-pm) |
| `[OFFSETn]` | −1023 … 1023 |

Band plan for MOT 800 custom / VHF / UHF sites when trunking a control channel in custom search. `,`-only set leaves the band plan unchanged; any format error aborts. If `[MOT_TYPE]` is not `CUSTOM`, other settings are ignored.

#### CSP — Get/Set Custom Search Settings *(PM)*

```text
# Controller → Radio
CSP,[SRCH_INDEX]\r   # ① get
# ② set (single line on the wire)
CSP,[SRCH_INDEX],[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[RSV],[HLD],[LOUT],[C-CH],[RSV],[RSV],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV]\r

# Radio → Controller
# ① (single line on the wire)
CSP,[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[RSV],[HLD],[LOUT],[C-CH],[RSV],[RSV],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[RSV],[RSV],[RSV]\r
CSP,OK\r             # ②
```

| Parameter | Description |
|---|---|
| `[SRCH_INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[LIMIT_L]` / `[LIMIT_H]` | Lower / upper limit (250000–13000000) |
| `[STP]` | Search step: `AUTO`, `500` 5k, `625` 6.25k, `750` 7.5k, `833` 8.33k, `1000` 10k, `1250` 12.5k, `1500` 15k, `2000` 20k, `2500` 25k, `5000` 50k, `10000` 100k |
| `[MOD]` | `AUTO` / `AM` / `FM` / `NFM` / `WFM` / `FMB` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | Delay time |
| `[HLD]` | System hold time (0–255) |
| `[LOUT]` | `0` unlocked / `1` lockout |
| `[C-CH]` | Control channel only: `0` OFF / `1` ON |
| `[QUICK_KEY]` | 0–99 / `.` |
| `[START_KEY]` | 0–9 / `.` |
| `[NUMBER_TAG]` | 0–999 / `NONE` |

`,`-only parameters unchanged in set; any format error aborts.

---

### Weather settings

#### WXS — Get/Set Weather Settings *(PM)*

```text
# Controller → Radio
WXS\r                                                  # ① get
WXS,[DLY],[ATT],[ALT_PRI],[RSV],[RSV],[RSV]\r   # ② set
# Radio → Controller
WXS,[DLY],[ATT],[ALT_PRI],[RSV],[RSV],[RSV]\r   # ①
WXS,OK\r                                               # ②
```

| Parameter | Description |
|---|---|
| `[DLY]` | Delay time |
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
| `[FIPS1-8]` | FIPS code, 6 digits (`000000`–`999999`), or `------` = none |

`,`-only parameters unchanged in set; any format error aborts.

---

### Other settings

#### TON — Get/Set Tone-Out Settings *(PM)*

```text
# Controller → Radio
TON,[INDEX]\r     # ① get
# ② set (single line on the wire)
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RSV],[RSV],[ALT_PATTERN],[RSV],[RSV],[RSV]\r
# Radio → Controller
TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RSV],[RSV],[ALT_PATTERN],[RSV],[RSV],[RSV]\r   # ①
TON,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[INDEX]` | 1–9, `0` = 10 |
| `[NAME]` | Name (max. 16 chars) |
| `[FRQ]` | Channel frequency |
| `[MOD]` | `AUTO` / `FM` / `NFM` |
| `[ATT]` | `0` / `1` |
| `[DLY]` | `0`, `1`, `2`, `5`, `10`, `30` / `INF` |
| `[ALT]` | `0` OFF / `1`–`9` |
| `[ALTL]` | `0` AUTO / `1`–`15` |
| `[TONE_A]` / `[TONE_B]` | Tone frequency in 0.1 Hz units (e.g. `10000` = 1000.0 Hz, `00000` = 0.0 Hz) |
| `[ALT_PATTERN]` | `0` OFF / `1` ON / `2` Slow / `3` Fast |
| `[RSV]` | Always only `,` |

#### CNT — Get/Set LCD Contrast Settings *(PM)*

```text
# Controller → Radio
CNT\r                 # ① get
CNT,[CONTRAST]\r      # ② set — 1–15
# Radio → Controller
CNT,[CONTRAST]\r      # ①
CNT,OK\r              # ②
```

#### SCN — Get/Set Scanner Option Settings *(PM)*

```text
# Controller → Radio
SCN\r             # ① get
# ② set (single line on the wire)
SCN,[DISP_MODE],[RSV],[CH_LOG],[G_ATT],[RSV],[RSV],[DISP_UID],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r
# Radio → Controller
SCN,[DISP_MODE],[RSV],[CH_LOG],[G_ATT],[RSV],[RSV],[DISP_UID],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r   # ①
SCN,OK\r          # ②
```

| Parameter | Description |
|---|---|
| `[DISP_MODE]` | Display mode: `1` / `2` / `3` |
| `[CH_LOG]` | Control channel logging: `0` OFF / `1` ON / `2` Extend |
| `[G_ATT]` | Global attenuator: `0` / `1` |
| `[DISP_UID]` | Display unit ID: `0` / `1` |
| `[RSV]` | Always only `,` |

#### VOL — Get/Set Volume Level Settings

```text
# Controller → Radio
VOL\r              # ① get
VOL,[LEVEL]\r      # ② set — 0–15
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

#### P25 — Get/Set APCO Data Settings

```text
# Controller → Radio
P25\r
# Radio → Controller
P25,[RSV],[RSV],[RSV]\r        # all fields reserved
```

#### DBC — Get/Set Default Band Coverage Settings *(PM)*

```text
# Controller → Radio
DBC,[BNAD_NO]\r                 # ① get
DBC,[BNAD_NO],[STEP],[MOD]\r    # ② set
# Radio → Controller
DBC,[STEP],[MOD]\r              # ①
DBC,OK\r                        # ②
```

| Parameter | Description |
|---|---|
| `[BNAD_NO]` | Band number of band coverage (1–27) |
| `[STP]` / `[STEP]` | `500` 5k, `625` 6.25k, `750` 7.5k, `833` 8.33k, `1000` 10k, `1250` 12.5k, `1500` 15k, `2000` 20k, `2500` 25k, `5000` 50k, `10000` 100k |
| `[MOD]` | `AM` / `NFM` / `FM` / `WFM` / `FMB` |

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

#### BSP — Get/Set Band Scope System Settings *(PM)*

```text
# Controller → Radio
BSP\r                                   # ① get
BSP,[FRQ],[STP],[SPN],[MAX_HOLD]\r      # ② set
# Radio → Controller
BSP,[FRQ],[STP],[SPN],[MAX_HOLD]\r      # ①
BSP,OK\r                                # ②
```

| Parameter | Description |
|---|---|
| `[FRQ]` | Center frequency |
| `[STP]` | Search step (same values as DBC) |
| `[SPN]` | Sweep span: `0.2M`, `0.4M`, `0.6M`, `0.8M`, `1M`, `2M`, `4M`, `6M`, `8M`, `10M`, `20M`, `40M`, `60M`, `80M`, `100M`, `120M`, `140M`, `160M`, `180M`, `200M`, `250M`, `300M`, `350M`, `400M`, `450M`, `500M` |
| `[MAX_HOLD]` | Max hold display: `0` OFF / `1` ON |

`,`-only parameters unchanged in set; any format error aborts.

---

### IF exchange list settings

#### GIE — Get Global IF Exchange Frequency *(PM)*

```text
# Controller → Radio
GIE\r
# Radio → Controller
GIE,[FRQ]\r      # one IF exchange frequency (250000–13000000)
GIE,-1\r         # no more
```

Call repeatedly until `-1`.

#### CIE — Clear IF Exchange Frequency *(PM)*

```text
# Controller → Radio
CIE,[FRQ]\r
# Radio → Controller
CIE,OK\r
```

Removes the frequency from the global IF exchange list.

#### RIE — Register IF Exchange Frequency *(PM)*

```text
# Controller → Radio
RIE,[FRQ]\r
# Radio → Controller
RIE,OK\r
```

Adds the frequency to the global IF exchange list.

---

### Test

#### BAV — Get Battery Voltage

```text
# Controller → Radio
BAV\r
# Radio → Controller
BAV,####\r       # A/D value 0–1023
```

Battery level [V] = (3.2 V × `####` × 2) / 1023. Test-mode command.

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

## 7.14 Font Data

The original pages show each glyph as a pixel bitmap. This conversion lists the code points and their labels; the bitmaps themselves are not reproduced. Codes `0x20`–`0x7E` are standard ASCII in both fonts. The relevant part for decoding `STS` output is the non-ASCII range.

### Large font — 8 × 16 dot

Characters in these areas are described as normal characters in this document.

| Code | Glyph / label |
|---|---|
| `0x18`–`0x1F` | Vertical fill bars, increasing height (1/8 → full) |
| `0x20`–`0x7E` | ASCII (`0x7F` blank) |
| `0x80` | ■ solid block |
| `0x81` | ↑ up arrow |
| `0x82` | ↓ down arrow |
| `0x83` | (blank) |
| `0x84`–`0x87` | ARROW (compass-arrow segments) |
| `0x88` | (blank) |
| `0x89`, `0x8A` | ARROW segments |
| `0x8B`, `0x8C` | (blank / block segment) |
| `0x8D`–`0x9F` | ARROW segments (multi-cell direction arrows) |
| `0xA0` | (blank) |
| `0xA1`, `0xA2` | ARROW segments |
| `0xA3` | (blank) |
| `0xA4`, `0xA5` | EAST (2-cell "E") |
| `0xA6`, `0xA7` | NORTH (2-cell "N") |
| `0xA8`, `0xA9` | WEST (2-cell "W") |
| `0xAA`, `0xAB` | SOUTH (2-cell "S") |
| `0xAC`–`0xAF` | ARROW segments (right arrow, bottom) |
| `0xB0`–`0xB3` | (blank) |
| `0xB4`–`0xBA` | ARROW segments |
| `0xBB` | (blank) |
| `0xBC`–`0xCB` | ARROW segments |
| `0xCC` | ° degree sign |
| `0xCD`–`0xCF` | (blank) |
| `0xD0`–`0xD3` | ARROW segments |
| `0xD4` | FUNC indicator (boxed "F") |
| `0xD5`–`0xD7` | BAR R1–R3 (signal bars, right) |
| `0xD8`–`0xDB` | ARROW segments |
| `0xDC` | BAR R4 |
| `0xDD`–`0xDF` | L/O (3-cell "L/O") |
| `0xE0`–`0xE2` | ARROW segments |
| `0xE3` | BAR L1 |
| `0xE4`–`0xE7` | BAR L1 R (L1 + R1…R4) |
| `0xE8` | BAR L3 |
| `0xE9`, `0xEA` | ARROW segments |
| `0xEB` | BAR L2 |
| `0xEC`–`0xEF` | BAR L2 R (L2 + R1…R4) |
| `0xF0`–`0xF3` | ARROW segments (down arrow) |
| `0xF4`–`0xF7` | BAR L3 R (L3 + R1…R4) |
| `0xF8` | BAR L4 |
| `0xF9`–`0xFC` | BAR L4 R (L4 + R1…R4) |
| `0xFD`–`0xFF` | (blank) |

### Small font — 8 × 8 dot

| Code | Glyph / label |
|---|---|
| `0x20`–`0x7E` | ASCII (`0x7F` blank) |
| `0x80` | ■ solid block |
| `0x81` | ↑ |
| `0x82` | ↓ |
| `0x83`, `0x84` | Battery low icon |
| `0x85`, `0x86` | Key pad (lock) icon |
| `0x87`–`0x8A` | Close Call icon |
| `0x8B` | Function icon ("F") |
| `0x8C` | Priority icon ("P") |
| `0x8D`–`0x90` | HOLD (4-cell) |
| `0x91`–`0x94` | Data Skip ("DSKP") |
| `0x95`–`0x97` | Lock Out ("L/O") |
| `0x98`–`0x9A` | AM |
| `0x9B`, `0x9C` | FM |
| `0x9D`, `0x9E` | NFM |
| `0x9F`, `0xA0` | WFM |
| `0xA1`, `0xA2` | Priority ("PRI") |
| `0xA3`–`0xA5` | Attenuate ("ATT") |
| `0xA6`–`0xAD` | Signal level bars |
| `0xAE`–`0xB0` | Active C… (active channel indicator) |
| `0xB1`–`0xB3` | Volume/… (box segments) |
| `0xB4` | Active (indicator) |
| `0xB5`–`0xB8` | CC DND |
| `0xB9`, `0xBA` | FMB |
| `0xBB`, `0xBC` | MUTE |
| `0xBD`–`0xBF` | (marker glyphs) |
| `0xC0` | MARK+C |
| `0xC1`–`0xC4` | SRCH |
| `0xC5`–`0xC7` | IFX |
| `0xC8`–`0xCA` | SCR |
| `0xCB` | (blank) |
| `0xCC` | ° degree sign |
| `0xCD`–`0xCF` | REP |
| `0xD0`–`0xD3` | MAX |
| `0xD4`–`0xD7` | (blank) |

---

## Errata notes (conversion)

These are observations about the source text, not part of the original spec:

- **KEY** — response is given as `KEY,OK\r`, but the examples show bare `OK\r`; Ex.2's last line shows `→ OK` (direction reversed). Verify against hardware.
- **COM** — written with `[/r]` instead of `[\r]`; the get form is `COM,` with a trailing comma.
- **GLG** — last field is printed `[RVS]`, almost certainly `[RSV]`; the empty response is shown with 10 commas in one place and 9 in another.
- **AGV / ABP** — "it just exit for improving interchangeability" means the commands *exist* only for BCD396XT compatibility.
- **ABP** — response is printed `ABP,[[RSV],...[\r] [\r]` (extra bracket, doubled return).
- **CIN / TIN** — responses still contain `[AUDIO_TYPE]`, but this spec never defines it.
- **BLT** — command uses `[EVNT]`, definition uses `[EVENT]`.
- **ACT** — response shows `[INDEX]` but the definition names it `[TGID_INDEX]`.
- **CSP** — `[NUMBER_TAG]` lists `NOE` (typo for `NONE`).
- **CSG** — says `0` = valid, `1` = invalid, then says all ranges can't be set to `0`; one of these is likely inverted.
- **TIN** — response shows `[PRI]]` (extra bracket).
- **CIN** — "Chan0nel" typo in `[REV_INDEX]` description.
- **DBC** — command uses `[STEP]`, definition uses `[STP]`; `[BNAD_NO]` is a typo for BAND_NO.
- **QSC/QSH** — FUNCTION text mentions "UASD" and "STP", neither of which appears in the parameter list.
