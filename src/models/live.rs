//! Live-control types and universal-action translation shared by all supported protocol families.
//! Nothing here depends on the UI, so command mapping and parsers can be unit-tested.

use std::time::{Duration, Instant};

use super::Capabilities;

/// Remote-command dialect used by the Scanner page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Legacy,
    EarlyComma,
    Gen1,
    Xt,
    HomePatrol,
    Sds,
}

/// One physical key exposed by the virtual remote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyDef {
    pub code: &'static str,
    pub label: &'static str,
}

/// Parsed LCD and indicator data from an `STS` reply.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisplayStatus {
    pub large: Vec<bool>,
    pub lines: Vec<DisplayLine>,
    pub battery_low: Option<bool>,
    pub weather_alert: bool,
    pub signal_bars: Option<u8>,
    pub squelch_open: bool,
    pub muted: bool,
}

/// One 16-character scanner display line and its display attributes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisplayLine {
    pub text: String,
    pub modes: String,
}

/// One scanner LCD cell translated from the documented large/small font tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LcdGlyph {
    pub text: String,
    pub class: &'static str,
    pub title: Option<String>,
}

fn label_cell(code: u8, start: u8, label: &str) -> Option<String> {
    let offset = usize::from(code.checked_sub(start)?);
    label
        .chars()
        .nth(offset)
        .map(|character| character.to_string())
}

/// Translate one preserved scanner display byte into a browser-renderable glyph.
pub fn lcd_glyph(character: char, large: bool) -> LcdGlyph {
    let value = character as u32;
    let code = if (0xE000..=0xE0FF).contains(&value) {
        Some((value - 0xE000) as u8)
    } else if value <= 0x1F {
        Some(value as u8)
    } else {
        None
    };
    let Some(code) = code else {
        return LcdGlyph {
            text: character.to_string(),
            class: "",
            title: None,
        };
    };
    let (text, class, description) = if (0x18..=0x1F).contains(&code) {
        const BARS: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
        (
            BARS[usize::from(code - 0x18)].to_owned(),
            "lcd-bar",
            "Vertical level bar",
        )
    } else if large {
        match code {
            0x80 => ("■".to_owned(), "lcd-icon", "Solid block"),
            0x81 => ("↑".to_owned(), "lcd-icon", "Up arrow"),
            0x82 => ("↓".to_owned(), "lcd-icon", "Down arrow"),
            0xA4 | 0xA5 => ("E".to_owned(), "lcd-icon", "East"),
            0xA6 | 0xA7 => ("N".to_owned(), "lcd-icon", "North"),
            0xA8 | 0xA9 => ("W".to_owned(), "lcd-icon", "West"),
            0xAA | 0xAB => ("S".to_owned(), "lcd-icon", "South"),
            0xCC => ("°".to_owned(), "", "Degree sign"),
            0xD4 => ("F".to_owned(), "lcd-icon boxed", "Function"),
            0xD5..=0xD7 | 0xDC | 0xE3..=0xE8 | 0xEB..=0xFC => {
                ("▮".to_owned(), "lcd-bar", "Signal level")
            }
            0xDD => ("L".to_owned(), "lcd-icon", "Lockout"),
            0xDE => ("/".to_owned(), "lcd-icon", "Lockout"),
            0xDF => ("O".to_owned(), "lcd-icon", "Lockout"),
            0x83
            | 0x88
            | 0x8B
            | 0x8C
            | 0xA0
            | 0xA3
            | 0xB0..=0xB3
            | 0xBB
            | 0xCD..=0xCF
            | 0xFD..=0xFF => (" ".to_owned(), "", "Blank"),
            _ => (
                "◆".to_owned(),
                "lcd-icon compass",
                "Compass graphic segment",
            ),
        }
    } else {
        let label = label_cell(code, 0x8D, "HOLD")
            .or_else(|| label_cell(code, 0x91, "DSKP"))
            .or_else(|| label_cell(code, 0x95, "L/O"))
            .or_else(|| label_cell(code, 0x98, "AM "))
            .or_else(|| label_cell(code, 0x9B, "FM"))
            .or_else(|| label_cell(code, 0x9D, "NF"))
            .or_else(|| label_cell(code, 0x9F, "WF"))
            .or_else(|| label_cell(code, 0xA1, "PR"))
            .or_else(|| label_cell(code, 0xA3, "ATT"))
            .or_else(|| label_cell(code, 0xB9, "FM"))
            .or_else(|| label_cell(code, 0xBB, "MU"))
            .or_else(|| label_cell(code, 0xC1, "SRCH"))
            .or_else(|| label_cell(code, 0xC5, "IFX"))
            .or_else(|| label_cell(code, 0xC8, "SCR"))
            .or_else(|| label_cell(code, 0xCD, "REP"))
            .or_else(|| label_cell(code, 0xD0, "MAX "))
            .or_else(|| label_cell(code, 0xD4, "NAC"));
        if let Some(label) = label {
            (label, "lcd-icon", "Scanner status label")
        } else {
            match code {
                0x80 => ("■".to_owned(), "lcd-icon", "Solid block"),
                0x81 => ("↑".to_owned(), "lcd-icon", "Up arrow"),
                0x82 => ("↓".to_owned(), "lcd-icon", "Down arrow"),
                0x83 | 0x84 => ("▰".to_owned(), "lcd-icon", "Battery"),
                0x85 | 0x86 => ("▣".to_owned(), "lcd-icon", "Keypad lock"),
                0x87..=0x8A | 0xB5..=0xB8 => ("●".to_owned(), "lcd-icon", "Close Call"),
                0x8B => ("F".to_owned(), "lcd-icon boxed", "Function"),
                0x8C => ("P".to_owned(), "lcd-icon boxed", "Priority"),
                0xA6..=0xAD => ("▮".to_owned(), "lcd-bar", "Signal level"),
                0xAE..=0xB4 => ("▣".to_owned(), "lcd-icon", "Active channel"),
                0xBD..=0xC0 => ("◆".to_owned(), "lcd-icon", "Marker"),
                0xCC => ("°".to_owned(), "", "Degree sign"),
                0xCB | 0xD7 => (" ".to_owned(), "", "Blank"),
                _ => (
                    "◇".to_owned(),
                    "lcd-icon unknown",
                    "Undocumented scanner glyph",
                ),
            }
        }
    };
    LcdGlyph {
        text,
        class,
        title: Some(format!("{description} (0x{code:02X})")),
    }
}

/// One sample emitted by `CSC`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpectrumPoint {
    pub rssi: u16,
    pub freq_hz: u64,
    pub squelch_open: bool,
}

/// Every feature the Scanner page can bind a control to. Some are only listed so the support table is complete.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    SignalMeter,
    Hold,
    HoldAtLevel,
    StepNext,
    StepPrev,
    Avoid,
    ResumeScan,
    Volume,
    Squelch,
    Mute,
    ModeSwitch,
    ToneOut,
    Discovery,
    Waterfall,
    Highway,
    QuickKeysSystems,
    QuickKeysGroups,
    QuickKeysFavorites,
    QuickKeysBanks,
    QuickTune,
    JumpToTag,
    Spectrum,
    LcdMirror,
    Keypad,
    Battery,
    WeatherAlert,
    PowerOff,
    Record,
}

/// How (or whether) a model supports a capability.
#[derive(Clone, Debug, PartialEq)]
pub enum Support {
    /// A dedicated command with an acknowledgment.
    Native,
    /// Achieved with key presses; there is no confirmation.
    Emulated,
    /// Needs Program Mode, which pauses scanning.
    NeedsProgramMode,
    /// The model cannot do this; the reason is shown to the user.
    Unsupported(&'static str),
    /// The model can, but the app does not do it yet.
    Planned(&'static str),
}

/// Static per-model data used by the Scanner page.
#[derive(Clone, Debug)]
pub struct ControlProfile {
    /// Model name, e.g. `BCD325P2`.
    pub model: &'static str,
    /// `None` means no remote volume.
    pub volume_range: Option<(u8, u8)>,
    /// `None` means no remote squelch.
    pub squelch_range: Option<(u8, u8)>,
    /// Receivable ranges in the wire unit of 100 Hz.
    pub freq_ranges: &'static [(u32, u32)],
    /// Service search banks as `(label, JPM index)`.
    pub service_banks: &'static [(&'static str, &'static str)],
    /// True if the model accepts `POF`.
    pub power_off: bool,
    pub family: Family,
    pub keymap: &'static [KeyDef],
    pub battery_status: bool,
    pub spectrum: bool,
    pub gps_status: bool,
    pub pwr_status: bool,
    pub live_quick_keys: bool,
    pub waterfall: bool,
    pub recording: bool,
}

impl ControlProfile {
    /// How this model supports `cap`.
    pub fn support(&self, cap: Capability) -> Support {
        use Capability::*;
        use Support::*;
        match cap {
            SignalMeter
                if self.pwr_status || matches!(self.family, Family::HomePatrol | Family::Sds) =>
            {
                Native
            }
            SignalMeter => Unsupported("This model cannot report signal strength."),
            Volume if self.volume_range.is_some() => Native,
            Volume => Unsupported("This model has no remote volume command."),
            Squelch if self.squelch_range.is_some() => Native,
            Squelch => Unsupported("This model has no remote squelch command."),
            QuickTune => Native,
            ModeSwitch if !matches!(self.family, Family::EarlyComma | Family::Legacy) => Native,
            ModeSwitch if self.family == Family::Legacy => Emulated,
            ToneOut if self.family == Family::Legacy => {
                Unsupported("The BCT8 has no Fire Tone-Out mode.")
            }
            ToneOut if self.family != Family::EarlyComma => Native,
            ModeSwitch | ToneOut => {
                Unsupported("This model has no safe direct command for changing to that mode.")
            }
            JumpToTag if matches!(self.family, Family::Xt | Family::HomePatrol | Family::Sds) => {
                Native
            }
            JumpToTag => Unsupported("Number tags are not supported by this model."),
            Hold | StepNext | StepPrev | Avoid | ResumeScan
                if matches!(self.family, Family::HomePatrol | Family::Sds) =>
            {
                Native
            }
            Hold | StepNext | StepPrev | Avoid | ResumeScan => Emulated,
            Mute if self.family == Family::Legacy => Native,
            Mute if self.family == Family::Sds => Emulated,
            Mute if self.family == Family::HomePatrol => {
                Unsupported("HomePatrol-family scanners have no remote mute or volume command.")
            }
            Mute => Emulated,
            QuickKeysSystems | QuickKeysGroups if self.family == Family::Legacy => {
                Unsupported("The BCT8 uses five scan banks instead of system/group quick keys.")
            }
            QuickKeysSystems | QuickKeysGroups if self.live_quick_keys => Native,
            QuickKeysSystems | QuickKeysGroups => NeedsProgramMode,
            HoldAtLevel if matches!(self.family, Family::HomePatrol | Family::Sds) => Native,
            HoldAtLevel => {
                Unsupported("Only HomePatrol and SDS scanners can hold at a chosen level.")
            }
            Discovery if matches!(self.family, Family::HomePatrol | Family::Sds) => {
                Planned("Discovery session setup is not implemented yet.")
            }
            Discovery => Unsupported("Discovery modes exist only on HomePatrol and SDS scanners."),
            Waterfall if self.waterfall => Native,
            Waterfall => Unsupported("A waterfall is only available on SDS scanners."),
            Highway if self.family == Family::Legacy => Native,
            Highway => Unsupported("Highway Patrol mode exists only on the BCT8."),
            QuickKeysFavorites if self.live_quick_keys => Native,
            QuickKeysFavorites => {
                Unsupported("Favorites Lists exist only on HomePatrol and SDS scanners.")
            }
            QuickKeysBanks if self.family == Family::Legacy => Native,
            QuickKeysBanks => Unsupported("Quick-key banks A–E exist only on the BCT8."),
            PowerOff if self.power_off => Native,
            PowerOff => Unsupported("This model has no remote power-off command."),
            Spectrum if self.spectrum => Native,
            Spectrum => Unsupported("This scanner cannot stream sweep signal data."),
            LcdMirror | Keypad => Native,
            Battery if self.battery_status => Native,
            Battery => Unsupported("This mobile scanner has no battery status."),
            WeatherAlert => Native,
            Record if self.recording => Native,
            Record => Unsupported("This scanner has no user-record control."),
        }
    }
}

const BCT8_KEYS: [KeyDef; 27] = [
    KeyDef {
        code: "00",
        label: "Resume/Hold",
    },
    KeyDef {
        code: "01",
        label: "HWY/PVT",
    },
    KeyDef {
        code: "02 0",
        label: "0",
    },
    KeyDef {
        code: "02 1",
        label: "1",
    },
    KeyDef {
        code: "02 2",
        label: "2",
    },
    KeyDef {
        code: "02 3",
        label: "3",
    },
    KeyDef {
        code: "02 4",
        label: "4",
    },
    KeyDef {
        code: "02 5",
        label: "5",
    },
    KeyDef {
        code: "02 6",
        label: "6",
    },
    KeyDef {
        code: "02 7",
        label: "7",
    },
    KeyDef {
        code: "02 8",
        label: "8",
    },
    KeyDef {
        code: "02 9",
        label: "9",
    },
    KeyDef {
        code: "03",
        label: ".",
    },
    KeyDef {
        code: "04",
        label: "E",
    },
    KeyDef {
        code: "05",
        label: "PRI",
    },
    KeyDef {
        code: "06",
        label: "L/O",
    },
    KeyDef {
        code: "07",
        label: "Up",
    },
    KeyDef {
        code: "08",
        label: "Down",
    },
    KeyDef {
        code: "09",
        label: "Search",
    },
    KeyDef {
        code: "10",
        label: "Service",
    },
    KeyDef {
        code: "11",
        label: "Data",
    },
    KeyDef {
        code: "12",
        label: "Delay",
    },
    KeyDef {
        code: "13",
        label: "Trunk",
    },
    KeyDef {
        code: "14",
        label: "State",
    },
    KeyDef {
        code: "15",
        label: "Mute",
    },
    KeyDef {
        code: "16",
        label: "Flash",
    },
    KeyDef {
        code: "17",
        label: "Alert",
    },
];

const BCT8_SERVICES: [(&str, &str); 9] = [
    ("Police", "1"),
    ("Weather", "2"),
    ("Fire/EMS", "3"),
    ("News", "4"),
    ("CB", "5"),
    ("Air", "6"),
    ("Railroad", "7"),
    ("Marine", "8"),
    ("Department of Transportation", "9"),
];

/// BCT8 legacy live-control profile.
pub fn bct8_profile(model: &'static str, caps: &Capabilities) -> ControlProfile {
    ControlProfile {
        model,
        volume_range: None,
        squelch_range: None,
        freq_ranges: caps.freq_ranges,
        service_banks: &BCT8_SERVICES,
        power_off: false,
        family: Family::Legacy,
        keymap: &BCT8_KEYS,
        battery_status: false,
        spectrum: false,
        gps_status: false,
        pwr_status: false,
        live_quick_keys: false,
        waterfall: false,
        recording: false,
    }
}

const HANDHELD_KEYS: [KeyDef; 21] = [
    KeyDef {
        code: "M",
        label: "Menu",
    },
    KeyDef {
        code: "F",
        label: "Func",
    },
    KeyDef {
        code: "H",
        label: "Hold",
    },
    KeyDef {
        code: "S",
        label: "Scan",
    },
    KeyDef {
        code: "L",
        label: "L/O",
    },
    KeyDef {
        code: "1",
        label: "1",
    },
    KeyDef {
        code: "2",
        label: "2",
    },
    KeyDef {
        code: "3",
        label: "3",
    },
    KeyDef {
        code: "4",
        label: "4",
    },
    KeyDef {
        code: "5",
        label: "5",
    },
    KeyDef {
        code: "6",
        label: "6",
    },
    KeyDef {
        code: "7",
        label: "7",
    },
    KeyDef {
        code: "8",
        label: "8",
    },
    KeyDef {
        code: "9",
        label: "9",
    },
    KeyDef {
        code: ".",
        label: "./No",
    },
    KeyDef {
        code: "0",
        label: "0",
    },
    KeyDef {
        code: "E",
        label: "E/Yes",
    },
    KeyDef {
        code: "^",
        label: "VFO",
    },
    KeyDef {
        code: "<",
        label: "VFO left",
    },
    KeyDef {
        code: ">",
        label: "VFO right",
    },
    KeyDef {
        code: "P",
        label: "Power/Light",
    },
];

const MOBILE_KEYS: [KeyDef; 24] = [
    KeyDef {
        code: "P",
        label: "Pri",
    },
    KeyDef {
        code: "W",
        label: "WX",
    },
    KeyDef {
        code: "G",
        label: "GPS",
    },
    KeyDef {
        code: "M",
        label: "Menu",
    },
    KeyDef {
        code: "F",
        label: "Func/VFO",
    },
    KeyDef {
        code: "H",
        label: "Hold",
    },
    KeyDef {
        code: "S",
        label: "Scan",
    },
    KeyDef {
        code: "L",
        label: "L/O",
    },
    KeyDef {
        code: "1",
        label: "1",
    },
    KeyDef {
        code: "2",
        label: "2",
    },
    KeyDef {
        code: "3",
        label: "3",
    },
    KeyDef {
        code: "4",
        label: "4",
    },
    KeyDef {
        code: "5",
        label: "5",
    },
    KeyDef {
        code: "6",
        label: "6",
    },
    KeyDef {
        code: "7",
        label: "7",
    },
    KeyDef {
        code: "8",
        label: "8",
    },
    KeyDef {
        code: "9",
        label: "9",
    },
    KeyDef {
        code: ".",
        label: "./No",
    },
    KeyDef {
        code: "0",
        label: "0",
    },
    KeyDef {
        code: "E",
        label: "E/Yes",
    },
    KeyDef {
        code: "Q",
        label: "Squelch",
    },
    KeyDef {
        code: "V",
        label: "Volume",
    },
    KeyDef {
        code: "<",
        label: "VFO left",
    },
    KeyDef {
        code: ">",
        label: "VFO right",
    },
];

/// Service banks of the BCD325P2 and BCD396XT.
pub const BANKS_WITH_MILITARY: [(&str, &str); 12] = [
    ("Public Safety", "PublicSafety"),
    ("News", "News"),
    ("HAM", "HAM"),
    ("Marine", "Marine"),
    ("Railroad", "Railroad"),
    ("Air", "Air"),
    ("CB", "CB"),
    ("FRS/GMRS/MURS", "FRS/GMRS/MURS"),
    ("Racing", "Racing"),
    ("FM Broadcast", "FM"),
    ("Special", "Special"),
    ("Military Air", "Military"),
];

/// The BC346XT has no Military Air bank.
pub const BANKS_WITHOUT_MILITARY: [(&str, &str); 11] = [
    ("Public Safety", "PublicSafety"),
    ("News", "News"),
    ("HAM", "HAM"),
    ("Marine", "Marine"),
    ("Railroad", "Railroad"),
    ("Air", "Air"),
    ("CB", "CB"),
    ("FRS/GMRS/MURS", "FRS/GMRS/MURS"),
    ("Racing", "Racing"),
    ("FM Broadcast", "FM"),
    ("Special", "Special"),
];

/// Differences between models of the XT/P2 family that the Scanner page cares about.
pub struct XtVariant {
    /// Highest `VOL` level.
    pub volume_max: u8,
    /// Highest (closed) `SQL` level.
    pub squelch_max: u8,
    pub military_air: bool,
    pub power_off: bool,
    pub battery_status: bool,
}

impl XtVariant {
    /// BCD325P2, BCD396XT: VOL 0–15, SQL 0–15, with `POF`.
    pub const HANDHELD: XtVariant = XtVariant {
        volume_max: 15,
        squelch_max: 15,
        military_air: true,
        power_off: true,
        battery_status: true,
    };
    /// BCD996XT, BCD996P2: VOL 0–29, SQL 0–19, no `POF`.
    pub const MOBILE: XtVariant = XtVariant {
        volume_max: 29,
        squelch_max: 19,
        military_air: true,
        power_off: false,
        battery_status: false,
    };
}

/// Profile for the XT/P2 family. These models share one command set and differ only in data.
pub fn xt_profile(model: &'static str, caps: &Capabilities, v: &XtVariant) -> ControlProfile {
    ControlProfile {
        model,
        // SPEC: VOL and SQL ranges differ per model (handhelds 0–15; BCD996XT/P2 VOL 0–29, SQL 0–19).
        volume_range: Some((0, v.volume_max)),
        squelch_range: Some((0, v.squelch_max)),
        freq_ranges: caps.freq_ranges,
        service_banks: if v.military_air {
            &BANKS_WITH_MILITARY
        } else {
            &BANKS_WITHOUT_MILITARY
        },
        // SPEC: POF is listed for the handhelds only; the BCD996 specs have no power-off command.
        power_off: v.power_off,
        family: Family::Xt,
        keymap: if v.battery_status {
            &HANDHELD_KEYS
        } else {
            &MOBILE_KEYS
        },
        battery_status: v.battery_status,
        spectrum: true,
        gps_status: !v.battery_status,
        pwr_status: true,
        live_quick_keys: false,
        waterfall: false,
        recording: false,
    }
}

/// Profile for the earlier comma-command scanners added in Phase 2.
pub fn comma_profile(
    model: &'static str,
    caps: &Capabilities,
    family: Family,
    handheld: bool,
    spectrum: bool,
    power_off: bool,
    service_banks: &'static [(&'static str, &'static str)],
) -> ControlProfile {
    ControlProfile {
        model,
        volume_range: Some(if handheld { (0, 15) } else { (0, 29) }),
        squelch_range: Some(if handheld { (0, 15) } else { (0, 19) }),
        freq_ranges: caps.freq_ranges,
        service_banks,
        power_off,
        family,
        keymap: if handheld {
            &HANDHELD_KEYS
        } else {
            &MOBILE_KEYS
        },
        battery_status: handheld,
        spectrum,
        gps_status: false,
        pwr_status: family != Family::EarlyComma,
        live_quick_keys: false,
        waterfall: false,
        recording: false,
    }
}

const HP_HANDHELD_KEYS: [KeyDef; 25] = [
    KeyDef {
        code: "M",
        label: "Menu",
    },
    KeyDef {
        code: "F",
        label: "Func",
    },
    KeyDef {
        code: "L",
        label: "Avoid",
    },
    KeyDef {
        code: "0",
        label: "0",
    },
    KeyDef {
        code: "1",
        label: "1",
    },
    KeyDef {
        code: "2",
        label: "2",
    },
    KeyDef {
        code: "3",
        label: "3",
    },
    KeyDef {
        code: "4",
        label: "4",
    },
    KeyDef {
        code: "5",
        label: "5",
    },
    KeyDef {
        code: "6",
        label: "6",
    },
    KeyDef {
        code: "7",
        label: "7",
    },
    KeyDef {
        code: "8",
        label: "8",
    },
    KeyDef {
        code: "9",
        label: "9",
    },
    KeyDef {
        code: ".",
        label: "./No",
    },
    KeyDef {
        code: "E",
        label: "E/Yes",
    },
    KeyDef {
        code: ">",
        label: "Rotary right",
    },
    KeyDef {
        code: "<",
        label: "Rotary left",
    },
    KeyDef {
        code: "^",
        label: "Rotary push",
    },
    KeyDef {
        code: "V",
        label: "Backlight",
    },
    KeyDef {
        code: "Y",
        label: "Replay",
    },
    KeyDef {
        code: "A",
        label: "System",
    },
    KeyDef {
        code: "B",
        label: "Department",
    },
    KeyDef {
        code: "C",
        label: "Channel",
    },
    KeyDef {
        code: "Z",
        label: "Zip",
    },
    KeyDef {
        code: "R",
        label: "Range",
    },
];

const HP_MOBILE_KEYS: [KeyDef; 27] = [
    KeyDef {
        code: "M",
        label: "Menu",
    },
    KeyDef {
        code: "F",
        label: "Rotary",
    },
    KeyDef {
        code: "L",
        label: "Avoid",
    },
    KeyDef {
        code: "0",
        label: "0",
    },
    KeyDef {
        code: "1",
        label: "1",
    },
    KeyDef {
        code: "2",
        label: "2",
    },
    KeyDef {
        code: "3",
        label: "3",
    },
    KeyDef {
        code: "4",
        label: "4",
    },
    KeyDef {
        code: "5",
        label: "5",
    },
    KeyDef {
        code: "6",
        label: "6",
    },
    KeyDef {
        code: "7",
        label: "7",
    },
    KeyDef {
        code: "8",
        label: "8",
    },
    KeyDef {
        code: "9",
        label: "9",
    },
    KeyDef {
        code: ".",
        label: "./No",
    },
    KeyDef {
        code: "E",
        label: "E/Yes",
    },
    KeyDef {
        code: ">",
        label: "Rotary right",
    },
    KeyDef {
        code: "<",
        label: "Rotary left",
    },
    KeyDef {
        code: "^",
        label: "Rotary push",
    },
    KeyDef {
        code: "V",
        label: "Volume",
    },
    KeyDef {
        code: "Q",
        label: "Squelch",
    },
    KeyDef {
        code: "Y",
        label: "Replay",
    },
    KeyDef {
        code: "A",
        label: "System",
    },
    KeyDef {
        code: "B",
        label: "Department",
    },
    KeyDef {
        code: "C",
        label: "Channel",
    },
    KeyDef {
        code: "Z",
        label: "Zip",
    },
    KeyDef {
        code: "T",
        label: "Service type",
    },
    KeyDef {
        code: "R",
        label: "Range",
    },
];

/// HomePatrol/SDS live-control profile. These scanners expose their database as handles, not
/// program-mode records, so database transfer remains outside the Scanner page.
pub fn hp_profile(
    model: &'static str,
    caps: &Capabilities,
    family: Family,
    mobile: bool,
) -> ControlProfile {
    let sds = family == Family::Sds;
    ControlProfile {
        model,
        volume_range: sds.then_some(if mobile { (0, 29) } else { (0, 15) }),
        squelch_range: sds.then_some(if mobile { (0, 19) } else { (0, 15) }),
        freq_ranges: caps.freq_ranges,
        service_banks: &[],
        power_off: sds,
        family,
        keymap: if mobile {
            &HP_MOBILE_KEYS
        } else {
            &HP_HANDHELD_KEYS
        },
        battery_status: !mobile || sds,
        spectrum: false,
        gps_status: false,
        pwr_status: false,
        live_quick_keys: true,
        waterfall: sds,
        recording: true,
    }
}

/// Parses a variable-length `STS` reply.
pub fn parse_sts(reply: &str, battery_field: bool) -> Result<DisplayStatus, super::Error> {
    let fields: Vec<&str> = reply.trim_end_matches(['\r', '\n']).split(',').collect();
    if fields.first() != Some(&"STS") {
        return Err(super::Error::UnexpectedReply {
            expected: "STS",
            reply: reply.to_owned(),
        });
    }
    let form = fields.get(1).copied().unwrap_or_default();
    if !(4..=8).contains(&form.len()) {
        return Err(super::Error::BadValue {
            cmd: "STS",
            pos: 1,
            value: form.to_owned(),
        });
    }
    let tail = 2 + form.len() * 2;
    if fields.len() < tail + 9 {
        return Err(super::Error::MissingField {
            cmd: "STS",
            pos: tail + 8,
        });
    }
    let lines = (0..form.len())
        .map(|i| DisplayLine {
            text: fields[2 + i * 2].trim_end().to_owned(),
            modes: fields[3 + i * 2].to_owned(),
        })
        .collect();
    Ok(DisplayStatus {
        large: form.bytes().map(|b| b == b'1').collect(),
        lines,
        battery_low: battery_field.then(|| fields[tail + 2] == "1"),
        weather_alert: !matches!(fields[tail + 3], "" | "0"),
        signal_bars: fields[tail + 6].parse::<u8>().ok().map(|v| v.min(5)),
        squelch_open: fields[tail] == "1",
        muted: fields[tail + 1] == "1",
    })
}

/// Parses one `CSC,<RSSI>,<FRQ>,<SQL>` stream line.
pub fn parse_csc(reply: &str) -> Result<SpectrumPoint, super::Error> {
    let mut fields = reply.trim_end_matches(['\r', '\n']).split(',');
    if fields.next() != Some("CSC") {
        return Err(super::Error::UnexpectedReply {
            expected: "CSC",
            reply: reply.to_owned(),
        });
    }
    let parse = |pos, value: Option<&str>| {
        let value = value.ok_or(super::Error::MissingField { cmd: "CSC", pos })?;
        value.parse::<u64>().map_err(|_| super::Error::BadValue {
            cmd: "CSC",
            pos,
            value: value.to_owned(),
        })
    };
    let rssi = u16::try_from(parse(1, fields.next())?).map_err(|_| super::Error::BadValue {
        cmd: "CSC",
        pos: 1,
        value: reply.to_owned(),
    })?;
    let freq_hz = parse(2, fields.next())? * 100;
    let squelch_open = parse(3, fields.next())? == 1;
    Ok(SpectrumPoint {
        rssi,
        freq_hz,
        squelch_open,
    })
}

/// The scanner's weather scan modes: `(label, JPM index)`.
pub const WEATHER_MODES: [(&str, &str); 8] = [
    ("Normal", "NORMAL"),
    ("Alert only", "A_ONLY"),
    ("SAME 1", "SAME_1"),
    ("SAME 2", "SAME_2"),
    ("SAME 3", "SAME_3"),
    ("SAME 4", "SAME_4"),
    ("SAME 5", "SAME_5"),
    ("All FIPS", "ALL_FIPS"),
];

/// NOAA weather channels, for display only: `(label, MHz)`.
pub const NOAA_CHANNELS: [(&str, f64); 7] = [
    ("WX1", 162.400),
    ("WX2", 162.425),
    ("WX3", 162.450),
    ("WX4", 162.475),
    ("WX5", 162.500),
    ("WX6", 162.525),
    ("WX7", 162.550),
];

/// The NOAA channel label for a frequency in MHz, if it is one of the seven.
pub fn noaa_label(mhz: f64) -> Option<&'static str> {
    NOAA_CHANNELS
        .iter()
        .find(|(_, f)| (f - mhz).abs() < 0.0005)
        .map(|(l, _)| *l)
}

/// What the scanner is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanMode {
    Scan,
    Highway,
    /// Service search bank (the `JPM` index).
    Service(&'static str),
    Custom,
    CloseCall,
    QuickSearch,
    /// Weather mode (the `JPM` index).
    Weather(&'static str),
    ToneOut,
}

/// HomePatrol/SDS hold depth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldLevel {
    System,
    Department,
    Channel,
}

/// The state of the scanner as far as the app can tell. `None` means the model cannot say.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LiveState {
    /// App-tracked: the XT/P2 protocol cannot report the mode, so this is the last one we set.
    pub mode: Option<ScanMode>,
    pub system: Option<String>,
    pub group: Option<String>,
    pub channel: Option<String>,
    pub favorite: Option<String>,
    pub favorite_index: Option<u32>,
    pub system_index: Option<u32>,
    pub department_index: Option<u32>,
    pub channel_index: Option<u32>,
    pub favorite_qk: Option<u8>,
    pub system_qk: Option<u8>,
    pub department_qk: Option<u8>,
    pub channel_target: Option<&'static str>,
    pub system_held: bool,
    pub department_held: bool,
    pub channel_held: bool,
    pub freq_hz: Option<u64>,
    /// Talkgroup ID when the scanner reports one instead of a frequency.
    pub tgid: Option<String>,
    pub modulation: Option<String>,
    /// CTCSS/DCS code or P25 NAC, as text.
    pub sub_audio: Option<String>,
    pub system_tag: Option<String>,
    pub channel_tag: Option<String>,
    /// Signal strength 0–5.
    pub signal_bars: Option<u8>,
    pub squelch_open: bool,
    pub volume: Option<u8>,
    pub squelch: Option<u8>,
    /// App-tracked: mute is emulated with volume 0.
    pub muted: bool,
    /// BCT8 automatic mute control is selected; `muted` still reflects current audio state.
    pub mute_automatic: bool,
    pub recording: bool,
}

/// Non-empty trimmed text, or `None`.
fn text(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_owned())
}

impl LiveState {
    /// True when the scanner reports no reception details at all.
    pub fn is_idle(&self) -> bool {
        self.freq_hz.is_none()
            && self.tgid.is_none()
            && self.system.is_none()
            && self.channel.is_none()
    }

    /// The reception frequency in MHz.
    pub fn freq_mhz(&self) -> Option<f64> {
        self.freq_hz.map(|f| f as f64 / 1_000_000.0)
    }

    /// Updates the reception fields from a `GLG` reply (the full line, including `GLG,`).
    pub fn apply_glg(&mut self, reply: &str) -> Result<(), super::Error> {
        let mut f = reply.trim_end().split(',');
        if f.next() != Some("GLG") {
            return Err(super::Error::UnexpectedReply {
                expected: "GLG",
                reply: reply.to_owned(),
            });
        }
        // SPEC: the empty reply is printed with different comma counts, so missing fields are empty.
        let mut next = || f.next().unwrap_or("");
        let frq = next().trim().to_owned();
        let modulation = next();
        let _attenuation = next();
        let tone = next();
        let (system, group, channel) = (next(), next(), next());
        let squelch = next();
        let _mute = next(); // the scanner's own mute flag, not the app's emulated mute
        let (system_tag, channel_tag) = (next(), next());
        let nac = next();

        // Observed on hardware: a decimal point means MHz; a bare number of 6+ digits is in 100 Hz
        // units; anything shorter is a talkgroup ID.
        self.freq_hz = None;
        self.tgid = None;
        if frq.contains('.') {
            self.freq_hz = frq
                .parse::<f64>()
                .ok()
                .map(|m| (m * 1_000_000.0).round() as u64);
        } else if frq.len() >= 6 && frq.bytes().all(|b| b.is_ascii_digit()) {
            self.freq_hz = frq.parse::<u64>().ok().map(|v| v * 100);
        } else if !frq.is_empty() {
            self.tgid = Some(frq);
        }
        self.modulation = text(modulation);
        self.system = text(system);
        self.group = text(group);
        self.channel = text(channel);
        self.squelch_open = squelch.trim() == "1";
        let tag = |s: &str| text(s).filter(|t| t != "NONE");
        self.system_tag = tag(system_tag);
        self.channel_tag = tag(channel_tag);

        // A tone code of 0 means none; a NAC of NONE means none.
        let mut sub = Vec::new();
        if let Some(t) = text(tone).filter(|t| t != "0") {
            sub.push(format!("Tone {t}"));
        }
        if let Some(n) = tag(nac) {
            sub.push(format!("NAC {n}"));
        }
        self.sub_audio = (!sub.is_empty()).then(|| sub.join(" · "));
        Ok(())
    }

    /// Updates the signal meter from a `PWR` reply (RSSI 0–1023).
    pub fn apply_pwr(&mut self, reply: &str) {
        let rssi = reply
            .strip_prefix("PWR,")
            .and_then(|r| r.split(',').next())
            .and_then(|r| r.trim().parse::<u16>().ok());
        self.signal_bars = rssi.map(bars_from_rssi);
    }
}

/// Maps RSSI 0–1023 to 0–5 bars.
pub fn bars_from_rssi(rssi: u16) -> u8 {
    ((u32::from(rssi.min(1023)) * 5) / 1023) as u8
}

/// Universal UI intents. The driver turns these into wire commands.
#[allow(dead_code)] // PowerOff has no button yet
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Hold,
    HoldLevel(HoldLevel),
    ResumeScan,
    StepNext,
    StepPrev,
    Avoid,
    AvoidPermanent,
    SetVolume(u8),
    SetSquelch(u8),
    /// Handled by the session (it needs the volume to restore); never reaches [`commands`].
    SetMute(bool),
    SetMuteAutomatic,
    JumpMode(ScanMode),
    QuickTune {
        freq_hz: u64,
        modulation: Option<String>,
    },
    JumpToTag {
        favorite: Option<u16>,
        sys: Option<u16>,
        chan: Option<u16>,
    },
    PowerOff,
    SetRecord(bool),
}

impl Action {
    /// Short name for messages.
    pub fn label(&self) -> &'static str {
        match self {
            Action::Hold => "Hold",
            Action::HoldLevel(_) => "Hold",
            Action::ResumeScan => "Scan",
            Action::StepNext => "Next",
            Action::StepPrev => "Previous",
            Action::Avoid => "Avoid",
            Action::AvoidPermanent => "Permanent avoid",
            Action::SetVolume(_) => "Volume",
            Action::SetSquelch(_) => "Squelch",
            Action::SetMute(_) => "Mute",
            Action::SetMuteAutomatic => "Automatic mute",
            Action::JumpMode(_) => "Mode change",
            Action::QuickTune { .. } => "Quick tune",
            Action::JumpToTag { .. } => "Jump to tag",
            Action::PowerOff => "Power off",
            Action::SetRecord(_) => "Record",
        }
    }
}

fn hp_target(
    state: &LiveState,
    level: HoldLevel,
) -> Result<(&'static str, u32, Option<u32>), String> {
    match level {
        HoldLevel::System => Ok((
            "SYS",
            state
                .system_index
                .ok_or("No current system handle is available.")?,
            None,
        )),
        HoldLevel::Department => Ok((
            "DEPT",
            state
                .department_index
                .ok_or("No current department handle is available.")?,
            state.system_index,
        )),
        HoldLevel::Channel => {
            let target = state
                .channel_target
                .ok_or("No current channel target is available.")?;
            let first = if matches!(target, "QS_FREQ" | "CS_FREQ" | "SWS_FREQ") {
                u32::try_from(state.freq_hz.ok_or("No current frequency is available.")? / 100)
                    .map_err(|_| "Current frequency is out of range.")?
            } else {
                state
                    .channel_index
                    .ok_or("No current channel handle is available.")?
            };
            let second = match target {
                "DEPT" => state.system_index,
                "SWS_FREQ" => state.department_index,
                _ => None,
            };
            Ok((target, first, second))
        }
    }
}

/// The wire commands (in order) that carry out `action` on this model, or a user-facing reason it can't.
pub fn commands(
    action: &Action,
    profile: &ControlProfile,
    state: &LiveState,
) -> Result<Vec<String>, String> {
    // SPEC: KEY,<code>,<mode>; H hold, S scan, L lockout, < and > step (VFO left/right).
    let key = |c: char| vec![format!("KEY,{c},P")];
    let hp = matches!(profile.family, Family::HomePatrol | Family::Sds);
    let legacy = profile.family == Family::Legacy;
    let legacy_key = |code: &str| vec![format!("KEY{code}")];
    let target_command = |command: &str, level: HoldLevel, suffix: &str| {
        let (target, first, second) = hp_target(state, level)?;
        Ok::<_, String>(vec![format!(
            "{command},{target},{first},{}{suffix}",
            second.map(|value| value.to_string()).unwrap_or_default()
        )])
    };
    let avoid_command = |status: u8| {
        let target = state
            .channel_target
            .ok_or("No current channel target is available.")?;
        let (target, first, second) = match target {
            "CFREQ" | "TGID" | "CCHIT" => (
                target,
                state
                    .channel_index
                    .ok_or("No current channel handle is available.")?,
                None,
            ),
            "QS_FREQ" | "CS_FREQ" | "SWS_FREQ" => (
                "AFREQ",
                u32::try_from(state.freq_hz.ok_or("No current frequency is available.")? / 100)
                    .map_err(|_| "Current frequency is out of range.")?,
                None,
            ),
            "STGID" => (
                "ATGID",
                state
                    .tgid
                    .as_deref()
                    .and_then(|value| value.parse().ok())
                    .ok_or("No current talkgroup ID is available.")?,
                state.system_index,
            ),
            _ => return Err("The current item cannot be avoided remotely.".into()),
        };
        Ok::<_, String>(vec![format!(
            "AVD,{target},{first},{},{status}",
            second.map(|value| value.to_string()).unwrap_or_default()
        )])
    };
    Ok(match action {
        Action::Hold if legacy => legacy_key("00"),
        Action::Hold if hp => target_command("HLD", HoldLevel::Channel, "")?,
        Action::HoldLevel(level) if hp => target_command("HLD", *level, "")?,
        Action::HoldLevel(_) => return Err("Hold levels are not supported by this model.".into()),
        Action::Hold => key('H'),
        Action::ResumeScan | Action::JumpMode(ScanMode::Scan) if legacy => legacy_key("00"),
        Action::ResumeScan | Action::JumpMode(ScanMode::Scan) if hp => {
            vec!["JPM,SCN_MODE,FFFFFFFF".into()]
        }
        Action::ResumeScan | Action::JumpMode(ScanMode::Scan) => key('S'),
        Action::Avoid if hp => avoid_command(2)?,
        Action::AvoidPermanent if hp => avoid_command(1)?,
        Action::AvoidPermanent => {
            return Err("Permanent avoid is not supported by this model.".into());
        }
        Action::Avoid if legacy => legacy_key("06"),
        Action::Avoid => key('L'),
        Action::StepNext if hp => target_command("NXT", HoldLevel::Channel, ",1")?,
        Action::StepPrev if hp => target_command("PRV", HoldLevel::Channel, ",1")?,
        Action::StepNext if legacy => legacy_key("07"),
        Action::StepPrev if legacy => legacy_key("08"),
        Action::StepNext => key('>'),
        Action::StepPrev => key('<'),
        Action::SetVolume(v) => {
            let (lo, hi) = profile
                .volume_range
                .ok_or("This model has no remote volume.")?;
            vec![format!("VOL,{}", (*v).clamp(lo, hi))]
        }
        Action::SetSquelch(v) => {
            let (lo, hi) = profile
                .squelch_range
                .ok_or("This model has no remote squelch.")?;
            vec![format!("SQL,{}", (*v).clamp(lo, hi))]
        }
        Action::SetMute(_) => return Err("Mute is handled by the session.".into()),
        Action::SetMuteAutomatic if legacy => vec!["MUA".into()],
        Action::SetMuteAutomatic => {
            return Err("Automatic mute control is only available on the BCT8.".into());
        }
        Action::JumpMode(ScanMode::Highway) if legacy => vec!["HWN".into()],
        Action::JumpMode(ScanMode::Service(bank)) if legacy => vec![format!("SV{bank}")],
        Action::JumpMode(ScanMode::Weather(_)) if legacy => vec!["SV2".into()],
        Action::JumpMode(ScanMode::Custom) if legacy => legacy_key("09"),
        Action::JumpMode(_) if legacy => {
            return Err("The BCT8 cannot jump directly to this mode.".into());
        }
        Action::JumpMode(_) if profile.family == Family::EarlyComma => {
            return Err("This model's mode changes require physical-key sequences that are not safely automatable.".into());
        }
        Action::JumpMode(ScanMode::Service(_)) if hp => {
            return Err("This family has no direct Service Scan jump command.".into());
        }
        Action::JumpMode(ScanMode::Service(bank)) => vec![format!("JPM,SVC_MODE,{bank}")],
        Action::JumpMode(ScanMode::Custom) => vec!["JPM,CTM_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::CloseCall) => vec!["JPM,CC_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::Weather(mode)) => vec![format!("JPM,WX_MODE,{mode}")],
        Action::JumpMode(ScanMode::ToneOut) => vec!["JPM,FTO_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::QuickSearch) => {
            return Err("Use Quick Tune to enter Quick Search.".into());
        }
        Action::JumpMode(ScanMode::Highway) => {
            return Err("Highway Patrol mode is only available on the BCT8.".into());
        }
        Action::QuickTune {
            freq_hz,
            modulation,
        } => {
            let wire = u32::try_from(freq_hz / 100).map_err(|_| "Frequency is out of range.")?;
            if !profile
                .freq_ranges
                .iter()
                .any(|(lo, hi)| (*lo..=*hi).contains(&wire))
            {
                return Err(format!(
                    "{:.4} MHz is outside the {} receive range.",
                    *freq_hz as f64 / 1e6,
                    profile.model
                ));
            }
            if hp {
                return Ok(vec![format!("QSH,{wire:08}")]);
            }
            if legacy {
                return Ok(vec![format!("RF{wire:08}?")]);
            }
            // SPEC: XT/P2 QSH places modulation after one reserved field. Always send a
            // modulation; callers without an override use the scanner's AUTO selection.
            let modulation = modulation.as_deref().unwrap_or("AUTO");
            vec![format!("QSH,{wire:08},,{modulation}")]
        }
        Action::JumpToTag {
            favorite,
            sys,
            chan,
        } => {
            if !matches!(
                profile.family,
                Family::Xt | Family::HomePatrol | Family::Sds
            ) {
                return Err("Number tags are not supported by this model.".into());
            }
            if favorite.is_none() && sys.is_none() && chan.is_none() {
                return Err("Enter a system tag, a channel tag, or both.".into());
            }
            if favorite.is_some_and(|tag| tag > 99)
                || (hp && sys.is_some_and(|tag| tag > 99))
                || (!hp && sys.is_some_and(|tag| tag > 999))
                || chan.is_some_and(|tag| tag > 999)
            {
                return Err("Number tags are 0–999.".into());
            }
            let field = |t: &Option<u16>| t.map(|t| t.to_string()).unwrap_or_default();
            if hp {
                vec![format!(
                    "JNT,{},{},{}",
                    field(favorite),
                    field(sys),
                    field(chan)
                )]
            } else {
                vec![format!("JNT,{},{}", field(sys), field(chan))]
            }
        }
        Action::PowerOff => {
            if !profile.power_off {
                return Err("This model has no remote power-off command.".into());
            }
            vec!["POF".into()]
        }
        Action::SetRecord(recording) => {
            if !profile.recording {
                return Err("This model has no user-record control.".into());
            }
            vec![format!("URC,{}", u8::from(*recording))]
        }
    })
}

/// One reception in the app-side activity log.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityEntry {
    /// Wall-clock time the reception started, e.g. `14:02:11`.
    pub clock: String,
    pub system: Option<String>,
    pub group: Option<String>,
    pub channel: Option<String>,
    pub freq_hz: Option<u64>,
    pub tgid: Option<String>,
    /// `None` while the reception is still going.
    pub duration: Option<Duration>,
    pub peak_bars: u8,
    #[serde(skip, default = "Instant::now")]
    started: Instant,
}

impl ActivityEntry {
    /// True if `s` is the same reception (same place on the same system) as this entry.
    fn same_as(&self, s: &LiveState) -> bool {
        self.channel == s.channel
            && self.freq_hz == s.freq_hz
            && self.tgid == s.tgid
            && self.system == s.system
    }
}

/// The most recent receptions, newest last.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityLog {
    pub entries: Vec<ActivityEntry>,
    /// True while the last entry is still open.
    #[serde(skip)]
    pub(crate) open: bool,
}

/// Entries kept in memory.
const ACTIVITY_LIMIT: usize = 500;

impl ActivityLog {
    /// A serializable snapshot with any current reception closed at `now`.
    pub fn persistence_snapshot(&self, now: Instant) -> Self {
        let mut snapshot = self.clone();
        if snapshot.open {
            if let Some(last) = snapshot.entries.last_mut() {
                last.duration = Some(now.saturating_duration_since(last.started));
            }
            snapshot.open = false;
        }
        snapshot
    }

    /// Feeds one status update in; returns true if the log changed.
    /// A reception starts when squelch opens and ends when it closes or the channel changes.
    pub fn observe(&mut self, s: &LiveState, now: Instant, clock: &str) -> bool {
        let mut changed = false;
        if self.open {
            let last = self.entries.last_mut().expect("open entry");
            if s.squelch_open && last.same_as(s) {
                let bars = s.signal_bars.unwrap_or(0);
                if bars > last.peak_bars {
                    last.peak_bars = bars;
                    changed = true;
                }
                return changed;
            }
            last.duration = Some(now.duration_since(last.started));
            self.open = false;
            changed = true;
        }
        if s.squelch_open {
            if self.entries.len() >= ACTIVITY_LIMIT {
                self.entries.remove(0);
            }
            self.entries.push(ActivityEntry {
                clock: clock.to_owned(),
                system: s.system.clone(),
                group: s.group.clone(),
                channel: s.channel.clone(),
                freq_hz: s.freq_hz,
                tgid: s.tgid.clone(),
                duration: None,
                peak_bars: s.signal_bars.unwrap_or(0),
                started: now,
            });
            self.open = true;
            changed = true;
        }
        changed
    }

    /// The log as CSV (oldest first).
    pub fn to_csv(&self) -> String {
        let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        let mut out =
            String::from("Time,System,Group,Channel,Frequency MHz,TGID,Duration s,Peak bars\n");
        for e in &self.entries {
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                q(&e.clock),
                q(e.system.as_deref().unwrap_or("")),
                q(e.group.as_deref().unwrap_or("")),
                q(e.channel.as_deref().unwrap_or("")),
                e.freq_hz
                    .map(|f| format!("{:.4}", f as f64 / 1e6))
                    .unwrap_or_default(),
                q(e.tgid.as_deref().unwrap_or("")),
                e.duration
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_default(),
                e.peak_bars,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    /// The BCD325P2 profile.
    fn profile() -> ControlProfile {
        models::find("BCD325P2").unwrap().control_profile().unwrap()
    }

    /// A reception reply fills names, tags, squelch and the frequency.
    #[test]
    fn parses_glg() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,4625000,FM,0,0,Fire,Dispatch,Main,1,0,12,3,NONE")
            .unwrap();
        assert_eq!(
            (
                s.system.as_deref(),
                s.group.as_deref(),
                s.channel.as_deref()
            ),
            (Some("Fire"), Some("Dispatch"), Some("Main"))
        );
        assert!(s.squelch_open);
        assert_eq!(s.freq_hz, Some(462_500_000));
        assert_eq!(
            (s.system_tag.as_deref(), s.channel_tag.as_deref()),
            (Some("12"), Some("3"))
        );
        assert_eq!(s.sub_audio, None);
    }

    /// Tone codes and NAC are combined; a decimal frequency is MHz.
    #[test]
    fn glg_sub_audio_and_decimal_frequency() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,162.5500,NFM,0,23,WX,,,1,0,NONE,NONE,293")
            .unwrap();
        assert_eq!(s.freq_hz, Some(162_550_000));
        assert_eq!(s.sub_audio.as_deref(), Some("Tone 23 · NAC 293"));
    }

    /// Both spellings of the empty reply are idle, and a talkgroup ID is not a frequency.
    #[test]
    fn idle_and_talkgroup() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,,,,,,,,,,,,").unwrap();
        assert!(s.is_idle());
        s.apply_glg("GLG,,,,,,,,,,").unwrap();
        assert!(s.is_idle());
        s.apply_glg("GLG,12345,,,,Sys,Grp,Chan,1,0,,,").unwrap();
        assert_eq!((s.freq_hz, s.tgid.as_deref()), (None, Some("12345")));
    }

    /// Other replies are rejected, and PWR maps to 0–5 bars.
    #[test]
    fn rejects_wrong_reply_and_maps_bars() {
        assert!(LiveState::default().apply_glg("ERR").is_err());
        let mut s = LiveState::default();
        s.apply_pwr("PWR,1023,4625000");
        assert_eq!(s.signal_bars, Some(5));
        s.apply_pwr("PWR,0,0");
        assert_eq!(s.signal_bars, Some(0));
        assert_eq!(bars_from_rssi(5000), 5);
    }

    /// Weather frequencies map to their WX channel and others do not.
    #[test]
    fn noaa_labels() {
        assert_eq!(noaa_label(162.4), Some("WX1"));
        assert_eq!(noaa_label(162.55), Some("WX7"));
        assert_eq!(noaa_label(162.6), None);
    }

    /// Actions translate to the documented wire commands.
    #[test]
    fn translates_actions() {
        let p = profile();
        let state = LiveState::default();
        let one = |a: Action| commands(&a, &p, &state).unwrap();
        assert_eq!(one(Action::Hold), ["KEY,H,P"]);
        assert_eq!(one(Action::StepNext), ["KEY,>,P"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Scan)), ["KEY,S,P"]);
        assert_eq!(
            one(Action::JumpMode(ScanMode::Service("HAM"))),
            ["JPM,SVC_MODE,HAM"]
        );
        assert_eq!(
            one(Action::JumpMode(ScanMode::Weather("SAME_2"))),
            ["JPM,WX_MODE,SAME_2"]
        );
        assert_eq!(one(Action::SetVolume(99)), ["VOL,15"]);
        assert_eq!(
            one(Action::JumpToTag {
                favorite: None,
                sys: Some(12),
                chan: None
            }),
            ["JNT,12,"]
        );
        assert_eq!(
            one(Action::JumpToTag {
                favorite: None,
                sys: None,
                chan: Some(3)
            }),
            ["JNT,,3"]
        );
        assert_eq!(
            one(Action::QuickTune {
                freq_hz: 462_562_500,
                modulation: Some("AUTO".into())
            }),
            ["QSH,04625625,,AUTO"]
        );
        assert_eq!(
            one(Action::QuickTune {
                freq_hz: 162_400_000,
                modulation: Some("NFM".into())
            }),
            ["QSH,01624000,,NFM"]
        );
        assert_eq!(
            one(Action::QuickTune {
                freq_hz: 162_400_000,
                modulation: None
            }),
            ["QSH,01624000,,AUTO"]
        );
    }

    #[test]
    fn translates_homepatrol_native_actions() {
        let profile = models::find("SDS100").unwrap().control_profile().unwrap();
        let state = LiveState {
            system_index: Some(10),
            department_index: Some(20),
            channel_index: Some(30),
            channel_target: Some("CFREQ"),
            freq_hz: Some(155_250_000),
            ..LiveState::default()
        };
        let one = |action| commands(&action, &profile, &state).unwrap();
        assert_eq!(one(Action::HoldLevel(HoldLevel::System)), ["HLD,SYS,10,"]);
        assert_eq!(
            one(Action::HoldLevel(HoldLevel::Department)),
            ["HLD,DEPT,20,10"]
        );
        assert_eq!(one(Action::StepNext), ["NXT,CFREQ,30,,1"]);
        assert_eq!(one(Action::Avoid), ["AVD,CFREQ,30,,2"]);
        assert_eq!(one(Action::AvoidPermanent), ["AVD,CFREQ,30,,1"]);
        assert_eq!(one(Action::ResumeScan), ["JPM,SCN_MODE,FFFFFFFF"]);
        assert_eq!(one(Action::SetRecord(true)), ["URC,1"]);
        assert_eq!(
            one(Action::JumpToTag {
                favorite: Some(1),
                sys: Some(2),
                chan: Some(3),
            }),
            ["JNT,1,2,3"]
        );
        assert_eq!(
            one(Action::QuickTune {
                freq_hz: 155_250_000,
                modulation: None,
            }),
            ["QSH,01552500"]
        );
    }

    #[test]
    fn translates_bct8_legacy_actions() {
        let profile = models::find("BCT8").unwrap().control_profile().unwrap();
        let state = LiveState::default();
        let one = |action| commands(&action, &profile, &state).unwrap();
        assert_eq!(one(Action::Hold), ["KEY00"]);
        assert_eq!(one(Action::StepNext), ["KEY07"]);
        assert_eq!(one(Action::Avoid), ["KEY06"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Service("5"))), ["SV5"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Weather("NORMAL"))), ["SV2"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Highway)), ["HWN"]);
        assert_eq!(one(Action::SetMuteAutomatic), ["MUA"]);
        assert_eq!(
            one(Action::QuickTune {
                freq_hz: 406_012_500,
                modulation: None,
            }),
            ["RF04060125?"]
        );
        assert!(commands(&Action::JumpMode(ScanMode::CloseCall), &profile, &state).is_err());
        assert_eq!(profile.support(Capability::Mute), Support::Native);
        assert_eq!(profile.support(Capability::QuickKeysBanks), Support::Native);
    }

    /// Bad input is refused with a reason instead of being sent.
    #[test]
    fn refuses_bad_input() {
        let p = profile();
        let state = LiveState::default();
        assert!(
            commands(
                &Action::QuickTune {
                    freq_hz: 600_000_000,
                    modulation: None
                },
                &p,
                &state
            )
            .is_err()
        );
        assert!(
            commands(
                &Action::JumpToTag {
                    favorite: None,
                    sys: None,
                    chan: None
                },
                &p,
                &state
            )
            .is_err()
        );
        assert!(
            commands(
                &Action::JumpToTag {
                    favorite: None,
                    sys: Some(1000),
                    chan: None
                },
                &p,
                &state
            )
            .is_err()
        );
        assert!(commands(&Action::SetMute(true), &p, &state).is_err());
    }

    /// Each model has a profile, and the BC346XT lacks the Military Air bank.
    #[test]
    fn profiles_per_model() {
        for m in models::all() {
            assert!(m.control_profile().is_some(), "{}", m.name());
        }
        let banks = |n: &str| {
            models::find(n)
                .unwrap()
                .control_profile()
                .unwrap()
                .service_banks
                .len()
        };
        assert_eq!((banks("BCD325P2"), banks("BC346XT")), (12, 11));
        let ranges = |n: &str| {
            models::find(n)
                .unwrap()
                .control_profile()
                .unwrap()
                .volume_range
        };
        assert_eq!(
            (ranges("BCD396XT"), ranges("BCD996XT")),
            (Some((0, 15)), Some((0, 29)))
        );
        assert!(
            commands(
                &Action::PowerOff,
                &models::find("BCD996P2").unwrap().control_profile().unwrap(),
                &LiveState::default()
            )
            .is_err()
        );
        assert!(matches!(
            profile().support(Capability::HoldAtLevel),
            Support::Unsupported(_)
        ));
        assert_eq!(profile().support(Capability::Mute), Support::Emulated);
        let br = models::find("BR330T").unwrap().control_profile().unwrap();
        assert_eq!(br.family, Family::EarlyComma);
        assert!(matches!(
            br.support(Capability::Spectrum),
            Support::Unsupported(_)
        ));
        assert!(
            models::find("BCT15X")
                .unwrap()
                .control_profile()
                .unwrap()
                .spectrum
        );
    }

    /// STS display fields and indicators are normalized for the UI.
    #[test]
    fn parses_sts_display_and_indicators() {
        let status = parse_sts(
            "STS,1111,Line 1          ,,Line 2          ,****************,Line 3          ,________________,Line 4          ,,1,0,1,$$$,0,0,5,RED,3",
            true,
        )
        .unwrap();
        assert_eq!(status.lines.len(), 4);
        assert!(status.large.iter().all(|large| *large));
        assert_eq!(status.lines[1].modes, "****************");
        assert_eq!(status.battery_low, Some(true));
        assert!(status.weather_alert);
        assert!(status.squelch_open);
        assert_eq!(status.signal_bars, Some(5));
    }

    /// CSC samples convert the scanner's 100 Hz unit to hertz.
    #[test]
    fn parses_csc_sample() {
        let point = parse_csc("CSC,768,04625625,1").unwrap();
        assert_eq!(
            point,
            SpectrumPoint {
                rssi: 768,
                freq_hz: 462_562_500,
                squelch_open: true
            }
        );
        assert!(parse_csc("PWR,10,20").is_err());
    }

    /// Receptions are logged on squelch open, closed on squelch close or channel change.
    #[test]
    fn activity_log() {
        let t0 = Instant::now();
        let mut log = ActivityLog::default();
        let mut s = LiveState {
            squelch_open: true,
            channel: Some("A".into()),
            signal_bars: Some(2),
            ..Default::default()
        };
        assert!(log.observe(&s, t0, "10:00:00"));
        s.signal_bars = Some(4);
        assert!(log.observe(&s, t0, "10:00:01"));
        assert!(!log.observe(&s, t0, "10:00:02"));
        s.channel = Some("B".into());
        log.observe(&s, t0 + Duration::from_secs(3), "10:00:03");
        assert_eq!(log.entries.len(), 2);
        assert_eq!(
            (log.entries[0].peak_bars, log.entries[0].duration),
            (4, Some(Duration::from_secs(3)))
        );
        s.squelch_open = false;
        log.observe(&s, t0 + Duration::from_secs(5), "10:00:05");
        assert_eq!(log.entries[1].duration, Some(Duration::from_secs(2)));
        assert!(log.to_csv().lines().count() == 3);
    }

    #[test]
    fn renders_documented_lcd_glyphs() {
        assert_eq!(lcd_glyph('A', true).text, "A");
        assert_eq!(lcd_glyph('\u{e018}', true).text, "▁");
        assert_eq!(lcd_glyph('\u{e081}', true).text, "↑");
        assert_eq!(lcd_glyph('\u{e0cc}', true).text, "°");
        assert_eq!(lcd_glyph('\u{e08d}', false).text, "H");
        assert_eq!(lcd_glyph('\u{e090}', false).text, "D");
        let unknown = lcd_glyph('\u{e0fe}', false);
        assert_eq!(unknown.class, "lcd-icon unknown");
        assert!(
            unknown
                .title
                .as_deref()
                .is_some_and(|title| title.contains("FE"))
        );
    }

    #[test]
    fn persistence_snapshot_closes_open_reception() {
        let started = Instant::now();
        let mut log = ActivityLog::default();
        let state = LiveState {
            squelch_open: true,
            channel: Some("Dispatch".into()),
            ..Default::default()
        };
        log.observe(&state, started, "10:00:00");
        let snapshot = log.persistence_snapshot(started + Duration::from_secs(4));
        assert!(!snapshot.open);
        assert_eq!(snapshot.entries[0].duration, Some(Duration::from_secs(4)));
    }
}
