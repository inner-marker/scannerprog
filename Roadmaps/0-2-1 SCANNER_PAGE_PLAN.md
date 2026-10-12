# Scanner Page — UI & Architecture Plan

> **Audience:** GitHub Copilot (and humans) implementing the live-control "Scanner" page of the Dioxus scanner app.
> **Status:** Draft v1 — decisions marked **[DECIDED]** are settled; items under [Open questions](#14-open-questions--verify-on-hardware) still need answers.
> **Protocol references:** the Markdown protocol specs in `docs/protocols/` (one per model family — see [§2](#2-supported-models--protocol-families)). When this plan and a spec disagree, the spec wins; note the discrepancy in a code comment.
> **Existing codebase:** this plan was written without seeing the app's code. **Fit it into the existing codebase.** Reuse the app's current structure, naming, state management, transport/serial code, settings storage, styling and components. You may change names, file layout, crate boundaries and type shapes in this plan to do that. What must be preserved is listed in [§15.1](#151-invariants--keep-these). Details are in [§15](#15-notes-for-copilot).

---

## 1. Purpose & scope

The Scanner page is a **universal remote head**: one interface that monitors and controls any supported Uniden scanner connected over USB serial (or network, where the model supports it).

**In scope [DECIDED]:** live control only:

- What is the radio receiving right now (system / group / channel, frequency or talkgroup, signal, squelch)
- Transport-style controls (hold, avoid/lockout, next/previous, scan/resume)
- Volume, squelch level, mute
- Mode switching (scan, searches, Close Call, weather, tone-out, etc.)
- Quick keys (what is enabled for scanning)
- Quick tune to a frequency
- Spectrum / waterfall where the radio can supply it
- An activity log of recent hits (app-side)
- A virtual remote: LCD mirror + keypad

**Out of scope (other pages):** database/memory programming (systems, groups, channels), scanner settings menus, Radio Reference integration, firmware tools, HomePatrol "Analyze" tools.

**Platforms [DECIDED]:** desktop (Linux, macOS, Windows) with mouse + keyboard, **and** laptops with touch screens. Mobile is not a target (USB limitations). Design for 1280×800 first; degrade gracefully down to ~900 px wide; every control must be usable by touch (≥ 40 px targets) with no hover-only information.

**Universality rule [DECIDED]:** every feature control is always **visible**. If the connected model can't do it, the control is **disabled with an explanation** (see [§7](#7-disabled-state-behavior)) — never hidden. The layout must not shift between models.

---

## 2. Supported models & protocol families

Each model belongs to one **protocol family**. Drivers are written per family; per-model differences are data (a `ModelProfile`), not code branches in the UI.

| Family | Models | Protocol style | Status source | Spec file(s) |
|---|---|---|---|---|
| **A — Legacy** | BCT8 | Two-letter commands, space-separated fixed-width fields, `N`/`F` toggles; unsolicited event messages | `LCD` reads + unsolicited `ID`/`QU`/`RI`/`HA`/`HP` | `BCT8_Remote_Command_Spec.md` |
| **B1 — Early comma** | BR330T | Comma-separated; no sites; Program Mode for memory | `GLG`, `GID`, `STS` | `BR330T_Remote_Command_Spec.md` |
| **B2 — Gen-1 comma** | BCD996T, BCT15 | As B1 + sites, `QSC`/`CSC`/`PWR`, `JPM`, `MNU` | `GLG`, `GID`, `STS`, `PWR` | `BCD996T_…`, `BCT15_…` |
| **C — XT / P2** | BCD396XT, BC346XT, BCD996XT, BCT15X, BCD325P2, BCD996P2 | Comma-separated; number tags; `JNT`; `VOL`/`SQL` | `GLG`, `GID`, `STS`, `PWR` | `BCD396XT_…`, `BC346XT_…`, `BCD996XT_…`, `BCT15X_…`, `BCD325P2_…`, `BCD996P2_…` |
| **D — HomePatrol** | BCD436HP, BCD536HP | Commands + **XML** (`PSI`/`GSI`/`GLT`/`MSI`); index handles; native `HLD`/`AVD`/`NXT`/`PRV` | `PSI` push (XML) | `BCDx36HP_Remote_Command_Spec.md` |
| **D′ — SDS** | SDS100, SDS150, SDS200 | Family D + waterfall (`GST`/`PWF`/`GWF`/`GW2`), `KAL`, `POF`, `GCS`, `VOL`/`SQL` | `PSI` push + `GST` | `SDS_Series_Remote_Command_Spec.md` |

**Identification on connect:**

1. Send `MDL\r`. Families B–D′ reply `MDL,<model>` (e.g. `MDL,BCD325P2`, `MDL,SDS150GBT`).
2. If that gives `ERR`/no reply, send `SI\r`. BCT8 replies `SI BCT8,<esn>,<cmd-version>`.
3. Then `VER\r` (families B–D′) or `VR\r` (BCT8) for the firmware string shown in the connection bar.
4. Map the model string to a `ModelProfile` (§8). Unknown model → connect read-only with a "generic" profile of the closest family and show a warning banner.

Baud rate: try the last-used rate first, then autodetect across each family's documented rates (BCT8: 2400–57600; others: 4800/9600–115200).

---

## 3. Design principles

1. **Actions, not commands.** The UI only ever emits universal `Action`s (`Hold`, `Avoid`, `Next`, `SetVolume(n)`…). Each family driver translates an action into its native command, or into key-press emulation, or reports it unsupported.
2. **One status model.** Every driver produces the same normalized `LiveState` (§8). Fields a model can't supply are `None`, and the UI renders them as "—" (not as zero).
3. **Capabilities are data.** `ModelProfile` declares, per `Capability`, whether it's `Native`, `Emulated` (via key presses — works but gets no confirmation), `NeedsProgramMode`, or `Unsupported(reason)`. UI controls bind to a capability and derive their enabled/disabled state from it plus the current `LiveState`.
4. **Never surprise the radio.** Anything that pauses scanning or changes mode (Program Mode, sweep/spectrum, quick tune) says so on the control *before* it is used.
5. **Readable at arm's length.** The now-playing card is the hero: large type and high contrast; it works in a dim room.
6. **Keyboard first, touch friendly.** Every primary action has a shortcut (§11) and a ≥ 40 px touch target.

---

## 4. Page layout

### 4.1 Wide layout (≥ 1280 px) — three columns

```text
┌──────────────────────────────────────────────────────────────────────────────────┐
│ ConnectionBar: [● BCD325P2 ▾] /dev/ttyACM0 · FW 1.02 · 115200   [Disconnect] [⋯] │
├──────────────────────────────────────────────────────────────────────────────────┤
│ ModeBar: [Scan] [Service] [Custom] [Close Call] [Quick Search] [WX] [Tone-Out]   │
│          [Discovery] [Waterfall]                   ProgramModeBadge (when active)│
├───────────────────────────────┬───────────────────────────┬──────────────────────┤
│ NowPlayingCard                │ ActivityLog               │ AudioPanel           │
│  SYSTEM  › DEPT/GROUP         │  14:02:11  Fire Disp  …   │  Volume  ───●────  9 │
│  CHANNEL NAME (large)         │  14:01:57  PD Main   …    │  Squelch ──●─────  3 │
│  851.0125 MHz · TGID 1234     │  …                        │  [Mute]              │
│  NFM · NAC 293 · Fire Disp    │                           ├──────────────────────┤
│  Signal ▮▮▮▯▯   ● SQUELCH OPEN │                           │ QuickKeysPanel       │
│                               │                           │  [Systems|Groups|FL] │
│ TransportBar                  │                           │  ▢1 ▣2 ▢3 … (grid)   │
│  [◀ Prev] [⏸ Hold] [Next ▶]   │                           │  Page 0–9 ◀ ▶        │
│  [⊘ Avoid ▾] [⟳ Scan/Resume]  │                           ├──────────────────────┤
│                               │                           │ QuickTune            │
│ HoldLevelSelector (D/D′ only) │                           │  [ 851.0125 ] MHz    │
│  (System · Dept · Channel)    │                           │  Mod [Auto▾] [Tune]  │
│                               │                           │ JumpToTag            │
│                               │                           │  Sys [  ] Ch [  ] Go │
├───────────────────────────────┴───────────────────────────┴──────────────────────┤
│ BottomDrawer (collapsible, remembers state): [Spectrum] [Virtual Remote] tabs    │
│  Spectrum: waterfall / sweep canvas      Virtual Remote: LCD mirror + keypad     │
└──────────────────────────────────────────────────────────────────────────────────┘
│ StatusStrip: battery · WX alert · GPS · last poll latency · errors (click → log) │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Medium layout (900–1279 px) — two columns

- Left: NowPlayingCard + TransportBar (unchanged).
- Right: one tabbed panel: **Controls** (Audio + QuickTune + JumpToTag) · **Quick Keys** · **Activity**.
- BottomDrawer unchanged.

### 4.3 Narrow layout (< 900 px) — single column

Stack in this order: ConnectionBar, NowPlayingCard, TransportBar, ModeBar (horizontally scrollable), tabbed panel (Controls · Quick Keys · Activity · Spectrum · Remote). Still keyboard- and touch-usable; not a mobile target.

### 4.4 Layout rules

- Panels never appear or disappear based on model; only their contents' enabled state changes.
- The NowPlayingCard height is fixed per breakpoint so live updates don't make the layout jump.
- The BottomDrawer opens to a resizable height (default 280 px, min 160, max 60 % of the window). Persist the open/closed state, the active tab and the height per window.

---

## 5. Components

For each component: **purpose**, **data**, **actions**, **capabilities used**, **states**.

### 5.1 `ConnectionBar`

- **Purpose:** choose, connect and identify the radio.
- **Data:** discovered ports (serial) and saved network endpoints; connection state; model, firmware, baud.
- **Actions:** connect, disconnect, rescan ports, open the connection log.
- **States:** Disconnected (radio picker prominent) · Connecting/Identifying (spinner + step text) · Connected (green dot) · Degraded (amber: timeouts > threshold) · Lost (red; auto-retry countdown).
- The picker shows one radio at a time [DECIDED]. Internally keep a `HashMap<ConnectionId, RadioSession>` so a side-by-side view can be added later without refactoring.

### 5.2 `ModeBar`

- **Purpose:** switch scanner mode.
- **Buttons (always shown):** Scan · Service Search · Custom Search · Close Call · Quick Search · Weather · Tone-Out · Discovery · Waterfall · Highway (BCT8).
- **Active mode** is highlighted from `LiveState.mode`.
- **Mechanism per family:** C/B2 → `JPM` (`SCN_MODE`, `SVC_MODE`, `CTM_MODE`, `CC_MODE`, `WX_MODE`, `FTO_MODE`); D/D′ → `JPM` (adds `QSH_MODE`, `TDIS_MODE`, `CDIS_MODE`; `WF_MODE` on SDS); B1 → key emulation; A → `SV` / `HW` / key emulation.
- Service Search opens a small popover to pick the service (C/B2: the `SVC_MODE` index list; D/D′: service types are toggled via `SVC`, not jumped to).
- `JPM` returning `NG` → toast "The scanner refused the mode change (it may be in a menu)".

### 5.3 `NowPlayingCard` (hero)

- **Data (from `LiveState`):** system/site name, group/department name, channel name, frequency **or** TGID (show both when known), modulation, tone/NAC/color code/RAN, service type (D/D′), number tags (C, D), signal level, squelch-open flag, hold/avoid flags, mode label.
- **Visual:** three text tiers (system › group small, channel large, details medium). A squelch-open indicator (pulsing dot plus a subtle card glow) shows when audio is present. Show a "HOLD" badge when held and an "AVOIDED"/"T-AVOID" badge on avoided items (D/D′).
- **Empty states:** "Scanning…" (no current hit), "Not connected", "In menu" (when `LiveState.in_menu`).
- **Signal meter:** normalized to 0–5 bars. Sources: C/B2 `PWR` RSSI (0–1023) mapped to bars, or `STS` `SIG_LVL`; D/D′ `Property Sig` (0–4) and `Rssi`. A/B1 → "—" with the disabled tooltip.

### 5.4 `TransportBar`

| Control | Capability | A (BCT8) | B1 (BR330T) | B2 / C | D / D′ |
|---|---|---|---|---|---|
| ◀ Prev | `StepPrev` | Emulated `KEY08` (▼) | Emulated `KEY,<,P` | Emulated `KEY,<,P` | Native `PRV,tkw,…` |
| ⏸ Hold / Resume | `Hold` | Emulated `KEY00` | Emulated `KEY,H,P` | Emulated `KEY,H,P` | Native `HLD,tkw,…` |
| Next ▶ | `StepNext` | Emulated `KEY07` (▲) | Emulated `KEY,>,P` | Emulated `KEY,>,P` | Native `NXT,tkw,…` |
| ⊘ Avoid | `Avoid` | Emulated `KEY06` | Emulated `KEY,L,P` | Emulated `KEY,L,P` | Native `AVD,…` with menu: Temporary / Permanent / Stop avoiding |
| ⟳ Scan | `ResumeScan` | Emulated `KEY00` | Emulated `KEY,S,P` | Emulated `KEY,S,P` | Native `JPM,SCN_MODE,0xFFFFFFFF` or release hold (verify §14) |

- **HoldLevelSelector** (D/D′ native; others disabled with "Only HomePatrol/SDS can hold at a chosen level"): segmented control *System · Department · Channel* that sets the `tkw` used by Hold/Next/Prev. Indices come from the latest PSI elements; NXT/PRV take a `COUNT` (1–8), and long-press repeats.
- The Avoid button on D/D′ is a split button: the click avoids temporarily, the chevron offers Permanent / Stop avoiding. On other families it's a plain button (L/O key).

### 5.5 `ActivityLog` (app-side, universal)

- An entry is appended when a new hit starts (the squelch-open edge with a changed channel/TGID). It records the timestamp, system, group, channel, freq/TGID, duration (closed on the squelch-close edge) and peak signal.
- Columns: Time · Channel · System/Group · Freq/TGID · Duration. Clicking a row offers *Hold on this* (D/D′ via indices; others: Quick-tune the frequency) and *Avoid*.
- Keep the last 500 entries in memory; export to CSV. Persisting across sessions is optional (phase 3).
- Family A: driven by unsolicited `ID S/E` and `QU +/-` events, which gives exact start/end times.

### 5.6 `AudioPanel`

- **Volume slider** (`SetVolume`): range from `ModelProfile.volume_range`. Native `VOL` on B1/B2/C/D′ (0–15 or 0–29 per model). **Disabled on BCT8 and BCD436HP/536HP** (no volume command). Debounce at 150 ms; show the value numerically.
- **Squelch slider** (`SetSquelch`): `SQL` with per-model range and endpoint labels ("Open" at 0, "Closed" at max where the spec defines it).
- **Mute toggle** (`Mute`): BCT8 native `MUN`/`MUF`/`MUA` (shows the three states); elsewhere emulated as volume → 0 and restore (disabled where volume is unsupported).

### 5.7 `QuickKeysPanel`

- **Tabs:** Favorites Lists (D/D′) · Systems · Groups/Departments · Banks (BCT8).
- **Grid:** 10 keys per page × 10 pages (system/FL/dept keys 0–99) or a single row of 10 (groups). Key states: *not assigned* (outlined, inert) · *enabled* (filled) · *disabled* (struck/dim).
- **Mechanism:**
  - D/D′: `FQK` / `SQK,<fl>` / `DQK,<fl>,<sys>`, applied live and instantly.
  - B1/B2/C: `QSL` / `QGL,<sys_index>` — **Program Mode only**. Edits are staged locally; an "Apply (pauses scanning)" button runs `PRG` → writes → `EPG` → `KEY,S,P` (EPG leaves the radio in Scan Hold, so resume explicitly). While staged, show a pending-changes bar.
  - A (BCT8): `SB` banks A–E (live, no Program Mode).
- The Groups tab needs a system context: on C/B use the current system index from `GID`/`SIN` lookup (phase 2); disable until known.

### 5.8 `QuickTune`

- A frequency input (MHz, validated against `ModelProfile.freq_ranges`) plus a modulation dropdown (where the model accepts it) and a **Tune** button.
- Mechanism: B/C → `QSH,<frq>,…` (empty fields keep the radio's current options); D/D′ → `QSH,<frq>`; A → `RF<frq>`.
- The label states "Switches the scanner to Quick Search hold".

### 5.9 `JumpToTag`

- Inputs: Favorites-List tag (D/D′ only), System tag, Channel tag, and a **Go** button. C: `JNT,<sys>,<chan>`; D/D′: `JNT,<fl>,<sys>,<chan>`. Disabled on A/B ("Number tags not supported").

### 5.10 `BottomDrawer` → `SpectrumView`

| Family | Source | Behavior |
|---|---|---|
| D′ (SDS) | `GWF`/`PWF` (240-point FFT), `GST` for center/span/marker | True waterfall: scrolling canvas + current-spectrum line; click to set marker; shows CF/Lower/Upper |
| D (x36HP) | `AST,BAND_SCOPE,…` stream (`AST,BAND_SCOPE,<freq>,<rssi>`) | Spectrum line built from sweeps; waterfall assembled app-side |
| C / B2 | `CSC,ON` stream (`CSC,<rssi>,<frq>,<sql>`) | "Sweep" view of the active custom search range; **changes the radio to Custom Search** (warn first) |
| A / B1 | — | Disabled: "This scanner can't stream signal data" |

Render on a `<canvas>` (via `document::eval` / JS interop or a WebGPU/2D canvas crate). Cap at 30 fps; colormap selectable (match SDS `COLOR_MODE` where relevant).

### 5.11 `BottomDrawer` → `VirtualRemote`

- **LCD mirror:** renders `STS`/`GST` lines in a monospace grid, honoring the per-line font size (`DSP_FORM`) and per-character modes (reverse `*`, underline `_`). Family A: `LCD` reads (`LCD CHN`, `LCD FRQ`, icons). Map non-ASCII glyph codes via the font tables in the specs (icons → small SVGs; phase 3).
- **Keypad:** generated from `ModelProfile.keymap` (key code → label → grid position), so each family shows its real keys. Keys the model lacks are shown disabled. Supports press, long-press (`L`) and hold/release (`H`/`R`) gestures; touch long-press maps to `L`.
- Polling `STS` is expensive, so poll it only while this tab is visible.

### 5.12 `StatusStrip`

Battery (STS `BAT` flag on handhelds; D `Property Battery`; SDS `GCS` voltage/percent/charging), weather alert (STS `WAT`, D `WxMode`), highway alert (BCT8 `HA`/`HP`), GPS fix (996XT/996P2 `GGA`/`RMC`; D `LCR`), poll latency, and an error counter (opens the connection log).

### 5.13 `ProgramModeBadge`

Visible whenever the driver is in Program Mode (radio shows "Remote Mode / Keypad Lock"). Has a **Leave** button. The driver must always exit Program Mode on disconnect, app close and panic paths.

---

## 6. Capability matrix

Legend: ✅ native · 🔁 emulated via key presses · 🔒 needs Program Mode · ⛔ unsupported. Per-model exceptions are in parentheses. Derived from the converted specs — **verify on hardware**.

| Capability | A BCT8 | B1 BR330T | B2 996T / BCT15 | C XT / P2 | D x36HP | D′ SDS |
|---|---|---|---|---|---|---|
| Identify (model/firmware) | ✅ `SI`/`VR` | ✅ | ✅ | ✅ | ✅ | ✅ |
| Now playing (names, freq) | ✅ `LCD` + `ID` events | ✅ `GLG`/`GID` | ✅ | ✅ | ✅ `PSI` | ✅ `PSI` |
| Signal meter | ⛔ | ⛔ | ✅ `PWR`/`STS` | ✅ `PWR`/`STS` | ✅ | ✅ |
| Squelch-open indicator | ✅ `QU` events | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hold / Resume | 🔁 | 🔁 | 🔁 | 🔁 | ✅ `HLD` | ✅ `HLD` |
| Hold at level (sys/dept/chan) | ⛔ | ⛔ | ⛔ | ⛔ | ✅ | ✅ |
| Next / Prev | 🔁 | 🔁 | 🔁 | 🔁 | ✅ `NXT`/`PRV` | ✅ |
| Avoid (temp/perm) | 🔁 L/O only | 🔁 L/O only | 🔁 L/O only | 🔁 L/O only | ✅ `AVD` | ✅ `AVD` |
| Volume | ⛔ | ✅ 0–15 | ✅ 0–29 | ✅ 0–15 handhelds / 0–29 mobiles | ⛔ | ✅ 0–15 (SDS100/150) / 0–29 (SDS200) |
| Squelch level | ⛔ | ✅ | ✅ | ✅ | ⛔ | ✅ |
| Mute | ✅ `MU` | 🔁 vol→0 | 🔁 | 🔁 | ⛔ | 🔁 |
| Mode switch | ✅/🔁 `SV`, `HW` | 🔁 | ✅ `JPM` | ✅ `JPM` | ✅ `JPM` | ✅ `JPM` (+`WF_MODE`) |
| Quick keys | ✅ `SB` banks | 🔒 `QSL`/`QGL` | 🔒 | 🔒 | ✅ `FQK`/`SQK`/`DQK` | ✅ |
| Quick tune | ✅ `RF` | ✅ `QSH` | ✅ `QSH` | ✅ `QSH` | ✅ `QSH` | ✅ `QSH` |
| Jump to number tag | ⛔ | ⛔ | ⛔ | ✅ `JNT` | ✅ `JNT` | ✅ `JNT` |
| Spectrum / waterfall | ⛔ | ⛔ | ✅ sweep `CSC` (mode change) | ✅ sweep `CSC` (mode change) | ✅ `AST,BAND_SCOPE` | ✅ waterfall `GWF`/`PWF` |
| LCD mirror | ✅ `LCD` | ✅ `STS` | ✅ | ✅ | ✅ `STS` | ✅ `STS`/`GST` |
| Virtual keypad | ✅ `KEYnn` | ✅ | ✅ | ✅ | ✅ | ✅ |
| Record | ⛔ | ⛔ | ⛔ | ⛔ | ✅ `URC` | ✅ `URC` |
| Battery | ⛔ | ✅ flag | ⛔ (mobiles) | ✅ flag (handhelds) | ✅ | ✅ `GCS` (detail) |
| Weather alert | ⛔ (highway alerts instead) | ✅ | ✅ | ✅ | ✅ | ✅ |
| Power off | ⛔ | ✅ `POF` | ⛔ | ✅ (396XT, 346XT, 325P2) / ⛔ (996XT, 996P2, BCT15X) | ⛔ | ✅ `POF` |

Notes:

- C-family `POF` availability differs per model; take it from `ModelProfile`, not the family.
- B2/C spectrum uses `CSC`, which **switches the radio into Custom Search**. The Spectrum tab shows a confirm step and restores the previous mode on close (best effort).
- Store volume and squelch ranges per model in `ModelProfile`; never hard-code them in components.

---

## 7. Disabled-state behavior

A control can be disabled for one of three reasons, and each looks different:

| Reason | Visual | Explanation text (template) |
|---|---|---|
| **Unsupported by model** | 40 % opacity, small "⛔" corner glyph | "Not available on {model}." + one-line why, e.g. "The BCT8 has no remote volume command." |
| **Not available right now** | 60 % opacity, no glyph | Context, e.g. "Connect a scanner first", "Exit the scanner's menu first", "No talkgroup to hold". |
| **Needs Program Mode** | Normal look + 🔒 glyph | Enabled; clicking shows the warning "This pauses scanning while changes are written." |

- **Mouse:** a tooltip appears on hover after 300 ms.
- **Touch / keyboard:** tapping or focusing a disabled control shows the same text in an inline popover (no hover needed). Disabled controls stay focusable (`aria-disabled="true"`, not the `disabled` attribute) so the explanation is reachable.
- **Emulated** capabilities look identical to native ones, but the action toast says "Sent key press" instead of a confirmation, since no acknowledgment is available.

---

## 8. Core types (Rust)

Put these in UI-independent code (e.g. a `scanner-core` crate or an existing non-UI module) so drivers can be tested without Dioxus. These shapes are illustrative. If the codebase already has equivalent types, extend or adapt those instead of duplicating them (see [§15.2](#152-flexible--adapt-freely-to-the-existing-code)).

```rust
// scanner-core/src/model.rs

/// Protocol family; selects the driver implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Legacy,      // A: BCT8
    EarlyComma,  // B1: BR330T
    Gen1,        // B2: BCD996T, BCT15
    Xt,          // C: BCD396XT, BC346XT, BCD996XT, BCT15X, BCD325P2, BCD996P2
    HomePatrol,  // D: BCD436HP, BCD536HP
    Sds,         // D′: SDS100, SDS150, SDS200
}

/// Every feature the Scanner page can bind a control to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    SignalMeter, Hold, HoldAtLevel, StepNext, StepPrev, Avoid, AvoidTemporary,
    ResumeScan, Volume, Squelch, Mute, ModeSwitch, QuickKeysSystems,
    QuickKeysGroups, QuickKeysFavorites, QuickTune, JumpToTag, Spectrum,
    Waterfall, LcdMirror, Keypad, Record, Battery, WeatherAlert, PowerOff,
}

/// How (or whether) a model supports a capability.
#[derive(Clone, Debug, PartialEq)]
pub enum Support {
    Native,                 // dedicated command with acknowledgment
    Emulated,               // achieved via KEY presses; no confirmation
    NeedsProgramMode,       // must enter PRG; pauses scanning
    Unsupported(&'static str), // reason shown in the disabled tooltip
}

/// Static per-model data; one entry per supported model.
pub struct ModelProfile {
    pub model: &'static str,                  // exact MDL/SI string, e.g. "BCD325P2"
    pub family: Family,                       // driver selector
    pub support: fn(Capability) -> Support,   // capability lookup
    pub volume_range: Option<(u8, u8)>,       // None = no remote volume
    pub squelch_range: Option<(u8, u8)>,      // None = no remote squelch
    pub freq_ranges_hz: &'static [(u64, u64)],// valid tune ranges
    pub keymap: &'static [KeyDef],            // keypad layout for VirtualRemote
    pub baud_rates: &'static [u32],           // autodetect candidates
}

/// Normalized live state produced by every driver; None = "model can't tell us".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LiveState {
    pub mode: Option<ScanMode>,               // current scanner mode
    pub system: Option<String>,               // system / site / search name
    pub group: Option<String>,                // group or department
    pub channel: Option<String>,              // channel or TGID alpha tag
    pub freq_hz: Option<u64>,                 // current frequency
    pub tgid: Option<String>,                 // talkgroup, formatted per system type
    pub modulation: Option<String>,           // AM/FM/NFM/WFM/FMB/Auto
    pub sub_audio: Option<String>,            // CTCSS/DCS/NAC/color code/RAN
    pub service_type: Option<String>,         // D/D′ only
    pub signal_bars: Option<u8>,              // normalized 0–5
    pub squelch_open: bool,                   // audio present
    pub held: Option<bool>,                   // hold state if known
    pub avoided: Option<AvoidState>,          // D/D′ only
    pub volume: Option<u8>,
    pub squelch: Option<u8>,
    pub muted: Option<bool>,
    pub in_menu: bool,                        // disables most actions
    pub program_mode: bool,                   // radio is in PRG
    pub indices: Option<HpIndices>,           // D/D′ handles for HLD/AVD/NXT/PRV
    pub battery: Option<Battery>,
    pub alerts: Vec<Alert>,                   // WX, highway, emergency
    pub display: Option<DisplayFrame>,        // LCD mirror lines (when polled)
}

/// Universal UI intents; drivers translate these to wire commands.
#[derive(Clone, Debug)]
pub enum Action {
    Hold, ResumeScan, StepNext { count: u8 }, StepPrev { count: u8 },
    Avoid(AvoidKind), SetHoldLevel(HoldLevel), SetVolume(u8), SetSquelch(u8),
    SetMute(bool), JumpMode(ScanMode), QuickTune { freq_hz: u64, modulation: Option<String> },
    JumpToTag { fl: Option<u8>, sys: Option<u8>, chan: Option<u16> },
    SetQuickKeys(QuickKeyEdit), Key { code: String, mode: KeyMode },
    StartSpectrum, StopSpectrum, PowerOff,
}
```

Driver trait:

```rust
// scanner-core/src/driver.rs

/// One implementation per Family.
#[async_trait::async_trait]
pub trait Driver: Send {
    /// Profile of the identified model (capabilities, ranges, keymap).
    fn profile(&self) -> &'static ModelProfile;
    /// Translate and send a UI action; Err carries user-facing text.
    async fn perform(&mut self, action: Action) -> Result<ActionOutcome, DriverError>;
    /// One polling tick; returns updated state (drivers merge pushed/unsolicited data).
    async fn poll(&mut self, want: PollWants) -> Result<LiveState, DriverError>;
    /// Must leave Program Mode and stop any streams (CSC/PWF/PSI).
    async fn shutdown(&mut self) -> Result<(), DriverError>;
}

/// What the UI currently needs, so drivers skip expensive polls.
pub struct PollWants {
    pub display: bool,   // VirtualRemote visible → poll STS/LCD
    pub spectrum: bool,  // Spectrum visible → keep stream running
}
```

---

## 9. Connection & polling architecture

- **Transport:** a `Transport` trait with `SerialTransport` (via `serialport` / `tokio-serial`) and `NetTransport` (for SDS200 Ethernet — confirm the protocol details first, see §14). Line framing: split on `\r`; also accept `\n` (SDS `GCS` ends in `\n`); and handle multi-line XML replies (`PSI`/`GSI`/`GLT`/`MSI` start `XXX,<XML>,` and end at the closing root tag).
- **One command in flight.** Every spec requires waiting for a reply before the next command. The driver owns a queue: UI actions jump ahead of routine polls; polls are dropped (not queued) while an action is pending.
- **Unsolicited/streamed messages** are routed by prefix before reply matching: BCT8 `ID S/E`, `+`/`-`, `PST`/`PRT`, `HA+`, `HP ALERT`; `CSC,…` stream; `PWF,…`; `AST,…`; PSI pushes. Replies to commands that stream (`CSC,ON`, `PWF`) must not block the queue.
- **Polling strategy by family:**
  - A: event-driven (`IDN`, `QUN`, `RIN` on connect) + `LCD` every 500 ms while the remote is visible.
  - B1: `GLG` every 250 ms; `STS` only if the display is wanted.
  - B2/C: `GLG` every 200 ms, `PWR` every 400 ms, `GID` on hit change, `STS` only if the display is wanted, `VOL`/`SQL` every 5 s.
  - D/D′: start `PSI` push (interval ~250–500 ms; syntax to verify) and parse the XML; `GSI` on demand; SDS: `GST` while the remote/waterfall is visible; `KAL` every few seconds (interval to verify).
- **Timeouts:** 1 s default per command, 3 s for Program Mode writes; 3 consecutive timeouts → Degraded; 10 → Lost, then auto-reconnect with backoff.
- **Errors:** map `ERR`/`NG`/`FER`/`ORER` to `DriverError` variants with user-facing text.
- **Logging:** keep a ring buffer of the last 1,000 TX/RX lines, viewable from the ConnectionBar "⋯" menu (very useful for debugging the spec discrepancies listed in each spec's errata).

---

## 10. Dioxus structure

> **Suggested, not required.** This is the layout to use if nothing exists yet. If the app already has its own conventions (an existing `scanner`/`radio` module, a serial or USB layer, a global store, a router layout, a settings module), follow those and map the pieces below onto them. A separate `scanner-core` crate is optional. A module inside the existing crate is fine if it doesn't depend on Dioxus.

```text
src/
  pages/scanner/
    mod.rs              // ScannerPage: layout + breakpoints
    connection_bar.rs
    mode_bar.rs
    now_playing.rs
    transport_bar.rs
    activity_log.rs
    audio_panel.rs
    quick_keys.rs
    quick_tune.rs
    jump_to_tag.rs
    bottom_drawer.rs     // tabs: SpectrumView, VirtualRemote
    spectrum.rs
    virtual_remote.rs
    status_strip.rs
    capability.rs        // CapButton / CapSlider wrappers implementing §7
  state/
    radio_session.rs     // owns Driver; coroutine loop
scanner-core/            // UI-free crate: model.rs, driver.rs, families/{legacy,early,gen1,xt,hp,sds}.rs, transport.rs, parse/
```

- **State:** `RadioSession` runs as a `use_coroutine` that owns the driver and receives `Action`s over its channel. It publishes `Signal<LiveState>`, `Signal<ConnectionStatus>`, `Signal<Vec<ActivityEntry>>` and `Signal<Option<SpectrumFrame>>` through context (`use_context_provider`).
- **Capability wrappers:** `CapButton { cap: Capability, on_click, .. }` reads the session profile plus `LiveState` and computes enabled/disabled/explanation per §7. **All feature controls use these wrappers**, so the disabled behavior stays consistent.
- **Spectrum rendering:** a dedicated component that receives frames through a signal and draws to a canvas; throttle to 30 fps.
- **Persistence:** drawer state, last connection and baud go into the app's existing settings file.

---

## 11. Keyboard shortcuts & touch

| Key | Action |
|---|---|
| `Space` | Hold / Resume |
| `→` / `←` | Next / Previous (hold `Shift` for ×5 on D/D′) |
| `A` | Avoid (D/D′: temporary; `Shift+A` permanent) |
| `S` | Resume scan |
| `↑` / `↓` | Volume up / down |
| `Shift+↑` / `Shift+↓` | Squelch up / down |
| `M` | Mute toggle |
| `T` | Focus QuickTune |
| `J` | Focus JumpToTag |
| `1`–`0` | Toggle quick key 1–10 on the current page (in the Quick Keys panel) |
| `` ` `` | Toggle the bottom drawer |
| `Ctrl+L` | Open the connection log |

- Shortcuts are disabled while a text input has focus. Show them in tooltips ("Hold (Space)").
- **Touch:** long-press = keypad long press (`L`) and NXT/PRV repeat; swipe left/right on the NowPlayingCard = Next/Previous; no hover-only UI.

---

## 12. Visual design

- **Theme:** dark by default (night use), light theme supported; follow the OS preference with a manual override.
- **Type scale:** channel name 32–40 px semibold; system/group 14–16 px uppercase-tracked; details 16–18 px tabular numerals for frequencies/TGIDs.
- **Color semantics:** squelch-open/active = green accent; hold = amber; avoided = red-muted; Program Mode = violet badge; disabled per §7.
- **Motion:** squelch indicator pulse ≤ 1 Hz; no layout animation on state updates (avoid jitter during fast scanning).
- **Density:** 8 px spacing grid; 40 px minimum touch targets; cards with 12 px radius.
- **Accessibility:** WCAG AA contrast; every icon button has a text label or `aria-label`; the live region announces channel changes only when held (avoid screen-reader spam while scanning).

---

## 13. Build phases & acceptance criteria

1. **Phase 1 — Skeleton + C family (your BCD325P2).** ConnectionBar, identification, `LiveState` from `GLG`/`PWR`/`GID`, NowPlayingCard, TransportBar (emulated), AudioPanel (`VOL`/`SQL`), ModeBar (`JPM`), QuickTune (`QSH`), JumpToTag (`JNT`), ActivityLog, and capability wrappers rendering disabled states for everything else.
   *Done when:* the BCD325P2 can be monitored and controlled for an hour without desync, and every unsupported control shows the correct explanation.
2. **Phase 2 — Families B1/B2/C complete.** QuickKeysPanel with staged Program-Mode apply, VirtualRemote (STS mirror + keypad from keymaps), CSC sweep spectrum, StatusStrip.
   *Done when:* every B/C model profile exists, and a mock driver replaying captured transcripts passes UI tests.
3. **Phase 3 — Families D/D′.** PSI XML parser, native HLD/AVD/NXT/PRV with HoldLevelSelector, FQK/SQK/DQK live quick keys, SDS waterfall (GWF/PWF), GCS battery, URC record.
4. **Phase 4 — Family A (BCT8).** Space-separated parser, unsolicited events, `LCD` mirror, `SB` banks, `MU` mute.
5. **Phase 5 — Polish.** Glyph rendering for non-ASCII LCD characters, activity-log persistence/export, multi-radio groundwork.

**Testing:** a `MockTransport` that replays recorded TX/RX transcripts per model (store them in `scanner-core/tests/fixtures/<model>/`), plus unit tests for each parser and for `ModelProfile` capability tables.

---

## 14. Open questions / verify on hardware

1. **SDS200 network control:** transport and port aren't described in the SDS spec; confirm before building `NetTransport`.
2. **`PSI` request syntax and interval units** (D/D′) — not given in the specs.
3. **`KAL` cadence** — how often the SDS needs a keep-alive.
4. **Resume after `HLD` on D/D′** — whether a second `HLD` releases hold, or `JPM,SCN_MODE` is required.
5. **`GW2` mnemonic** — the SDS spec prints it as `GWF`.
6. **`KEY` reply form** — the specs show both `KEY,OK` and bare `OK`; accept both.
7. **BCT15X `SIN` `[STATE]` position** — differs between set and get in the spec (only matters to the programming page, but log it).
8. Whether to expose HomePatrol **Analyze** tools on this page later (currently out of scope).
9. Whether **side-by-side multi-radio** is wanted in a later version (architecture already allows it).

---

## 15. Notes for Copilot

**This plan describes the target behavior, not a required code shape.** It was written without access to the existing codebase. Before you implement anything, read the relevant existing code: pages and routing, state management, any existing scanner, serial or USB code, settings and credential storage, shared UI components and styles. Then fit the plan to that code.

### 15.1 Invariants — keep these

These are what the user actually cares about. Keep them however the code is organized:

1. **Controls are never hidden by model.** Unsupported features stay visible but disabled, with an explanation. The three disabled reasons from §7 stay visually distinct. The layout doesn't shift between models.
2. **Capability-driven UI.** Whether a control is enabled comes from a per-model capability table (Native / Emulated / NeedsProgramMode / Unsupported with a reason). Don't scatter `if model == …` checks through components. Route every feature control through one shared wrapper or helper.
3. **Universal actions.** Components send model-agnostic actions. Translating them into each family's commands happens in the driver layer.
4. **Protocol code stays out of UI components.** Parsing and command building live in a layer that doesn't depend on Dioxus and can be unit-tested.
5. **Serial discipline.** Only one command is in flight per radio. Unsolicited and streamed messages are routed before reply matching.
6. **Program Mode safety.** The user is warned before any action that enters Program Mode. The driver always leaves Program Mode on disconnect, app close and error paths, and resumes scanning afterwards where the plan says to.
7. **Specs win.** When a protocol spec and this plan conflict, follow the spec and leave a `// SPEC:` comment explaining the difference.
8. **Inline comments in all code**, including notes on protocol quirks that cite the spec section.

### 15.2 Flexible — adapt freely to the existing code

- Component, module, type and function names. Treat the names in this plan as suggestions.
- File and folder layout (§10), and whether there's a separate `scanner-core` crate.
- The exact shapes of `Family`, `Capability`, `Support`, `ModelProfile`, `LiveState`, `Action` and `Driver` (§8). Merge them with existing types, or extend those types, rather than duplicating them.
- State plumbing: use the app's existing pattern (context, global signals, a store, etc.) instead of the `use_coroutine` design if one already exists.
- Transport, settings, logging, error types, theming and icons: reuse what's there.
- Build-phase order (§13), if existing code makes another order easier.

### 15.3 When you deviate

- Prefer the existing code's conventions over this plan's names and structure.
- Don't rewrite or rename working code just to match this plan unless that's needed to keep an invariant.
- Leave a short `// PLAN:` comment where the implementation intentionally differs from this document, saying what changed and why. When you finish a phase, summarize these deviations so the plan can be updated.
- If an invariant seems to conflict with the existing architecture, stop and ask. Don't silently drop it.
