# Uniden BCD325P2 Remote Command Reference

Reorganized from Uniden's *BCD325P2 Operation Specification*, §7.13 "Remote Command" and §7.14 "Font Data" (Remote Protocol document ver. 1.02, pp. 209–256).

Everything in sections 1–9 comes from the spec. Section 10 lists errors and inconsistencies found in the spec, and section 11 holds practical notes that are **not** from Uniden. Treat those as suggestions to verify.

## Contents

1. [Serial link settings](#1-serial-link-settings)
2. [General protocol rules](#2-general-protocol-rules)
3. [Common parameter values](#3-common-parameter-values)
4. [Command index](#4-command-index)
5. [Memory model and programming recipes](#5-memory-model-and-programming-recipes)
6. [Commands: remote control, info, program mode](#6-commands-remote-control-info-program-mode)
7. [Commands: settings and memory programming](#7-commands-settings-and-memory-programming)
8. [CTCSS/DCS code list](#8-ctcssdcs-code-list)
9. [Display character set](#9-display-character-set)
10. [Errata found in the spec](#10-errata-found-in-the-spec)
11. [Practical notes (not from the spec)](#11-practical-notes-not-from-the-spec)

### Notation used here

In the format blocks, `→` is controller to radio and `←` is radio to controller. `\r` is a single carriage return (0x0D). Lines starting with `#` inside format blocks are annotations for this reference and are never sent. Field positions are counted with the command name as field 0 when you split a line on commas.

---

## 1. Serial link settings

| Setting | Value |
|---|---|
| Baud rate | 4800, 9600, 19200, 38400, 57600, or 115200 bps |
| Start / stop bits | 1 / 1 |
| Data bits | 8 |
| Parity | None |
| Character code | ASCII |
| Flow control | None |
| Line terminator | Carriage return only (`\r`) |

---

## 2. General protocol rules

1. **One command at a time.** Wait for the scanner's response before sending the next command.
2. **Program Mode.** All memory-access commands work only in Program Mode. Enter with `PRG`, leave with `EPG`. The command index in section 4 marks which commands require it.
3. **Error responses** (not repeated under each command):

   | Response | Meaning |
   |---|---|
   | `ERR\r` | Command format error or value error |
   | `NG\r` | Command is not valid at this time (e.g., wrong mode) |
   | `FER\r` | Framing error |
   | `ORER\r` | Overrun error |

4. **Single line.** Long formats are wrapped in the spec for page width, but every command and response is one line.
5. **Partial sets.** In a set command, any parameter sent as an empty field (just the comma) is left unchanged.
6. **Atomic sets.** A set command is aborted entirely if any format error is detected.
7. **Indexes.** `[INDEX]` / `[xxx_INDEX]` is a handle into the scanner's internal memory chain (a dynamic memory allocation structure). Indexes are used to access records and to trace forward/reverse and up/down links. Valid range is 1 to the maximum memory block count (about 45000). A value of `-1` means "none" or "allocation failed."
8. **Frequency format** (`[FRQ]`, `[BASEx]`, `[LIMIT_x]`): an 8-digit number with no decimal point, from the 1 GHz digit down to the 100 Hz digit. Example: `08510125` = 851.0125 MHz. In other words, the value is the frequency in units of 100 Hz.
9. **TGID format** depends on the trunked system type. The spec refers to a separate appendix that is not included in this document.
10. **Names** (`[NAME]`): if set to only space characters, the name reverts to its default.
11. **Latitude** (`[LATITUDE]`): `DDMMSSssL` in DMS format.
    - `DD` degrees 00–90, `MM` minutes 00–59, `SS` seconds 00–59, `ss` hundredths 00–99 (all zero-padded to two digits)
    - `L` is `N` or `S`
    - Example: 40°42′51.12″ N → `40425112N`
12. **Longitude** (`[LONGITUDE]`): `DDDMMSSssL` in DMS format.
    - `DDD` degrees 000–180 (three digits), then `MM`, `SS`, `ss` as above
    - `L` is `W` or `E`
    - Example: 74°00′23.05″ W → `074002305W`
13. **Protect bit.** When a system's protect bit is on, get commands return most of that system's fields (and its sites, groups, channels, etc.) as empty reserved fields. Link fields such as `REV_INDEX`, `FWD_INDEX`, heads/tails, and parent indexes are still returned. Each command's notes say which fields survive.
14. **Type-specific fields.** Fields that do not apply to a system or site type are returned empty, and are ignored in set commands.

---

## 3. Common parameter values

These parameters appear in many commands with the same meaning. Individual command sections note any exceptions.

| Parameter | Values |
|---|---|
| `[RSV]` | Reserved. Always sent and returned as an empty field (just the comma), except where noted. |
| `[MOD]` | Modulation: `AUTO`, `AM`, `FM`, `NFM`, `WFM`, `FMB` (subset for some commands) |
| `[ATT]` | Attenuator: `0` off, `1` on |
| `[DLY]` | Delay time in seconds: `-10`, `-5`, `-2`, `0`, `1`, `2`, `5`, `10`, `30` (negative = resume early) |
| `[HLD]` | Hold time: `0`–`255` |
| `[LOUT]` | Lockout: `0` unlocked, `1` locked out |
| `[PRI]` | Priority: `0` off, `1` on |
| `[QUICK_KEY]` | Systems, sites, searches: `0`–`99`, or `.` for none. Groups: `1`–`9`, `0` (= 10), or `.` for none |
| `[START_KEY]` | Startup configuration key: `0`–`9`, or `.` for none |
| `[NUMBER_TAG]` | `0`–`999`, or `NONE` |
| `[AGC_ANALOG]` / `[AGC_DIGITAL]` | AGC for analog / digital audio: `0` off, `1` on |
| `[P25WAITING]` | Digital waiting time in ms: `0`, `100`, `200`, … `900`, `1000` |
| `[ALT]` | Alert tone: `0` off, `1`–`9` tone number (location alerts use `1`–`4`) |
| `[ALTL]` | Alert tone level: `0` auto, `1`–`15` |
| `[ALT_COLOR]` | Alert light color: `OFF`, `RED` |
| `[ALT_PATTERN]` | Alert light pattern: `0` on, `1` slow, `2` fast |
| `[VOL_OFFSET]` | Volume offset: `-3` to `+3` |
| `[AUDIO_TYPE]` | `0` all, `1` analog only, `2` digital only |
| `[CTCSS/DCS]` | Code `0`–`239`; see [section 8](#8-ctcssdcs-code-list) |
| `[P25NAC]` | `0`–`FFF` = NAC (hex); `1000`–`100F` = DMR color code 0–15; `SRCH` = NAC/color code search (set); `NONE` = none (status responses) |
| `[RANGE]` | Range for GPS-enabled records: `1`–`250`, in units of 0.5 mile or km |
| `[GPS_ENABLE]` | GPS location detection: `0` off, `1` on |
| `[PROTECT]` | System protect bit: `0` off, `1` on |

### System / site types

Used by `[SYS_TYPE]` (CSY, SIN) and `[SITE_TYPE]` (GID).

| Code | Meaning |
|---|---|
| `CNV` | Conventional |
| `MOT` | Motorola |
| `EDC` | EDACS Narrow / Wide |
| `EDS` | EDACS SCAT |
| `LTR` | LTR |
| `P25S` | P25 Standard (Phase 1 / Phase 2 / X2-TDMA) |
| `P25F` | P25 One Frequency Trunk |
| `TRBO` | MotoTRBO |
| `DMR` | DMR One Frequency Trunk |

### Broadcast Screen bit field (`[BSC]`)

A 16-character string of `0` (off) and `1` (on). Positions, left to right:

| Position | 1 | 2 | 3 | 4 | 5 | 6 | 7–16 |
|---|---|---|---|---|---|---|---|
| Band | Pager | FM | UHF TV | VHF TV | NOAA WX | Reserved | Custom Band 1 … Band 10 |

Custom band limits are set with `BBS`.

### Step codes

Search steps (`CSP`, `DBC`, `BSP`) use these codes. `AUTO` and `750` availability varies by command.

| Code | Step | Code | Step | Code | Step |
|---|---|---|---|---|---|
| `AUTO` | Auto (CSP only) | `833` | 8.33 kHz | `2000` | 20 kHz |
| `500` | 5 kHz | `1000` | 10 kHz | `2500` | 25 kHz |
| `625` | 6.25 kHz | `1250` | 12.5 kHz | `5000` | 50 kHz |
| `750` | 7.5 kHz | `1500` | 15 kHz | `10000` | 100 kHz |

Motorola band-plan steps (`MCP`, `CBP`) are a separate, longer list; see `MCP`.

---

## 4. Command index

"PRG" = only accepted in Program Mode.

| # | Category | Cmd | Function | PRG |
|---|---|---|---|---|
| 1 | Remote control | `GID` | Get current talkgroup ID status | |
| 2 | Remote control | `KEY` | Push key | |
| 3 | Remote control | `POF` | Power off | |
| 4 | Remote control | `QSH` | Go to quick search hold mode | |
| 5 | Remote control | `QSC` | Set current frequency and get reception status | |
| 6 | Remote control | `CSC` | Go to custom search and get reception status | |
| 7 | Remote control | `PWR` | Get RSSI level | |
| 8 | Remote control | `STS` | Get current status (display contents) | |
| 9 | Remote control | `GLG` | Get reception status | |
| 10 | Remote control | `JPM` | Jump mode | |
| 11 | Remote control | `JNT` | Jump to number tag | |
| 12 | Remote control | `MNU` | Menu mode | |
| 13 | System info | `MDL` | Get model info | |
| 14 | System info | `VER` | Get firmware version | |
| 15 | Program control | `PRG` | Enter Program Mode | |
| 16 | Program control | `EPG` | Exit Program Mode | |
| 17 | System settings | `BLT` | Get/set backlight | ✓ |
| 18 | System settings | `BSV` | Get/set battery info | ✓ |
| 19 | System settings | `COM` | Get/set COM port setting | ✓ |
| 20 | System settings | `CLR` | Clear all memory | ✓ |
| 21 | System settings | `KBP` | Get/set key beep and key lock | ✓ |
| 22 | System settings | `OMS` | Get/set opening message | ✓ |
| 23 | System settings | `PRI` | Get/set priority mode | ✓ |
| 24 | System settings | `AGV` | Get/set auto gain control | ✓ |
| 25 | System settings | `SCT` | Get system count | ✓ |
| 26 | Scan settings | `SIH` | Get system index head | ✓ |
| 27 | Scan settings | `SIT` | Get system index tail | ✓ |
| 28 | Scan settings | `QSL` | Get/set system/site quick lockout | ✓ |
| 29 | Scan settings | `QGL` | Get/set group quick lockout | ✓ |
| 30 | Scan settings | `CSY` | Create system | ✓ |
| 31 | Scan settings | `DSY` | Delete system | ✓ |
| 32 | Scan settings | `SIN` | Get/set system info | ✓ |
| 33 | Scan settings | `TRN` | Get/set trunk info | ✓ |
| 34 | Scan settings | `AST` | Append site | ✓ |
| 35 | Scan settings | `SIF` | Get/set site info | ✓ |
| 36 | Scan settings | `MCP` | Get/set Motorola custom band plan | ✓ |
| 37 | Scan settings | `ABP` | Get/set APCO-P25 band plan | ✓ |
| 38 | Scan settings | `TFQ` | Get/set trunk frequency info | ✓ |
| 39 | Scan settings | `AGC` | Append channel group | ✓ |
| 40 | Scan settings | `AGT` | Append talkgroup ID group | ✓ |
| 41 | Scan settings | `DGR` | Delete group / site | ✓ |
| 42 | Scan settings | `GIN` | Get/set group info | ✓ |
| 43 | Scan settings | `ACC` | Append channel / trunk frequency | ✓ |
| 44 | Scan settings | `ACT` | Append talkgroup ID | ✓ |
| 45 | Scan settings | `DCH` | Delete channel | ✓ |
| 46 | Scan settings | `CIN` | Get/set channel info | ✓ |
| 47 | Scan settings | `TIN` | Get/set talkgroup ID info | ✓ |
| 48 | Scan settings | `GLI` | Get lockout TGID (review L/O ID) | ✓ |
| 49 | Scan settings | `SLI` | Get search L/O TGID | ✓ |
| 50 | Scan settings | `ULI` | Unlock TGID | ✓ |
| 51 | Scan settings | `LOI` | Lock out TGID | ✓ |
| 52 | Scan settings | `REV` | Get reverse index | ✓ |
| 53 | Scan settings | `FWD` | Get forward index | ✓ |
| 54 | Scan settings | `RMB` | Get remaining memory blocks | ✓ |
| 55 | Scan settings | `MEM` | Get memory used | ✓ |
| 56 | Location | `LIH` | Get location alert system index head | ✓ |
| 57 | Location | `LIT` | Get location alert system index tail | ✓ |
| 58 | Location | `CLA` | Create location alert system | ✓ |
| 59 | Location | `DLA` | Delete location alert system | ✓ |
| 60 | Location | `LIN` | Get/set location alert system info | ✓ |
| 61 | Search / Close Call | `SCO` | Get/set search/Close Call settings | ✓ |
| 62 | Search / Close Call | `BBS` | Get/set broadcast screen band | ✓ |
| 63 | Search / Close Call | `SHK` | Get/set search key settings | ✓ |
| 64 | Search / Close Call | `GLF` | Get global lockout frequency | ✓ |
| 65 | Search / Close Call | `ULF` | Unlock global L/O frequency | ✓ |
| 66 | Search / Close Call | `LOF` | Lock out frequency | ✓ |
| 67 | Search / Close Call | `CLC` | Get/set Close Call settings | ✓ |
| 68 | Service search | `SSP` | Get/set service search settings | ✓ |
| 69 | Custom search | `CSG` | Get/set custom search group | ✓ |
| 70 | Custom search | `CBP` | Get/set C-Ch only custom search MOT band plan | ✓ |
| 71 | Custom search | `CSP` | Get/set custom search settings | ✓ |
| 72 | Weather | `WXS` | Get/set weather settings | ✓ |
| 73 | Weather | `SGP` | Get/set SAME group settings | ✓ |
| 74 | Tone-out | `TON` | Get/set tone-out settings | ✓ |
| 75 | LCD | `CNT` | Get/set LCD contrast | ✓ |
| 76 | Scanner options | `SCN` | Get/set scanner option settings | ✓ |
| 77 | Volume | `VOL` | Get/set volume level | |
| 78 | Squelch | `SQL` | Get/set squelch level | |
| 79 | APCO data | `P25` | Get/set APCO data settings | |
| 80 | Band coverage | `DBC` | Get/set default band coverage | ✓ |
| 81 | GPS | `GDO` | Get/set GPS display options | ✓ |
| 82 | Band scope | `BSP` | Get/set band scope settings | ✓ |
| 83 | IF exchange | `GIE` | Get global IF exchange frequency | ✓ |
| 84 | IF exchange | `CIE` | Clear IF exchange frequency | ✓ |
| 85 | IF exchange | `RIE` | Register IF exchange frequency | ✓ |
| 86 | Test | `BAV` | Get battery voltage | |
| 87 | Test | `WIN` | Get window voltage | |

---

## 5. Memory model and programming recipes

The spec defines the individual commands; this section pulls together how they link up. All link fields below come from the spec's command definitions.

### Structure

Every record lives in one pool of memory blocks and is addressed by an index. Records of the same kind form doubly linked lists (`REV_INDEX` / `FWD_INDEX`), and each parent stores the head and tail index of its child list.

```text
# System list: head from SIH, tail from SIT, walk with SIN's FWD_INDEX (or FWD)
System (SIN)
│
├── Conventional system (SYS_TYPE = CNV)
│   └── Channel groups      # SIN fields CHN_GRP_HEAD / CHN_GRP_TAIL
│       │                   # each read with GIN (GRP_TYPE = C)
│       └── Channels        # GIN fields CHN_HEAD / CHN_TAIL, each read with CIN
│
└── Trunked system (any other SYS_TYPE)
    ├── Trunk settings      # TRN (same index as the system)
    ├── Sites               # SIN fields CHN_GRP_HEAD / CHN_GRP_TAIL point to sites here
    │   │                   # each read with SIF
    │   ├── Band plans      # MCP (Motorola custom) / ABP (P25), keyed by site index
    │   └── Trunk freqs     # SIF fields CHN_HEAD / CHN_TAIL, each read with TFQ
    ├── TGID groups         # TRN fields TGID_GRP_HEAD / TGID_GRP_TAIL
    │   │                   # each read with GIN (GRP_TYPE = T)
    │   └── TGIDs           # GIN fields CHN_HEAD / CHN_TAIL, each read with TIN
    └── L/O TGID groups     # TRN fields ID_LOUT_GRP_HEAD / ID_LOUT_GRP_TAIL
                            # (lockouts are easier to read with GLI / SLI)

# Location alert systems are a separate list per type:
# LIH,<type> / LIT,<type> for head/tail, LIN to read, FWD/REV to walk
```

`FWD,[INDEX]` and `REV,[INDEX]` work on any record type (system, site, group, channel, TGID, location alert system) and return `-1` at the end of the list. That gives you a generic way to walk any list without parsing the record's own link fields.

### Capacity

From `MEM` and `SCT`:

| Item | Maximum |
|---|---|
| Memory blocks | about 45000 (check free blocks with `RMB`) |
| Systems | 500 |
| Sites | 1000 (256 per system, per `SIF` `SEQ_NO`) |
| Channels | 25000 |
| Location alert systems | 1000 |

### Reading the whole scan database

```text
# Enter Program Mode first; every command below requires it
→ PRG\r
← PRG,OK\r

# Get the first system's index
→ SIH\r
← SIH,[SYS_INDEX]\r                # -1 means no systems stored

# For each system:
→ SIN,[SYS_INDEX]\r                # field 1 = SYS_TYPE, 13 = FWD_INDEX, 14/15 = child head/tail
→ TRN,[SYS_INDEX]\r                # trunked systems only

#   Conventional: walk groups from CHN_GRP_HEAD with GIN, then channels with CIN
#   Trunked: walk sites from CHN_GRP_HEAD with SIF, then frequencies with TFQ;
#            walk TGID groups from TRN's TGID_GRP_HEAD with GIN, then TGIDs with TIN

# Move to the next system using SIN's FWD_INDEX (or FWD,[SYS_INDEX]) until it is -1

# Always leave Program Mode when done
→ EPG\r
← EPG,OK\r
```

### Creating a conventional system with one channel

Index values below are examples. Use whatever the scanner returns.

```text
→ PRG\r                            # enter Program Mode
← PRG,OK\r

→ CSY,CNV,0\r                      # create a conventional system, protect bit off
← CSY,123\r                        # new system index (-1 = out of memory)

# Name the system etc. with SIN,123,... (see the SIN field layout in section 7)

→ AGC,123\r                        # append a channel group to system 123
← AGC,456\r                        # new group index

→ GIN,456,Fireground,1,0,,,,\r     # name the group, quick key 1, unlocked; GPS fields unchanged
← GIN,OK\r

→ ACC,456\r                        # append a channel to group 456
← ACC,789\r                        # new channel index

# Channel 789: "Dispatch", 154.5500 MHz NFM, no tone, all flags off, audio type "all"
→ CIN,789,Dispatch,01545500,NFM,0,0,0,0,0,0,0,,0,,NONE,OFF,0,0\r
← CIN,OK\r

→ EPG\r                            # exit Program Mode
← EPG,OK\r
```

### Creating a trunked system

1. `CSY,[TYPE],[PROTECT]` returns the system index.
2. Set system fields with `SIN` and trunk fields with `TRN`.
3. `AST,[SYS_INDEX],` returns a site index. Configure it with `SIF`, and with `MCP` or `ABP` if a custom band plan is needed (set `SIF` `MOT_TYPE` to `CUSTOM` before using `MCP`).
4. `ACC,[SITE_INDEX]` appends a trunk frequency. Set it with `TFQ`.
5. `AGT,[SYS_INDEX]` appends a TGID group. Set it with `GIN`.
6. `ACT,[GRP_INDEX]` appends a TGID. Set it with `TIN`.

### Deleting

| Record | Command |
|---|---|
| System | `DSY,[SYS_INDEX]` |
| Channel group, TGID group, or site | `DGR,[INDEX]` |
| Channel, TGID, or trunk frequency | `DCH,[INDEX]` |
| Location alert system | `DLA,[INDEX]` |
| Everything (factory reset of memory) | `CLR` |

---

## 6. Commands: remote control, info, program mode

These work outside Program Mode (except where the scanner is in a state that rejects them).

### GID — Get current talkgroup ID status

```text
→ GID\r
← GID,[SITE_TYPE],[TGID],[ID_SRCH_MODE],[NAME1],[NAME2],[NAME3]\r
```

| Field | Meaning |
|---|---|
| `[SITE_TYPE]` | System/site type of the current site (see [section 3](#system--site-types)) |
| `[TGID]` | Talkgroup ID, in the system's TGID format |
| `[ID_SRCH_MODE]` | `0` ID Scan mode, `1` ID Search mode |
| `[NAME1]` | System / site name |
| `[NAME2]` | Group name |
| `[NAME3]` | TGID name |

Returns the TGID currently shown on the LCD. After you read a TGID once, the scanner returns `,,,,,` (empty fields) until the next reception. It also returns empty fields when no TGID is displayed.

### KEY — Push a key

```text
→ KEY,[KEY_CODE],[KEY_MODE]\r
← KEY,OK\r
```

| `[KEY_CODE]` | Key | `[KEY_CODE]` | Key |
|---|---|---|---|
| `M` | Menu | `0`–`9` | Number keys |
| `F` | Func | `.` | ./no/pri |
| `H` | Hold | `E` | E/yes/gps |
| `S` | Scan/srch | `>` | VFO right (mode must be `P`) |
| `L` | L/O | `<` | VFO left (mode must be `P`) |
| `P` | Power/light/lock | `^` | VFO push |

| `[KEY_MODE]` | Meaning |
|---|---|
| `P` | Press |
| `L` | Long press |
| `H` | Hold (press and hold until a release is received) |
| `R` | Release (cancels hold) |

A key hold (`H`) times out 10 seconds after the hold command if there is no further communication.

```text
# Press Menu
→ KEY,M,P\r

# Press and hold L/O (long press)
→ KEY,L,L\r

# F + Scan, as shown in the spec
→ KEY,F,P\r                        # "hold" F
→ KEY,S,P\r                        # press Scan
→ KEY,F,P\r                        # "release" F (see errata: H/R modes may be intended)
```

### POF — Power off

```text
→ POF\r
← POF,OK\r
```

Turns off the scanner. It accepts no further commands afterward.

### QSH — Go to quick search hold mode

```text
→ QSH,[FRQ],[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r
← QSH,OK\r                         # or QSH,NG\r
```

| Field | Meaning |
|---|---|
| `[FRQ]` | Frequency to tune |
| `[MOD]` | `AUTO`, `AM`, `FM`, `NFM`, `WFM`, `FMB` |
| `[CODE_SRCH]` | `0` off, `1` CTCSS/DCS search, `2` P25 NAC / color code search |
| `[BSC]` | Broadcast screen bit field ([section 3](#broadcast-screen-bit-field-bsc)) |
| `[REP]` | Repeater find: `0` off, `1` on |
| Others | See [common parameters](#3-common-parameter-values) |

Switches to Quick Search Hold (VFO) mode on the given frequency. The parameters update the Srch/CloCall options. The command works with only `[FRQ]` set. It is rejected in Menu Mode, during direct entry, and during Quick Save.

### QSC — Set current frequency and get reception status

```text
→ QSC,[FRQ],[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r
← QSC,[RSSI],[FRQ],[SQL]\r         # or QSC,NG\r
```

Same parameters as `QSH`. The response adds `[RSSI]` (A/D value 0–1023) and `[SQL]` (squelch: `0` closed, `1` open). Same mode restrictions as `QSH`.

### CSC — Custom search with streaming status

```text
→ CSC,ON\r
← CSC,[RSSI],[FRQ],[SQL]\r         # repeated, one line per frequency searched
← CSC,[RSSI],[FRQ],[SQL]\r
# ...continues until stopped

→ CSC,OFF\r
← CSC,OK\r
```

Outputs custom search status for each frequency continuously until `CSC,OFF`. Same mode restrictions as `QSH`.

### PWR — Get RSSI level

```text
→ PWR\r
← PWR,[RSSI],[FRQ]\r               # RSSI A/D value 0-1023, current frequency
```

### STS — Get current status (display contents)

```text
→ STS\r
← STS,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE], ... ,[Ln_CHAR],[Ln_MODE],[SQL],[MUT],[BAT],[WAT],[RSV],[RSV],[SIG_LVL],[BK_COLOR],[BK_DIMMER]\r
# n = number of display lines = number of digits in DSP_FORM (4 to 8)
```

| Field | Meaning |
|---|---|
| `[DSP_FORM]` | 4–8 digits, one per display line: `0` small font, `1` large font. Its length tells you how many line pairs follow. |
| `[Lx_CHAR]` | Line text, 16 characters, fixed length (may contain non-ASCII glyph codes; see [section 9](#9-display-character-set)) |
| `[Lx_MODE]` | 16 attribute characters: space = normal, `*` = reverse, `_` = underline. Empty if the whole line is normal. |
| `[SQL]` | Squelch: `0` closed, `1` open |
| `[MUT]` | Mute: `0` off, `1` on |
| `[BAT]` | Battery low: `0` no alert, `1` alert |
| `[WAT]` | Weather alert: `0` none, `1` alert, or the SAME code of the alert |
| `[RSV]` | Reserved (spec says always `0`, but its examples show empty) |
| `[SIG_LVL]` | Signal level `0`–`5` |
| `[BK_COLOR]` | Backlight color: always `RED` |
| `[BK_DIMMER]` | `0` off, `1` low, `2` middle, `3` high |

Example from the spec (menu screen):

```text
← STS,1111, -- M E N U -- ,________________,Program System  ,****************,Program Location,,Srch/CloCall Opt,,1,0,0,0,,,0,RED,\r
# DSP_FORM=1111 -> 4 lines, all large font
# Line 1 underlined, line 2 (the selected item) reversed, lines 3-4 normal
# Squelch open, not muted, no battery or weather alert, signal level 0
```

### GLG — Get reception status

```text
→ GLG\r
← GLG,[FRQ/TGID],[MOD],[ATT],[CTCSS/DCS],[NAME1],[NAME2],[NAME3],[SQL],[MUT],[SYS_TAG],[CHAN_TAG],[P25NAC]\r
```

| Field | Meaning |
|---|---|
| `[FRQ/TGID]` | Frequency or TGID |
| `[MOD]` | `AM`, `FM`, `NFM`, `WFM`, `FMB` |
| `[CTCSS/DCS]` | Detected tone code `0`–`231` (see section 8) |
| `[NAME1]` | System, site, or search name |
| `[NAME2]` | Group name |
| `[NAME3]` | Channel name |
| `[SQL]` / `[MUT]` | Squelch / mute status (as in `STS`) |
| `[SYS_TAG]` / `[CHAN_TAG]` | Current system / channel number tag: `0`–`999` or `NONE` |
| `[P25NAC]` | `0`–`FFF` NAC, `1000`–`100F` color code, `NONE` |

Returns all-empty fields until the scanner detects a frequency or TGID.

### JPM — Jump mode

```text
→ JPM,[JUMP_MODE],[INDEX]\r
← JPM,OK\r                         # NG if the mode cannot be switched now
```

| `[JUMP_MODE]` | Mode | Valid `[INDEX]` |
|---|---|---|
| `SCN_MODE` | Scan mode | Channel index |
| `SVC_MODE` | Service search | `PublicSafety`, `News`, `HAM`, `Marine`, `Railroad`, `Air`, `CB`, `FRS/GMRS/MURS`, `Racing`, `FM`, `Special`, `Military` |
| `CTM_MODE` | Custom search | `RESERVE` |
| `CC_MODE` | Close Call only | `RESERVE` |
| `WX_MODE` | Weather scan | `NORMAL`, `A_ONLY`, `SAME_1` … `SAME_5`, `ALL_FIPS` |
| `FTO_MODE` | Tone-out | `RESERVE` |

### JNT — Jump to number tag

```text
→ JNT,[SYS_TAG],[CHAN_TAG]\r       # each 0-999 or NONE
← JNT,OK\r
```

| `SYS_TAG` | `CHAN_TAG` | Result |
|---|---|---|
| blank | blank | Error |
| blank | set | Jump to that channel tag in the current system |
| set | blank | Jump to the first channel of that system tag |
| set | set | Jump to that system and channel |

### MNU — Menu mode

```text
→ MNU,[MENU_INDEX]\r
← MNU,OK\r                         # NG if the mode cannot be switched now
```

| `[MENU_INDEX]` | Menu |
|---|---|
| `SVC_MENU` | Service search select |
| `WX_MENU` | Weather select |
| `CCBAND_MENU` | Close Call band filter |
| `SCR_OPT_MENU` | Broadcast screen band |
| `GL_LIST_MENU` | Search global lockout list review |
| `SETTING_MENU` | Settings |

### MDL — Get model info

```text
→ MDL\r
← MDL,BCD325P2\r
```

### VER — Get firmware version

```text
→ VER\r
← VER,Version 1.00.00\r            # example; actual text varies by firmware
```

### PRG — Enter Program Mode

```text
→ PRG\r
← PRG,OK\r                         # or PRG,NG\r
```

Rejected in Menu Mode, during direct entry, and during Quick Save. While in Program Mode the scanner shows "Remote Mode" on line 1 and "Keypad Lock" on line 2.

### EPG — Exit Program Mode

```text
→ EPG\r
← EPG,OK\r
```

The scanner leaves Program Mode and goes to Scan Hold mode.

### VOL — Get/set volume level

```text
→ VOL\r                            # get
← VOL,[LEVEL]\r
→ VOL,[LEVEL]\r                    # set, 0-15
← VOL,OK\r
```

### SQL — Get/set squelch level

```text
→ SQL\r                            # get
← SQL,[LEVEL]\r
→ SQL,[LEVEL]\r                    # set: 0 open, 1-14, 15 closed
← SQL,OK\r
```

### P25 — Get APCO data settings

```text
→ P25\r
← P25,[RSV],[RSV],[ERR_RATE]\r     # ERR_RATE = P25 decode error rate, 0-99
```

The spec lists this as get/set but only documents the get form.

### BAV — Get battery voltage (test mode)

```text
→ BAV\r
← BAV,####\r                       # A/D value 0-1023
# Battery voltage [V] = (3.2 * #### * 2) / 1023
```

### WIN — Get window voltage (test mode)

```text
→ WIN\r
← WIN,###,[FRQ]\r                  # A/D value 0-255, current frequency
```

---

## 7. Commands: settings and memory programming

Every command in this section requires Program Mode.

A pattern to know before reading the field layouts: **get responses usually do not echo the index you asked for, and they include read-only link fields** (`REV_INDEX`, `FWD_INDEX`, parent index, child head/tail, `SEQ_NO`) **that are absent from the set form.** You cannot send a get response straight back as a set command. The field-position tables below show both layouts.

### 7.1 System settings

#### BLT — Get/set backlight

```text
→ BLT\r                            # get
← BLT,[EVNT],[RSV],[DIMMER]\r
→ BLT,[EVNT],[RSV],[DIMMER]\r      # set
← BLT,OK\r
```

| Field | Values |
|---|---|
| `[EVNT]` | `IF` infinite, `10` 10 s, `30` 30 s, `KY` keypress, `SQ` squelch |
| `[DIMMER]` | `1` low, `2` middle, `3` high |

#### BSV — Get/set battery info

```text
→ BSV\r                            # get
← BSV,[BAT_SAVE],[CHARGE_TIME]\r
→ BSV,[BAT_SAVE],[CHARGE_TIME]\r   # set
← BSV,OK\r
```

`[BAT_SAVE]`: `0` off, `1` on. `[CHARGE_TIME]`: `1`–`16`.

#### COM — Get/set COM port setting

```text
→ COM\r                            # get
← COM,[BAUDRATE],[RSV]\r
→ COM,[BAUDRATE],[RSV]\r           # set
← COM,OK\r
```

`[BAUDRATE]`: `OFF`, `4800`, `9600`, `19200`, `38400`, `57600`, `115200`.

After receiving `COM,OK`, do not send the next command for 2 seconds. The baud rate is not reset by `CLR`.

#### CLR — Clear all memory

```text
→ CLR\r
← CLR,OK\r
```

Resets all memory to initial settings, except the PC control baud rate. **Takes dozens of seconds**, so use a long read timeout.

#### KBP — Get/set key beep and key lock

```text
→ KBP\r                            # get
← KBP,[LEVEL],[LOCK],[SAFE]\r
→ KBP,[LEVEL],[LOCK],[SAFE]\r      # set
← KBP,OK\r
```

| Field | Values |
|---|---|
| `[LEVEL]` | Beep level: `0` auto, `1`–`15`, `99` off |
| `[LOCK]` | Key lock: `0` off, `1` on |
| `[SAFE]` | Key safe: `0` off, `1` on |

#### OMS — Get/set opening message

```text
→ OMS\r                            # get
← OMS,[L1_CHAR],[L2_CHAR],[L3_CHAR],[L4_CHAR]\r
→ OMS,[L1_CHAR],[L2_CHAR],[L3_CHAR],[L4_CHAR]\r   # set, max 16 chars per line
← OMS,OK\r
```

A line containing only spaces reverts to the default message.

#### PRI — Get/set priority mode

```text
→ PRI\r                            # get
← PRI,[PRI_MODE],[MAX_CHAN],[INTERVAL]\r
→ PRI,[PRI_MODE],[MAX_CHAN],[INTERVAL]\r          # set
← PRI,OK\r
```

| Field | Values |
|---|---|
| `[PRI_MODE]` | `0` off, `1` on, `2` Plus on |
| `[MAX_CHAN]` | Priority channels checked at once: `1`–`100` |
| `[INTERVAL]` | Priority scan interval: `1`–`10` |

#### AGV — Get/set auto gain control

```text
→ AGV\r                            # get
← AGV,[RSV],[RSV],[A_RES],[A_REF],[A_GAIN],[D_RES],[D_GAIN]\r
→ AGV,[RSV],[RSV],[A_RES],[A_REF],[A_GAIN],[D_RES],[D_GAIN]\r   # set
← AGV,OK\r
# The spec prints the last field as [A_GAIN] twice; the table defines it as [D_GAIN]
```

| Field | Values |
|---|---|
| `[A_RES]` | Analog response time: `-4` to `+6` |
| `[A_REF]` | Analog reference gain: `-5` to `+5` |
| `[A_GAIN]` | Analog gain range: `0`–`15` |
| `[D_RES]` | Digital response time: `-8` to `+8` |
| `[D_GAIN]` | Digital reference gain: `-5` to `+5` |

#### SCT — Get system count

```text
→ SCT\r
← SCT,###\r                        # number of stored systems, 0-500
```

### 7.2 System list and quick keys

#### SIH / SIT — Get system index head / tail

```text
→ SIH\r
← SIH,[SYS_INDEX]\r                # first system in the list
→ SIT\r
← SIT,[SYS_INDEX]\r                # last system in the list
```

#### QSL — Get/set system/site quick key lockout

```text
→ QSL\r                            # get
← QSL,[PAGE0],[PAGE1],...,[PAGE9]\r
→ QSL,[PAGE0],[PAGE1],...,[PAGE9]\r                # set
← QSL,OK\r
```

Each `[PAGEn]` is 10 digits, one per quick key, each `0`–`2`:

| Digit | Meaning | Scanner shows |
|---|---|---|
| `0` | Not assigned | `-` |
| `1` | On | The key number |
| `2` | Off | `*` |

Key order within each page matches the LCD icons: `PAGE0` = keys 1–9 then 0; `PAGE1` = 11–19 then 10; `PAGE2` = 21–29 then 20; and so on up to `PAGE9` = 91–99 then 90. You cannot turn on or off a quick key that has no system or site assigned.

#### QGL — Get/set group quick key lockout

```text
→ QGL,[SYS_INDEX]\r                # get
← QGL,##########\r
→ QGL,[SYS_INDEX],##########\r     # set
← QGL,OK\r
```

Same `0`/`1`/`2` digit meanings as `QSL`, for the group quick keys of one system, in order 1–9 then 0. You cannot turn on or off a quick key that has no group.

### 7.3 Systems

#### CSY — Create system

```text
→ CSY,[SYS_TYPE],[PROTECT]\r
← CSY,[SYS_INDEX]\r                # -1 = no memory available
```

`[SYS_TYPE]` is one of the codes in [section 3](#system--site-types). The returned index is the handle for later `SIN` / `TRN` calls.

#### DSY — Delete system

```text
→ DSY,[SYS_INDEX]\r
← DSY,OK\r
```

#### SIN — Get/set system info

```text
→ SIN,[INDEX]\r                    # get
← SIN,[SYS_TYPE],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[RSV],[RSV],[RSV],[RSV],[RSV],[REV_INDEX],[FWD_INDEX],[CHN_GRP_HEAD],[CHN_GRP_TAIL],[SEQ_NO],[START_KEY],[RSV],[RSV],[RSV],[RSV],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING],[PROTECT],[RSV]\r

→ SIN,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[DLY],[RSV],[RSV],[RSV],[RSV],[RSV],[START_KEY],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r   # set
← SIN,OK\r
```

| Field | Values |
|---|---|
| `[SYS_TYPE]` | System type (read-only; set at creation) |
| `[NAME]` | Up to 16 characters |
| `[QUICK_KEY]` | `0`–`99` or `.` |
| `[HLD]` | System hold time `0`–`255` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next system in scan order |
| `[CHN_GRP_HEAD]` / `[CHN_GRP_TAIL]` | Conventional: first/last channel group. Trunked: first/last **site**. |
| `[SEQ_NO]` | System sequence number `1`–`500` |
| `[START_KEY]` | `0`–`9` or `.` |
| `[PROTECT]` | Protect bit (read-only here; set at creation) |

Field positions (field 0 = `SIN`):

| Pos | Get response | Set command |
|---|---|---|
| 1 | SYS_TYPE | INDEX |
| 2 | NAME | NAME |
| 3 | QUICK_KEY | QUICK_KEY |
| 4 | HLD | HLD |
| 5 | LOUT | LOUT |
| 6 | DLY | DLY |
| 7–11 | RSV ×5 | RSV ×5 |
| 12 | REV_INDEX | START_KEY |
| 13 | FWD_INDEX | RSV |
| 14 | CHN_GRP_HEAD | RSV |
| 15 | CHN_GRP_TAIL | RSV |
| 16 | SEQ_NO | RSV |
| 17 | START_KEY | RSV |
| 18 | RSV | RSV |
| 19 | RSV | NUMBER_TAG |
| 20 | RSV | AGC_ANALOG |
| 21 | RSV | AGC_DIGITAL |
| 22 | RSV | P25WAITING |
| 23 | NUMBER_TAG | |
| 24 | AGC_ANALOG | |
| 25 | AGC_DIGITAL | |
| 26 | P25WAITING | |
| 27 | PROTECT | |
| 28 | RSV | |

The set form has six reserved fields after `START_KEY` while the get form has five. One of them is likely a typo, so verify on a throwaway system (see errata).

With the protect bit on, only `SYS_TYPE`, `NAME`, `REV_INDEX`, `FWD_INDEX`, `CHN_GRP_HEAD`, and `CHN_GRP_TAIL` are returned; the rest come back empty.

#### TRN — Get/set trunk info

```text
→ TRN,[INDEX]\r                    # get (INDEX = system index)
← TRN,[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[RSV],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV]x10,[TGID_GRP_HEAD],[TGID_GRP_TAIL],[ID_LOUT_GRP_HEAD],[ID_LOUT_GRP_TAIL],[MOT_ID],[EMG_COLOR],[EMG_PATTERN],[P25NAC],[PRI_ID_SCAN]\r

→ TRN,[INDEX],[ID_SEARCH],[S_BIT],[END_CODE],[AFS],[RSV],[RSV],[EMG],[EMGL],[FMAP],[CTM_FMAP],[RSV]x10,[MOT_ID],[EMG_COLOR],[EMG_PATTERN],[P25NAC],[PRI_ID_SCAN]\r   # set
← TRN,OK\r
# [RSV]x10 = ten consecutive empty fields
```

| Field | Values |
|---|---|
| `[ID_SEARCH]` | `0` ID Scan, `1` ID Search |
| `[S_BIT]` | Motorola status bit: `0` ignore, `1` yes |
| `[END_CODE]` | Motorola end code: `0` ignore, `1` analog, `2` analog and digital |
| `[AFS]` | EDACS ID format: `0` decimal, `1` AFS |
| `[EMG]` | Emergency alert: `0` ignore, `1`–`9` alert tone |
| `[EMGL]` | Emergency alert level: `0` off, `1`–`15` |
| `[FMAP]` | Fleet map: `0`–`15` preset, `16` custom |
| `[CTM_FMAP]` | Custom fleet map: 8 characters, one size code per block 0–7, each `0`–`9` or `A`–`E` (size codes 0–14) |
| `[TGID_GRP_HEAD]` / `[TGID_GRP_TAIL]` | First/last TGID group |
| `[ID_LOUT_GRP_HEAD]` / `[ID_LOUT_GRP_TAIL]` | First/last lockout TGID group |
| `[MOT_ID]` | Motorola/P25 ID format: `0` decimal, `1` hex |
| `[EMG_COLOR]` | `OFF`, `RED` |
| `[EMG_PATTERN]` | `0` on, `1` slow, `2` fast |
| `[P25NAC]` | `0`–`FFF`, `1000`–`100F`, or `SRCH` |
| `[PRI_ID_SCAN]` | Priority ID scan: `0` off, `1` on |

Field positions (field 0 = `TRN`):

| Pos | Get response | Set command |
|---|---|---|
| 1 | ID_SEARCH | INDEX |
| 2 | S_BIT | ID_SEARCH |
| 3 | END_CODE | S_BIT |
| 4 | AFS | END_CODE |
| 5–6 | RSV ×2 | AFS, RSV |
| 7 | EMG | RSV |
| 8 | EMGL | EMG |
| 9 | FMAP | EMGL |
| 10 | CTM_FMAP | FMAP |
| 11 | RSV | CTM_FMAP |
| 12–20 | RSV ×9 | RSV ×9 |
| 21 | TGID_GRP_HEAD | RSV |
| 22 | TGID_GRP_TAIL | MOT_ID |
| 23 | ID_LOUT_GRP_HEAD | EMG_COLOR |
| 24 | ID_LOUT_GRP_TAIL | EMG_PATTERN |
| 25 | MOT_ID | P25NAC |
| 26 | EMG_COLOR | PRI_ID_SCAN |
| 27 | EMG_PATTERN | |
| 28 | P25NAC | |
| 29 | PRI_ID_SCAN | |

With the protect bit on, only the four head/tail fields are returned.

### 7.4 Sites and trunk frequencies

#### AST — Append site

```text
→ AST,[SYS_INDEX],[RSV]\r
← AST,[SITE_INDEX]\r               # -1 = no memory available
```

#### SIF — Get/set site info

```text
→ SIF,[INDEX]\r                    # get (INDEX = site index)
← SIF,[RSV],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[RSV],[RSV],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV],[MOT_TYPE],[EDACS_TYPE],[P25WAITING],[RSV]\r

→ SIF,[INDEX],[NAME],[QUICK_KEY],[HLD],[LOUT],[MOD],[ATT],[C-CH],[RSV],[RSV],[START_KEY],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE],[RSV],[MOT_TYPE],[EDACS_TYPE],[P25WAITING],[RSV]\r   # set
← SIF,OK\r
```

| Field | Values |
|---|---|
| `[NAME]` | Up to 16 characters |
| `[QUICK_KEY]` | `0`–`99` or `.` |
| `[HLD]` | Site hold time `0`–`255` |
| `[MOD]` | `AUTO`, `FM`, `NFM` |
| `[C-CH]` | Control channel only: always `1` (on) |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next site |
| `[SYS_INDEX]` | Parent system |
| `[CHN_HEAD]` / `[CHN_TAIL]` | First/last trunk frequency of the site |
| `[SEQ_NO]` | Site sequence number `1`–`256` |
| `[LATITUDE]` / `[LONGITUDE]` / `[RANGE]` / `[GPS_ENABLE]` | Location-based scanning settings (formats in section 2) |
| `[MOT_TYPE]` | Motorola/EDACS band type: `STD`, `SPL`, `CUSTOM` |
| `[EDACS_TYPE]` | `WIDE`, `NARROW` |

Field positions (field 0 = `SIF`):

| Pos | Get response | Set command |
|---|---|---|
| 1 | RSV | INDEX |
| 2–8 | NAME, QUICK_KEY, HLD, LOUT, MOD, ATT, C-CH | same |
| 9–10 | RSV ×2 | RSV ×2 |
| 11 | REV_INDEX | START_KEY |
| 12 | FWD_INDEX | LATITUDE |
| 13 | SYS_INDEX | LONGITUDE |
| 14 | CHN_HEAD | RANGE |
| 15 | CHN_TAIL | GPS_ENABLE |
| 16 | SEQ_NO | RSV |
| 17 | START_KEY | MOT_TYPE |
| 18 | LATITUDE | EDACS_TYPE |
| 19 | LONGITUDE | P25WAITING |
| 20 | RANGE | RSV |
| 21 | GPS_ENABLE | |
| 22 | RSV | |
| 23 | MOT_TYPE | |
| 24 | EDACS_TYPE | |
| 25 | P25WAITING | |
| 26 | RSV | |

With the protect bit on, only `REV_INDEX`, `FWD_INDEX`, `SYS_INDEX`, `CHN_HEAD`, and `CHN_TAIL` are returned.

#### MCP — Get/set Motorola custom band plan

```text
→ MCP,[INDEX]\r                    # get (INDEX = site index)
← MCP,[LOWER1],[UPPER1],[STEP1],[OFFSET1], ... ,[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r
→ MCP,[INDEX],[LOWER1],[UPPER1],[STEP1],[OFFSET1], ... ,[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r   # set
← MCP,OK\r
# Six band-plan entries, 4 fields each = 24 fields after the index
```

Band plan for Motorola 800 custom / VHF / UHF sites. Set the site's `MOT_TYPE` to `CUSTOM` with `SIF` before using this. Sending only commas leaves the plan unchanged. With the protect bit on, all fields return empty.

| Field | Values |
|---|---|
| `[LOWERn]` / `[UPPERn]` | Frequency limits (8-digit format) |
| `[OFFSETn]` | `-1023` to `1023` |
| `[STEPn]` | Code from the table below |

| Code | Step | Code | Step | Code | Step | Code | Step |
|---|---|---|---|---|---|---|---|
| `500` | 5.0k | `2500` | 25.0k | `4500` | 45.0k | `6875` | 68.75k |
| `625` | 6.25k | `3000` | 30.0k | `5000` | 50.0k | `7000` | 70.0k |
| `1000` | 10.0k | `3125` | 31.25k | `5500` | 55.0k | `7500` | 75.0k |
| `1250` | 12.5k | `3500` | 35.0k | `5625` | 56.25k | `8000` | 80.0k |
| `1500` | 15.0k | `3750` | 37.5k | `6000` | 60.0k | `8125` | 81.25k |
| `1875` | 18.75k | `4000` | 40.0k | `6250` | 62.5k | `8500` | 85.0k |
| `2000` | 20.0k | `4375` | 43.75k | `6500` | 65.0k | `8750` | 87.5k |
| | | | | | | `9000` | 90.0k |
| | | | | | | `9375` | 93.75k |
| | | | | | | `9500` | 95.0k |
| | | | | | | `10000` | 100.0k |

#### ABP — Get/set APCO-P25 band plan

```text
→ ABP,[INDEX]\r                    # get (INDEX = site index)
← ABP,[BASE_FREQ_0],[SPACING_FREQ_0],[BASE_FREQ_1],[SPACING_FREQ_1], ... ,[BASE_FREQ_F],[SPACING_FREQ_F]\r
→ ABP,[INDEX],[BASE_FREQ_0],[SPACING_FREQ_0], ... ,[BASE_FREQ_F],[SPACING_FREQ_F]\r   # set
← ABP,OK\r
# 16 band-plan entries (0-F), 2 fields each = 32 fields after the index
```

Both values are **hexadecimal**:

- `BASE_FREQ_n` = (base frequency in Hz) / 5. Range 25.0000–960.0000 MHz in 5 Hz steps.
- `SPACING_FREQ_n` = (spacing in Hz) / 125. Range 0.125–128.0 kHz in 0.125 kHz steps.

```text
# Spec example: base 851.00625 MHz, spacing 6.25 kHz
# BASE    = 851006250 / 5 = 170201250 = 0xA2510A2
# SPACING = 6250 / 125    = 50        = 0x32
```

An empty band plan entry returns `0`. With the protect bit on, all fields return empty.

#### TFQ — Get/set trunk frequency info

```text
→ TFQ,[CHN_INDEX]\r                # get
← TFQ,[FRQ],[LCN],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[NUMBER_TAG],[VOL_OFFSET],[RSV],[COLOR_CODE]\r

→ TFQ,[CHN_INDEX],[FRQ],[LCN],[LOUT],[RSV],[NUMBER_TAG],[VOL_OFFSET],[RSV],[COLOR_CODE]\r   # set
← TFQ,OK\r
```

| Field | Values |
|---|---|
| `[FRQ]` | Trunk frequency |
| `[LCN]` | EDACS wide/narrow: `1`–`30`; LTR: `1`–`20`; DMR/MotoTRBO: `0`–`4094`. Ignored for Motorola and EDACS SCAT. |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next frequency in the site |
| `[SYS_INDEX]` | Parent system |
| `[GRP_INDEX]` | Parent **site** |
| `[NUMBER_TAG]` / `[VOL_OFFSET]` | Used only for EDACS SCAT systems |
| `[COLOR_CODE]` | `0`–`15`, or `SRCH` |

| Pos | Get response | Set command |
|---|---|---|
| 1 | FRQ | CHN_INDEX |
| 2 | LCN | FRQ |
| 3 | LOUT | LCN |
| 4 | REV_INDEX | LOUT |
| 5 | FWD_INDEX | RSV |
| 6 | SYS_INDEX | NUMBER_TAG |
| 7 | GRP_INDEX | VOL_OFFSET |
| 8 | RSV | RSV |
| 9 | NUMBER_TAG | COLOR_CODE |
| 10 | VOL_OFFSET | |
| 11 | RSV | |
| 12 | COLOR_CODE | |

With the protect bit on, only `REV_INDEX`, `FWD_INDEX`, `SYS_INDEX`, and `GRP_INDEX` are returned.

### 7.5 Groups

#### AGC — Append channel group

```text
→ AGC,[SYS_INDEX]\r
← AGC,[GRP_INDEX]\r                # -1 = no memory available
```

#### AGT — Append TGID group

```text
→ AGT,[SYS_INDEX]\r
← AGT,[GRP_INDEX]\r                # -1 = no memory available
```

#### DGR — Delete group or site

```text
→ DGR,[INDEX]\r                    # channel group, TGID group, or site index
← DGR,OK\r
```

#### GIN — Get/set group info

```text
→ GIN,[GRP_INDEX]\r                # get
← GIN,[GRP_TYPE],[NAME],[QUICK_KEY],[LOUT],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[CHN_HEAD],[CHN_TAIL],[SEQ_NO],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE]\r

→ GIN,[GRP_INDEX],[NAME],[QUICK_KEY],[LOUT],[LATITUDE],[LONGITUDE],[RANGE],[GPS_ENABLE]\r   # set
← GIN,OK\r
```

| Field | Values |
|---|---|
| `[GRP_TYPE]` | `C` channel group, `T` TGID group |
| `[NAME]` | Up to 16 characters |
| `[QUICK_KEY]` | `1`–`9`, `0` (= 10), or `.` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next group in the system |
| `[SYS_INDEX]` | Parent system |
| `[CHN_HEAD]` / `[CHN_TAIL]` | First/last channel or TGID in the group |
| `[SEQ_NO]` | Group sequence number within the system |

| Pos | Get response | Set command |
|---|---|---|
| 1 | GRP_TYPE | GRP_INDEX |
| 2 | NAME | NAME |
| 3 | QUICK_KEY | QUICK_KEY |
| 4 | LOUT | LOUT |
| 5 | REV_INDEX | LATITUDE |
| 6 | FWD_INDEX | LONGITUDE |
| 7 | SYS_INDEX | RANGE |
| 8 | CHN_HEAD | GPS_ENABLE |
| 9 | CHN_TAIL | |
| 10 | SEQ_NO | |
| 11 | LATITUDE | |
| 12 | LONGITUDE | |
| 13 | RANGE | |
| 14 | GPS_ENABLE | |

With the protect bit on, only `NAME`, `REV_INDEX`, `FWD_INDEX`, `SYS_INDEX`, `CHN_HEAD`, and `CHN_TAIL` are returned.

### 7.6 Channels and TGIDs

#### ACC — Append channel or trunk frequency

```text
→ ACC,[GRP_INDEX]\r                # channel group index -> appends a channel
                                   # site index          -> appends a trunk frequency
← ACC,[CHN_INDEX]\r                # -1 = no memory available
```

#### ACT — Append TGID

```text
→ ACT,[GRP_INDEX]\r                # TGID group index
← ACT,[TGID_INDEX]\r               # -1 = no memory available
```

#### DCH — Delete channel, TGID, or trunk frequency

```text
→ DCH,[INDEX]\r
← DCH,OK\r
```

#### CIN — Get/set channel info

```text
→ CIN,[INDEX]\r                    # get
← CIN,[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[AUDIO_TYPE],[P25NAC],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN],[VOL_OFFSET]\r

→ CIN,[INDEX],[NAME],[FRQ],[MOD],[CTCSS/DCS],[TLOCK],[LOUT],[PRI],[ATT],[ALT],[ALTL],[RSV],[AUDIO_TYPE],[P25NAC],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN],[VOL_OFFSET]\r   # set
← CIN,OK\r
```

| Field | Values |
|---|---|
| `[NAME]` | Up to 16 characters |
| `[FRQ]` | Channel frequency (8-digit format) |
| `[MOD]` | `AUTO`, `AM`, `FM`, `NFM`, `WFM`, `FMB` |
| `[CTCSS/DCS]` | `0`–`239` (section 8) |
| `[TLOCK]` | CTCSS/DCS tone lockout: `0` off, `1` on |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next channel in the group |
| `[SYS_INDEX]` / `[GRP_INDEX]` | Parent system / group |
| `[P25NAC]` | `0`–`FFF`, `1000`–`100F`, `SRCH` |
| Others | See [common parameters](#3-common-parameter-values) |

| Pos | Get response | Set command |
|---|---|---|
| 1 | NAME | INDEX |
| 2 | FRQ | NAME |
| 3 | MOD | FRQ |
| 4 | CTCSS/DCS | MOD |
| 5 | TLOCK | CTCSS/DCS |
| 6 | LOUT | TLOCK |
| 7 | PRI | LOUT |
| 8 | ATT | PRI |
| 9 | ALT | ATT |
| 10 | ALTL | ALT |
| 11 | REV_INDEX | ALTL |
| 12 | FWD_INDEX | RSV |
| 13 | SYS_INDEX | AUDIO_TYPE |
| 14 | GRP_INDEX | P25NAC |
| 15 | RSV | NUMBER_TAG |
| 16 | AUDIO_TYPE | ALT_COLOR |
| 17 | P25NAC | ALT_PATTERN |
| 18 | NUMBER_TAG | VOL_OFFSET |
| 19 | ALT_COLOR | |
| 20 | ALT_PATTERN | |
| 21 | VOL_OFFSET | |

With the protect bit on, only the four index fields are returned.

#### TIN — Get/set TGID info

```text
→ TIN,[INDEX]\r                    # get
← TIN,[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SYS_INDEX],[GRP_INDEX],[RSV],[AUDIO_TYPE],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN],[VOL_OFFSET],[TDMA_SLOT]\r

→ TIN,[INDEX],[NAME],[TGID],[LOUT],[PRI],[ALT],[ALTL],[RSV],[AUDIO_TYPE],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN],[VOL_OFFSET],[TDMA_SLOT]\r   # set
← TIN,OK\r
```

| Field | Values |
|---|---|
| `[TGID]` | Talkgroup ID in the system's TGID format |
| `[TDMA_SLOT]` | `ANY`, `1`, `2` |
| Others | As in `CIN` |

| Pos | Get response | Set command |
|---|---|---|
| 1 | NAME | INDEX |
| 2 | TGID | NAME |
| 3 | LOUT | TGID |
| 4 | PRI | LOUT |
| 5 | ALT | PRI |
| 6 | ALTL | ALT |
| 7 | REV_INDEX | ALTL |
| 8 | FWD_INDEX | RSV |
| 9 | SYS_INDEX | AUDIO_TYPE |
| 10 | GRP_INDEX | NUMBER_TAG |
| 11 | RSV | ALT_COLOR |
| 12 | AUDIO_TYPE | ALT_PATTERN |
| 13 | NUMBER_TAG | VOL_OFFSET |
| 14 | ALT_COLOR | TDMA_SLOT |
| 15 | ALT_PATTERN | |
| 16 | VOL_OFFSET | |
| 17 | TDMA_SLOT | |

With the protect bit on, only the four index fields are returned.

### 7.7 TGID lockouts

#### GLI — Get lockout TGID (for Review L/O ID)

```text
→ GLI,[SYS_INDEX]\r
← GLI,[TGID]\r                     # one locked-out TGID per call
← GLI,-1\r                         # no more
```

Call repeatedly until it returns `-1`. With the protect bit on, it returns only `-1`.

#### SLI — Get search lockout TGID

```text
→ SLI,[SYS_INDEX]\r
← SLI,[TGID]\r
← SLI,-1\r                         # no more
```

Like `GLI`, but returns only locked-out TGIDs that do **not** belong to any group in the system. Call repeatedly until `-1`.

#### ULI — Unlock TGID

```text
→ ULI,[SYS_INDEX],[TGID]\r         # removes the TGID from the L/O list
← ULI,OK\r
```

#### LOI — Lock out TGID

```text
→ LOI,[SYS_INDEX],[TGID]\r         # adds the TGID to the L/O list
← LOI,OK\r
```

### 7.8 List navigation and memory status

#### REV / FWD — Get reverse / forward index

```text
→ REV,[INDEX]\r
← REV,[INDEX]\r                    # previous record, -1 if none
→ FWD,[INDEX]\r
← FWD,[INDEX]\r                    # next record, -1 if none
```

Works for any system, site, group, channel, TGID, or location alert system index.

#### RMB — Get remaining memory blocks

```text
→ RMB\r
← RMB,#####\r                      # free blocks, not zero-padded
```

#### MEM — Get memory used

```text
→ MEM\r
← MEM,[MEMORY_USED],[SYS],[SITE],[CHN],[LOC]\r
```

| Field | Range |
|---|---|
| `[MEMORY_USED]` | Percent used, `0`–`100` |
| `[SYS]` | Systems created, `0`–`500` |
| `[SITE]` | Sites created, `0`–`1000` |
| `[CHN]` | Channels created, `0`–`25000` |
| `[LOC]` | Location alert systems created, `0`–`1000` |

### 7.9 Location alert systems

`[LAS_TYPE]` values: `POI` (point of interest), `DROAD` (dangerous road), `DXING` (dangerous crossing).

#### LIH / LIT — Get location alert system index head / tail

```text
→ LIH,[LAS_TYPE]\r
← LIH,[INDEX]\r                    # first location alert system of this type
→ LIT,[LAS_TYPE]\r
← LIT,[INDEX]\r                    # last location alert system of this type
```

#### CLA — Create location alert system

```text
→ CLA,[LAS_TYPE]\r
← CLA,[INDEX]\r                    # -1 = no memory available
```

#### DLA — Delete location alert system

```text
→ DLA,[INDEX]\r
← DLA,OK\r
```

#### LIN — Get/set location alert system info

```text
→ LIN,[INDEX]\r                    # get
← LIN,[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[REV_INDEX],[FWD_INDEX],[SEQ_NO],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR],[ALT_COLOR],[ALT_PATTERN]\r

→ LIN,[INDEX],[LAS_TYPE],[NAME],[LOUT],[ALT],[ALTL],[LATITUDE],[LONGITUDE],[RANGE],[SPEED],[DIR],[ALT_COLOR],[ALT_PATTERN]\r   # set
← LIN,OK\r
```

| Field | Values |
|---|---|
| `[NAME]` | Up to 16 characters |
| `[ALT]` | `0` off, `1`–`4` tone number |
| `[ALTL]` | `0` auto, `1`–`15` |
| `[REV_INDEX]` / `[FWD_INDEX]` | Previous / next location alert system |
| `[SEQ_NO]` | Sequence number |
| `[RANGE]` | `1`–`80`, in units of **0.05** mile or km (differs from the 0.5 unit used elsewhere) |
| `[SPEED]` | Speed limit `0`–`200` mph or km/h |
| `[DIR]` | Heading in degrees; `360` = all directions. Spec examples: `0` N, `44` NE, `90` E, `134` SE, `180` S, `224` SW, `270` W, `314` NW |

### 7.10 Search and Close Call

#### SCO — Get/set search / Close Call settings

```text
→ SCO\r                            # get
← SCO,[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE],[RSV],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r
→ SCO,[RSV],[MOD],[ATT],[DLY],[RSV],[CODE_SRCH],[BSC],[REP],[RSV],[RSV],[MAX_STORE],[RSV],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r   # set
← SCO,OK\r
```

`[CODE_SRCH]`: `0` off, `1` CTCSS/DCS, `2` P25 NAC / color code. `[REP]`: repeater find `0`/`1`. `[MAX_STORE]`: max auto store `1`–`256`. `[BSC]`: see section 3.

#### BBS — Get/set broadcast screen band

```text
→ BBS,[INDEX]\r                    # get, INDEX 1-9 or 0 (= 10)
← BBS,[LIMIT_L],[LIMIT_H]\r
→ BBS,[INDEX],[LIMIT_L],[LIMIT_H]\r                # set, limits 00000000-99999999
← BBS,OK\r
```

Defines the custom bands 1–10 referenced by the `[BSC]` bit field.

#### SHK — Get/set search key settings

```text
→ SHK\r                            # get
← SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[RSV],[RSV],[RSV]\r
→ SHK,[SRCH_KEY_1],[SRCH_KEY_2],[SRCH_KEY_3],[RSV],[RSV],[RSV]\r   # set
← SHK,OK\r
```

Values for each search key: `.` (not assigned), `PublicSafety`, `News`, `HAM`, `Marine`, `Railroad`, `Air`, `CB`, `FRS/GMRS/MURS`, `Racing`, `FM`, `Special`, `Military`, `CUSTOM_1` … `CUSTOM_10`, `TONE_OUT`, `B_SCOPE`.

#### GLF — Get global lockout frequency

```text
→ GLF\r
← GLF,[FRQ]\r                      # one locked-out frequency per call
← GLF,-1\r                         # no more
```

Call repeatedly until `-1`. Frequency range given as `250000`–`9600000` (25–960 MHz).

#### ULF — Unlock global lockout frequency

```text
→ ULF,[FRQ]\r                      # removes it from the L/O list
← ULF,OK\r
```

#### LOF — Lock out frequency

```text
→ LOF,[FRQ]\r                      # adds it to the L/O list
← LOF,OK\r
```

#### CLC — Get/set Close Call settings

```text
→ CLC\r                            # get
← CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN]\r
→ CLC,[CC_MODE],[CC_OVERRIDE],[RSV],[ALTB],[ALTL],[ALTP],[CC_BAND],[LOUT],[HLD],[QUICK_KEY],[NUMBER_TAG],[ALT_COLOR],[ALT_PATTERN]\r   # set
← CLC,OK\r
```

| Field | Values |
|---|---|
| `[CC_MODE]` | `0` off, `1` CC priority, `2` CC do-not-disturb |
| `[CC_OVERRIDE]` | `0` off, `1` on |
| `[ALTB]` | Alert beep: `0` off, `1`–`9` |
| `[ALTL]` | `0` auto, `1`–`15` |
| `[ALTP]` | Close Call pause: `3`, `5`, `10`, `15`, `30`, `45`, `60` seconds, or `INF` |
| `[CC_BAND]` | 7 digits, `0`/`1`, left to right: VHF Low1, VHF Low2, Air, VHF High1, VHF High2, UHF, 800 MHz+ |
| `[LOUT]` / `[HLD]` / `[QUICK_KEY]` | Lockout, hold time, quick key for CC hits with scan |

### 7.11 Service and custom search

#### SSP — Get/set service search settings

```text
→ SSP,[SRCH_INDEX]\r               # get
← SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r
→ SSP,[SRCH_INDEX],[DLY],[ATT],[HLD],[LOUT],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r   # set
← SSP,OK\r
# Unlike most get commands, this response echoes the index
```

| `[SRCH_INDEX]` | Range | `[SRCH_INDEX]` | Range |
|---|---|---|---|
| `1` | Public Safety | `7` | CB Radio |
| `2` | News | `8` | FRS/GMRS/MURS |
| `3` | HAM Radio | `9` | Racing |
| `4` | Marine | `11` | FM Broadcast |
| `5` | Railroad | `12` | Special |
| `6` | Air | `15` | Military Air |

#### CSG — Get/set custom search group

```text
→ CSG\r                            # get
← CSG,##########\r
→ CSG,##########\r                 # set
← CSG,OK\r
```

10 digits for custom search ranges 1–10 in LCD icon order: `0` valid (enabled), `1` invalid (disabled). The spec states you cannot set all ranges to `0`; this may be a typo for `1` (disabling all), so test before relying on it.

#### CBP — Get/set control-channel-only custom search MOT band plan

```text
→ CBP,[SRCH_INDEX]\r               # get, INDEX 1-9 or 0 (= 10)
← CBP,[MOT_TYPE],[LOWER1],[UPPER1],[STEP1],[OFFSET1], ... ,[LOWER6],[UPPER6],[STEP6],[OFFSET6]\r
→ CBP,[SRCH_INDEX],[MOT_TYPE],[LOWER1],[UPPER1],[STEP1],[OFFSET1], ... ,[OFFSET6]\r   # set
← CBP,OK\r
```

Band plan used when custom search is trunking a control channel. `[MOT_TYPE]`: `STD`, `SPL`, `CUSTOM`. If it is not `CUSTOM`, the remaining fields are ignored. Step codes and offsets as in `MCP`.

#### CSP — Get/set custom search settings

```text
→ CSP,[SRCH_INDEX]\r               # get, INDEX 1-9 or 0 (= 10)
← CSP,[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[RSV],[HLD],[LOUT],[C-CH],[RSV],[RSV],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r
→ CSP,[SRCH_INDEX],[NAME],[LIMIT_L],[LIMIT_H],[STP],[MOD],[ATT],[DLY],[RSV],[HLD],[LOUT],[C-CH],[RSV],[RSV],[QUICK_KEY],[START_KEY],[RSV],[NUMBER_TAG],[AGC_ANALOG],[AGC_DIGITAL],[P25WAITING]\r   # set
← CSP,OK\r
```

| Field | Values |
|---|---|
| `[NAME]` | Up to 16 characters |
| `[LIMIT_L]` / `[LIMIT_H]` | `250000`–`9600000` (25–960 MHz) |
| `[STP]` | `AUTO`, `500`, `625`, `750`, `833`, `1000`, `1250`, `1500`, `2000`, `2500`, `5000`, `10000` |
| `[MOD]` | `AUTO`, `AM`, `FM`, `NFM`, `WFM`, `FMB` |
| `[C-CH]` | Control channel only: `0` off, `1` on |

### 7.12 Weather and tone-out

#### WXS — Get/set weather settings

```text
→ WXS\r                            # get
← WXS,[DLY],[ATT],[ALT_PRI],[RSV],[AGC_ANALOG],[RSV]\r
→ WXS,[DLY],[ATT],[ALT_PRI],[RSV],[AGC_ANALOG],[RSV]\r   # set
← WXS,OK\r
```

`[ALT_PRI]`: weather alert priority, `0` off, `1` on.

#### SGP — Get/set SAME group settings

```text
→ SGP,[SAME_INDEX]\r               # get, INDEX 1-5
← SGP,[NAME],[FIPS1],[FIPS2],...,[FIPS8]\r
→ SGP,[SAME_INDEX],[NAME],[FIPS1],...,[FIPS8]\r          # set
← SGP,OK\r
```

`[NAME]` up to 16 characters. Each `[FIPSn]` is a 6-digit code `000000`–`999999`, or `------` for none.

#### TON — Get/set tone-out settings

```text
→ TON,[INDEX]\r                    # get, INDEX 1-9 or 0 (= 10)
← TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RSV],[ALT_COLOR],[ALT_PATTERN],[AGC_ANALOG],[RSV],[RSV]\r
→ TON,[INDEX],[NAME],[FRQ],[MOD],[ATT],[DLY],[ALT],[ALTL],[TONE_A],[RSV],[TONE_B],[RSV],[RSV],[RSV],[ALT_COLOR],[ALT_PATTERN],[AGC_ANALOG],[RSV],[RSV]\r   # set
← TON,OK\r
# Response echoes the index
```

| Field | Values |
|---|---|
| `[MOD]` | `AUTO`, `FM`, `NFM` |
| `[DLY]` | `0`, `1`, `2`, `5`, `10`, `30`, or `INF` (no negative values here) |
| `[TONE_A]` / `[TONE_B]` | Tone frequency in 0.1 Hz units, 5 digits. `10000` = 1000.0 Hz, `00000` = 0.0 Hz |

### 7.13 Display, options, and other settings

#### CNT — Get/set LCD contrast

```text
→ CNT\r                            # get
← CNT,[CONTRAST]\r
→ CNT,[CONTRAST]\r                 # set, 1-15
← CNT,OK\r
```

#### SCN — Get/set scanner option settings

```text
→ SCN\r                            # get
← SCN,[DISP_MODE],[RSV],[CH_LOG],[G_ATT],[RSV],[P25_LPF],[DISP_UID],[RSV]x14\r
→ SCN,[DISP_MODE],[RSV],[CH_LOG],[G_ATT],[RSV],[P25_LPF],[DISP_UID],[RSV]x14\r   # set
← SCN,OK\r
# [RSV]x14 = fourteen trailing empty fields
```

| Field | Values |
|---|---|
| `[DISP_MODE]` | `1`, `2`, or `3` (display mode 1–3) |
| `[CH_LOG]` | Control channel logging: `0` off, `1` on, `2` extended |
| `[G_ATT]` | Global attenuator: `0` off, `1` on |
| `[P25_LPF]` | P25 low-pass filter: `0` off, `1` on |
| `[DISP_UID]` | Display unit ID: `0` off, `1` on |

#### DBC — Get/set default band coverage

```text
→ DBC,[BAND_NO]\r                  # get, band 1-31
← DBC,[STEP],[MOD]\r
→ DBC,[BAND_NO],[STEP],[MOD]\r     # set
← DBC,OK\r
```

`[STEP]`: `500`, `625`, `750`, `833`, `1000`, `1250`, `1500`, `2000`, `2500`, `5000`, `10000`. `[MOD]`: `AM`, `NFM`, `FM`, `WFM`, `FMB`. The spec does not list which frequency range each band number covers.

#### GDO — Get/set GPS display options

```text
→ GDO\r                            # get
← GDO,[DISP_MODE],[UNIT],[TIME_FORMAT],[TIME_ZONE],[POS_FORMAT]\r
→ GDO,[DISP_MODE],[UNIT],[TIME_FORMAT],[TIME_ZONE],[POS_FORMAT]\r   # set
← GDO,OK\r
```

| Field | Values |
|---|---|
| `[DISP_MODE]` | `0` ETA, `1` clock, `2` elevation, `3` speed, `4` location |
| `[UNIT]` | `0` mile, `1` km |
| `[TIME_FORMAT]` | `0` 12-hour, `1` 24-hour |
| `[TIME_ZONE]` | `-14.0` to `14.0` in 0.5 h steps, e.g. `-5.0` |
| `[POS_FORMAT]` | `DMS` or `DEG` |

#### BSP — Get/set band scope settings

```text
→ BSP\r                            # get
← BSP,[FRQ],[STP],[SPN],[MAX_HOLD]\r
→ BSP,[FRQ],[STP],[SPN],[MAX_HOLD]\r                # set
← BSP,OK\r
```

| Field | Values |
|---|---|
| `[FRQ]` | Center frequency |
| `[STP]` | `500`, `625`, `750`, `833`, `1000`, `1250`, `1500`, `2000`, `2500`, `5000`, `10000` |
| `[SPN]` | Sweep span: `0.2M`, `0.4M`, `0.6M`, `0.8M`, `1M`, `2M`, `4M`, `6M`, `8M`, `10M`, `20M`, `40M`, `60M`, `80M`, `100M`, `120M`, `140M`, `160M`, `180M`, `200M`, `250M`, `300M`, `350M`, `400M`, `450M`, `500M` |
| `[MAX_HOLD]` | Max hold display: `0` off, `1` on |

### 7.14 IF exchange list

#### GIE — Get global IF exchange frequency

```text
→ GIE\r
← GIE,[FRQ]\r                      # one frequency per call
← GIE,-1\r                         # no more
```

Call repeatedly until `-1`. Frequency range `250000`–`9600000`.

#### CIE — Clear IF exchange frequency

```text
→ CIE,[FRQ]\r                      # removes the frequency from the list
← CIE,OK\r
```

#### RIE — Register IF exchange frequency

```text
→ RIE,[FRQ]\r                      # adds the frequency to the list
← RIE,OK\r
```

---

## 8. CTCSS/DCS code list

Used by `[CTCSS/DCS]` in `CIN` and `GLG`.

| Code | Meaning |
|---|---|
| `0` | None / all (no tone squelch) |
| `127` | Search |
| `64`–`113` | CTCSS tones (below) |
| `128`–`239` | DCS codes (below) |

### CTCSS (codes 64–113)

| Code | Tone | Code | Tone | Code | Tone | Code | Tone | Code | Tone |
|---|---|---|---|---|---|---|---|---|---|
| 64 | 67.0 | 74 | 94.8 | 84 | 131.8 | 94 | 171.3 | 104 | 203.5 |
| 65 | 69.3 | 75 | 97.4 | 85 | 136.5 | 95 | 173.8 | 105 | 206.5 |
| 66 | 71.9 | 76 | 100.0 | 86 | 141.3 | 96 | 177.3 | 106 | 210.7 |
| 67 | 74.4 | 77 | 103.5 | 87 | 146.2 | 97 | 179.9 | 107 | 218.1 |
| 68 | 77.0 | 78 | 107.2 | 88 | 151.4 | 98 | 183.5 | 108 | 225.7 |
| 69 | 79.7 | 79 | 110.9 | 89 | 156.7 | 99 | 186.2 | 109 | 229.1 |
| 70 | 82.5 | 80 | 114.8 | 90 | 159.8 | 100 | 189.9 | 110 | 233.6 |
| 71 | 85.4 | 81 | 118.8 | 91 | 162.2 | 101 | 192.8 | 111 | 241.8 |
| 72 | 88.5 | 82 | 123.0 | 92 | 165.5 | 102 | 196.6 | 112 | 250.3 |
| 73 | 91.5 | 83 | 127.3 | 93 | 167.9 | 103 | 199.5 | 113 | 254.1 |

All tones in Hz.

### DCS (codes 128–239)

| Code | DCS | Code | DCS | Code | DCS | Code | DCS |
|---|---|---|---|---|---|---|---|
| 128 | 023 | 156 | 156 | 184 | 332 | 212 | 532 |
| 129 | 025 | 157 | 162 | 185 | 343 | 213 | 546 |
| 130 | 026 | 158 | 165 | 186 | 346 | 214 | 565 |
| 131 | 031 | 159 | 172 | 187 | 351 | 215 | 606 |
| 132 | 032 | 160 | 174 | 188 | 356 | 216 | 612 |
| 133 | 036 | 161 | 205 | 189 | 364 | 217 | 624 |
| 134 | 043 | 162 | 212 | 190 | 365 | 218 | 627 |
| 135 | 047 | 163 | 223 | 191 | 371 | 219 | 631 |
| 136 | 051 | 164 | 225 | 192 | 411 | 220 | 632 |
| 137 | 053 | 165 | 226 | 193 | 412 | 221 | 654 |
| 138 | 054 | 166 | 243 | 194 | 413 | 222 | 662 |
| 139 | 065 | 167 | 244 | 195 | 423 | 223 | 664 |
| 140 | 071 | 168 | 245 | 196 | 431 | 224 | 703 |
| 141 | 072 | 169 | 246 | 197 | 432 | 225 | 712 |
| 142 | 073 | 170 | 251 | 198 | 445 | 226 | 723 |
| 143 | 074 | 171 | 252 | 199 | 446 | 227 | 731 |
| 144 | 114 | 172 | 255 | 200 | 452 | 228 | 732 |
| 145 | 115 | 173 | 261 | 201 | 454 | 229 | 734 |
| 146 | 116 | 174 | 263 | 202 | 455 | 230 | 743 |
| 147 | 122 | 175 | 265 | 203 | 462 | 231 | 754 |
| 148 | 125 | 176 | 266 | 204 | 464 | 232 | 006 |
| 149 | 131 | 177 | 271 | 205 | 465 | 233 | 007 |
| 150 | 132 | 178 | 274 | 206 | 466 | 234 | 015 |
| 151 | 134 | 179 | 306 | 207 | 503 | 235 | 017 |
| 152 | 143 | 180 | 311 | 208 | 506 | 236 | 021 |
| 153 | 145 | 181 | 315 | 209 | 516 | 237 | 050 |
| 154 | 152 | 182 | 325 | 210 | 523 | 238 | 141 |
| 155 | 155 | 183 | 331 | 211 | 526 | 239 | 214 |

Codes 128–231 run in numeric DCS order. Codes 232–239 (DCS 006, 007, 015, 017, 021, 050, 141, 214) were appended out of order, so don't compute DCS codes arithmetically.

---

## 9. Display character set

`STS` returns the raw characters on the LCD, and the scanner uses byte values above 0x7F for icons and graphics. Decode `STS` output as single bytes (e.g., Latin-1), not UTF-8, or these bytes will cause decode errors.

There are two fonts: large (8×16 dots) and small (8×8 dots). The `[DSP_FORM]` digit for each line tells you which font that line uses. 0x20–0x7E are standard ASCII in both fonts.

### Large font (8×16) special codes

| Code(s) | Glyph |
|---|---|
| 0x18–0x1F | Vertical bar graph, 1/8 to full height (eight levels) |
| 0x80 | Solid block |
| 0x81 / 0x82 | Up arrow / down arrow |
| Other codes 0x84–0xF3 not listed here | Pieces of large multi-cell direction arrows (used for GPS heading display), except the blank codes in the last row |
| 0xA4–0xA5 | "E" (EAST) |
| 0xA6–0xA7 | "N" (NORTH) |
| 0xA8–0xA9 | "W" (WEST) |
| 0xAA–0xAB | "S" (SOUTH) |
| 0xCC | Degree sign |
| 0xD4 | FUNC indicator |
| 0xD5–0xD7, 0xDC | Signal bars, right side, levels R1–R4 |
| 0xDD–0xDF | "L/O" (lockout), three cells |
| 0xE3, 0xEB, 0xE8, 0xF8 | Signal bars, left side only, levels L1–L4 |
| 0xE4–0xE7, 0xEC–0xEF, 0xF4–0xF7, 0xF9–0xFC | Combined left+right signal bars (left L1–L4, each with right R1–R4) |
| 0x83, 0x88, 0x8B, 0x8C, 0xA0, 0xA3, 0xB0–0xB3, 0xBB, 0xCD–0xCF, 0xFD–0xFF | Blank / unused |

### Small font (8×8) special codes

| Code(s) | Glyph |
|---|---|
| 0x80 | Solid block |
| 0x81 / 0x82 | Up arrow / down arrow |
| 0x83–0x84 | Battery level icon |
| 0x85–0x86 | Keypad (lock) icon |
| 0x87–0x8A | Close Call icon |
| 0x8B | Function icon |
| 0x8C | Priority icon |
| 0x8D–0x90 | "HOLD" |
| 0x91–0x94 | "DSKP" (data skip) |
| 0x95–0x97 | "L/O" (lockout) |
| 0x98–0x9A | "AM" |
| 0x9B–0x9C | "FM" |
| 0x9D–0x9E | "NFM" |
| 0x9F–0xA0 | "WFM" |
| 0xA1–0xA2 | "PRI" (priority) |
| 0xA3–0xA5 | "ATT" (attenuator) |
| 0xA6–0xAD | Signal level bars |
| 0xAE–0xB0, 0xB4 | Active channel indicators |
| 0xB1–0xB3 | Volume / squelch bar frame |
| 0xB5–0xB8 | Close Call DND icon |
| 0xB9–0xBA | "FMB" |
| 0xBB–0xBC | "MUTE" |
| 0xBD–0xBF | Small marker glyphs |
| 0xC0 | Marker + C |
| 0xC1–0xC4 | "SRCH" |
| 0xC5–0xC7 | "IFX" (IF exchange) |
| 0xC8–0xCA | "SCR" (broadcast screen) |
| 0xCC | Degree sign |
| 0xCD–0xCF | "REP" (repeater find) |
| 0xD0–0xD3 | "MAX" |
| 0xD4–0xD6 | "NAC" |
| 0xCB, 0xD7 | Blank |

Multi-cell labels like "HOLD" span consecutive codes and must be read together. The spec shows these only as pixel images; the labels above are taken from the image captions and glyph shapes.

---

## 10. Errata found in the spec

These are inconsistencies in the source document. Confirm actual behavior against your scanner and firmware.

| Command | Issue |
|---|---|
| General | Note 3 says error responses are `ERR`, `NG`, etc., but some command definitions show `XXX,NG` (e.g., `QSH,NG`, `PRG,NG`). Handle both forms. |
| `KEY` | Format says response is `KEY,OK`, but examples show bare `OK`. The F + Scan example "holds" and "releases" F with `P` (press) both times, and shows the final response with the wrong arrow direction. `H` / `R` modes are probably intended. |
| `STS` | Table says `[RSV]` is always `0`; examples show it empty. Example responses also omit the leading `STS,`. |
| `GLG` | The "empty" response is printed with two different comma counts (10 and 9). Parse by splitting rather than expecting a fixed string. `[CTCSS/DCS]` range is given as 0–231, but the code list goes to 239. |
| `BLT` | Format uses `[EVNT]`, table uses `[EVENT]`. Same field. |
| `COM` | Uses `[/r]` instead of `[\r]`. |
| `AGV` | Format lists `[A_GAIN]` twice; the last field is `[D_GAIN]` per the table. |
| `SIN` | Set form has six `[RSV]` fields between `START_KEY` and `NUMBER_TAG`; get form has five. One is likely wrong. |
| `TFQ` | Get response shows `[RSV][COLOR_CODE]` with no comma between them; almost certainly a missing comma. |
| `ACT` | Response shown as `ACT,[INDEX]` but described as `[TGID_INDEX]`. Same value. |
| `CSP` | Get response shows `[START_KEY][RSV]` with no comma between them. |
| `TIN` | Typo `[PRI]]` in the get response. |
| `CIN` | Typo "Chan0nel" in the `REV_INDEX` description. |
| `DBC` | Uses `[BNAD_NO]` (typo for band number) and `[STP]` in the table vs `[STEP]` in the format. |
| `P25` | Listed as get/set but only the get form is documented. |
| `CSG` | Says you cannot set all ranges to `0`, but `0` means valid, so `1` (all disabled) is probably meant. |
| Frequency fields | Most frequencies use the 8-digit format (`08510125`), but `GLF`, `ULF`, `LOF`, `CSP`, `GIE`, `CIE`, `RIE` give ranges as `250000`–`9600000`. Same units (100 Hz); it is unclear whether leading zeros are required there. |
| `QSH` / `QSC` | Description mentions an "STP" parameter that does not exist in the format. |
| TGID format | Referenced appendix is not included in this document. |

---

## 11. Practical notes (not from the spec)

These are suggestions based on how the protocol is described, not Uniden documentation.

- **Device node.** Over USB the scanner normally appears as a CDC-ACM serial device (`/dev/ttyACM0`). Your user needs access (typically membership in the `dialout` group). The baud setting matters little over USB, but set 115200 to match.
- **ModemManager.** It may probe new `ttyACM` devices with AT commands. Stop it or add a udev rule with `ENV{ID_MM_DEVICE_IGNORE}="1"` for the scanner.
- **Always exit Program Mode.** Wrap programming sessions so `EPG` runs even on errors, or the scanner stays keypad-locked in "Remote Mode."
- **Read before write.** Since get and set layouts differ, a safe edit pattern is: read with get, map fields into the set layout by name, change what you need, and leave everything else as empty fields so the scanner keeps the existing values (rule 5 in section 2).
- **Verify suspicious layouts first.** Test `SIN` (the RSV count) and `TFQ` on a scratch system created with `CSY` before writing to your real programming, then delete it with `DSY`.
- **Back up first.** Dump the full database (section 5 recipe) to a file before any bulk write. `CLR` wipes everything and is slow.
- **Timeouts.** Use a generous read timeout for `CLR` (tens of seconds) and wait 2 seconds after `COM,OK`.
- **Field counting.** Split responses on commas and index by position. Names can't contain commas in this protocol, so avoid them when setting names.
- **Gaps the capture can fill.** The TGID format appendix and the `DBC` band list are missing here. Capturing what Sentinel sends while you edit a trunked system, or reading those values back with get commands, is the quickest way to fill them in.
