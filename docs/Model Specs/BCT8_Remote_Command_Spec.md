# BCT8 Operation Specification — Remote Command

> Markdown conversion of Uniden's *BCT8 Operation Specification, Appendix B — Remote Command*, version 2.00 (51 pages; file name "V201").
> The BCT8 is a highway-patrol-oriented TrunkTracker and uses Uniden's oldest command style: two-letter commands, values packed into fixed-width fields separated by spaces (not commas), and `N`/`F` suffixes for ON/OFF. Banks and ID scan lists are lettered `A`–`E`.
> Wire formats are reproduced as written in the source. The source writes the return code as `[¥r]` (the Japanese-locale backslash); it is shown here as `\r`. Apparent typos in the original are listed under [Errata notes](#errata-notes-conversion) at the end.

---

## Contents

- [Remote communication format](#remote-communication-format)
- [Format of this document](#format-of-this-document)
- [Command list](#command-list)
- [Command reference](#command-reference)
- [Talkgroup ID formats](#talkgroup-id-formats)
- [Errata notes (conversion)](#errata-notes-conversion)

---

## Remote communication format

| Item | Value |
|---|---|
| BPS rate | 2400 / 4800 / 9600 / 19200 / 38400 / 57600 bps |
| Start / Stop bit | 1 bit / 1 bit |
| Data length | 8 bit |
| Parity check | None |
| Code | ASCII |
| Flow control | None |
| Return code | Carriage Return (`\r`) only |

Notes:

1. When controlling the scanner from a program, insert a waiting time between commands.
2. In Menu mode, only KEY emulation commands are valid.
3. A command that changes one scanner setting may also change other setup items. Most of these depend on the scanner's specifications (e.g. the `PM` or `PR` command).

## Format of this document

Each command is described as:

- **`COMMAND NAME`** — summary of the command's function
- **Controller → Radio** — command format
- **Radio → Controller** — response format

Error messages aren't described per command; the unit returns:

| Response | Meaning |
|---|---|
| `ERR\r` | Command format error / value error |
| `NG\r` | The command is invalid at this time |
| `FER\r` | Framing error |
| `ORER\r` | Overrun error |

Conventions:

- `\r` means "hit the Enter key" / "send the Return code".
- Channel banks and search numbers are lettered: BANK1 = `A`, BANK2 = `B`, … BANK5 = `E`.
- ID scan lists are lettered the same way: LIST1 = `A`, … LIST5 = `E`.
- ID locations within a list are `1`–`9`, with `0` meaning location 10.
- Frequencies are 8 digits, from the 1 GHz digit down to the 100 Hz digit (e.g. `04060125` = 406.0125 MHz).
- Placeholder characters (`@`, `#`, `%`, `$`, `&`, `?`, `X`, `*`) each stand for one character position.

---

## Command list

As listed in the source (page numbers refer to the original). `AV` is documented but missing from this list, and `BC` is listed twice.

| No. | Command | Function | Page |
|---:|---|---|---:|
| 1 | AC | Clear (Initialize) all memory | 4 |
| 2 | AF | Confirm/Set EDACS AFS ID Format | 4 |
| 3 | BC | Confirm Base, Space, Offset Configuration | 5 |
| 4 | BC | Confirm Base, Space, Offset Configuration | 5 |
| 5 | BT | Confirm/Set S-BIT function | 6 |
| 6 | CP | Set Base, Space, Offset Configuration for Custom system | 50 |
| 7 | CR | Confirm Base, Space, Offset Configuration for Custom system | 50 |
| 8 | DL | Confirm/Set DELAY function | 6 |
| 9 | DS | Confirm/Set DATA SKIP function | 7 |
| 10 | EL | Confirm/Set Enter Lock feature | 7 |
| 11 | FB | Confirm/Program fleet block | 8 |
| 12 | FI | Confirm/Set Frequency Identification function | 8 |
| 13 | HA | Informs when highway alert condition changes | 9 |
| 14 | HP | Informs when highway alert signal received | 9 |
| 15 | HW | Confirm/Set Highway scan mode | 10 |
| 16 | IC | Confirm/Move/Program Talk Group ID Location No. | 10 |
| 17 | ID | Informs when ID reception starts or ends | 14 |
| 18 | IL | Read / Register / Delete L/O ID memory | 17 |
| 19 | IS | Confirm/Select ID scan lists | 22 |
| 20 | KEY | KEY emulation command | 23 |
| 21 | LCD | Read LCD | 24 |
| 22 | LO | Confirm/Set LOCKOUT Channel | 25 |
| 23 | MA | Confirm / Move the channel No. | 26 |
| 24 | MD | Confirm the Scanner mode | 27 |
| 25 | MU | Confirm/Set status of speaker muting | 28 |
| 26 | PC | Confirm/Set priority channel No. of a bank | 28 |
| 27 | PI | Confirm/Set Priority Talk ID Memory Location | 29 |
| 28 | PM | Read / Program a channel frequency | 30 |
| 29 | PR | Confirm/Set PRIORITY function | 31 |
| 30 | QU | Informs when squelch condition changes | 31 |
| 31 | RF | Confirm/Tune the commanded frequency | 32 |
| 32 | RG | Confirm/Set EDACS ID Range mode | 33 |
| 33 | RI | Informs when priority receiving condition changes | 33 |
| 34 | RM | Confirm receive mode | 34 |
| 35 | SB | Confirm/Select scan banks | 34 |
| 36 | SI | Confirm Scanner Information | 35 |
| 37 | SQ | Confirm squelch condition | 35 |
| 38 | SS | Read / Register a frequency in search skip memory | 36 |
| 39 | ST | Confirm frequency step | 37 |
| 40 | SV | Confirm/Set Service scan mode | 37 |
| 41 | TB | Confirm/Set Trunking bank On/Off | 38 |
| 42 | TC | Confirm/Set Control channel only mode | 39 |
| 43 | TD | Confirm/Set Motorola disconnect Tone option | 41 |
| 44 | TG | Program Talk Group ID | 41 |
| 45 | TR | Set trunking to bank | 46 |
| 46 | US | Confirm/Select U.S. state | 47 |
| 47 | VR | Confirm the version of the Product | 48 |
| 48 | WD | Confirm/Set Warning light Dimmer | 48 |
| 49 | WL | Confirm warning light status | 49 |
| 50 | WM | Confirm warning mute condition | 49 |
| — | AV | Confirm/Set alert volume (documented on p. 5, not in the list) | 5 |

---

## Command reference

Many commands follow the same ON/OFF pattern: send the bare command to read the state, or the command plus `N` (ON) / `F` (OFF) to set it. The radio answers a read with `xxN`/`xxF` and a set with `OK`.

### AC — Clear (Initialize) all memory

```text
# Controller → Radio
AC\r
# Radio → Controller
OK\r
```

Clears all memory to initial settings. Valid at any time. Takes about 3 seconds. After initialization the scanner starts in highway scanning (state `AK`).

### AF — Confirm/Set EDACS AFS ID Format

```text
# Controller → Radio
AF\r        # ① read AFS ID format ON/OFF
AFN\r       # ② AFS ID format ON
AFF\r       #    AFS ID format OFF
# Radio → Controller
AFN\r       # ① ON
AFF\r       #    OFF
OK\r        # ②
```

Append a bank letter to address a specific bank, e.g. `AF A` or `AFN A`.

### AV — Confirm/Set alert volume

```text
# Controller → Radio
AV\r        # ① read alert volume
AV@\r       # ② set alert volume (no alert sound)
# Radio → Controller
AV@\r       # ①
OK\r        # ②
```

`@` = volume level: `H` high / `M` middle / `L` low. Add `?` at the end to hear the alert sound, e.g. `AV?` or `AVH?`.

### BC — Confirm Base, Space, Offset Configuration

```text
# Controller → Radio
BC @#\r
# Radio → Controller
BC @# %%%%%%%% $$$$ XXX\r
```

| Field | Meaning |
|---|---|
| `@` | Bank (`A`–`E`) |
| `#` | Configuration No. (`1`, `2`, `3`) |
| `%%%%%%%%` | Base frequency |
| `$$$$` | Spacing: multiple of 5.0 kHz (`0050`, `0100`, `0150` … `1000`) or of 12.5 kHz (`0125`, `0250`, `0375` … `1000`) |
| `XXX` | Offset channel (`380`–`759`) |

Example from the source:

```text
BC C1 01380000 0500 0380\r   # bank C (3), config 1, base 138.0000 MHz, spacing 50 kHz, offset 380
```

### BT — Confirm/Set S-BIT function

```text
# Controller → Radio
BT\r        # ① read S-BIT ON/OFF
BTN\r       # ② S-BIT ON
BTF\r       #    S-BIT OFF
# Radio → Controller
BTN\r       # ① ON
BTF\r       #    OFF
OK\r        # ②
```

Append a bank letter to address a specific bank, e.g. `BT A` or `BTN A`.

### CP — Set Base, Space, Offset Configuration for Custom system

```text
# Controller → Radio
CP & # @ %%%%%%%% ******** $$$$ ???? X\r
# Radio → Controller
OK\r
```

| Field | Meaning |
|---|---|
| `&` | Bank (`A`–`E`) |
| `#` | Trunking type: `A` Type 2 Custom / `B` Type 1 Custom |
| `@` | Configuration No. (`1`–`6`) |
| `%%%%%%%%` | Base (lower) frequency |
| `********` | Upper frequency |
| `$$$$` | Spacing: multiple of 5.0 kHz (`0050` × n, n = 1–20) or of 12.5 kHz (`0125` × n, n = 1–8) |
| `????` | Offset channel (`0000`–`1023`) |
| `X` | Polarity (`+` or `-`) |

### CR — Confirm Base, Space, Offset Configuration for Custom system

```text
# Controller → Radio
CR &@\r
# Radio → Controller
CR &@ %%%%%%%% ******** $$$$ ???? X\r
```

`&` = bank (`A`–`E`), `@` = configuration No. (`1`–`6`); the other fields are as for CP.

### DL — Confirm/Set DELAY function

```text
# Controller → Radio
DL\r        # ① read delay ON/OFF
DLN\r       # ② 2-second delay ON
DLF\r       #    delay OFF
# Radio → Controller
DLN\r       # ① ON
DLF\r       #    OFF
OK\r        # ②
```

### DS — Confirm/Set DATA SKIP function

```text
# Controller → Radio
DS\r        # ① read data skip ON/OFF
DSN\r       # ② data skip ON
DSF\r       #    data skip OFF
# Radio → Controller
DSN\r       # ① ON
DSF\r       #    OFF
OK\r        # ②
```

### EL — Confirm/Set Enter Lock feature

```text
# Controller → Radio
EL\r        # ① read Enter Lock ON/OFF
ELN\r       # ② Enter Lock ON
ELF\r       #    Enter Lock OFF
# Radio → Controller
ELN\r       # ① ON
ELF\r       #    OFF
OK\r        # ②
```

### FB — Confirm/Program fleet block

```text
# Controller → Radio
FB & #\r        # ① read fleet block size
FB & # %%\r     # ② program fleet block
# Radio → Controller
FB & # %%\r     # ①
OK\r            # ②
```

`&` = bank (`A`–`E`); `#` = fleet map block No. (`0`–`7`); `%%` = block size indicator (`00`–`14`).

### FI — Confirm/Set Frequency Identification function

```text
# Controller → Radio
FI\r        # ① read ON/OFF
FIN\r       # ② ON
FIF\r       #    OFF
# Radio → Controller
FIN\r       # ① ON
FIF\r       #    OFF
OK\r        # ②
```

### HA — Informs when highway alert condition changes

```text
# Controller → Radio
HA\r        # ① is HA reporting active?
HAN\r       # ② activate
HAF\r       #    deactivate (printed "HWF" in source)
# Radio → Controller
HAN\r       # ① active
HAF\r       #    inactive
OK\r        # ②
```

While active, the unit monitors the highway alert condition and sends:

- `HA+\r` when it changes from NO ALERT to ALERT
- `-\r` when it changes from ALERT to NO ALERT

### HP — Informs when highway alert signal is received

```text
# Controller → Radio
HP\r        # ① is HP reporting active?
HPN\r       # ② activate
HPF\r       #    deactivate
# Radio → Controller
HPN\r       # ① active
HPF\r       #    inactive
OK\r        # ②
```

While active, the unit sends `HP ALERT` when the highway alert signal becomes active.

### HW — Confirm/Set Highway scan mode

```text
# Controller → Radio
HW\r        # ① read highway scan mode
HWN\r       # ② highway scan only
HW+\r       #    highway scan plus private memory scan
# Radio → Controller
HWN\r       # ① highway scan only
HW+\r       #    highway + private
OK\r        # ②
```

### IC — Confirm/Move/Program Talk Group ID Location No.

```text
# Controller → Radio
IC\r            # ① read ID location number
IC @%\r         # ② move ID memory to list @, location %
IC @% <ID>\r    # ③ program a talkgroup ID at list @, location %
# Radio → Controller
IC @% <ID>\r    # ①, ②
IC @% ------\r  # ①, ② location not programmed
OK\r            # ③
```

`@` = ID scan list (`A`–`E`); `%` = ID location (`1`–`9`, `0` = 10). Example: `IC A0\r` moves to list A, location 10.

`<ID>` is in the format of the bank's system type; see [Talkgroup ID formats](#talkgroup-id-formats). Examples:

```text
IC A0 001-05\r     # Motorola Type 1: block 0, fleet 1, subfleet 5
IC A0 001234\r     # Motorola Type 2: ID 1234
IC A0 001064\r     # LTR: area 0, home repeater 01, ID 64
IC A0 01-025\r     # EDACS, AFS format: agency 01, fleet 02, subfleet 5
IC A0 000149\r     # EDACS, decimal format (same talkgroup)
```

### ID — Informs when ID reception starts or ends

```text
# Controller → Radio
ID\r        # ① is ID reporting active?
IDN\r       # ② ON
IDF\r       #    OFF
# Radio → Controller
IDN\r       # ① ON
IDF\r       #    OFF
OK\r        # ②
```

While ON, the unit reports the received ID and frequency when ID reception starts (`S`) and ends (`E`):

```text
ID S <ID> %%%%%%%%\r     # reception starts
ID E <ID> %%%%%%%%\r     # reception ends
```

`<ID>` is in the format of the system type ([Talkgroup ID formats](#talkgroup-id-formats)). Which frequency `%%%%%%%%` reports depends on the system:

| System | On start (`S`) | On end (`E`) |
|---|---|---|
| Motorola Type 1 | Control channel frequency | Voice channel frequency |
| Motorola Type 2 | Control channel frequency | Voice channel frequency |
| LTR | Home channel frequency | GOTO channel frequency |
| EDACS | Voice channel frequency | Working channel frequency |

Examples from the source:

```text
ID S 001-03 08510125\r   # Type 1: block 0, fleet 1, subfleet 3; CC 851.0125 MHz
ID S 001234 08510125\r   # Type 2: ID 1234; CC 851.0125 MHz
ID S 001064 08510250\r   # LTR: area 0, home rpt 01, ID 64; home ch 851.0250 MHz
ID S 01-025 08510125\r   # EDACS, AFS format
ID S 000149 08510125\r   # EDACS, decimal format
ID E 001-03 08510125\r   # Type 1 reception ends; voice ch 851.0125 MHz
ID E 001064 08510250\r   # LTR reception ends; GOTO ch 851.0250 MHz
ID E 001234 08510125\r   # Type 2 reception ends; voice ch 851.0125 MHz
ID E 01-025 08510125\r   # EDACS reception ends, AFS format; working ch 851.0125 MHz
ID E 000149 08510125\r   # EDACS reception ends, decimal format
```

### IL — Read / Register / Delete L/O ID memory

```text
# Controller → Radio
IL###\r         # ① read lockout memory ### (001–100), or highway trunked ID lockout memory (101–130)
ILR <ID>\r      # ② register an ID into L/O memory
ILD <ID>\r      # ③ delete an ID from L/O memory
ILD###\r        # ③ delete highway trunked ID lockout memory ### (101–130)
```

`<ID>` uses the [Talkgroup ID formats](#talkgroup-id-formats). For EDACS the source calls it the "EDACS Emergency ID". Register examples: `ILR 001-03` (Type 1), `ILR 024106` (Type 2), `ILR 001064` (LTR), `ILR 01-011` (EDACS).
Delete examples: `ILD 001-03`, `ILD 024106`, `ILD 001064`, `ILD 01-011`; `ILD101` deletes highway trunked lockout 101.

**EDACS block lockout** (lock out a whole agency or agency-fleet):

```text
ILR &&-\r       # lock out all of agency &&           e.g. ILR 02-
ILR &&-##\r     # lock out all of agency && fleet ##  e.g. ILR 02-01
ILD &&-\r       # delete agency lockout               e.g. ILD 02-
ILD &&-##\r     # delete agency-fleet lockout         e.g. ILD 02-01
```

Responses:

```text
# ① Read
IL ------\r                 # not registered
IL <ID>\r                   # e.g. IL 001-03, IL 024106, IL 001064, IL 01-011
IL &&----\r                 # EDACS: all-agency lockout
IL &&-##-\r                 # EDACS: all agency-fleet lockout (printed "ILD &&-##-")
IL ###### %%%%%%%%\r        # highway trunked ID + control channel frequency
# ② Register
OK\r                        # registered
ON\r                        # already in L/O memory
FULL\r                      # L/O memory full
# ③ Delete
OK\r                        # deleted
OFF\r                       # ID wasn't in L/O memory
```

Notes: the scanner can't register a highway trunking talkgroup, and doesn't send `OFF` when deleting one.

### IS — Confirm/Select ID scan lists

```text
# Controller → Radio
IS\r            # ① read selected ID scan lists
IS @%○...\r     # ② select ID scan lists (any of A–E)
# Radio → Controller
IS @%○...\r     # ①, ② selected lists
```

Example: `IS ACE\r` selects lists A, C and E (B and D are not selected).

### KEY — KEY emulation command

```text
# Controller → Radio
KEY○○\r         # ○○ = key emulate code
# Radio → Controller
OK\r
```

Add `H` after the code for a hold-press. Examples: `KEY06H\r` hold-presses [L/O]; `KEY02 6\r` presses [6].

| Code | Key | Code | Key |
|---|---|---|---|
| `KEY00` | [RESUME/HOLD] | `KEY01` | [HWY/PVT] |
| `KEY02` | [0]–[9] (digit follows after a space) | `KEY03` | [.] |
| `KEY04` | [E] | `KEY05` | [PRI] |
| `KEY06` | [L/O] | `KEY07` | [▲] |
| `KEY08` | [▼] | `KEY09` | [SRCH] |
| `KEY10` | [SVC] | `KEY11` | [DATA] |
| `KEY12` | [DLY] | `KEY13` | [TRUNK] |
| `KEY14` | [STATE] | `KEY15` | [MUTE] |
| `KEY16` | [FLASH] | `KEY17` | [ALERT] |

### LCD — Read LCD

```text
# Controller → Radio
LCD @@@\r       # ① read one icon's status
LCD BNK\r       # ② read the selected bank indicators
LCD CHN\r       # ③ read the channel indication
LCD FRQ\r       # ④ read the frequency indication
LCD\r           # ⑤ read everything at once
```

① response: `@@@ +` icon on · `@@@ -` icon off · `@@@ *` icon flashing.

| No. | Icon | `@@@` | No. | Icon | `@@@` |
|---:|---|---|---:|---|---|
| 1 | TRUNK | `TRUNK` | 16 | CB | `CB` |
| 2 | P | `P` | 17 | AIR | `AIR` |
| 3 | E | `E` | 18 | RR | `RR` |
| 4 | M | `M` | 19 | MRN | `MRN` |
| 5 | L | `L` | 20 | HOLD | `HOLD` |
| 6 | BANK | `BANK` | 21 | ▲ | `UP` |
| 7 | LIST | `LIST` | 22 | SRCH | `SRCH` |
| 8 | M-LOCK | `MLOCK` | 23 | ▼ | `DOWN` |
| 9 | RMT | `RMT` | 24 | PRI | `PRI` |
| 10 | HWY | `HWY` | 25 | L/O | `LOUT` |
| 11 | PVT | `PVT` | 26 | DLY | `DLY` |
| 12 | POL | `POL` | 27 | DATA | `DATA` |
| 13 | WX | `WX` | 28 | FLASH | `FLASH` |
| 14 | FIRE/EMS | `FIRE` | 29 | . | `DCML` |
| 15 | NEWS | `NEWS` | | | |

Examples for ②–④:

```text
BNK ++*--\r                  # banks 1,2 on; 3 flashing; 4,5 off
CHN [250][      ]\r          # channel indication
CHN [ AK][ **]\r
CHN [1-1][***]\r
FRQ [956.0000][           ]\r  # frequency indication
FRQ [id    SCAN][         ]\r
```

⑤ returns all of the above at once.

### LO — Confirm/Set LOCKOUT Channel

```text
# Controller → Radio
LO\r        # ① read lockout ON/OFF
LON\r       # ② lockout ON
LOF\r       #    lockout OFF
# Radio → Controller
LON\r       # ① ON
LOF\r       #    OFF
OK\r        # ②
```

### MA — Confirm / Move the channel No.

```text
# Controller → Radio
MA\r            # ① read current channel
MA@@@\r         # ② move to channel @@@ (001–250), e.g. MA015
# Radio → Controller
C@@@ F%%%%%%%% T# D# L# A# R# N$$$\r    # ①, ②
```

| Field | Meaning |
|---|---|
| `C@@@` | Channel No. |
| `F%%%%%%%%` | Frequency |
| `TN` / `TF` | Trunking frequency / conventional frequency |
| `DN` / `DF` | Delay ON / OFF |
| `LN` / `LF` | Lockout ON / OFF |
| `AN` / `AF` | Attenuator ON / OFF |
| `RN` / `RF` | Auto record ON / OFF |
| `N$$$` | CTCSS/DCS tone No. |

Example from the source (it omits the `R#` field):

```text
C015 F04060125 TF DN LF AF N000\r   # ch 15, 406.0125 MHz conventional, delay on, lockout off, att off, no CTCSS
```

### MD — Confirm the Scanner mode

```text
# Controller → Radio
MD\r
# Radio → Controller
MD@@\r          # @@ = current scanner mode
```

| `@@` | Mode | `@@` | Mode |
|---|---|---|---|
| `00` | Private Scan | `13` | Motorola ID scan |
| `01` | Private Scan hold | `14` | Motorola ID manual |
| `02` | Band Search | `15` | EDACS ID search |
| `03` | Band Search Hold | `16` | EDACS ID search hold |
| `04` | Service Scan | `17` | EDACS ID scan |
| `05` | Service Scan Hold | `18` | EDACS ID manual |
| `06` | Highway Scan | `19` | LTR ID search |
| `07` | Highway Scan Hold | `20` | LTR ID search hold |
| `08` | Highway Trunking | `21` | LTR ID scan |
| `09` | Highway Trunking Hold | `22` | LTR ID manual |
| `10` | Program channel | `23` | Manual frequency |
| `11` | Motorola ID search | `24` | Program trunked bank menu |
| `12` | Motorola ID search hold | `25` | Remote control menu |

### MU — Confirm/Set status of speaker muting

```text
# Controller → Radio
MU\r        # ① read mute control mode
MU?\r       # ② read whether audio is muted right now
MUN\r       # ③ force mute ON
MUF\r       #    force mute OFF
MUA\r       #    automatic mute control
# Radio → Controller
MUN\r       # ① forced ON
MUF\r       #    forced OFF
MUA\r       #    automatic
MU ON\r     # ② muted
MU OFF\r    #    not muted
OK\r        # ③
```

### PC — Confirm/Set priority channel No. of a bank

```text
# Controller → Radio
PC @\r          # ① read priority channel of bank @ (A–E), e.g. PC A
PC @%%%\r       # ② set priority channel %%% (001–250), e.g. PC A014
# Radio → Controller
PC @%%%\r       # ①, ② e.g. PC A014 = bank A priority is channel 14
```

### PI — Confirm/Set Priority Talk ID Memory Location

```text
# Controller → Radio
PI @\r              # ① read priority ID location of list @ (A–E) in the current trunk bank
PI @#\r             # ② set priority to list @, location # (1–9, 0), e.g. PI A1
# Radio → Controller
PI @# %%%%%%\r      # ① e.g. PI A1 001234 = list A, location 1, ID 001234
OK\r                # ②
```

### PM — Read / Program a channel frequency

```text
# Controller → Radio
PM@@@\r                 # ① read channel @@@ (001–250), e.g. PM014
PM@@@ %%%%%%%%\r        # ② program conventional frequency, e.g. PM014 04060125
PM@@@T%%%%%%%%\r        # ② program as a trunking channel (T flag)
# Radio → Controller
C@@@ F%%%%%%%% T# D# L# A# R# N$$$\r    # ①, ② (fields as for MA)
```

Example response:

```text
C015 F04060125 TF DN LF AF RF N000\r   # ch 15, 406.0125 MHz conventional, delay on, lockout off, att off, record off, CTCSS 00.0 Hz
```

Note: PM initializes the delay setting, so send DL after PM if needed.

### PR — Confirm/Set PRIORITY function

```text
# Controller → Radio
PR\r        # ① read priority ON/OFF
PRN\r       # ② priority ON
PRF\r       #    priority OFF
PR+\r       #    Priority Only (highway scan only)
# Radio → Controller
PRN\r       # ① ON
PRF\r       #    OFF
PR+\r       #    Priority Only ON
OK\r        # ②
```

### QU — Informs when squelch condition changes

```text
# Controller → Radio
QU\r        # ① is QU reporting active?
QUN\r       # ② ON
QUF\r       #    OFF
# Radio → Controller
QUN\r       # ① ON
QUF\r       #    OFF
OK\r        # ②
```

While ON, the unit sends `+\r` when squelch opens and `-\r` when it closes.

### RF — Confirm/Tune the commanded frequency

```text
# Controller → Radio
RF@@@@@@@@\r        # ① tune to a frequency
RF@@@@@@@@?\r       # ① tune and echo the tuned frequency
RF\r                # ② read tuned frequency
# Radio → Controller
OK\r                # ① (without ?)
RF@@@@@@@@\r        # ① (with ?) and ②
```

Examples: `RF04060125\r` tunes to 406.0125 MHz; `RF00290038\r` tunes to 29.0050 MHz (rounded to the default step).

### RG — Confirm/Set EDACS ID Range mode

```text
# Controller → Radio
RG\r            # ① read range mode
RG @@-\r        # ② range = agency @@ (00–15), e.g. RG 01-
RG @@-##\r      #    range = agency @@, fleet ## (00–15), e.g. RG 01-01
RGF\r           # ③ clear range mode (printed "RGF [¥r]")
# Radio → Controller
RGN\r           # ① range mode ON
RGF\r           #    range mode OFF
OK\r            # ②, ③
```

### RI — Informs when priority receiving condition changes

```text
# Controller → Radio
RI\r        # ① is RI reporting active?
RIN\r       # ② activate
RIF\r       #    deactivate
# Radio → Controller
RIN\r       # ① active
RIF\r       #    inactive
OK\r        # ②
```

While active, the unit sends `PST\r` when it stops on the priority channel, and `PRT\r` when it returns from it.

### RM — Confirm receive mode

```text
# Controller → Radio
RM\r
# Radio → Controller
RM @@\r     # RM AM or RM FM
```

Accepted in conventional and trunking mode.

### SB — Confirm/Select scan banks

```text
# Controller → Radio
SB\r            # ① read selected banks
SB @%○...\r     # ② select banks (any of A–E), e.g. SB ACE
# Radio → Controller
SB @%○...\r     # ①, ② e.g. SB ACE = banks A, C, E selected
```

### SI — Confirm Scanner Information

```text
# Controller → Radio
SI\r
# Radio → Controller
SI @@@@@@@@,%%%%%%%%%%,&&&\r
```

`@@@@@@@@` = model name; `%%%%%%%%%%` = ESN (not used); `&&&` = remote command version. Example: `SI BCT8,0000000000,200`.

Note that this is the only response that uses commas.

### SQ — Confirm squelch condition

```text
# Controller → Radio
SQ\r
# Radio → Controller
+\r         # squelch open
-\r         # squelch closed
```

### SS — Read / Register a frequency in search skip memory

```text
# Controller → Radio
SS###\r             # ① read skip memory ### (001–400)
SS@@@@@@@@\r        # ② register a band-search frequency, e.g. SS04060125
# Radio → Controller
SS@@@@@@@@\r        # ① band search / highway patrol channel
SS@@@@@@@@ #\r      # ① service search (# = service kind, see SV)
SS@@@@@@@@\r        # ② registered
ON\r                # ② already in skip memory
```

| Skip memory No. | Used for |
|---|---|
| `001`–`100` | Band search |
| `101`–`150` | Highway Patrol priority channel |
| `151`–`250` | Highway Patrol other channel |
| `251`–`350` | Police and DOT service search |
| `351`–`400` | Other service search |

Register applies only to band-search frequencies. Example read response: `SS08594875 1` = 859.4875 MHz is skipped in the Police service.

### ST — Confirm frequency step

```text
# Controller → Radio
ST\r
# Radio → Controller
ST ###\r        # 5K or 12.5K
```

### SV — Confirm/Set Service scan mode

```text
# Controller → Radio
SV\r        # ① read service kind
SV@\r       # ② set service scan mode
# Radio → Controller
SV@\r       # ①
OK\r        # ②
```

| `@` | Service | `@` | Service |
|---|---|---|---|
| `1` | Police | `6` | Air |
| `2` | Weather | `7` | Railroad |
| `3` | Fire/EMS | `8` | Marine |
| `4` | News | `9` | Department of Transportation |
| `5` | CB | | |

### TB — Confirm/Set Trunking bank On/Off

```text
# Controller → Radio
TB\r            # ① read the active trunk bank
TB #\r          # ② read bank # (A–E)
TBN #\r         # ③ trunking ON for bank #
TBF #\r         #    trunking OFF for bank #
# Radio → Controller
TB # @@@@@ %\r  # ①, ② e.g. TB A E2-800 N
OK\r            # ③
```

`%` = `N` trunking ON / `F` trunking OFF. `@@@@@` = trunking type:

| Value | Meaning |
|---|---|
| `E2-800` | Motorola Type 2 800 MHz |
| `E2-VHI` | Motorola Type 2 VHF |
| `E2-UHF` | Motorola Type 2 UHF |
| `TYPE1` | Motorola Type 1 |
| `EDCS WIDE` | Wide-band EDACS |
| `EDCS SCT` | EDACS SCAT |
| `LT` | LTR |
| `E2-CUS` | Motorola Type 2 Custom |
| `E1-CUS` | Motorola Type 1 Custom |

### TC — Confirm/Set Control channel only mode

```text
# Controller → Radio
TC @\r          # ① read control-channel-only mode of bank @ (A–E)
TCN @ ##\r      # ② ON, with optional channel assignment plan ## (P1–P4), e.g. TCN A P1
TCF @\r         #    OFF
# Radio → Controller
TCN @ ##\r      # ① ON (e.g. TCN A P1, or TCN A without a plan)
TCF @\r         #    OFF
OK\r            # ②
```

### TD — Confirm/Set Motorola disconnect Tone option

```text
# Controller → Radio
TD\r        # ① read disconnect (end-of-transmission) tone option
TDN\r       # ② ON
TDF\r       #    OFF
# Radio → Controller
TDN\r       # ① ON
TDF\r       #    OFF
OK\r        # ②
```

Append a bank letter to address a specific bank, e.g. `TD A` or `TDN A`.

### TG — Program Talk Group ID

Like IC, but with an explicit bank.

```text
# Controller → Radio
TG ? @%\r           # ① read ID at bank ?, list @, location %
TG ? @% <ID>\r      # ② program ID
# Radio → Controller
TG ? @% <ID>\r      # ①
OK\r                # ②
```

`?` = bank (`A`–`E`); `@` = ID scan list (`A`–`E`); `%` = ID location (`1`–`9`, `0` = 10). `<ID>` uses the [Talkgroup ID formats](#talkgroup-id-formats). Examples:

```text
TG A A0 001-05\r    # Type 1: block 0, fleet 1, subfleet 5
TG A A0 0127-3\r    # Type 1: block 0, fleet 127, subfleet 3
TG A A0 001234\r    # Type 2: ID 1234
TG A A0 001064\r    # LTR: area 0, home rpt 01, ID 64
TG A A0 01-025\r    # EDACS, AFS format
TG A A0 000149\r    # EDACS, decimal format
```

**EDACS partial IDs** (match a whole agency or agency-fleet):

```text
# Controller → Radio (program)
TG ? @% &&-\r       # all of agency && (01–15),           e.g. TG A A0 01-
TG ? @% &&-##\r     # all of agency &&, fleet ## (00–15), e.g. TG A A0 01-02
# Radio → Controller (read)
TG ? @% &&----\r    # e.g. TG A A0 01----
TG ? @% &&-##-\r    # e.g. TG A A0 01-02-
```

### TR — Set trunking to bank

```text
# Controller → Radio
TR & # %%%%%%%% $$$$ ??? X\r
# Radio → Controller
OK\r
```

| Field | Meaning |
|---|---|
| `&` | Bank (`A`–`E`) |
| `#` | Trunking type (see below) |
| `%%%%%%%%` | Base frequency (Motorola UHF/VHF only) |
| `$$$$` | Spacing (Motorola UHF/VHF only): multiple of 5.0 kHz (`0050` × n, n = 1–20) or of 12.5 kHz (`0125` × n, n = 1–8) |
| `???` | Offset channel, optional (Motorola UHF/VHF only): `380`–`759` |
| `X` | Base configuration No., optional: `1`, `2` or `3` |

| `#` | Trunking type |
|---|---|
| `1` | Type 1 |
| `2` | Type 2 800 MHz |
| `3` | Type 2 900 MHz (not used) |
| `4` | Type 2 UHF |
| `5` | Type 2 VHF |
| `6` | Wide-band EDACS |
| `7` | Narrow-band EDACS (not used) |
| `8` | EDACS SCAT |
| `9` | LTR |

### US — Confirm/Select U.S. state

```text
# Controller → Radio
US\r        # ① read state
US##\r      # ② select state ## (01–50)
# Radio → Controller
US##\r      # ①
OK\r        # ②
```

| `##` | State | `##` | State | `##` | State |
|---|---|---|---|---|---|
| 01 | Alaska | 18 | Louisiana | 35 | Ohio |
| 02 | Alabama | 19 | Massachusetts | 36 | Oklahoma |
| 03 | Arkansas | 20 | Maryland | 37 | Oregon |
| 04 | Arizona | 21 | Maine | 38 | Pennsylvania |
| 05 | California | 22 | Michigan | 39 | Rhode Island |
| 06 | Colorado | 23 | Minnesota | 40 | South Carolina |
| 07 | Connecticut | 24 | Missouri | 41 | South Dakota |
| 08 | District of Columbia | 25 | Mississippi | 42 | Tennessee |
| 09 | Delaware | 26 | Montana | 43 | Texas |
| 10 | Florida | 27 | North Carolina | 44 | Utah |
| 11 | Georgia | 28 | North Dakota | 45 | Virginia |
| 12 | Iowa | 29 | Nebraska | 46 | Vermont |
| 13 | Idaho | 30 | New Hampshire | 47 | Washington |
| 14 | Illinois | 31 | New Jersey | 48 | Wisconsin |
| 15 | Indiana | 32 | New Mexico | 49 | West Virginia |
| 16 | Kansas | 33 | Nevada | 50 | Wyoming |
| 17 | Kentucky | 34 | New York | | |

### VR — Confirm the version of the Product

```text
# Controller → Radio
VR\r
# Radio → Controller
VR@.@@\r        # e.g. VR1.00
```

This value is not the software version.

### WD — Confirm/Set Warning light Dimmer

```text
# Controller → Radio
WD\r        # ① read warning light dimmer
WD@\r       # ② set dimmer
# Radio → Controller
WD@\r       # ①
OK\r        # ②
```

`@` = `H` high brightness / `D` dimmed / `F` light off. Add `?` at the end to sound the alert and show the dimmer setting, e.g. `WD?` or `WDH?`.

### WL — Confirm warning light status

```text
# Controller → Radio
WL\r
# Radio → Controller
WLN\r       # warning light on
WLF\r       # warning light off
```

### WM — Confirm warning mute condition

```text
# Controller → Radio
WM\r
# Radio → Controller
WMF\r       # warning not muted
WM+\r       # warning muted momentarily
WMN\r       # warning fully muted
```

---

## Talkgroup ID formats

Used by IC, ID, IL, PI and TG. Each is 6 characters.

| System | Format | Fields | Example |
|---|---|---|---|
| Motorola Type 1 | `&##-$$` or `&###-$` | `&` block (`0`–`7`), `##`/`###` fleet, `$$`/`$` subfleet | `001-05` = block 0, fleet 1, subfleet 5; `0127-3` = block 0, fleet 127, subfleet 3 |
| Motorola Type 2 | `######` | Talkgroup ID | `001234` = ID 1234 |
| LTR | `%$$###` | `%` area code (`0`, `1`), `$$` home repeater (`01`–`20`), `###` ID (`000`–`254`) | `001064` = area 0, repeater 01, ID 64 |
| EDACS (AFS) | `&&-##$` | `&&` agency (`00`–`15`), `##` fleet (`00`–`15`), `$` subfleet (`0`–`7`) | `01-025` = agency 01, fleet 02, subfleet 5 |
| EDACS (decimal) | `######` | Decimal ID | `000149` (same talkgroup as `01-025`) |

Whether EDACS IDs use AFS or decimal format is set with the [AF command](#af--confirmset-edacs-afs-id-format).

---

## Errata notes (conversion)

These are observations about the source text, not part of the original spec:

- **Command list** — `BC` is listed twice (items 3 and 4) and `AV` is missing, although it's documented on page 5.
- **Error list** — "Flaming error" for framing error.
- **AF** — "If you ass the Bank No." (add).
- **HA** — the deactivate command is printed `HWF`; the response and the pattern of the other commands say `HAF`. The return-to-normal notification is printed as a bare `-\r`, not `HA-\r`.
- **BC example** — the offset is printed `0380` (4 digits) though the field is `XXX` (3 digits), and "Bank No.: 3" is given for bank `C`.
- **IC / TG** — Motorola Type 1 block No. is given as `0`–`9` in some places and `0`–`7` in others.
- **ID** — the Type 1 ID is described as `&##-&&` in places where `&##-$$` is meant.
- **IL** — the EDACS agency-fleet read response is printed `ILD &&-##-`, but its example is `IL 02-01-`; the agency-only example `IL 02-` doesn't match the `IL &&----` format above it.
- **LCD** — uses `[\r]` while the rest of the document uses `[¥r]`; the ① format is printed `LCD @` but the parameter is `@@@`; "EXMAPLE" typo.
- **KEY** — `[RESUMU/HOLD]` (RESUME).
- **MA** — "from 1 GHz digit to 1GHz digit" (should end at the 100 Hz digit); the example omits the `R#` field.
- **PM** — "PM command initialize delay mode, because DL commands is commanded after commanding PM command" — i.e. PM resets the delay setting.
- **RF** — the example `RF00290038` is said to tune 29.0050 MHz (rounded to the default step).
- **RG** — clear command printed `RGF [¥r]` with a space.
- **TC** — the OK reply appears on a page of its own, after the example.
- **TR** — `#` is described as "1,2,3,4,5,6,7" but the list runs to 9.
- **CP / CR** — the configuration number is printed `(1--6)`.
- **SS** — the register response example shows `SS04060125` rather than `OK`.
