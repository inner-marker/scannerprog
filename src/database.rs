#![allow(dead_code)]

//! Model-neutral scan database: the values, records and tree that the UI edits
//! and that `.ron` files store. Models translate this to and from their wire format.

use std::collections::HashMap;

use crate::models::Error;
use std::fmt; // Display impls produce the exact wire format of each value

use std::str::FromStr; // FromStr impls parse the wire format back

/// Handle to a record in the scanner's memory pool (1 to about 45000).
/// "-1" on the wire (end of list / none) is represented as Option::None.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Index(pub u16);

impl fmt::Display for Index {
    /// Format the memory index as an unpadded decimal number.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0) // plain decimal, no padding
    }
}

/// Maximum scanner memory-block count, also used to stop a broken linked-list walk.
pub const MAX_BLOCKS: usize = 45_000;

/// The previous/next pointers every record carries.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Links {
    pub rev: Option<Index>, // REV_INDEX: previous record in the same list
    pub fwd: Option<Index>, // FWD_INDEX: next record in the same list
}

/// Frequency in units of 100 Hz, as the scanner stores it.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Freq(pub u32);

impl Freq {
    /// Build from Hz, rounding to the nearest 100 Hz.
    pub fn from_hz(hz: u64) -> Self {
        Freq(((hz + 50) / 100) as u32)
    }
    /// Value in Hz.
    pub fn hz(self) -> u64 {
        self.0 as u64 * 100
    }
    /// Value in MHz, for display only.
    pub fn mhz(self) -> f64 {
        self.0 as f64 / 10_000.0
    }
}

impl FromStr for Freq {
    type Err = std::num::ParseIntError;
    /// Parse the scanner's stored 100 Hz frequency unit.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Freq) // leading zeros are fine for u32 parsing
    }
}

impl fmt::Display for Freq {
    /// Write the wire representation as eight zero-padded digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08}", self.0) // always 8 digits, zero-padded
    }
}

/// Alpha tag, up to 16 printable ASCII characters, no commas.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(pub(crate) String);

impl Name {
    /// Maximum number of printable ASCII characters accepted by the scanner.
    pub const MAX_LEN: usize = 16;

    /// Validate a name you intend to send to the scanner.
    pub fn new(s: &str) -> Result<Self, Error> {
        // Commas split command fields, so a printable comma is still unsafe in a name.
        let printable = s.bytes().all(|b| (0x20..=0x7E).contains(&b)); // ASCII glyphs only
        if s.len() > Self::MAX_LEN || s.contains(',') || !printable {
            return Err(Error::InvalidName(s.to_string()));
        }
        Ok(Name(s.to_string()))
    }

    /// Borrow the validated name without allocating another string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    /// Write the name as a command field; empty means "leave unchanged" to the scanner.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0) // empty name on a set = leave unchanged
    }
}

/// Talkgroup ID as sent by the scanner. Format depends on the system type
/// (decimal, hex, AFS "aa-fff", etc.), so it is kept raw until that is mapped out.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tgid(pub String);

impl FromStr for Tgid {
    type Err = std::convert::Infallible;
    /// Preserve the talkgroup text because its format depends on the system type.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tgid(s.to_string()))
    }
}

impl fmt::Display for Tgid {
    /// Write the original talkgroup text unchanged.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Declare an enum whose variants map one-to-one onto wire strings.
macro_rules! wire_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident = $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($(#[$vmeta])* $variant),+
        }

        impl FromStr for $name {
            type Err = ();
            /// Parse only the exact wire spelling assigned to a variant.
            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    $($wire => Ok(Self::$variant),)+ // exact match on the wire text
                    _ => Err(()),
                }
            }
        }

        impl fmt::Display for $name {
            /// Write this variant's fixed wire spelling.
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(match self {
                    $(Self::$variant => $wire),+
                })
            }
        }
    };
}

wire_enum! {
    /// System type, fixed when the system is created with CSY.
    SystemType {
        Conventional = "CNV",
        Motorola = "MOT",
        /// EDACS narrow/wide
        Edacs = "EDC",
        EdacsScat = "EDS",
        Ltr = "LTR",
        /// P25 Phase 1 / Phase 2 / X2-TDMA
        P25Standard = "P25S",
        P25OneFreq = "P25F",
        MotoTrbo = "TRBO",
        DmrOneFreq = "DMR",
    }
}

impl Default for SystemType {
    /// Use the simplest system type for newly created records.
    fn default() -> Self {
        SystemType::Conventional
    }
}

impl SystemType {
    /// Trunked systems have sites, trunk frequencies and TGID groups.
    pub fn is_trunked(self) -> bool {
        self != SystemType::Conventional
    }
}

wire_enum! {
    /// Modulation. Not every command accepts every value (sites: AUTO/FM/NFM).
    Modulation { Auto = "AUTO", Am = "AM", Fm = "FM", Nfm = "NFM", Wfm = "WFM", Fmb = "FMB" }
}

wire_enum! {
    /// GIN group type.
    GroupType { Channel = "C", Tgid = "T" }
}

impl Default for GroupType {
    /// New groups contain channels unless explicitly marked as talkgroup groups.
    fn default() -> Self {
        GroupType::Channel
    }
}

wire_enum! {
    /// Alert light color (the BCD325P2 only has red).
    AlertColor { Off = "OFF", Red = "RED" }
}

wire_enum! {
    /// Alert light pattern.
    AlertPattern { On = "0", Slow = "1", Fast = "2" }
}

wire_enum! {
    /// Which audio a channel or TGID passes.
    AudioType { All = "0", AnalogOnly = "1", DigitalOnly = "2" }
}

wire_enum! {
    /// TDMA slot for a TGID.
    TdmaSlot { Any = "ANY", Slot1 = "1", Slot2 = "2" }
}

wire_enum! {
    /// Motorola end code handling (TRN).
    EndCode { Ignore = "0", Analog = "1", AnalogAndDigital = "2" }
}

wire_enum! {
    /// Motorola/EDACS band type for a site (SIF). CUSTOM enables MCP band plans.
    MotBandType { Standard = "STD", Splinter = "SPL", Custom = "CUSTOM" }
}

wire_enum! {
    /// EDACS channel spacing (SIF).
    EdacsType { Wide = "WIDE", Narrow = "NARROW" }
}

wire_enum! {
    /// Location alert type (CLA, LIH, LIT, LIN).
    LocationAlertType { Poi = "POI", DangerousRoad = "DROAD", DangerousCrossing = "DXING" }
}

/// Delay time in seconds; only these values are valid (negative = resume early).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DelayTime(i8);

impl DelayTime {
    /// Delay values supported by the scanner, in seconds.
    pub const ALLOWED: [i8; 9] = [-10, -5, -2, 0, 1, 2, 5, 10, 30];

    /// Return a delay only when it is one of the scanner's supported values.
    pub fn new(secs: i8) -> Option<Self> {
        Self::ALLOWED.contains(&secs).then_some(DelayTime(secs)) // reject anything else
    }
    /// Return the delay in seconds.
    pub fn seconds(self) -> i8 {
        self.0
    }
}

impl FromStr for DelayTime {
    type Err = ();
    /// Parse a signed delay and reject values outside the scanner's allowed set.
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse::<i8>().ok().and_then(Self::new).ok_or(())
    }
}

impl fmt::Display for DelayTime {
    /// Write the delay as a signed decimal number of seconds.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Quick key or startup key. "." on the wire means deliberately unassigned.
/// Ranges differ by record: systems/sites 0-99, groups 1-9 and 0, startup keys 0-9.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAssignment {
    Unassigned,
    Key(u8),
}

impl FromStr for KeyAssignment {
    type Err = ();
    /// Parse a dot as unassigned and any other valid value as a key number.
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "." => Ok(KeyAssignment::Unassigned),
            n => n.parse().map(KeyAssignment::Key).map_err(|_| ()),
        }
    }
}

impl fmt::Display for KeyAssignment {
    /// Write an unassigned key as a dot, matching the scanner's convention.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyAssignment::Unassigned => f.write_str("."),
            KeyAssignment::Key(k) => write!(f, "{k}"),
        }
    }
}

/// Number tag 0-999, or "NONE".
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberTag {
    Unassigned,
    Tag(u16),
}

impl FromStr for NumberTag {
    type Err = ();
    /// Parse `NONE` or a decimal tag in the range 0 through 999.
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "NONE" => Ok(NumberTag::Unassigned),
            n => n.parse().ok().filter(|&v| v <= 999).map(NumberTag::Tag).ok_or(()),
        }
    }
}

impl fmt::Display for NumberTag {
    /// Write an unassigned tag as `NONE`; otherwise write its number.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberTag::Unassigned => f.write_str("NONE"),
            NumberTag::Tag(t) => write!(f, "{t}"),
        }
    }
}

/// CTCSS tone values for wire codes 64-113, in tenths of a hertz (670 = 67.0 Hz).
const CTCSS_TENTHS_HZ: [u16; 50] = [
    670, 693, 719, 744, 770, 797, 825, 854, 885, 915, // codes 64-73
    948, 974, 1000, 1035, 1072, 1109, 1148, 1188, 1230, 1273, // 74-83
    1318, 1365, 1413, 1462, 1514, 1567, 1598, 1622, 1655, 1679, // 84-93
    1713, 1738, 1773, 1799, 1835, 1862, 1899, 1928, 1966, 1995, // 94-103
    2035, 2065, 2107, 2181, 2257, 2291, 2336, 2418, 2503, 2541, // 104-113
];

/// DCS codes for codes 128-239, written as their usual octal labels
/// (23 means "DCS 023"). The last eight were appended out of order.
const DCS_LABELS: [u16; 112] = [
    23, 25, 26, 31, 32, 36, 43, 47, 51, 53, // codes 128-137
    54, 65, 71, 72, 73, 74, 114, 115, 116, 122, // 138-147
    125, 131, 132, 134, 143, 145, 152, 155, 156, 162, // 148-157
    165, 172, 174, 205, 212, 223, 225, 226, 243, 244, // 158-167
    245, 246, 251, 252, 255, 261, 263, 265, 266, 271, // 168-177
    274, 306, 311, 315, 325, 331, 332, 343, 346, 351, // 178-187
    356, 364, 365, 371, 411, 412, 413, 423, 431, 432, // 188-197
    445, 446, 452, 454, 455, 462, 464, 465, 466, 503, // 198-207
    506, 516, 523, 526, 532, 546, 565, 606, 612, 624, // 208-217
    627, 631, 632, 654, 662, 664, 703, 712, 723, 731, // 218-227
    732, 734, 743, 754, 6, 7, 15, 17, 21, 50, // 228-237 (006 onward appended)
    141, 214, // 238-239
];

/// Decoded meaning of a tone code.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Off,
    Search,
    Ctcss { tenths_hz: u16 },
    Dcs { code: u16 }, // octal label as a number, print with {:03}
}

/// Wire tone code (0-239). Only valid codes can be constructed.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToneCode(u16);

impl ToneCode {
    /// Validate a raw code from the wire.
    pub fn from_wire(code: u16) -> Option<Self> {
        // The gaps are reserved by the protocol, so accepting them would create invalid tones.
        match code {
            0 | 127 | 64..=113 | 128..=239 => Some(ToneCode(code)),
            _ => None, // 1-63, 114-126 and 240+ are undefined
        }
    }

    /// Encode a tone; None if the tone is not in Uniden's table.
    pub fn from_tone(tone: Tone) -> Option<Self> {
        // The tables are the source of truth; unsupported tone values have no wire code.
        let code = match tone {
            Tone::Off => 0,
            Tone::Search => 127,
            Tone::Ctcss { tenths_hz } => {
                64 + CTCSS_TENTHS_HZ.iter().position(|&t| t == tenths_hz)? as u16
            }
            Tone::Dcs { code } => 128 + DCS_LABELS.iter().position(|&d| d == code)? as u16,
        };
        Some(ToneCode(code))
    }

    /// Decode the code into a tone.
    pub fn tone(self) -> Tone {
        // Only valid wire codes are constructed, so the table indexes below stay in range.
        match self.0 {
            0 => Tone::Off,
            127 => Tone::Search,
            c @ 64..=113 => Tone::Ctcss { tenths_hz: CTCSS_TENTHS_HZ[(c - 64) as usize] },
            c => Tone::Dcs { code: DCS_LABELS[(c - 128) as usize] }, // from_wire guarantees 128-239
        }
    }

    /// Return the raw integer code used in scanner commands.
    pub fn wire(self) -> u16 {
        self.0
    }
}

impl FromStr for ToneCode {
    type Err = ();
    /// Parse and validate a tone code from decimal wire text.
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse().ok().and_then(Self::from_wire).ok_or(())
    }
}

impl fmt::Display for ToneCode {
    /// Write the raw tone code as decimal digits.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The P25NAC field: NAC, DMR color code, search, or none.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitalCode {
    Nac(u16),      // 0x000-0xFFF
    ColorCode(u8), // 0-15, sent as 0x1000 + code
    Search,        // "SRCH" (set commands and configuration)
    Unset,         // "NONE" (status replies such as GLG)
}

impl FromStr for DigitalCode {
    type Err = ();
    /// Parse the P25NAC field as a special value, NAC, or DMR color code.
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "SRCH" => Ok(DigitalCode::Search),
            "NONE" => Ok(DigitalCode::Unset),
            hex => match u32::from_str_radix(hex, 16).map_err(|_| ())? {
                v @ 0..=0xFFF => Ok(DigitalCode::Nac(v as u16)),
                v @ 0x1000..=0x100F => Ok(DigitalCode::ColorCode((v - 0x1000) as u8)),
                _ => Err(()),
            },
        }
    }
}

impl fmt::Display for DigitalCode {
    /// Write the value in the format used by the P25NAC field.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigitalCode::Nac(n) => write!(f, "{n:X}"), // unpadded hex; verify padding on hardware
            DigitalCode::ColorCode(c) => write!(f, "{:X}", 0x1000 + *c as u32),
            DigitalCode::Search => f.write_str("SRCH"),
            DigitalCode::Unset => f.write_str("NONE"),
        }
    }
}

/// TFQ color code: 0-15 or search.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorCodeSetting {
    Code(u8),
    Search,
}

impl FromStr for ColorCodeSetting {
    type Err = ();
    /// Parse `SRCH` or a color code from 0 through 15.
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "SRCH" => Ok(ColorCodeSetting::Search),
            n => n.parse().ok().filter(|&c| c <= 15).map(ColorCodeSetting::Code).ok_or(()),
        }
    }
}

impl fmt::Display for ColorCodeSetting {
    /// Write either the decimal color code or `SRCH`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorCodeSetting::Code(c) => write!(f, "{c}"),
            ColorCodeSetting::Search => f.write_str("SRCH"),
        }
    }
}

/// Degrees, minutes, and seconds in hundredths, plus hemisphere.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dms {
    pub degrees: u8,
    pub minutes: u8,       // 0-59
    pub centiseconds: u16, // SSss: 0-5999 (51.12 s = 5112)
    pub positive: bool,    // true = North or East
}

impl Dms {
    /// Signed decimal degrees, handy for maps and distance math.
    pub fn to_decimal(self) -> f64 {
        let v = self.degrees as f64
            + self.minutes as f64 / 60.0
            + self.centiseconds as f64 / 360_000.0; // hundredths of a second per degree
        if self.positive { v } else { -v }
    }

    /// Parse the fixed-width form; degree width is 2 for latitude and 3 for longitude.
    fn parse(s: &str, deg_digits: usize, pos: char, neg: char) -> Option<Self> {
        if !s.is_ascii() || s.len() != deg_digits + 7 {
            return None; // degrees + MM + SSss + hemisphere letter
        }
        let (num, hemi) = s.split_at(s.len() - 1);
        let positive = match hemi.chars().next()? {
            c if c == pos => true,
            c if c == neg => false,
            _ => return None,
        };
        // The earlier ASCII and width checks make these byte slices safe.
        let degrees = num[..deg_digits].parse().ok()?;
        let minutes = num[deg_digits..deg_digits + 2].parse().ok()?;
        let centiseconds = num[deg_digits + 2..].parse().ok()?;
        if minutes > 59 || centiseconds > 5999 {
            return None;
        }
        Some(Dms { degrees, minutes, centiseconds, positive })
    }

    /// Write degrees/minutes/seconds at the requested width and with its hemisphere.
    fn write(&self, f: &mut fmt::Formatter<'_>, deg_digits: usize, pos: char, neg: char) -> fmt::Result {
        let hemi = if self.positive { pos } else { neg };
        write!(f, "{:0w$}{:02}{:04}{}", self.degrees, self.minutes, self.centiseconds, hemi, w = deg_digits)
    }
}

/// Latitude, wire form DDMMSSssN / DDMMSSssS.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Latitude(pub Dms);

impl FromStr for Latitude {
    type Err = ();
    /// Parse fixed-width latitude and reject values beyond either pole.
    fn from_str(s: &str) -> Result<Self, ()> {
        Dms::parse(s, 2, 'N', 'S').filter(|d| d.degrees <= 90).map(Latitude).ok_or(())
    }
}

impl fmt::Display for Latitude {
    /// Write latitude with two degree digits and an N/S suffix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.write(f, 2, 'N', 'S')
    }
}

/// Longitude, wire form DDDMMSSssE / DDDMMSSssW.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Longitude(pub Dms);

impl FromStr for Longitude {
    type Err = ();
    /// Parse fixed-width longitude and reject values beyond the antimeridian.
    fn from_str(s: &str) -> Result<Self, ()> {
        Dms::parse(s, 3, 'E', 'W').filter(|d| d.degrees <= 180).map(Longitude).ok_or(())
    }
}

impl fmt::Display for Longitude {
    /// Write longitude with three degree digits and an E/W suffix.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.write(f, 3, 'E', 'W')
    }
}

/// Location-based scanning settings shared by sites, groups, and location alerts.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GeoFence {
    pub latitude: Option<Latitude>,
    pub longitude: Option<Longitude>,
    /// Sites/groups: 1-250 in 0.5 mile/km units. Location alerts: 1-80 in 0.05 units.
    pub range: Option<u8>,
    pub enabled: Option<bool>, // GPS_ENABLE
}

/// What the scanner reports about its own memory (RMB and MEM).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ScannerMemory {
    pub free_blocks: usize,
    pub percent_used: u8,
    pub systems: u32,
    pub sites: u32,
    pub channels: u32,
    pub location_alerts: u32,
}

/// One system, from SIN.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SystemRecord {
    pub sys_type: SystemType, // always present, even when protected
    pub name: Name,
    pub quick_key: Option<KeyAssignment>, // 0-99 or "."
    pub hold_time: Option<u8>,            // 0-255
    pub lockout: Option<bool>,
    pub delay: Option<DelayTime>,
    pub links: Links,
    /// Conventional: first/last channel group. Trunked: first/last site.
    pub child_head: Option<Index>,
    pub child_tail: Option<Index>,
    pub seq_no: Option<u16>, // 1-500, position in scan order
    pub start_key: Option<KeyAssignment>,
    pub number_tag: Option<NumberTag>,
    pub agc_analog: Option<bool>,
    pub agc_digital: Option<bool>,
    pub p25_waiting_ms: Option<u16>,
    /// Empty (None) when the system is protected, since the scanner hides it too.
    pub protect: Option<bool>,
}

/// Trunking settings for a trunked system, from TRN (same index as the system).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TrunkRecord {
    pub id_search: Option<bool>,  // false = ID Scan, true = ID Search
    pub status_bit: Option<bool>, // Motorola status bit
    pub end_code: Option<EndCode>,
    pub afs: Option<bool>,             // EDACS ID format: false decimal, true AFS
    pub emergency_alert: Option<u8>,   // 0 ignore, 1-9 tone
    pub emergency_level: Option<u8>,   // 0 off, 1-15
    pub fleet_map: Option<u8>,         // 0-15 preset, 16 custom
    pub custom_fleet_map: Option<String>, // 8 size codes, 0-E each
    pub tgid_group_head: Option<Index>,
    pub tgid_group_tail: Option<Index>,
    pub lockout_group_head: Option<Index>,
    pub lockout_group_tail: Option<Index>,
    pub hex_ids: Option<bool>, // Motorola/P25 ID display: false decimal, true hex
    pub emergency_color: Option<AlertColor>,
    pub emergency_pattern: Option<AlertPattern>,
    pub nac: Option<DigitalCode>,
    pub priority_id_scan: Option<bool>,
}

/// One site of a trunked system, from SIF.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SiteRecord {
    pub name: Name,
    pub quick_key: Option<KeyAssignment>,
    pub hold_time: Option<u8>,
    pub lockout: Option<bool>,
    pub modulation: Option<Modulation>, // AUTO, FM, NFM
    pub attenuator: Option<bool>,
    pub control_channel_only: Option<bool>, // spec: always on
    pub links: Links,
    pub system: Option<Index>,
    pub freq_head: Option<Index>, // first trunk frequency
    pub freq_tail: Option<Index>,
    pub seq_no: Option<u16>, // 1-256
    pub start_key: Option<KeyAssignment>,
    pub geofence: GeoFence,
    pub mot_band_type: Option<MotBandType>, // MOT/EDACS only
    pub edacs_type: Option<EdacsType>,      // EDACS only
    pub p25_waiting_ms: Option<u16>,
}

/// One entry of a Motorola custom band plan.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MotBand {
    pub lower: Freq,
    pub upper: Freq,
    pub step_code: u16, // e.g. 1250 = 12.5 kHz; see the MCP step table
    pub offset: i16,    // -1023 to 1023
}

/// Motorola custom band plan (MCP), six entries; unused entries are None.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MotBandPlan {
    pub bands: [Option<MotBand>; 6],
}

/// One entry of a P25 band plan, stored in plain units.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct P25Band {
    pub base_hz: u64,    // wire: hex(base_hz / 5)
    pub spacing_hz: u32, // wire: hex(spacing_hz / 125)
}

/// APCO P25 band plan (ABP), sixteen entries (0-F); "0" on the wire = unused.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct P25BandPlan {
    pub bands: [Option<P25Band>; 16],
}

/// One trunk frequency of a site, from TFQ.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TrunkFreqRecord {
    pub freq: Option<Freq>,
    pub lcn: Option<u16>, // EDACS 1-30, LTR 1-20, DMR/TRBO 0-4094; ignored for MOT/SCAT
    pub lockout: Option<bool>,
    pub links: Links,
    pub system: Option<Index>,
    pub site: Option<Index>, // the spec calls this GRP_INDEX, but it is the parent site
    pub number_tag: Option<NumberTag>, // EDACS SCAT only
    pub vol_offset: Option<i8>,        // EDACS SCAT only
    pub color_code: Option<ColorCodeSetting>,
}

/// A channel group or TGID group, from GIN.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct GroupRecord {
    pub group_type: GroupType,
    pub name: Name,
    pub quick_key: Option<KeyAssignment>, // 1-9, 0 (=10), or "."
    pub lockout: Option<bool>,
    pub links: Links,
    pub system: Option<Index>,
    pub child_head: Option<Index>, // first channel or TGID
    pub child_tail: Option<Index>,
    pub seq_no: Option<u16>,
    pub geofence: GeoFence,
}

/// A conventional channel, from CIN.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ChannelRecord {
    pub name: Name,
    pub freq: Option<Freq>,
    pub modulation: Option<Modulation>,
    pub tone: Option<ToneCode>,
    pub tone_lockout: Option<bool>,
    pub lockout: Option<bool>,
    pub priority: Option<bool>,
    pub attenuator: Option<bool>,
    pub alert_tone: Option<u8>,  // 0 off, 1-9
    pub alert_level: Option<u8>, // 0 auto, 1-15
    pub links: Links,
    pub system: Option<Index>,
    pub group: Option<Index>,
    pub audio_type: Option<AudioType>,
    pub nac: Option<DigitalCode>,
    pub number_tag: Option<NumberTag>,
    pub alert_color: Option<AlertColor>,
    pub alert_pattern: Option<AlertPattern>,
    pub vol_offset: Option<i8>, // -3 to +3
}

/// A talkgroup ID entry, from TIN.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct TgidRecord {
    pub name: Name,
    pub tgid: Option<Tgid>,
    pub lockout: Option<bool>,
    pub priority: Option<bool>,
    pub alert_tone: Option<u8>,
    pub alert_level: Option<u8>,
    pub links: Links,
    pub system: Option<Index>,
    pub group: Option<Index>,
    pub audio_type: Option<AudioType>,
    pub number_tag: Option<NumberTag>,
    pub alert_color: Option<AlertColor>,
    pub alert_pattern: Option<AlertPattern>,
    pub vol_offset: Option<i8>,
    pub tdma_slot: Option<TdmaSlot>,
}

/// A location alert (POI, dangerous road, dangerous crossing), from LIN.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct LocationAlertRecord {
    pub alert_type: LocationAlertType,
    pub name: Name,
    pub lockout: Option<bool>,
    pub alert_tone: Option<u8>, // 0 off, 1-4
    pub alert_level: Option<u8>,
    pub links: Links,
    pub seq_no: Option<u16>,
    pub geofence: GeoFence, // range here is 1-80 in 0.05 mile/km units; enabled is unused
    pub speed_limit: Option<u8>, // 0-200 mph or km/h
    pub heading: Option<u16>,    // degrees; 360 = any direction
    pub alert_color: Option<AlertColor>,
    pub alert_pattern: Option<AlertPattern>,
}

/// Everything read from the scanner's scan memory.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ScanDatabase {
    /// Name of the scanner model this database was read from or is meant for.
    #[serde(default)]
    pub model: Option<String>,
    pub systems: Vec<System>, // in scan order (system list link order)
}

impl ScanDatabase {
    /// (systems, sites, channels incl. talkgroups) as counted in this database.
    pub fn counts(&self) -> (usize, usize, usize) {
        // Count channels and talkgroups together because both consume the channel allowance.
        let (mut sites, mut channels) = (0, 0);
        for s in &self.systems {
            match &s.kind {
                SystemKind::Conventional { groups } => {
                    channels += groups.iter().map(|g| g.channels.len()).sum::<usize>();
                }
                SystemKind::Trunked { sites: st, tgid_groups, .. } => {
                    sites += st.len();
                    channels += tgid_groups.iter().map(|g| g.tgids.len()).sum::<usize>();
                }
            }
        }
        (self.systems.len(), sites, channels)
    }

    /// Memory blocks used: one per system, site, group, channel, talkgroup and trunk frequency.
    pub fn blocks_used(&self) -> usize {
        // Each system, group, site, and child record occupies its own memory block.
        self.systems
            .iter()
            .map(|s| {
                1 + match &s.kind {
                    SystemKind::Conventional { groups } => {
                        groups.iter().map(|g| 1 + g.channels.len()).sum::<usize>()
                    }
                    SystemKind::Trunked { sites, tgid_groups, .. } => {
                        sites.iter().map(|s| 1 + s.frequencies.len()).sum::<usize>()
                            + tgid_groups.iter().map(|g| 1 + g.tgids.len()).sum::<usize>()
                    }
                }
            })
            .sum()
    }

    /// Validate a name using the scanner's character and length limits.
    ///
    /// Checks a given name for validity according to the scanner's rules.
    ///
    /// Allowed characters:
    /// - a-z
    /// - A-Z
    /// - 0-9
    /// - !@#$%&*()-/<>.?
    /// - space
    ///
    /// Maximum length: 16 characters.
    pub fn validate_name(name: &str) -> bool {
        // Empty names and names past the scanner's limit are rejected before character checks.
        if name.is_empty() || name.len() > 16 {
            return false;
        }
        for c in name.chars() {
            if !(c.is_alphanumeric() || c.is_whitespace() || "!@#$%&*()-/<>.?".contains(c)) {
                return false;
            }
        }
        true
    }


    /// Check the database against the rules the scanner enforces.
    /// Returns every problem found; an empty list means valid.
    /// 
    /// # All Validation Steps:
    /// - Up to 500 systems, 
    /// - 1,000 total sites (max 256 per system), 
    /// - 20 groups per system, and 
    /// - 25,000 channels (500 max IDs or 1,000 frequencies per system)
    pub fn validate(&self) -> Vec<ValidationError> {
        // Keep collecting problems instead of stopping at the first one, so the user can fix them together.
        let mut errors = Vec::new();
        // Keep the first system index so a duplicate can point to both entries.
        let mut quick_keys: HashMap<u8, usize> = HashMap::new();
        let mut start_keys: HashMap<u8, usize> = HashMap::new();

        // Validate each system in the database.
        for (i, sys) in self.systems.iter().enumerate() {
            // Validate the system's quick key for duplicates.
            if let Some(KeyAssignment::Key(k)) = sys.info.quick_key {
                if let Some(&first) = quick_keys.get(&k) {
                    errors.push(ValidationError::DuplicateSystemQuickKey { key: k, first, duplicate: i });
                } else {
                    quick_keys.insert(k, i);
                }
            }

            // Validate the system's start key for duplicates.
            if let Some(KeyAssignment::Key(k)) = sys.info.start_key {
                if let Some(&first) = start_keys.get(&k) {
                    errors.push(ValidationError::DuplicateStartupKey { key: k, first, duplicate: i });
                } else {
                    start_keys.insert(k, i);
                }
            }

            // Validate the group's quick keys for duplicates.
            let group_keys: Vec<Option<KeyAssignment>> = match &sys.kind {
                SystemKind::Conventional { groups } => groups.iter().map(|g| g.info.quick_key).collect(),
                SystemKind::Trunked { tgid_groups, .. } => tgid_groups.iter().map(|g| g.info.quick_key).collect(),
            };

            // Track seen group quick keys to detect duplicates.
            // Group quick keys only need to be unique within their parent system.
            let mut seen: HashMap<u8, usize> = HashMap::new();
            for (g, key) in group_keys.into_iter().enumerate() {
                if let Some(KeyAssignment::Key(k)) = key {
                    if let Some(&first) = seen.get(&k) {
                        errors.push(ValidationError::DuplicateGroupQuickKey { system: i, key: k, first, duplicate: g });
                    } else {
                        seen.insert(k, g);
                    }
                }
            }

            // Validate the system's name.
            if !Self::validate_name(sys.info.name.as_str()) {
                errors.push(ValidationError::InvalidSystemName {
                    system: i,
                    name: sys.info.name.to_string(),
                });
            }

            // Validate the names of groups, channels, sites, and talkgroup groups.
            match &sys.kind {
                // Validate the names of groups and channels for conventional systems.
                SystemKind::Conventional { groups } => {
                    for (g, group) in groups.iter().enumerate() {
                        if !Self::validate_name(group.info.name.as_str()) {
                            errors.push(ValidationError::InvalidGroupName {
                                system: i,
                                group: g,
                                name: group.info.name.to_string(),
                            });
                        }
                        for (c, channel) in group.channels.iter().enumerate() {
                            if !Self::validate_name(channel.info.name.as_str()) {
                                errors.push(ValidationError::InvalidChannelName {
                                    system: i,
                                    group: g,
                                    channel: c,
                                    name: channel.info.name.to_string(),
                                });
                            }
                        }
                    }
                }
                // Validate the names of sites and talkgroup groups for trunked systems.
                SystemKind::Trunked { sites, tgid_groups, .. } => {
                    // for each site, validate its name.
                    for (s, site) in sites.iter().enumerate() {
                        if !Self::validate_name(site.info.name.as_str()) {
                            errors.push(ValidationError::InvalidSiteName {
                                system: i,
                                site: s,
                                name: site.info.name.to_string(),
                            });
                        }
                    }
                    // for each talkgroup group, validate its name and the names of its talkgroups.
                    for (g, group) in tgid_groups.iter().enumerate() {
                        if !Self::validate_name(group.info.name.as_str()) {
                            errors.push(ValidationError::InvalidTalkgroupGroupName {
                                system: i,
                                group: g,
                                name: group.info.name.to_string(),
                            });
                        }
                        // for each talkgroup in the group, validate its name.
                        for (t, talkgroup) in group.tgids.iter().enumerate() {
                            if !Self::validate_name(talkgroup.info.name.as_str()) {
                                errors.push(ValidationError::InvalidTalkgroupName {
                                    system: i,
                                    group: g,
                                    talkgroup: t,
                                    name: talkgroup.info.name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        errors
    }

    /// Make a new placeholder database.
    /// 
    /// The database is initialized with one system, one group, and one channel, all placeholder entries.
    pub fn new_placeholder() -> Self {
        Self {
            model: None,
            systems: vec![System {
                index: Index::default(),
                info: SystemRecord::default(),
                kind: SystemKind::Conventional {
                    groups: vec![ChannelGroup {
                        index: Index::default(),
                        info: GroupRecord::default(),
                        channels: vec![Channel {
                            index: Index::default(),
                            info: ChannelRecord::default(),
                        }],
                    }],
                },
            }],
        }
    }
}

/// A rule violation found by [`ScanDatabase::validate`]. Positions are 0-based
/// indices into the database's system list and the system's group list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateSystemQuickKey { key: u8, first: usize, duplicate: usize },
    DuplicateStartupKey { key: u8, first: usize, duplicate: usize },
    DuplicateGroupQuickKey { system: usize, key: u8, first: usize, duplicate: usize },
    InvalidSystemName { system: usize, name: String },
    InvalidGroupName { system: usize, group: usize, name: String },
    InvalidChannelName { system: usize, group: usize, channel: usize, name: String },
    InvalidSiteName { system: usize, site: usize, name: String },
    InvalidTalkgroupGroupName { system: usize, group: usize, name: String },
    InvalidTalkgroupName { system: usize, group: usize, talkgroup: usize, name: String },
    TooMany { what: &'static str, count: usize, max: usize },
    OutOfMemory { used: usize, max: usize },
    UnsupportedSystemType { system: usize, sys_type: String, model: &'static str },
    FrequencyOutOfRange { system: usize, place: String, freq: String, model: &'static str },
}

impl fmt::Display for ValidationError {
    /// Turn a validation problem into a short message for the user.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateSystemQuickKey { key, first, duplicate } => write!(
                f, "Systems {} and {} both use quick key {key}", first + 1, duplicate + 1),
            Self::DuplicateStartupKey { key, first, duplicate } => write!(
                f, "Systems {} and {} both use startup key {key}", first + 1, duplicate + 1),
            Self::DuplicateGroupQuickKey { system, key, first, duplicate } => write!(
                f, "In system {}, groups {} and {} both use quick key {key}",
                system + 1, first + 1, duplicate + 1),
            Self::InvalidSystemName { system, name } => write!(
                f, "System {} has invalid name {name:?}", system + 1),
            Self::InvalidGroupName { system, group, name } => write!(
                f, "Group {} in system {} has invalid name {name:?}", group + 1, system + 1),
            Self::InvalidChannelName { system, group, channel, name } => write!(
                f, "Channel {} in group {} of system {} has invalid name {name:?}",
                channel + 1, group + 1, system + 1),
            Self::InvalidSiteName { system, site, name } => write!(
                f, "Site {} in system {} has invalid name {name:?}", site + 1, system + 1),
            Self::InvalidTalkgroupGroupName { system, group, name } => write!(
                f, "Talkgroup group {} in system {} has invalid name {name:?}",
                group + 1, system + 1),
            Self::InvalidTalkgroupName { system, group, talkgroup, name } => write!(
                f, "Talkgroup {} in group {} of system {} has invalid name {name:?}",
                talkgroup + 1, group + 1, system + 1),
            Self::TooMany { what, count, max } => write!(
                f, "The database has {count} {what}; the scanner holds at most {max}"),
            Self::OutOfMemory { used, max } => write!(
                f, "The database needs {used} memory blocks; the scanner has {max}"),
            Self::UnsupportedSystemType { system, sys_type, model } => write!(
                f, "System {} is of type {sys_type}, which the {model} does not support", system + 1),
            Self::FrequencyOutOfRange { system, place, freq, model } => write!(
                f, "In system {}, {place} uses {freq}, outside the {model}'s coverage", system + 1),
        }
    }
}

/// A system and everything under it.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct System {
    pub index: Index, // where it was read from; stale after writes
    pub info: SystemRecord,
    pub kind: SystemKind,
}

/// Conventional and trunked systems have different children.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum SystemKind {
    Conventional {
        groups: Vec<ChannelGroup>,
    },
    Trunked {
        trunk: TrunkRecord,
        sites: Vec<Site>,
        tgid_groups: Vec<TgidGroup>,
    },
}

/// A conventional channel group or a trunked talkgroup group, with its entries.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ChannelGroup {
    pub index: Index,
    pub info: GroupRecord, // group_type == GroupType::Channel
    pub channels: Vec<Channel>,
}

/// One conventional channel and the scanner record that describes it.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Channel {
    pub index: Index,
    pub info: ChannelRecord,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// A trunked-system site, including its band plan and control frequencies.
pub struct Site {
    pub index: Index,
    pub info: SiteRecord,
    pub band_plan: Option<BandPlan>,
    pub frequencies: Vec<TrunkFreq>,
}

/// Site band plan, depending on system type.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum BandPlan {
    Motorola(MotBandPlan),
    P25(P25BandPlan),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// One trunk frequency stored under a trunked-system site.
pub struct TrunkFreq {
    pub index: Index,
    pub info: TrunkFreqRecord,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// A group of talkgroup IDs within a trunked system.
pub struct TgidGroup {
    pub index: Index,
    pub info: GroupRecord, // group_type == GroupType::Tgid
    pub tgids: Vec<TgidEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// One talkgroup ID and its scanner settings.
pub struct TgidEntry {
    pub index: Index,
    pub info: TgidRecord,
}
