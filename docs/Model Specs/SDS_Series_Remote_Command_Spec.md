# SDS Series — Remote Command Specification

**Applies to:** Uniden **SDS100** (UB383Z), **SDS200** (UB384Z) and **SDS150** (UB391Z).
`MDL` returns `SDS100`, `SDS200` or `SDS150GBT`.

> Markdown conversion of Uniden's *SDS Series Remote Command Specification*, version 2.00 (2025‑07‑07), 37 pages. Uniden based it on the BCD536HP/BCD436HP Remote Command Specification v1.05, and most of the protocol is unchanged from those models.
> This is the HomePatrol-style protocol, a different family from the BCD396XT/BCD325P2/BCD996P2 documents. The scanner keeps its own HomePatrol database (Favorites Lists → Systems → Departments → Channels), so there are no memory-programming commands (`PRG`, `CSY`, `CIN`, …). Most status and list data comes back as **XML** (`GSI`, `PSI`, `GLT`, `MSI`, `AST`), and items are addressed by an `Index` handle that the scanner assigns.
> SDS-specific additions: waterfall status and FFT data (`GST`, `PWF`, `GWF`, `GW2`), keep-alive (`KAL`), power off (`POF`), charge status (`GCS`), volume/squelch (`VOL`, `SQL`), a waterfall jump mode, and a new `STS` layout. Band Scope, Raw Data Output and RF Power Plot are marked *Removed in SDS100*, and `BFH` is gone.
> The source is a spreadsheet export; its sheets ("tabs") are turned into sections here. Wire formats are reproduced as written. Apparent typos in the original are listed under [Errata notes](#errata-notes-conversion) at the end.

---

## Contents

- [Revision history](#revision-history)
- [Command list](#command-list)
- [Command reference](#command-reference) — MDL, VER, KEY, QSH, STS, JNT, NXT, PRV, FQK, SQK, DQK, PSI, GSI, GLT, HLD, AVD, SVC, JPM, DTM, LCR, AST, APR, URC, MNU, MSI, MSV, MSB, GST, PWF, GWF, GW2, KAL, POF, GCS, VOL, SQL
- [GLT command (list retrieval)](#glt-command-list-retrieval)
- [Target keywords and 1st/2nd parameters (GLT, NXT/PRV, HLD, AVD)](#target-keywords-and-1st2nd-parameters)
- [PSI / GSI scanner information](#psi--gsi-scanner-information)
  - [Elements](#elements)
  - [Mode-dependent elements](#mode-dependent-elements)
  - [Attributes](#attributes)
  - [ViewDescription](#viewdescription)
  - [ReplayDescription](#replaydescription)
  - [Basic rules (MyId, Index, Name)](#basic-rules-myid-index-name)
- [Analyze commands (AST / APR)](#analyze-commands-ast--apr)
- [MSI response XML](#msi-response-xml)
- [Service types (SVC)](#service-types-svc)
- [Key codes (KEY)](#key-codes-key)
- [Sub audio (SAS / SAD)](#sub-audio-sas--sad)
- [Errata notes (conversion)](#errata-notes-conversion)

---

## Revision history

| Date | Version | Contents |
|---|---|---|
| 2018/04/13 | 0.01 | Created, based on the BCD536HP/BCD436HP Remote Command Specification version 1.05 |
| 2023/12/11 | 1.01 | Added Waterfall |
| 2023/12/22 | 1.02 | Added GST command |
| 2024/12/04 | 1.03 | Added POF / GCS commands |
| 2025/07/07 | 2.00 | Added GW2 command; added KAL command |

---

## Command list

The source's "Program Mode Only" column is empty for every command.

| No. | Command | Function |
|---:|---|---|
| 1 | MDL | Get Model Info |
| 2 | VER | Get Firmware Version |
| 3 | KEY | Push KEY |
| 4 | QSH | Go to quick search hold mode |
| 5 | STS | Get Current Status |
| 6 | JNT | Jump Number tag |
| 7 | NXT | Next |
| 8 | PRV | Previous |
| 9 | FQK | Get/Set Favorites List Quick Keys Status |
| 10 | SQK | Get/Set System Quick Keys Status |
| 11 | DQK | Get/Set Department Quick Keys Status |
| 12 | PSI | Push Scanner Information |
| 13 | GSI | Get Scanner Information |
| 14 | GLT | Get xxx list |
| 15 | HLD | Hold |
| 16 | AVD | Set Avoid Option |
| 17 | SVC | Get/Set Service Type Settings |
| 18 | JPM | Jump Mode |
| 19 | DTM | Get/Set Date and Time |
| 20 | LCR | Get/Set Location and range |
| 21 | AST | Analyze Start |
| 22 | APR | Analyze Pause/Resume |
| 23 | URC | User Record Control |
| 24 | MNU | Menu Mode command |
| 25 | MSI | Menu Status Info |
| 26 | MSV | Menu Set Value |
| 27 | MSB | Menu Structure Back |
| 28 | GST | Get scanner status |
| 29 | PWF | Push Waterfall FFT Information |
| 30 | GWF | Get Waterfall FFT Information |
| — | GW2 | Get Waterfall FFT Information without separator (unnumbered in source) |
| — | KAL | Keep Alive (unnumbered in source) |
| 31 | POF | Power OFF |
| 32 | GCS | Get Charge Status |
| 33 | VOL | Get/Set Volume Level Settings |
| 34 | SQL | Get/Set Squelch Level Settings |

---

## Command reference

Conventions (as in the other Uniden specs): every line ends with `\r`; `[FRQ]` is the 8‑digit frequency format from the 1 GHz digit down to the 100 Hz digit (e.g. `4060000` = 406.0 MHz, as used in the AVD example).

### MDL — Get Model Info

```text
# Controller → Radio
MDL\r
# Radio → Controller
MDL,[MODEL_NAME]\r      # SDS100, SDS200 or SDS150GBT
```

### VER — Get Firmware Version

```text
# Controller → Radio
VER\r
# Radio → Controller
VER,[VERSION]\r         # "Version x.xx.xx"
```

### KEY — Push KEY

```text
# Controller → Radio
KEY,[KEY_CODE],[KEY_MODE]\r
# Radio → Controller
KEY,OK\r
```

See [Key codes](#key-codes-key) for `[KEY_CODE]`. `[KEY_MODE]` values are not given in this source; on earlier Uniden models they are `P` press, `L` long press, `H` hold, `R` release, and the [popup example](#viewdescription) uses `KEY,E,P`.

### QSH — Go to quick search hold mode

```text
# Controller → Radio
QSH,[FRQ]\r
# Radio → Controller
QSH,OK\r
```

Invalid while the scanner is in Menu Mode, during Direct Entry, or during Quick Save.

### STS — Get Current Status

```text
# Controller → Radio
STS\r
# Radio → Controller (single line on the wire)
STS,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE],[L3_CHAR],[L3_MODE],...,[L20_CHAR],[L20_MODE],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV],[RSV]\r
```

| Parameter | Description |
|---|---|
| `[DSP_FORM]` | Display form, 5–20 digits; each `0` = small font, `1` = large font |
| `[Ln_CHAR]` | Line *n* characters, 24 or 30 chars (fixed length) |
| `[Ln_MODE]` | Line *n* display mode, 24 or 30 chars |
| `[RSV]` | Reserved (9 fields; the example shows they still carry values) |

**Display mode characters (`[Ln_MODE]`):** space = normal, `*` = reverse, `_` = underline. The number of `[Lx_CHAR]`/`[Lx_MODE]` pairs depends on the display form.

Example (main menu):

```text
→ STS\r
← STS,
  1111111111,                    # DSP_FORM (10 lines)
  --- M E N U ---         ,      # L1_CHAR
  ________________________,      # L1_MODE (underlined)
  Set Scan Selection      ,      # L2_CHAR
  ,                              # L2_MODE
  Manage Full Database    ,      # L3_CHAR
  ************************,      # L3_MODE (reversed)
  Manage Favorites        ,      # L4_CHAR
  ,                              # L4_MODE
  Set Your Location       ,      # L5_CHAR
  ,                              # L5_MODE
  Select Service Types    ,      # L6_CHAR
  ,                              # L6_MODE
  Srch/CloCall Opt        ,      # L7_CHAR
  ,                              # L7_MODE
  Search for...           ,      # L8_CHAR
  ,                              # L8_MODE
  Close Call              ,      # L9_CHAR
  ,                              # L9_MODE
  Analyze                 ,      # L10_CHAR
  ,                              # L10_MODE
  0,1,0,0,,,0,OFF,3\r            # the 9 [RSV] fields
```

`GST` returns the same screen plus waterfall state; PSI/GSI are the richer status sources.

### JNT — Jump Number tag

```text
# Controller → Radio
JNT,[FL_TAG],[SYS_TAG],[CHAN_TAG]\r
# Radio → Controller
JNT,OK\r
```

| Parameter | Description |
|---|---|
| `[FL_TAG]` | Favorites List number tag (0–99) |
| `[SYS_TAG]` | System number tag (0–99) |
| `[CHAN_TAG]` | Channel number tag (0–999) |

### NXT — Next

```text
# Controller → Radio
NXT,[tkw],[xxx1],[xxx2],[COUNT]\r
# Radio → Controller
NXT,OK\r
```

### PRV — Previous

```text
# Controller → Radio
PRV,[tkw],[xxx1],[xxx2],[COUNT]\r
# Radio → Controller
PRV,OK\r
```

For NXT and PRV: `[tkw]` (target keyword), `[xxx1]` and `[xxx2]` are described under [Target keywords](#target-keywords-and-1st2nd-parameters); `[COUNT]` is the number of steps to move (1–8).

### FQK — Get/Set Favorites List Quick Keys Status

```text
# Controller → Radio
FQK\r                          # (1) get
FQK,[S0],[S1],...,[S99]\r      # (2) set
# Radio → Controller
FQK,[S0],[S1],...,[S99]\r      # (1)
FQK,OK\r                       # (2)
```

### SQK — Get/Set System Quick Keys Status

```text
# Controller → Radio
SQK,[FAV_QK]\r                          # (1) get, for one Favorites List quick key
SQK,[FAV_QK],[S0],[S1],...,[S99]\r      # (2) set
# Radio → Controller
SQK,[FAV_QK],[SYS_QK],[S0],[S1],...,[S99]\r   # (1) — note the extra [SYS_QK] field
SQK,OK\r                                      # (2)
```

### DQK — Get/Set Department Quick Keys Status

```text
# Controller → Radio
DQK,[FAV_QK],[SYS_QK]\r                          # (1) get
DQK,[FAV_QK],[SYS_QK],[S0],[S1],...,[S99]\r      # (2) set
# Radio → Controller
DQK,[FAV_QK],[SYS_QK],[S0],[S1],...,[S99]\r      # (1)
DQK,OK\r                                         # (2)
```

Quick key status values `[S0]`–`[S99]` (FQK, SQK, DQK):

| Value | Meaning |
|---|---|
| `0` | Quick key does not exist |
| `1` | Quick key exists and is disabled |
| `2` | Quick key exists and is enabled |

If the controller sends `0` (does not exist), the radio ignores it.

### PSI — Push Scanner Information

The scanner outputs scanner information (XML) periodically; the interval is set by a parameter. See [PSI / GSI scanner information](#psi--gsi-scanner-information). The source does not give the request syntax.

### GSI — Get Scanner Information

The scanner returns scanner information (XML) once. See [PSI / GSI scanner information](#psi--gsi-scanner-information). The source does not give the request syntax.

### GLT — Get xxx list

Retrieves a list (favorites lists, systems, departments, channels, …) from the scanner as XML. See [GLT command](#glt-command-list-retrieval).

### HLD — Hold

Holds on a system, department, or channel. It can't hold a Favorites List or a site frequency.

```text
# Controller → Radio
HLD,[tkw],[xxx1],[xxx2]\r
# Radio → Controller
HLD,OK\r
```

See [Target keywords](#target-keywords-and-1st2nd-parameters) for the parameters.

### AVD — Set Avoid Option

Avoids or un-avoids an item. It can't avoid a Favorites List or a site frequency.

```text
# Controller → Radio
AVD,[tkw],[xxx1],[xxx2],[STATUS]\r      # comma before [STATUS] missing in source; see example below
# Radio → Controller
AVD,OK\r
```

| `[STATUS]` | Meaning |
|---|---|
| `1` | Permanent Avoid |
| `2` | Temporary Avoid |
| `3` | Stop Avoiding |

Use GSI or GLT to read avoid status. Example from the source: `AVD,AFREQ,4060000,,1\r` avoids 406.0 MHz.

### SVC — Get/Set Service Type Settings

```text
# Controller → Radio
SVC\r                                                      # (1) get
SVC,[PST1],[PST2],...,[PST37],[CST1],...,[CST10]\r         # (2) set
# Radio → Controller
SVC,[PST1],[PST2],...,[PST37],[CST1],...,[CST10]\r         # (1)
SVC,OK\r                                                   # (2)
```

Each field is `0` Off (not scanned) or `1` On (scanned). `PST1`–`PST37` are the preset service types and `CST1`–`CST10` the custom ones; see [Service types](#service-types-svc).

### JPM — Jump Mode

```text
# Controller → Radio
JPM,[JUMP_MODE],[INDEX]\r
# Radio → Controller
JPM,OK\r
```

| `[JUMP_MODE]` | `[INDEX]` |
|---|---|
| `SCN_MODE` | Channel index (`0xFFFFFFFF` = start scanning from the top channel) |
| `CTM_MODE` | Reserve |
| `QSH_MODE` | Reserve |
| `CC_MODE` | Reserve |
| `WX_MODE` | `NORMAL`, `A_ONLY`, `SAME_1` … `SAME_5`, `ALL_FIPS` |
| `FTO_MODE` | Reserve |
| `WF_MODE` | Reserve (waterfall mode) |
| `IREC_MODE` | Reserve |
| `UREC_MODE` | Folder name |
| `TDIS_MODE` | Session name (trunk discovery) |
| `CDIS_MODE` | Session name (conventional discovery) |

If a temporary clock was set, jumping to discovery mode or WX alert mode returns `NG`.

### DTM — Get/Set Date and Time

```text
# Controller → Radio
DTM\r                                                          # (1) get
DTM,[DayLightSaving],[YYYY],[MM],[DD],[hh],[mm],[ss]\r         # (2) set
# Radio → Controller
DTM,[DayLightSaving],[YYYY],[MM],[DD],[hh],[mm],[ss],[RTC Status]\r   # (1)
DTM,OK\r                                                              # (2)
```

`[RTC Status]`: `0` RTC NG / `1` RTC OK.

### LCR — Get/Set Location and range

```text
# Controller → Radio
LCR\r                                      # (1) get
LCR,[LATITUDE],[LONGITUDE],[RANGE]\r       # (2) set
# Radio → Controller
LCR,[LATITUDE],[LONGITUDE],[RANGE]\r       # (1)
LCR,OK\r                                   # (2)
```

Latitude and longitude are in **degree** format (not the DMS format of older models).

### AST — Analyze Start

See [Analyze commands](#analyze-commands-ast--apr).

### APR — Analyze Pause/Resume

See [Analyze commands](#analyze-commands-ast--apr).

### URC — User Record Control

```text
# Controller → Radio
URC\r                   # (1) get
URC,[STATUS]\r          # (2) set
# Radio → Controller
URC,[STATUS]\r          # (1)
URC,OK\r                # (2) success
URC,ERR,[ERROR CODE]\r  # (2) failure
```

`[STATUS]`: `0` Stop / `1` Start.

| `[ERROR CODE]` | Meaning |
|---|---|
| `0001` | File access error |
| `0002` | Low battery |
| `0003` | Session over limit |
| `0004` | RTC lost |

### MNU — Menu Mode

```text
# Controller → Radio
MNU,[MENU_ID],[INDEX]\r
# Radio → Controller
MNU,OK\r
```

| `[MENU_ID]` | `[INDEX]` | Menu position |
|---|---|---|
| `TOP` | — | Top (main) menu |
| `MONITOR_LIST` | — | Select Lists to Monitor menu |
| `SCAN_SYSTEM` | System index | System menu |
| `SCAN_DEPARTMENT` | Department index | Department menu |
| `SCAN_SITE` | Site index | Site menu |
| `SCAN_CHANNEL` | Channel index | Channel menu |
| `SRCH_RANGE` | Custom bank index | Custom Search Bank menu |
| `SRCH_OPT` | — | Search/Close Call Options menu |
| `CC` | — | Close Call menu |
| `CC_BAND` | — | Close Call Band menu |
| `WX` | — | WX Operation menu |
| `FTO_CHANNEL` | FTO channel index | Tone-out Channel menu |
| `SETTINGS` | — | Settings menu |
| `BRDCST_SCREEN` | — | Broadcast Screen menu |

### MSI — Menu Status Info

```text
# Controller → Radio
MSI\r
# Radio → Controller
MSI,<XML>,\r
<?xml version="1.0" encoding="utf-8"?>\r
<MSI Name=" Title " Index="xxxxxx " >\r
  ...
</MSI>\r
```

See [MSI response XML](#msi-response-xml).

### MSV — Menu Set Value

```text
# Controller → Radio
MSV,[RSV],[VALUE]\r
# Radio → Controller
MSV,OK\r
```

`[VALUE]`: for a select-type menu, the index of the selected item; for an input-type menu, the entered string. If the value contains a comma, replace each comma with a tab (`\t`).

### MSB — Menu Structure Back

```text
# Controller → Radio
MSB,[RSV],[RET_LEVEL]\r
# Radio → Controller
MSB,OK\r
```

| `[RET_LEVEL]` | Action |
|---|---|
| `RETURN_PREVOUS_MODE` | Exit menu mode (spelling as in source) |
| *(empty)* | Go back one level |

### GST — Get scanner status (supports Waterfall)

```text
# Controller → Radio
GST\r
# Radio → Controller (single line on the wire)
GST,[DSP_FORM],[L1_CHAR],[L1_MODE],[L2_CHAR],[L2_MODE],[L3_CHAR],[L3_MODE],...,[L20_CHAR],[L20_MODE],[MUTE],[LED1],[LED2],[WF_MODE],[FREQ],[MOD],[MF_POS],[CF],[LOWER],[UPPER],[COLOR_MODE],[FFT_SIZE]\r
```

| Parameter | Description |
|---|---|
| `[DSP_FORM]` | Display form, 5–20 digits; each `0` = small font, `1` = large font |
| `[Ln_CHAR]` / `[Ln_MODE]` | Line *n* characters / display mode, 24 or 30 chars (fixed length) |
| `[MUTE]` | Mute status (values not given) |
| `[LED1]` | Alert LED: `0` OFF, `1` BLUE, `2` RED, `3` MAGENTA, `4` GREEN, `5` CYAN, `6` YELLOW, `7` WHITE |
| `[LED2]` | Battery charge LED (same color codes) — **SDS150 only** |
| `[WF_MODE]` | Scanner mode: `0` Normal mode (any mode other than waterfall, e.g. scan or search), `1` Waterfall, `2` Menu / Direct Entry |
| `[FREQ]` | Marker frequency |
| `[MOD]` | Modulation |
| `[MF_POS]` | Marker position |
| `[CF]` | Center frequency |
| `[LOWER]` / `[UPPER]` | Lower / upper frequency |
| `[COLOR_MODE]` | `0` Color, `1` Black background, `2` White background |
| `[FFT_SIZE]` | FFT area size: `0` 25%, `1` 50%, `2` 75%, `3` 100% |

Notes:

- Display mode characters: space = normal, `*` = reverse, `_` = underline. The number of line pairs depends on the display form.
- If an `[Lx_CHAR]` or `[Lx_MODE]` is all spaces (`0x20`), it is sent as an empty string.
- A comma inside `[Lx_CHAR]` is sent as a tab (`\t`).

### PWF — Push Waterfall FFT Information

```text
# Controller → Device
PWF,[FFT_TYPE],[ON/OFF]\r
# Device → Controller (pushed repeatedly while on)
PWF,[DATA1],[DATA2],...,[DATA_n],\r
```

`[FFT_TYPE]`: `1` = displayed FFT (240 points). `[ON/OFF]` values are not given in the source.

### GWF — Get Waterfall FFT Information

```text
# Controller → Radio
GWF,[TYPE],[ON/OFF]\r
# Radio → Controller
GWF,[DATA1],[DATA2],...,[DATA240],\r
```

`[TYPE]` (described as `[FFT_TYPE]`): displayed FFT (240 points).

### GW2 — Get Waterfall FFT Information without separator (binary)

```text
# Controller → Radio
GWF,[TYPE],[ON/OFF]\r        # printed as GWF in the source (see errata)
# Radio → Controller
GWF,[FFT_DATA]\r             # printed as GWF in the source
```

`[FFT_DATA]`: displayed FFT (240 points) as **binary data** with no comma separators.

### KAL — Keep Alive

```text
# Controller → Radio
KAL\r
```

The radio does **not** send a response to KAL.

### POF — Power OFF

```text
# Controller → Device
POF\r
# Device → Controller
POF,OK\r
```

### GCS — Get Charge Status

```text
# Controller → Device
GCS\r
# Device → Controller — note the \n line ending as printed
GCS,CST=[DATA1],VOLT=[DATA2]mV:[DATA3]%,CURR=[DATA4]mA,TEMP=[DATA5]C\n
```

| Field | Meaning |
|---|---|
| `[DATA1]` | Charge status (table below) |
| `[DATA2]` | Battery voltage (mV) |
| `[DATA3]` | Remaining battery capacity (%) |
| `[DATA4]` | Charge (+) / discharge (−) current (mA) |
| `[DATA5]` | Battery temperature (°C) |

| `CST` | Name | Meaning |
|---|---|---|
| `0` | `CHG_SUBSTS_NO_CHG` | No external power or no battery |
| `1` | `CHG_SUBSTS_BQ274_INIT` | Initializing gauge IC (BQ27426) |
| `2` | `CHG_SUBSTS_TEMP_NG` | Abnormal temperature |
| `3` | `CHG_SUBSTS_PWR_NG` | Abnormal power |
| `4` | `CHG_SUBSTS_FULL_CHG` | Full charge |
| `5` | `CHG_SUBSTS_RECHG` | Recharge |
| `6` | `CHG_SUBSTS_CHARGE` | Charging |

Example: `CST=4,VOLT=4184mV:100%,CURR=0000mA,TEMP= 27.65C`

### VOL — Get/Set Volume Level Settings

```text
# Controller → Radio
VOL\r              # (1) get
VOL,[LEVEL]\r      # (2) set
# Radio → Controller
VOL,[LEVEL]\r      # (1)
VOL\r              # (2) as printed (other set commands reply ...,OK)
```

`[LEVEL]`: `0`–`15` on SDS100/SDS150, `0`–`29` on SDS200.

### SQL — Get/Set Squelch Level Settings

```text
# Controller → Radio
SQL\r              # (1) get
SQL,[LEVEL]\r      # (2) set
# Radio → Controller
SQL,[LEVEL]\r      # (1)
SQL\r              # (2) as printed
```

`[LEVEL]`: `0`–`15` on SDS100/SDS150, `0`–`19` on SDS200.

---

## GLT command (list retrieval)

```text
# Controller → Radio
GLT,FL\r                          # (1)  Favorites List
GLT,SYS,[fl_index]\r              # (2)  System
GLT,DEPT,[system_index]\r         # (3)  Department
GLT,SITE,[system_index]\r         # (4)  Site
GLT,CFREQ,[dept_index]\r          # (5)  Conventional Frequency
GLT,TGID,[dept_index]\r           # (6)  TGID
GLT,SFREQ,[site_index]\r          # (7)  Site Frequency
GLT,AFREQ\r                       # (8)  Search Avoiding Frequencies
GLT,ATGID,[system_index]\r        # (9)  Search Avoiding TGID
GLT,FTO\r                         # (10) Fire Tone Out
GLT,CS_BANK\r                     # (11) Custom Search Bank
GLT,UREC\r                        # (12) User Record
GLT,IREC_FILE\r                   # (13) Inner Record File
GLT,UREC_FILE,[folder_index]\r    # (14) User Record File
GLT,TRN_DISCOV\r                  # (15) Trunk Discovery
GLT,CNV_DISCOV\r                  # (16) Conventional Discovery
```

The response is XML: one element per list entry, named after the list type, with these attributes:

| No. | Element | Attributes |
|---:|---|---|
| 1 | `FL` | `Index`, `Name`, `Monitor`, `Q_Key`, `N_Tag` |
| 2 | `SYS` | `Index`, `MyId`, `Name`, `Avoid`, `Type`, `Q_Key`, `N_Tag` |
| 3 | `DEPT` | `Index`, `MyId`, `Name`, `Avoid`, `Q_Key` |
| 4 | `SITE` | `Index`, `MyId`, `Name`, `Avoid`, `Q_Key` |
| 5 | `CFREQ` | `Index`, `MyId`, `Name`, `Avoid`, `Freq`, `Mod`, `SAS`, `SAL`, `SvcType`, `N_Tag` |
| 6 | `TGID` | `Index`, `MyId`, `Name`, `Avoid`, `TGID`, `Audio Type`, `SvcType`, `N_Tag` |
| 7 | `SFREQ` | `Index`, `Freq` |
| 8 | `AFREQ` | `Freq`, `Avoid` |
| 9 | `ATGID` | `TGID`, `Avoid`, `index`, `Name`, `DeptName`, `DeptIndex` |
| 10 | `FTO` | `Index`, `Freq`, `Mod`, `Name`, `ToneA`, `ToneB` |
| 11 | `CS_BANK` | `Index`, `Name`, `Lower`, `Upper`, `Mod`, `Step` |
| 12 | `UREC` | `Index`, `Name` (folder name) |
| 13 | `IREC_FILE` | `Index`, `Name` (file name), `Time` |
| 14 | `UREC_FILE` | `Index`, `Name` (file name), `Time` |
| 15 | `TRN_DISCOV` | `Name` (session name), `Delay`, `Logging`, `Duration`, `CompareDB`, `SystemName`, `SystemType`, `SiteName`, `TimeOutTimer`, `AutoStore` |
| 16 | `CNV_DISCOV` | `Name` (session name), `Lower`, `Upper`, `Mod`, `Step`, `Delay`, `Logginig`, `CompareDB`, `Duration`, `TimeOutTimer`, `AutoStore` |

Abbreviations: `Q_Key` quick key · `N_Tag` number tag · `Freq` frequency · `Mod` modulation · `SAS` sub audio setting (CTCSS/DCS/P25 NAC/color code/RAN/area) · `SAL` sub audio lockout (tone L/O). `Avoid` is `Off` / `T-Avoid` / `Avoid`.

The `Index` is a handle that the controller uses for HLD and AVD. `MyId` corresponds to the RadioReference database ID (see [Basic rules](#basic-rules-myid-index-name)).

Example:

```text
GLT,FL\r
GLT,<XML>,\r
<?xml version="1.0" encoding="utf-8"?>\r
<GLT>\r
  <FL Index="0" Name="Favorites List 1" Monitor="On" Q_Key="1" N_Tag="None" />\r
  <FL Index="1" Name="Favorites List 2" Monitor="On" Q_Key="2" N_Tag="2" />\r
  <FL Index="2" Name="Favorites List 3" Monitor="Off" Q_Key="3" N_Tag="999" />\r
</GLT>\r
```

(The source writes `\r` as `¥r`, the Japanese-locale rendering of the backslash.)

---

## Target keywords and 1st/2nd parameters

The target keyword (`[tkw]`) and the 1st/2nd parameters (`[xxx1]`, `[xxx2]`) for GLT, NXT/PRV, HLD and AVD. GLT takes only a 1st parameter. `[none]` = the parameter is empty; `—` = invalid command.

| Target | Keyword | GLT 1st | NXT/PRV 1st | NXT/PRV 2nd | HLD 1st | HLD 2nd | AVD 1st | AVD 2nd |
|---|---|---|---|---|---|---|---|---|
| Favorites List | `FL` | [none] | — | | — | | — | |
| System | `SYS` | Parent FL index | Sys index | [none] | Sys index | [none] | Sys index | [none] |
| Department | `DEPT` | Parent Sys index | Dept index | Parent Sys index | Dept index | Parent Sys index | Dept index | [none] |
| Site | `SITE` | Parent Sys index | Site index | [none] | Site index | [none] | Site index | [none] |
| Conventional frequency | `CFREQ` | Parent Dept index | Chan index | [none] | Chan index | [none] | Chan index | [none] |
| TGID in ID Scan | `TGID` | Parent Dept index | Chan index | [none] | Chan index | [none] | Chan index | [none] |
| TGID in ID Search | `STGID` | — | TGID | Site index | TGID | Site index | — (use `ATGID`) | |
| Site frequency | `SFREQ` | Parent Site index | — | | — | | — | |
| Avoiding TGID in ID Search | `ATGID` | Parent Sys index | — | | — | | TGID | Parent Sys index |
| Search avoiding frequency | `AFREQ` | [none] | — | | — | | Frequency | [none] |
| Close Call | `CC` | — | [none] | [none] | [none] | [none] | — (use `AFREQ`) | |
| WX | `WX` | [none] | WX chan index | [none] | WX chan index | [none] | — | |
| Tone-Out mode | `FTO` | [none] | FTO chan index | [none] | FTO chan index | [none] | — | |
| Search with Scan frequency | `SWS_FREQ` | — | Frequency | Parent Dept index | Frequency | Parent Dept index | — (use `AFREQ`) | |
| CC Hits channel | `CCHIT` | Parent Dept index | CC chan index | [none] | CC chan index | [none] | CC chan index | [none] |
| Custom Search bank | `CS_BANK` | [none] | — | | — | | — | |
| Custom Search frequency | `CS_FREQ` | — | Frequency | Parent Bank index | Frequency | Parent Bank index | — (use `AFREQ`) | |
| Quick Search frequency | `QS_FREQ` | — | Frequency | [none] | Frequency | [none] | — (use `AFREQ`) | |
| Repeater Find frequency | `RPTR_FREQ` | — | Frequency | [none] | Frequency | [none] | — (can't avoid a repeater frequency) | |
| Inner record file | `IREC_FILE` | [none] | *(blank)* | [none] | File index | [none] | — (can't avoid) | |
| User record folder | `UREC_FOLDER` | [none] | — (can't select folder) | | — (can't select folder) | | — (can't avoid) | |
| User record file | `UREC_FILE` | Folder index | File index | [none] | File index | [none] | — (can't avoid) | |
| Trunk discovery | `TRN_DISCOV` | [none] | — | | — | | TGID | [none] |
| Conventional discovery | `CNV_DISCOV` | [none] | — | | — | | Frequency | [none] |
| Band scope | `BAND_SCOPE` | — | Frequency | [none] | Frequency | [none] | — | |

**Note 1.** To avoid 406.0 MHz in Quick Search mode, `AVD,AFREQ,4060000,,1\r` is correct; `AVD,QS_FREQ,4060000,,1\r` is a bad command.

**Note 2.** If the app sends HLD, NXT or PRV in Repeater Find mode, the scanner cancels Repeater Find and returns to the previous mode (Custom Search / Quick Search / Close Call).

**Note 3.** The "Unknown" department in ID Search is a virtual department. You can hold it and step NXT/PRV through it, but you can't avoid it. The "Unknown" department needs the parent system index as the 2nd parameter; other departments don't, and either blank or the system index is accepted.

---

## PSI / GSI scanner information

A PC/tablet app needs the scanner's internal state to display it. On `GSI` the scanner sends this information once; on `PSI` it outputs it periodically at an interval set by a parameter. The format is XML with `ScannerInfo` as the root element.

Example (scan mode):

```xml
<?xml version="1.0" encoding="utf-8"?>
<ScannerInfo Mode="Trunk Scan Hold" V_Screen="trunk_scan">
  <MonitorList Name="Full Database" Index="4294967295" ListType="FullDb" Q_Key="None" N_Tag="None" DB_Counter="3" />
  <System Name="Calcasieu" Index="283" Avoid="Off" SystemType="Conventional" Q_Key="None" N_Tag="None" Hold="On" />
  <Department Name="Calcasieu Parish - Parish Fire &amp; Medical" Index="286" Avoid="Off" Q_Key="None" Hold="Off" />
  <ConvFrequency Name="DeQuincy Fire Department" Index="290" Avoid="Off" Freq=" 154.4150MHz"
       Mod="NFM" N_Tag="None" Hold="On" SvcType="Fire Dispatch" P_Ch="Off" SAS="All" SAD="None" LVL="0" IFX="Off" />
  <AGC A_AGC="Off" D_AGC="Off" />
  <DualWatch PRI="Off" CC="Off" WX="Off" />
  <Property
       VOL="0" SQL="9" Sig="0"
       WiFi="3" Att="Off" Rec="Off"
       KeyLock="Off" P25Status="None"
       Mute="Mute" Backlight="100"
       Rssi="0.377"
  />
  <ViewDescription>
    <InfoArea1 Text="F0:01234-6*789" />
    <InfoArea2 Text="S3:01234-6*---" />
    <PopupScreen Text="Quick Save?\n"/>
  </ViewDescription>
<!-- example ends here in the source, without closing </ScannerInfo> -->
```

### Elements

Present in all modes:

| Element | Notes |
|---|---|
| `ScannerInfo` | Root node |
| `Property` | |
| `AGC` | |
| `DispFormat` | Listed, but no attributes are documented |
| `ViewDescription` | When the radio is showing the override area |
| `ReplayDescription` | When the radio is in replay mode |

Elements inside `ViewDescription`: `InfoArea1`, `InfoArea2`, `OverWrite`, `PopupScreen`, `PlainText`.
Elements inside `ReplayDescription`: `File`, `ReplayMode`.

### Mode-dependent elements

✓ = element is present in that mode. The modes are grouped as in the source: **Scan mode** (conventional_scan, trunk_scan, custom_with_scan, cchits_with_scan), **Search** (custom_search, quick_search, close_call, cc_searching), **Signal** (tone_out, wx_alert), **Temporary** (reverse_frequency, repeater_find), **Discovery** (discovery_conventional, discovery_trunking) and **Analyze** (analyze_system_status, rf_power_plot, analyze) and **WF** (waterfall). No element is marked for rf_power_plot.

| Element | conv_scan | trunk_scan | custom_w_scan | cchits_w_scan | custom_search | quick_search | close_call | cc_searching | tone_out | wx_alert | reverse_freq | repeater_find | disc_conv | disc_trunk | sys_status | rf_power_plot | analyze | waterfall |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| MonitorList | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | | | | |
| System | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | | | | |
| Department | ✓ | ✓ | ✓ | ✓ | | | | | | | | | | | | | | |
| Site | | ✓ | | | | | | | | | | | | | | | | |
| ConvFrequency | ✓ | | | | | | | | | | | | | | | | | |
| TGID | | ✓ | | | | | | | | | | | | | | | | |
| SiteFrequency | | ✓ | *(blank)* | *(blank)* | | | | | | | | | | | | | | |
| SrchFrequency | | | ✓ | | ✓ | ✓ | ✓ | | | ✓ | ✓ | ✓ | | | | | | |
| CcHitsChannel | | | | ✓ | | | | | | | | | | | | | | |
| DualWatch | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | | |
| SearchRange | | | ✓ | | ✓ | ✓ | | | | | | | | | | | | |
| SearchBanks | | | | | ✓ | | | | | | | | | | | | | |
| CC_Bands | | | | | | | | ✓ | | | | | | | | | | |
| CC_Counters | | | | | | | | ✓ | | | | | | | | | | |
| ToneOutChannel | | | | | | | | | ✓ | | | | | | | | | |
| WxChannel | | | | | | | | | | ✓ | | | | | | | | |
| WxMode | | | | | | | | | | ✓ | | | | | | | | |
| ConventionalDiscovery | | | | | | | | | | | | | ✓ | | | | | |
| TrunkingDiscovery | | | | | | | | | | | | | | ✓ | | | | |
| SystemStatus | | | | | | | | | | | | | | | ✓ | | | |
| Analyze | | | | | | | | | | | | | | | | | ✓ | |
| WaterfallBand | | | | | | | | | | | | | | | | | | ✓ |
| WaterfallSettings | | | | | | | | | | | | | | | | | | ✓ |

### Attributes

Index values are `0`‑based handles unless noted. `Name` is ASCII, max. 64 characters.

**ScannerInfo**

| Attribute | Values |
|---|---|
| `Mode` | `Scan Mode`, `Scan Hold`, `Tone-Out`, `Custom Search`, `Custom Search Hold`, `Quick Search`, `Quick Search Hold`, `Service Scan`, `Service Scan Hold`, `Trunk Scan`, `Trunk Scan Hold`, `Close Call Only`, `Close Call`, `Menu tree` |
| `V_Screen` | `plain_text`, `conventional_scan`, `trunk_scan`, `custom_with_scan`, `cchits_with_scan`, `custom_search`, `quick_search`, `close_call`, `cc_searching`, `tone_out`, `wx_alert`, `discovery_conventional`, `discovery_trunking`, `reverse_frequency`, `repeater_find`, `direct_entry`, `menu_selection`, `menu_input`, `analyze_system_status`, `analyze` |

**Property**

| Attribute | Values |
|---|---|
| `F` | `Off` / `On` (function key state) |
| `VOL` | `0`–`29` or `0`–`15` |
| `SQL` | `0`–`19` or `0`–`15` |
| `Sig` | `0`–`4` |
| `WiFi` | `Off` / `0`–`3` / `AP` |
| `Battery` | `0.0`–`3.3` |
| `Att` | `Off` / `On` / `G-Att` |
| `Rec` | `Off` / `On` |
| `KeyLock` | `Off` / `On` |
| `P25Status` | `None`, `Data`, `P25`, `DMR`, `CAP`, `CON`, `DT3`, `XPT`, `NX9`, `NX4`, `ND9`, `ND4`, `IDS`, `NXD` |
| `Mute` | `Unmute` / `Mute` |
| `A_Led` | `Off`, `Blue`, `Red`, `Magenta`, `Green`, `Cyan`, `Yellow`, `White` |
| `Dir` | `Up` / `Down` |
| `Rssi` | `0`– |

The example also shows `Backlight="100"`, which is not in the attribute list.

**AGC** — `A_AGC`, `D_AGC`: `Off` / `On`.

**DualWatch**

| Attribute | Values |
|---|---|
| `PRI` | `Off` / `DND` / `Priority` |
| `CC` | `Off` / `DND` / `Priority` |
| `WX` | `Off` / `Priority` |

**MonitorList**

| Attribute | Values |
|---|---|
| `Name` | ASCII, max. 64 |
| `Index` | `0`– |
| `ListType` | `FullDb` / `FL` / `SWS` |
| `Q_Key` | `0`–`99` / `None` |
| `N_Tag` | `0`–`99` / `None` |
| `DB_Counter` | `0`–`65535`; wraps to `0` after 65535 |

**System**

| Attribute | Values |
|---|---|
| `Name` | ASCII, max. 64 |
| `Index` | `0`– |
| `Avoid` | `Off` / `T-Avoid` / `Avoid` |
| `SystemType` | `Conventional`, `Motorola`, `EDACS`, `LTR`, `P25 Trunk`, `P25 One Frequency`, `MotoTRBO Trunk`, `DMR One Frequency`, `NXDN Trunk`, `NXDN One Frequency` |
| `Q_Key` | `0`–`99` / `None` |
| `N_Tag` | `0`–`99` / `None` |
| `Hold` | `Off` / `On` |

**Department** — `Name`, `Index`, `Avoid` (`Off`/`T-Avoid`/`Avoid`), `Q_Key` (`0`–`99`/`None`), `Hold` (`Off`/`On`).

**Site** — `Name`, `Index`, `Avoid` (`Off`/`T-Avoid`/`Avoid`), `Q_Key` (`0`–`99`/`None`), `Hold` (`Off`/`On`), `Mod` (`Auto`/`NFM`/`FM`).

**ConvFrequency**

| Attribute | Values |
|---|---|
| `Name` | ASCII, max. 64 |
| `Index` | `0`– |
| `Avoid` | `Off` / `T-Avoid` / `Avoid` |
| `Freq` | `xxxx.xxxxMHz` |
| `Mod` | `Auto`, `AM`, `NFM`, `FM`, `WFM`, `FMB` |
| `N_Tag` | `0`–`999` / `None` |
| `Hold` | `Off` / `On` |
| `SvcType` | See [Service types](#service-types-svc) |
| `P_Ch` | `Off` / `On` (priority channel) |
| `SAS` | See [Sub audio](#sub-audio-sas--sad) |
| `SAL` | `Off` / `On` |
| `SAD` | See [Sub audio](#sub-audio-sas--sad) |
| `RecSlot` | `Slot 1` / `Slot 2` / `None` |
| `LVL` | `-3`…`3` (volume offset) |
| `IFX` | `Off` / `On` |
| `TGID` | `TGID xxxx` / `None` |
| `U_Id` | `UID xxxx` / `None` |

**TGID**

| Attribute | Values |
|---|---|
| `Name` | ASCII, max. 64 |
| `Index` | `0`– |
| `Avoid` | `Off` / `T-Avoid` / `Avoid` |
| `TGID` | `TGID:xxxx` |
| `SetSlot` | `Slot 1` / `Slot 2` / `Any` |
| `RecSlot` | `Slot 1` / `Slot 2` / `None` |
| `N_Tag` | `0`–`999` / `None` |
| `Hold` | `Off` / `On` |
| `SvcType` | See [Service types](#service-types-svc) |
| `P_Ch` | `Off` / `On` |
| `LVL` | `-3`…`3` |

**SiteFrequency** — `Freq` (`xxxx.xxxxMHz`), `SAS`, `SAD`, `IFX` (`Off`/`On`).

**SearchBanks**

| Attribute | Values |
|---|---|
| `Index` | `0`–`9` |
| `BankStatus` | 10 digits, each `0` Off / `1` On, order `0123456789` |
| `Name` | ASCII, max. 64 |
| `BankNo` | `0`–`9` |

**CC_Bands** — `BandStatus`: 7 digits, each `0` Off / `1` On, order `0123456`.

**CC_Counters** — no attributes are documented.

**SrchFrequency** — `Avoid` (`Off`/`T-Avoid`/`Avoid`), `Freq`, `Mod`, `Hold`, `SAD`, `RecSlot` (`Slot 1`/`Slot 2`/`None`), `TGID` (`TGID xxxx`/`None`), `U_Id` (`UID xxxx`/`None`), `IFX`.

**CcHitsChannel** — `Name`, `Index`, `Avoid`, `CH_No` (`0`–`9`), `Freq`, `Mod`, `Hold`, `SAD`, `LVL` (`-3`…`3`), `IFX`.

**SearchRange** — `Lower`, `Upper` (`xxxx.xxxxMHz`), `Mod`, `Step` (values not given).

**ToneOutChannel** — `Name`, `Index` (`1`–), `CH_No` (`0`–`31`), `Freq`, `Mod`, `Hold`, `LVL`, `IFX`, `ToneA`, `ToneB` (`xxxxHz`).

**WxMode** — `Mode`: `"Monitor Weather"` or `"Weather Alert"`; `SAME`: `"Alert Only"` or a SAME group name.

**WxChannel** — `Name`, `Index` (`0`–), `CH_No` (`1`–`7`), `Freq`, `Mod` (`FM`), `Hold`, `LVL`, `IFX`.

**ConventionalDiscovery** — `Lower`, `Upper`, `Mod`, `Step`, `Freq`, `SAD`, `RecSlot`, `PastTime`, `HitCount`, `TGID` (`TGID xxxx`/`None`), `U_Id` (`UID xxxx`/`None`), `IFX`.

**TrunkingDiscovery** — `SystemName`, `SiteName` (ASCII, max. 64), `TGID`, `TgidName`, `SAD`, `RecSlot`, `PastTime`, `HitCount`, `U_Id`.

**SystemStatus**

| Attribute | Values |
|---|---|
| `SystemName` | ASCII, max. 64 |
| `SiteName` | ASCII, max. 64 |
| `Signal` | `0`–`100` |
| `Quality` | `0`–`100` |
| `Activity` | `0`–`100` |
| `SystemID` | `0`–`0x1FFFF` |
| `SystemSubID` | `0`–`99` |
| `SiteID` | `0`–`4095` |
| `WacnID` | `0`–`0xFFFFF` |
| `NAC` | `0`–`0xFFF` |
| `Color` | `0`–`15` |
| `RAN` | `0`–`63` |
| `Area` | `0`–`1` |
| `Att` | `Off` / `G-Att` |
| `Freqs` | `0`–`16` |
| `P25Status` | as in Property |

**RfPowerPlot** — `Frequency` (`xxxx.xxxxMHz`), `Modulation` (`Auto`/`AM`/`NFM`/`FM`/`WFM`/`FMB`), `SampleRate` (`100ms`/`200ms`/`400ms`/`800ms`), `Att` (`Off`/`G-Att`), and `B01`–`B34` (each `0`–`100`).

**Analyze** — `Msg1`, `Msg2`, `SystemName`, `SiteName` (ASCII, max. 64), `Att` (`Off`/`G-Att`). Used by LCN Finder, Current Activity, LCN Monitor and Activity Log.

**WaterfallBand**

| Attribute | Values |
|---|---|
| `Lower`, `Center`, `Upper` | `xxxx.xxxxMHz` |
| `Mod` | `Auto`, `AM`, `NFM`, `FM`, `WFM`, `FMB` |
| `Step` | `5kHz`, `6.25kHz`, `7.5kHz`, `833kHz` [*sic*], `10kHz`, `12.5kHz`, `15kHz`, `20kHz`, `25kHz`, `50kHz`, `100kHz` |
| `Span` | `360kHz`, `720kHz`, `1.44MHz`, `2.88MHz`, `5.76MHz`, `8.64MHz`, `17.28MHz` |
| `Limit` | `0` / `1` |

**WaterfallSettings** — `MF` (marker frequency, `xxxx.xxxxMHz`), `Gain` (`Auto` / `0`–`15`).

### ViewDescription

**Info Area and Override.** The main screen has special areas. Info Area 1 and Info Area 2 show the quick key status in scan mode, or the bank status in custom search mode. The Over Write area shows an error or scanning message in place of the channel name. (The source's screen illustrations are carried over from the BCD536HP/BCD436HP document.)

```xml
<ViewDescription>
  <InfoArea1 Text="F0:01234-6*789" />
  <InfoArea2 Text="S3:01234-6*---" />
  <OverWrite Text="No thing to scan">
</ViewDescription>
```

**Popup Screen.** A temporary view shown on the main screen for 1–2 seconds, like an Android toast:

```xml
<ViewDescription>
  <PopupScreen Text="Global ATT\nOn"/>
</ViewDescription>
```

A popup can also have buttons. That kind is not cleared automatically; the scanner waits for the user to press a button, like a dialog box:

```xml
<ViewDescription>
  <PopupScreen Text="USB Cable Detected&#xD;
     Select USB mode&#xD;&#xD;Mass Storage=&quot;E&quot; / Serial Port=&quot;.&quot;">
    <Button Text="&quot;E&quot; (Yes)" KeyCode="E" />
    <Button Text="&quot;.&quot; (No)" KeyCode="." />
  </PopupScreen>
```

This popup has 2 buttons. If the 'E' (Yes) button is pressed, the app should send `KEY,E,P` (the button's `KeyCode`).

**PlainText view.** A view mode of the main screen:

```xml
<ViewDescription>
  <PlainText Text="Copyright 2014"/>
  <PlainText Text="Uniden America Corp."/>
  <PlainText Text="All Rights Reserved."/>
  <PlainText Text=""/>
</ViewDescription>
```

### ReplayDescription

```xml
<ReplayDescription>
  <File Index="2" />
  <ReplayMode Mode="USER_REC" />
</ReplayDescription>
```

### Basic rules (MyId, Index, Name)

**MyId.** Systems, departments, sites and channels in the Full Database have a MyId, and so do copies made from it; ones the user created do not. MyId relates to the RadioReference ID and is shown as `xxId=xx` (e.g. `CountyId=5`, `AgencyId=15`). Search with Scan has no MyId.

| HPDB ID | Description | RRDB ID |
|---|---|---|
| `CountyId` | Conventional system (county) | `ctid` |
| `AgencyId` | Conventional system (agency) | `aid` |
| `TrunkId` | Trunked system | `sid` |
| `CGroupId` | Conventional department | `scid` |
| `CFreqId` | Conventional frequency | `fid` |
| `SiteId` | Trunked site | `siteId` |
| `TGroupId` | Trunked department | `tgCid` |
| `Tid` | Trunked channel | `tgId` |

**Index.** Used to hold or avoid systems, departments and channels. It is assigned when the data is loaded into RAM, and is **invalid if `DB_Counter` has changed**.

**Name.** ASCII `0x20`–`0x7E`, max. 64 characters.

---

## Analyze commands (AST / APR)

Before sending an AST command that uses a site index (Current Activity, LCN Monitor, Activity Log, LCN Finder), go to Scan Mode so the HPDB data is loaded.

### Current Activity

```text
# Controller → Radio
AST,CURRENT_ACTIVITY,[Site Index]\r
```

The response is XML, sent every 200 ms. Each row is either a control channel (No, LCN, Frequency, SystemID, SiteID, TGID type) or a voice channel (No, LCN, Frequency, TGID, Unit ID, MOD, TGID type).

| Attribute | Meaning |
|---|---|
| `LCN` | LCN (decimal) |
| `Freq` | Frequency |
| `TGID` | Talkgroup ID (decimal) |
| `UnitID` | Unit ID (decimal) |
| `MOD` | `Analog` / `Digital` / `Encrypted` |
| `TgidType` | `Control Channel`, `Encrypted`, `Patch`, `Unknown`, `TGID`, `I-CALL` |
| `SystemID` | System ID (hex) |
| `SiteID` | Site ID (decimal) |

```text
AST,<XML>,\r
<?xml version="1.0" encoding="utf-8"?>\r
<AST>\r
  <CurrentActivity LCN="1" Freq="851.0125" SystemID="0001h" SiteID="0" TgidType="Control Channel" />\r
  <CurrentActivity LCN="2" Freq="851.0375" TGID="16" UnitID="32" MOD="Analog" TgidType="TGID" />\r
  <CurrentActivity LCN="3" Freq="851.0625" TGID="64" UnitID="128" MOD="Analog" TgidType="TGID" />\r
  ...
  <CurrentActivity LCN="32" Freq="851.6125" TGID="256" UnitID="512" MOD="Analog" Tgidype="TGID" />\r
</AST>\r
```

### LCN Monitor

```text
# Controller → Radio
AST,LCN_MONITOR,[Site Index]\r
```

The response is XML, sent every 1 s. Attributes: `LCN` (decimal), `Freq`, `ReceiveStaus` (`1` or `0`; spelling as in source).

```text
AST,<XML>,\r
<?xml version="1.0" encoding="utf-8"?>\r
<AST>\r
  <LcnMonitor LCN="1" Freq = "851.0125" ReceiveStaus="1" />\r
  <LcnMonitor LCN="2" Freq = "851.0250" ReceiveStaus="0" />\r
  ...
  <LcnMonitor LCN="32" Freq = "851.4000" ReceiveStaus="0" />\r
</AST>\r
```

### Activity Log

If a temporary clock was set, entering Activity Log mode returns `NG`.

```text
# Controller → Radio
AST,ACTIVITY_LOG,[Site Index]\r
# Radio → Controller (one line per message)
AST,ACTIVITY_LOG,[Time],[Data],[Message],[Description]
```

| Field | Meaning |
|---|---|
| `Time` | `MM/DD/YYYY hh:mm:ss` |
| `Data` | Received raw data (format depends on system type) |
| `Message` | Message type (depends on system type) |
| `Description1`–`5` | Message description; how many depends on the message type |

#### Motorola

`Data` = `"<cmd>/<prv>/<id>"` — `cmd` command field 0–1023 (decimal), `prv` private bit 0/1, `id` ID field 0–65535 (decimal).

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 | Desc 5 |
|---|---|---|---|---|---|
| System ID | Sid: | | | | |
| Site ID | Site: | | | | |
| Talkgroup Voice Channel Grant | Tid: | Uid: | Lcn: | Sts: | Mod: |
| Talkgroup Voice Channel Grant Update | Tid: | | Lcn: | Sts: | |
| I-Call Voice Channel Grant Update | Uid: | | Lcn: | | |
| Individual Call | Uid: | Uid: | Lcn: | | |
| Patch/MultiSelect Voice Channel Grant | Pid: | Uid: | Lcn: | Sts: | Mod: |
| Patch/Multiselect Voice Channel Grant Update | Pid: | | Lcn: | Sts: | |
| Patch List | Pid: | Mid | | | |
| Patch Cancel | Pid: | | | | |
| Control | | | | | |
| First OSW | | | | | |
| Receive Error | | | | | |

Descriptions: `Sid` system ID (hex) · `Site` site ID (decimal) · `Tid` talkgroup ID (decimal) · `Uid` unit ID (decimal) · `Pid` patch ID (decimal) · `Mid` patch member ID (decimal) · `Lcn` LCN (decimal) · `Mod` `Analog` / `Digital`.
`Sts` (status bit): `Normal Talkgroup`, `All Talkgroup`, `Emergency`, `Talkgroup Patch`, `Emergency Patch`, `Emergency Multi-Group`, `Multi-Select`, `DES Encryption Talkgroup`, `DES All Talkgroup`, `DES Emergency`, `DES Talkgroup Patch`, `DES Emergency Patch`, `DES Emergency Multi-Group`, `Multi-Select DES TG`.

#### P25 Standard

`Data` = `"<opecode>/<data>"` — `opecode` 1 byte `00`–`FF` (hex), `data` TSBK 12 bytes (24 hex digits).

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 | Desc 5 |
|---|---|---|---|---|---|
| Group Voice Channel Grant | Lcn: | Gad: | Sad: | | |
| Group Voice Channel Grant Explicit | LcnT: | Gad: | Sad: | LcnR: | |
| Group Voice Channel Grant Update | Lcn: | Gad: | Lcn: | Gad: | |
| Group Voice Channel Grant Update Explicit | LcnT: | LcnR: | Gad: | | |
| Unit To Unit Voice Channel Grant | Lcn: | Tad: | Sad: | | |
| Unit To Unit Voice Channel Grant Extended | LcnT: | Tad: | Sad: | LcnR: | |
| Unit To Unit Answer Request | Tad: | Src: | | | |
| Unit To Unit Answer Request Extended | Tad: | Src: | | | |
| Unit To Unit Voice Channel Grant Update | Lcn: | Tad: | Sad: | | |
| Unit To Unit Voice Channel Grant Update Extended | LcnT: | Tad: | Sad: | LcnR: | |
| Identifier Update for TDMA | Iden: | Type: | Tofs: | Csp: | Bfrq: |
| Time and Date Announcement | Iden: | Bw: | Tofs: | Csp: | Bfrq: |
| RFSS Status Broadcast | Sid: | Sub: | Site: | Lcn: | |
| RFSS Status Broadcast Extended | Sid: | Sub: | Site: | LcnT: | LcnR: |
| Network Status Broadcast | Wacn: | Sid: | Lcn: | | |
| Network Status Broadcast Extended | Wacn: | Sid: | LcnT: | LcnR: | |
| Identifier Update for non-VHF/UHF Bands | Iden: | Bw: | Tofs: | Csp: | Bfrq: |

Messages with no descriptions: Telephone Voice Channel Grant · Telephone Interconnect Answer Request · Identifier Update for X2TDMA · Individual Data Channel Grant · Group Data Channel Grant · Group Data Channel Announcement · Group Data Channel Announcement Explicit · SNDCP Data Channel Grant · SNDCP Data Page Request · SNDCP Data Channel Announcement Explicit · Status Update · Status Query · Message Update · Radio Unit Monitor Command · Call Alert · Acknowledge Response FNE · Queued Response · Extended Function Command · Deny Response · Group Affiliation Response · Secondary Control Channel Broadcast Explicit · Group Affiliation Query · Location Registration Response · Unit Registration Response · Unit Registration Command · Authentication Command · De-Registration Acknowledge · Identifier Update for VHF/UHF Bands · Roaming Address Command · Roaming Address Update · System Service Broadcast · Secondary Control Channel Broadcast · Adjacent Status Broadcast · Protection Parameter Broadcast · Protection Parameter Update · Receive Error.

Descriptions (decimal unless noted): `Lcn` LCN · `LcnT` transmit channel LCN · `LcnR` receive channel LCN · `Gad` group address · `Sad` source address · `Tad` target address · `Src` source ID · `Iden` identifier · `Bw` bandwidth · `Tofs` transmit offset · `Csp` channel spacing · `Bfrq` base frequency · `Sid` system ID (hex) · `Sub` RF sub-system ID · `Site` site ID · `Wacn` WACN ID (hex) · `Type` channel type.

#### EDACS

`Data` = `"<data>"` — message data, 28 bits, `0000000`–`FFFFFFF` (hex).

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 |
|---|---|---|---|---|
| Site ID | Site: | | Lcn: | |
| Talkgroup Voice Channel Grant | Tid: | Uid: | Lcn: | Sts: |
| Talkgroup Voice Channel Grant Update | Tid: | | Lcn: | Sts: |
| I-Call Voice Channel Grant Update | Uid: | | Lcn: | Sts: |
| Patch Voice Channel Grant | Pid: | Uid: | Lcn: | Sts: |
| Patch Voice Channel Grant Update | Pid: | | Lcn: | Sts: |
| Patch List | Pid: | Mid: | | |
| First OSW | | | | |
| Receive Error | | | | |

Descriptions: `Site` site ID · `Tid` talkgroup ID (decimal 1–2047: AFS; decimal 2048–65535: decimal) · `Uid` unit ID · `Pid` patch ID · `Mid` patch member ID · `Lcn` LCN (all decimal).
`Sts`: `Normal Talkgroup`, `Talkgroup Patch`, `Emergency`, `Emergency Patch`, `Digital Talkgroup`, `Digital Patch`, `Digital Emergency`, `Digital Emergency Patch`, `I-Call`, `Digital I-Call`.

#### LTR

`Data` = `"<area_code>/<goto>/<home>/<id>/<free>"` — area code 0/1, goto repeater 0–31, home repeater 0–31, ID field 0–255, free repeater 0–31 (all decimal).

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 |
|---|---|---|---|---|
| Repeater Idle | Tid: | Rpt: | Goto: | Free: |
| Talkgroup Voice Channel Grant Update | Tid: | Rpt: | Goto: | Free: |
| Turn-off Code | Tid: | Rpt: | Goto: | Free: |

Descriptions: `Tid` talkgroup ID (Area-Home-Id) · `Rpt` transmitting repeater · `Goto` goto repeater · `Free` free repeater.

#### DMR / MotoTRBO

`Data` = `"<opcode>/<fid>/<id>/<ch>/<slot>/<prv>/<emergency>"`

| Field | Meaning | Range |
|---|---|---|
| `opcode` | Full/Short Link Control opcode, or Control Signal Block opcode | `00`–`3F` (hex) |
| `fid` | Feature ID | `00` DMR, `06` Connect Plus, `10` Capacity Plus (hex) |
| `id` | TGID | 0–16777215 (decimal) |
| `ch` | LCN | 0–4095 (decimal) |
| `slot` | TDMA slot | `1`, `2`, or `15` (none) |
| `prv` | Privacy | 0 / 1 |
| `emergency` | Emergency | 0 / 1 |

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 | Desc 5 |
|---|---|---|---|---|---|
| Talkgroup Voice Channel Grant | Tid: | Uid: | Color Code: | Lcn: | Slot: |
| Talkgroup Voice Channel Link Control | Tid: | Uid: | Color Code: | Lcn: | Slot: |
| Unit to Unit Voice Channel Grant | Uid Src: | Uid Dst: | Color Code: | Lcn: | Slot: |
| Unit to Unit Voice Channel Link Control | Uid Src: | Uid Dst: | Color Code: | Lcn: | Slot: |
| Broadcast Talkgroup Voice Channel Grant | Tid: | Uid: | Color Code: | Lcn: | Slot: |
| Capacity Plus Voice Channel Grant | Tid: | Uid: | Color Code: | Lcn: | Slot: |
| Capacity Plus Update | Sid: | Site: | Color Code: | Lcn: | Slot: |
| Capacity Plus Site ID | Sid: | Site: | Color Code: | Lcn: | Slot: |
| Linked Capacity Plus Site ID | Sid: | Site: | Color Code: | Lcn: | Slot: |
| Connect Plus Voice Channel Grant | Tid: | Uid: | Color Code: | Lcn: | Slot: |
| Connect Plus Update | | | | | |
| Connect Plus Network ID | Sid: | Site: | Color Code: | Lcn: | Slot: |
| DMR Network ID | Sid: | Site: | Color Code: | Lcn: | Slot: |
| Idle | | | | | |

Descriptions: `Sid` network ID (hex) · `Site` site ID · `Tid` talkgroup ID · `Uid` unit ID · `Uid Src` / `Uid Dst` source / destination unit ID · `Color Code` · `Lcn` · `Slot` TDMA slot (all decimal unless noted).

#### NXDN

`Data` = `"<call type>/<home ch>/<id>/<ch>/<prv>/<emergency>"`

| Field | Meaning | Range |
|---|---|---|
| `call type` | Call type | 0–7 (decimal) |
| `home ch` | Home channel | 0–31 (IDAS only, decimal) |
| `id` | TGID | NEXEDGE 0–65535, IDAS 0–2047 (decimal) |
| `ch` | LCN | 0–1023 (decimal) |
| `prv` | Privacy | 0 / 1 |
| `emergency` | Emergency | 0 / 1 |

Where two description rows are shown, the first is for a group call and the second for an individual call.

| Message | Desc 1 | Desc 2 | Desc 3 | Desc 4 |
|---|---|---|---|---|
| Replying to requesting communication | Tid: / Uid Src: | Uid: / Uid Dst: | Area Code: | Home Ch: |
| Performing voice communication | Tid: / Uid Src: | Uid: / Uid Dst: | RAN: or Area Code: | |
| Sending Encryption init vector | | | | |
| Assignment of traffic channel to VC | Tid: / Uid Src: | Uid: / Uid Dst: | RAN: or Area Code: | LCN: or Home Ch: |
| Existence of assigned traffic channel to VC | Tid: / Uid Src: | Uid: / Uid Dst: | RAN: or Area Code: | LCN: or Home Ch: |
| Transmission released | | | | |
| Idle | | | | |
| Disconnecting | | | | |
| Site configuration information | Sys: | Site: | RAN: | Cch LCN: or DFA |
| Service information which site provides | Sys: | Site: | RAN: | |
| Information of site's control channel | Sys: | Site: | RAN: | Cch LCN: or DFA |
| IDAS go to Repeater | Tid: / Uid Src: | Uid: / Uid Dst: | Area Code: | Go to Repeater: |

Descriptions (decimal): `Sys` system ID · `Site` site ID · `Tid` talkgroup ID · `Uid` unit ID · `Uid Src` / `Uid Dst` individual call source / destination unit ID · `RAN` NEXEDGE RAN · `Area Code` IDAS area code · `LCN` NEXEDGE LCN · `Go to Repeater` IDAS repeater channel · `Home Ch` IDAS home channel · `Cch LCN` NEXEDGE control channel · `DFA` NEXEDGE direct frequency assignment.

### LCN Finder

```text
# Controller → Radio
AST,LCN_FINDER,[Site Index]\r
```

The response is XML, sent every 500 ms.

| Attribute | Meaning |
|---|---|
| `Freq` | Frequency |
| `AccuracyStatus` | 30 digits, one accuracy level per LCN slot: `0` Unknown, `1`–`4` Level 1–4, `5` Found, `6` Disable |
| `Condition` | `Searching` / `All Lcn Found` |

```text
AST,<XML>,\r
<?xml version="1.0" encoding="utf-8"?>\r
<AST>\r
  <LcnFinder Freq="851.0125" AccuracyStatus="566666666666666666666666666666"
  <LcnFinder Freq="851.0375" AccuracyStatus="610000000000000000000000000000"
  <LcnFinder Freq="851.0625" AccuracyStatus="603000000000000000000000000000"
  ...
  <LcnFinder Freq="851.6125" AccuracyStatus="600000000000000000000000000000"
  <LcnFinder Condition="Searching"/>
</AST>\r
```

### Band Scope

> **Removed in SDS100** — this section is crossed out in the source and kept here for reference only.

```text
# Controller → Radio
AST,BAND_SCOPE,[Center frequency],[Span],[Step],[Modulation]\r
# Radio → Controller — one line each time the frequency changes (every 10 ms)
AST,BAND_SCOPE,[Frequency],[RSSI_LEVEL]\r
```

| Parameter | Values |
|---|---|
| `[Center frequency]` | `250000`–`13000000` |
| `[Span]` | `200`, `400`, `600`, `800`, `1000`, `2000`, `4000`, `6000`, `8000`, `10000`, `20000`, `40000`, `60000`, `80000`, `100000`, `200000` |
| `[Step]` | `500`, `625`, `750`, `833`, `1000`, `1500`, `2000`, `2500`, `5000`, `10000` |
| `[Modulation]` | `Auto`, `AM`, `NFM`, `FM`, `WFM`, `FMB` |
| `[RSSI_LEVEL]` | RSSI level `0`–`100` |

### Raw Data Output

> **Removed in SDS100** — this section is crossed out in the source and kept here for reference only.

```text
# Controller → Radio
AST,RAW_DATA_OUTPUT,[Frequency],[Modulation],[Filter],[Global Attenuator]\r
```

| Parameter | Values |
|---|---|
| `[Frequency]` | `250000`–`13000000` |
| `[Modulation]` | `Auto`, `AM`, `NFM`, `FM`, `WFM`, `FMB` |
| `[Filter]` | `1` On / `0` Off |
| `[Global Attenuator]` | `1` On / `0` Off |

The radio then outputs discriminator A/D sampling raw data (10‑bit signed), split into a high byte and a low byte:

| Byte | b7 | b6 | b5 | b4 | b3 | b2 | b1 | b0 |
|---|---|---|---|---|---|---|---|---|
| H | **1** | 0 | 0 | bit9 | bit8 | bit7 | bit6 | bit5 |
| L | **0** | 0 | 0 | bit4 | bit3 | bit2 | bit1 | bit0 |

Bit 7 marks the byte (1 = high, 0 = low). Raw data output is only available on the USB port. To use other remote commands, send the pause command (APR) first.

### System Status

```text
# Controller → Radio
AST,SYSTEM_STATUS,[site_index]\r
# Radio → Controller
AST,OK\r
```

The status itself is reported through the `SystemStatus` element of PSI/GSI.

### RF Power Plot

> **Removed in SDS100** — this section is crossed out in the source and kept here for reference only.

```text
# Controller → Radio
AST,RF_POWER_PLOT,[Frequency],[Modulation],[Sampling Rate]\r
# Radio → Controller
AST,OK\r
```

| Parameter | Values |
|---|---|
| `[Frequency]` | `250000`–`13000000` |
| `[Modulation]` | `Auto`, `AM`, `NFM`, `FM`, `WFM`, `FMB` |
| `[Sampling Rate]` | `100`, `200`, `400`, `800` (ms) |

The plot data is reported through the `RfPowerPlot` element of PSI/GSI.

### APR — Analyze Pause/Resume

```text
# Controller → Radio
APR,[Analyze Mode]\r
# Radio → Controller
APR,OK\r
```

`[Analyze Mode]`: `SYSTEM_STATUS`, `RF_POWER_PLOT`, `CURRENT_ACTIVITY`, `LCN_MONITOR`, `ACTIVITY_LOG`, `RAW_DATA_OUTPUT` (the RF power plot and raw data modes are still listed here, although those AST modes are marked removed).

---

## MSI response XML

**MSI** (root element)

| Attribute | Meaning |
|---|---|
| `Name` | Menu title |
| `Index` | Menu index |
| `MenuType` | `TypeSelect` / `TypeInput` / `TypeLocation` / `TypeError` |
| `Value` | Current set value |
| `Selected` | *(no description given)* |

**MenuItem** — `Name` (item name), `Index` (item index), `Value` (item's current value).

**MenuInput** — `MaxLength` (`1`–`64`), `EnableKeys` (characters the user can enter), `AddedInformation` (string).

**MenuLocation** — `MaxLength` (`1`–), `EnableKeys` (characters the user can enter), `IsLatitude` (`"1"` latitude / `"0"` longitude).

**MenuErrorMsg** — `Text` (error message), `ScanButton` (`"1"` enabled / `"0"` disabled).

---

## Service types (SVC)

`non` = unused slot.

| ID | Service type | ID | Service type | ID | Service type |
|---|---|---|---|---|---|
| PST1 | Multi-Dispatch | PST14 | Public Works | PST27 | non |
| PST2 | Law Dispatch | PST15 | Aircraft | PST28 | non |
| PST3 | Fire Dispatch | PST16 | Federal | PST29 | Emergency Ops |
| PST4 | EMS Dispatch | PST17 | Business | PST30 | Military |
| PST5 | non | PST18 | non | PST31 | Media |
| PST6 | Multi-Tac | PST19 | non | PST32 | Schools |
| PST7 | Law Tac | PST20 | Railroad | PST33 | Security |
| PST8 | Fire-Tac | PST21 | Other | PST34 | Utilities |
| PST9 | EMS-Tac | PST22 | Multi-Talk | PST35 | non |
| PST10 | non | PST23 | Law Talk | PST36 | non |
| PST11 | Interop | PST24 | Fire-Talk | PST37 | Corrections |
| PST12 | Hospital | PST25 | EMS-Talk | | |
| PST13 | Ham | PST26 | Transportation | | |

Custom service types (the `[CSTn]` fields of SVC; the sheet labels them `ST1`–`ST10`):

| ID | Name | ID | Name |
|---|---|---|---|
| ST1 | Custom 1 | ST6 | Custom 6 |
| ST2 | Custom 2 | ST7 | Custom 7 |
| ST3 | Custom 3 | ST8 | Custom 8 |
| ST4 | Custom 4 | ST9 | Racing Officials |
| ST5 | Custom 5 | ST10 | Racing Teams |

---

## Key codes (KEY)

| Key code | BCD536HP | SDS100 | Note |
|---|---|---|---|
| `M` | MENU | Menu | Menu key |
| `F` | (rotary knob) | Func | F key |
| `L` | AVOID | AVOID | Avoid key |
| `0`–`9` | 0–9 | 0–9 | Number keys |
| `.` | . NO | . NO | Dot key |
| `E` | E yes | E yes | Enter key |
| `>` | (rotary knob) | (rotary knob) | Rotary right |
| `<` | (rotary knob) | (rotary knob) | Rotary left |
| `^` | (rotary knob) | (rotary knob) | Rotary knob push |
| `V` | VOL | Backlight | Volume knob push |
| `Q` | SQ | (none) | Squelch knob push |
| `Y` | REPLAY | REPLAY | Replay key |
| `A` | SOFT 1 | SOFT 1 | Soft key 1 (BCD436HP: System) |
| `B` | SOFT 2 | SOFT 2 | Soft key 2 (BCD436HP: Dept) |
| `C` | SOFT 3 | SOFT 3 | Soft key 3 (BCD436HP: Channel) |
| `Z` | ZIP | Zip | Zip key |
| `T` | SREV | (none) | Service Type key |
| `R` | RANG | RANG | Range key |

---

## Sub audio (SAS / SAD)

> **Not included in the SDS PDF.** The attribute descriptions still refer to the "Sub Audio" sheet, but this document doesn't contain it. The table below is carried over unchanged from the BCD436HP/BCD536HP specification v1.05, which the SDS document is based on.

`SAS` (Sub Audio Settings) is what a channel is set to decode; `SAD` (Sub Audio Detected) is what was received. Both take the same set of values, except for the top row:

| | All | Analog (CTCSS/DCS) | Digital (P25 NAC / color code / RAN / area) |
|---|---|---|---|
| SAS | `All` | `Tone Search` | `NAC Search` |
| SAD | `None` | `None` | `None` |

**Digital values and their code ranges** (the hex column in the source):

| Code range | Value |
|---|---|
| `0x0000`–`0x0FFF` | NAC `000h`–`FFFh` (P25) |
| `0x1000`–`0x100F` | Color Code `0`–`15` (DMR/MotoTRBO) |
| `0x2000`–`0x203F` | RAN `0`–`63` (NXDN) |
| `0x3000`–`0x3001` | Area `0`–`1` (NXDN IDAS) |

**Analog values — CTCSS (50 tones, Hz):**
67.0, 69.3, 71.9, 74.4, 77.0, 79.7, 82.5, 85.4, 88.5, 91.5, 94.8, 97.4, 100.0, 103.5, 107.2, 110.9, 114.8, 118.8, 123.0, 127.3, 131.8, 136.5, 141.3, 146.2, 151.4, 156.7, 159.8, 162.2, 165.5, 167.9, 171.3, 173.8, 177.3, 179.9, 183.5, 186.2, 189.9, 192.8, 196.6, 199.5, 203.5, 206.5, 210.7, 218.1, 225.7, 229.1, 233.6, 241.8, 250.3, 254.1

**Analog values — DCS (112 codes):**
006, 007, 015, 017, 021, 023, 025, 026, 031, 032, 036, 043, 047, 050, 051, 053, 054, 065, 071, 072, 073, 074, 114, 115, 116, 122, 125, 131, 132, 134, 141, 143, 145, 152, 155, 156, 162, 165, 172, 174, 205, 212, 214, 223, 225, 226, 243, 244, 245, 246, 251, 252, 255, 261, 263, 265, 266, 271, 274, 306, 311, 315, 325, 331, 332, 343, 346, 351, 356, 364, 365, 371, 411, 412, 413, 423, 431, 432, 445, 446, 452, 454, 455, 462, 464, 465, 466, 503, 506, 516, 523, 526, 532, 546, 565, 606, 612, 624, 627, 631, 632, 654, 662, 664, 703, 712, 723, 731, 732, 734, 743, 754

The source does not give numeric codes for the analog values; they appear by name (e.g. `CTCSS 67.0Hz`, `DCS 023`).

---

## Errata notes (conversion)

These are observations about the source text, not part of the original spec:

- **Page footers** — pages are numbered out of both 37 and 38 (leftover from the 38-page BCD536HP/BCD436HP document).
- **Key codes** — the second column is headed "SDS100", but its values (Menu, Func, Backlight, CHAN…) match the old BCD436HP column.
- **GW2** — the command and response are printed as `GWF`, identical to GWF; the binary format suggests a separate `GW2` mnemonic. Verify against hardware.
- **GWF** — the command uses `[TYPE]`, the definition `[FFT_TYPE]`.
- **PWF / GWF** — `[ON/OFF]` values aren't given; "Dispalyed" (sic).
- **GCS** — the response is printed ending with `\n`, unlike the `\r` used everywhere else.
- **VOL / SQL** — the set reply is printed as bare `VOL` / `SQL`; SQL's `[LEVEL]` is described as "Volume Level".
- **GST** — `[MUTE]` values aren't given; "convertered" (sic). The "Normal Mode" description is garbled.
- **STS** — the 9 trailing fields are all labelled `[RSV]`, yet the example shows values (`0,1,0,0,,,0,OFF,3`).
- **Sub Audio** — referenced by SAS/SAD (printed as `" Sub Audio"` with a leading space) but not included in this document.
- **Revision history** — "basd" (based).
- **Command list** — the "Program Mode Only" column is empty throughout; GW2 and KAL are unnumbered.
- **PSI / GSI** — the request syntax (including PSI's interval parameter) is not given; only the XML content is described.
- **KEY** — `[KEY_MODE]` values are not listed in this document.
- **STS** — refers to a separate "Font Data Specification" for non-ASCII codes, which isn't included.
- **NXT / PRV** — the response is labelled (2) although there is no (1).
- **AVD** — the format is printed `[xxx2][STATUS]` with no comma, but the source's own example (`AVD,AFREQ,4060000,,1\r`) uses one.
- **SQK** — the get response includes `[SYS_QK]`, which the set command doesn't.
- **FQK / SQK / DQK** — "radiowill" (missing space).
- **MSB** — `RETURN_PREVOUS_MODE` (sic).
- **MNU** — "Syetm Index", "Clsoe Call Band Menu" typos.
- **GLT** — "Avoding" (sic); the CNV_DISCOV attribute is spelled `Logginig`; the ATGID attribute is lowercase `index`.
- **Target keyword table** — `IREC_FILE`'s NXT/PRV 1st parameter is blank; "Parent Sit Index" for SFREQ; "invarild" for invalid; "Unkown" department.
- **PSI example** — closing `</ScannerInfo>` is missing; `Backlight` appears in the example but not in the Property attribute list.
- **Attributes** — `LVL` is printed `-3/-2-/-1/0/1/2/3` (stray hyphen); `WaterfallBand` `Step` lists `833kHz` (should be 8.33 kHz); `DispFormat` and `CC_Counters` have no attributes documented.
- **Current Activity example** — last line has `Tgidype` for `TgidType`.
- **LCN Monitor** — attribute spelled `ReceiveStaus`.
- **LCN Finder example** — `<LcnFinder …` lines are missing their closing `/>`.
- **Band Scope** — the response field is `[RSSI_LEVEL]`, but its description calls it `Status`.
- **Service types** — custom types are called `CSTn` in SVC but `STn` in the table.
- **Spelling** — "Analize" (Analyze) and "Pauze" (Pause) are used throughout the source.
