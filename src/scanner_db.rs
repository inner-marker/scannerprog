#![allow(dead_code)]

//! Data model for the Uniden BCD325P2 scan database.
//! Built from the BCD325P2 remote command protocol (ver. 1.02).
//! Standard library only; every rust block in the document forms this one module.

use std::fmt; // Display impls produce the exact wire format of each value
use std::str::FromStr; // FromStr impls parse the wire format back

/// Everything that can go wrong while talking to the scanner or parsing its replies.
#[derive(Debug)]
pub enum Error {
    /// Serial port or other I/O failure.
    Io(std::io::Error),
    /// The scanner answered with ERR, NG, FER, or ORER (bare or as "CMD,NG").
    Scanner { cmd: String, reply: String },
    /// The reply did not start with the expected command name, or was not "OK".
    UnexpectedReply { expected: &'static str, reply: String },
    /// A required field was empty, or the reply ended before this position.
    MissingField { cmd: &'static str, pos: usize },
    /// A field could not be parsed as the expected type.
    BadValue { cmd: &'static str, pos: usize, value: String },
    /// A create/append command returned -1 (no free memory blocks).
    OutOfMemory(&'static str),
    /// Following FWD links never reached -1, so the chain is looping.
    LinkLoop(Index),
    /// A name was too long, contained a comma, or used non-printable characters.
    InvalidName(String),
    /// A write function was handed the wrong kind of system.
    WrongSystemKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::Scanner { cmd, reply } => write!(f, "scanner rejected {cmd}: {reply}"),
            Error::UnexpectedReply { expected, reply } => {
                write!(f, "expected {expected} reply, got {reply:?}")
            }
            Error::MissingField { cmd, pos } => write!(f, "{cmd}: field {pos} missing or empty"),
            Error::BadValue { cmd, pos, value } => {
                write!(f, "{cmd}: field {pos} has bad value {value:?}")
            }
            Error::OutOfMemory(cmd) => write!(f, "{cmd}: scanner out of memory blocks"),
            Error::LinkLoop(idx) => write!(f, "link chain loops (seen at index {idx})"),
            Error::InvalidName(n) => write!(f, "invalid name {n:?}"),
            Error::WrongSystemKind => write!(f, "wrong system kind for this operation"),
        }
    }
}

impl std::error::Error for Error {}

// Lets `?` convert serial-port errors automatically.
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Handle to a record in the scanner's memory pool (1 to about 45000).
/// "-1" on the wire (end of list / none) is represented as Option::None.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Index(pub u16);

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0) // plain decimal, no padding
    }
}

/// Upper bound used to detect a looping chain while walking lists.
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Freq) // leading zeros are fine for u32 parsing
    }
}

impl fmt::Display for Freq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08}", self.0) // always 8 digits, zero-padded
    }
}

/// Alpha tag, up to 16 printable ASCII characters, no commas.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(String);

impl Name {
    pub const MAX_LEN: usize = 16;

    /// Validate a name you intend to send to the scanner.
    pub fn new(s: &str) -> Result<Self, Error> {
        let printable = s.bytes().all(|b| (0x20..=0x7E).contains(&b)); // ASCII glyphs only
        if s.len() > Self::MAX_LEN || s.contains(',') || !printable {
            return Err(Error::InvalidName(s.to_string()));
        }
        Ok(Name(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tgid(s.to_string()))
    }
}

impl fmt::Display for Tgid {
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
            fn from_str(s: &str) -> Result<Self, ()> {
                match s {
                    $($wire => Ok(Self::$variant),)+ // exact match on the wire text
                    _ => Err(()),
                }
            }
        }

        impl fmt::Display for $name {
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
    pub const ALLOWED: [i8; 9] = [-10, -5, -2, 0, 1, 2, 5, 10, 30];

    pub fn new(secs: i8) -> Option<Self> {
        Self::ALLOWED.contains(&secs).then_some(DelayTime(secs)) // reject anything else
    }
    pub fn seconds(self) -> i8 {
        self.0
    }
}

impl FromStr for DelayTime {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse::<i8>().ok().and_then(Self::new).ok_or(())
    }
}

impl fmt::Display for DelayTime {
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
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "." => Ok(KeyAssignment::Unassigned),
            n => n.parse().map(KeyAssignment::Key).map_err(|_| ()),
        }
    }
}

impl fmt::Display for KeyAssignment {
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
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "NONE" => Ok(NumberTag::Unassigned),
            n => n.parse().ok().filter(|&v| v <= 999).map(NumberTag::Tag).ok_or(()),
        }
    }
}

impl fmt::Display for NumberTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberTag::Unassigned => f.write_str("NONE"),
            NumberTag::Tag(t) => write!(f, "{t}"),
        }
    }
}

/// CTCSS tones for codes 64-113, in tenths of a Hz (670 = 67.0 Hz).
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
        match code {
            0 | 127 | 64..=113 | 128..=239 => Some(ToneCode(code)),
            _ => None, // 1-63, 114-126 and 240+ are undefined
        }
    }

    /// Encode a tone; None if the tone is not in Uniden's table.
    pub fn from_tone(tone: Tone) -> Option<Self> {
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
        match self.0 {
            0 => Tone::Off,
            127 => Tone::Search,
            c @ 64..=113 => Tone::Ctcss { tenths_hz: CTCSS_TENTHS_HZ[(c - 64) as usize] },
            c => Tone::Dcs { code: DCS_LABELS[(c - 128) as usize] }, // from_wire guarantees 128-239
        }
    }

    pub fn wire(self) -> u16 {
        self.0
    }
}

impl FromStr for ToneCode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse().ok().and_then(Self::from_wire).ok_or(())
    }
}

impl fmt::Display for ToneCode {
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
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "SRCH" => Ok(ColorCodeSetting::Search),
            n => n.parse().ok().filter(|&c| c <= 15).map(ColorCodeSetting::Code).ok_or(()),
        }
    }
}

impl fmt::Display for ColorCodeSetting {
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

    /// Parse the fixed-width form; deg_digits is 2 for latitude, 3 for longitude.
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
        let degrees = num[..deg_digits].parse().ok()?;
        let minutes = num[deg_digits..deg_digits + 2].parse().ok()?;
        let centiseconds = num[deg_digits + 2..].parse().ok()?;
        if minutes > 59 || centiseconds > 5999 {
            return None;
        }
        Some(Dms { degrees, minutes, centiseconds, positive })
    }

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
    fn from_str(s: &str) -> Result<Self, ()> {
        Dms::parse(s, 2, 'N', 'S').filter(|d| d.degrees <= 90).map(Latitude).ok_or(())
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.write(f, 2, 'N', 'S')
    }
}

/// Longitude, wire form DDDMMSSssE / DDDMMSSssW.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Longitude(pub Dms);

impl FromStr for Longitude {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        Dms::parse(s, 3, 'E', 'W').filter(|d| d.degrees <= 180).map(Longitude).ok_or(())
    }
}

impl fmt::Display for Longitude {
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

/// Sequential reader over the comma-separated fields of one reply.
pub struct Fields<'a> {
    cmd: &'static str,
    parts: Vec<&'a str>,
    pos: usize, // next field to read; field 0 is the command name
}

impl<'a> Fields<'a> {
    /// Split a reply and check it starts with `cmd` and is not an error.
    pub fn new(cmd: &'static str, reply: &'a str) -> Result<Self, Error> {
        let reply = reply.trim_end_matches(|c| c == '\r' || c == '\n');
        let parts: Vec<&str> = reply.split(',').collect();
        let is_err = |s: &str| matches!(s, "ERR" | "NG" | "FER" | "ORER");
        // Errors come back bare ("NG") or prefixed ("SIN,NG")
        if is_err(parts[0]) || (parts.len() == 2 && is_err(parts[1])) {
            return Err(Error::Scanner { cmd: cmd.to_string(), reply: reply.to_string() });
        }
        if parts[0] != cmd {
            return Err(Error::UnexpectedReply { expected: cmd, reply: reply.to_string() });
        }
        Ok(Fields { cmd, parts, pos: 1 })
    }

    /// Next field as raw text; error if the reply ran out.
    fn raw(&mut self) -> Result<&'a str, Error> {
        let pos = self.pos;
        self.pos += 1;
        self.parts.get(pos).copied().ok_or(Error::MissingField { cmd: self.cmd, pos })
    }

    fn bad(&self, pos: usize, value: &str) -> Error {
        Error::BadValue { cmd: self.cmd, pos, value: value.to_string() }
    }

    /// Skip n fields (reserved fields).
    pub fn skip(&mut self, n: usize) -> Result<(), Error> {
        for _ in 0..n {
            self.raw()?;
        }
        Ok(())
    }

    /// Raw text, possibly empty (used for names).
    pub fn text(&mut self) -> Result<String, Error> {
        Ok(self.raw()?.to_string())
    }

    /// Empty field -> None, otherwise parse with FromStr.
    pub fn opt<T: FromStr>(&mut self) -> Result<Option<T>, Error> {
        let pos = self.pos;
        let s = self.raw()?;
        if s.is_empty() {
            return Ok(None);
        }
        s.parse().map(Some).map_err(|_| self.bad(pos, s))
    }

    /// Like opt, but an empty field is an error.
    pub fn req<T: FromStr>(&mut self) -> Result<T, Error> {
        let pos = self.pos;
        self.opt()?.ok_or(Error::MissingField { cmd: self.cmd, pos })
    }

    /// "0"/"1" flag; empty -> None.
    pub fn flag(&mut self) -> Result<Option<bool>, Error> {
        let pos = self.pos;
        match self.raw()? {
            "" => Ok(None),
            "0" => Ok(Some(false)),
            "1" => Ok(Some(true)),
            other => Err(self.bad(pos, other)),
        }
    }

    /// Index field; "-1" or empty -> None.
    pub fn link(&mut self) -> Result<Option<Index>, Error> {
        let pos = self.pos;
        let s = self.raw()?;
        if s.is_empty() || s == "-1" {
            return Ok(None);
        }
        match s.parse::<u16>() {
            Ok(v) if v > 0 => Ok(Some(Index(v))), // valid indexes start at 1
            _ => Err(self.bad(pos, s)),
        }
    }

    /// Pair of REV/FWD index fields.
    pub fn links(&mut self) -> Result<Links, Error> {
        Ok(Links { rev: self.link()?, fwd: self.link()? })
    }

    /// Latitude, longitude, range, GPS-enable in that order.
    pub fn geofence(&mut self) -> Result<GeoFence, Error> {
        Ok(GeoFence {
            latitude: self.opt()?,
            longitude: self.opt()?,
            range: self.opt()?,
            enabled: self.flag()?,
        })
    }
}

/// Parse a single-index reply such as "SIH,123", "CSY,-1", or "FWD,456".
pub fn parse_index_reply(cmd: &'static str, reply: &str) -> Result<Option<Index>, Error> {
    Fields::new(cmd, reply)?.link()
}

/// Ask the scanner how many memory blocks are still free (RMB).
pub fn read_free_blocks<S: Scanner>(sc: &mut S) -> Result<usize, Error> {
    let reply = sc.send("RMB")?;
    let value = Fields::new("RMB", &reply)?.raw()?.to_string();
    value
        .parse()
        .map_err(|_| Error::BadValue { cmd: "RMB", pos: 1, value })
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

/// Read the scanner's own memory statistics. Must be called in Program Mode.
pub fn read_scanner_memory<S: Scanner>(sc: &mut S) -> Result<ScannerMemory, Error> {
    let free_blocks = read_free_blocks(sc)?;
    let reply = sc.send("MEM")?;
    let mut f = Fields::new("MEM", &reply)?;
    Ok(ScannerMemory {
        free_blocks,
        percent_used: f.req()?,
        systems: f.req()?,
        sites: f.req()?,
        channels: f.req()?,
        location_alerts: f.req()?,
    })
}

/// Check a "CMD,OK" reply.
pub fn expect_ok(cmd: &'static str, reply: &str) -> Result<(), Error> {
    match Fields::new(cmd, reply)?.raw()? {
        "OK" => Ok(()),
        _ => Err(Error::UnexpectedReply { expected: cmd, reply: reply.to_string() }),
    }
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

impl SystemRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("SIN", reply)?;
        let sys_type = f.req()?; // 1
        let name = Name(f.text()?); // 2
        let quick_key = f.opt()?; // 3
        let hold_time = f.opt()?; // 4
        let lockout = f.flag()?; // 5
        let delay = f.opt()?; // 6
        f.skip(5)?; // 7-11 reserved
        let links = f.links()?; // 12-13
        let child_head = f.link()?; // 14
        let child_tail = f.link()?; // 15
        let seq_no = f.opt()?; // 16
        let start_key = f.opt()?; // 17
        f.skip(5)?; // 18-22 reserved
        Ok(SystemRecord {
            sys_type,
            name,
            quick_key,
            hold_time,
            lockout,
            delay,
            links,
            child_head,
            child_tail,
            seq_no,
            start_key,
            number_tag: f.opt()?,     // 23
            agc_analog: f.flag()?,    // 24
            agc_digital: f.flag()?,   // 25
            p25_waiting_ms: f.opt()?, // 26
            protect: f.flag()?,       // 27 (28 is reserved, ignored)
        })
    }
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

impl TrunkRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("TRN", reply)?;
        let id_search = f.flag()?; // 1
        let status_bit = f.flag()?; // 2
        let end_code = f.opt()?; // 3
        let afs = f.flag()?; // 4
        f.skip(2)?; // 5-6 reserved
        let emergency_alert = f.opt()?; // 7
        let emergency_level = f.opt()?; // 8
        let fleet_map = f.opt()?; // 9
        let custom_fleet_map = Some(f.text()?).filter(|s| !s.is_empty()); // 10
        f.skip(10)?; // 11-20 reserved
        Ok(TrunkRecord {
            id_search,
            status_bit,
            end_code,
            afs,
            emergency_alert,
            emergency_level,
            fleet_map,
            custom_fleet_map,
            tgid_group_head: f.link()?,    // 21
            tgid_group_tail: f.link()?,    // 22
            lockout_group_head: f.link()?, // 23
            lockout_group_tail: f.link()?, // 24
            hex_ids: f.flag()?,            // 25
            emergency_color: f.opt()?,     // 26
            emergency_pattern: f.opt()?,   // 27
            nac: f.opt()?,                 // 28
            priority_id_scan: f.flag()?,   // 29
        })
    }
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

impl SiteRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("SIF", reply)?;
        f.skip(1)?; // 1 reserved
        let name = Name(f.text()?); // 2
        let quick_key = f.opt()?; // 3
        let hold_time = f.opt()?; // 4
        let lockout = f.flag()?; // 5
        let modulation = f.opt()?; // 6
        let attenuator = f.flag()?; // 7
        let control_channel_only = f.flag()?; // 8
        f.skip(2)?; // 9-10 reserved
        let links = f.links()?; // 11-12
        let system = f.link()?; // 13
        let freq_head = f.link()?; // 14
        let freq_tail = f.link()?; // 15
        let seq_no = f.opt()?; // 16
        let start_key = f.opt()?; // 17
        let geofence = f.geofence()?; // 18-21
        f.skip(1)?; // 22 reserved
        Ok(SiteRecord {
            name,
            quick_key,
            hold_time,
            lockout,
            modulation,
            attenuator,
            control_channel_only,
            links,
            system,
            freq_head,
            freq_tail,
            seq_no,
            start_key,
            geofence,
            mot_band_type: f.opt()?,  // 23
            edacs_type: f.opt()?,     // 24
            p25_waiting_ms: f.opt()?, // 25 (26 reserved, ignored)
        })
    }
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

impl MotBandPlan {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("MCP", reply)?;
        let mut bands = [None; 6];
        for band in bands.iter_mut() {
            // Four fields per entry: lower, upper, step, offset
            let parts = (f.opt()?, f.opt()?, f.opt()?, f.opt()?);
            if let (Some(lower), Some(upper), Some(step_code), Some(offset)) = parts {
                *band = Some(MotBand { lower, upper, step_code, offset });
            }
        }
        Ok(MotBandPlan { bands })
    }
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

impl P25BandPlan {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("ABP", reply)?;
        let mut bands = [None; 16];
        for (i, band) in bands.iter_mut().enumerate() {
            let pos = 1 + i * 2; // base field position for error reports
            let base = f.text()?;
            let spacing = f.text()?;
            if base.is_empty() || base == "0" {
                continue; // unused entry, or hidden by the protect bit
            }
            let base = u64::from_str_radix(&base, 16).map_err(|_| f.bad(pos, &base))?;
            let spacing = u32::from_str_radix(&spacing, 16).map_err(|_| f.bad(pos + 1, &spacing))?;
            *band = Some(P25Band { base_hz: base * 5, spacing_hz: spacing * 125 });
        }
        Ok(P25BandPlan { bands })
    }
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

impl TrunkFreqRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("TFQ", reply)?;
        let freq = f.opt()?; // 1
        let lcn = f.opt()?; // 2
        let lockout = f.flag()?; // 3
        let links = f.links()?; // 4-5
        let system = f.link()?; // 6
        let site = f.link()?; // 7
        f.skip(1)?; // 8 reserved
        let number_tag = f.opt()?; // 9
        let vol_offset = f.opt()?; // 10
        f.skip(1)?; // 11 reserved (spec omits the comma after it; see section 11)
        Ok(TrunkFreqRecord {
            freq,
            lcn,
            lockout,
            links,
            system,
            site,
            number_tag,
            vol_offset,
            color_code: f.opt()?, // 12
        })
    }
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

impl GroupRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("GIN", reply)?;
        Ok(GroupRecord {
            group_type: f.req()?,  // 1
            name: Name(f.text()?), // 2
            quick_key: f.opt()?,   // 3
            lockout: f.flag()?,    // 4
            links: f.links()?,     // 5-6
            system: f.link()?,     // 7
            child_head: f.link()?, // 8
            child_tail: f.link()?, // 9
            seq_no: f.opt()?,      // 10
            geofence: f.geofence()?, // 11-14
        })
    }
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

impl ChannelRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("CIN", reply)?;
        let name = Name(f.text()?); // 1
        let freq = f.opt()?; // 2
        let modulation = f.opt()?; // 3
        let tone = f.opt()?; // 4
        let tone_lockout = f.flag()?; // 5
        let lockout = f.flag()?; // 6
        let priority = f.flag()?; // 7
        let attenuator = f.flag()?; // 8
        let alert_tone = f.opt()?; // 9
        let alert_level = f.opt()?; // 10
        let links = f.links()?; // 11-12
        let system = f.link()?; // 13
        let group = f.link()?; // 14
        f.skip(1)?; // 15 reserved
        Ok(ChannelRecord {
            name,
            freq,
            modulation,
            tone,
            tone_lockout,
            lockout,
            priority,
            attenuator,
            alert_tone,
            alert_level,
            links,
            system,
            group,
            audio_type: f.opt()?,    // 16
            nac: f.opt()?,           // 17
            number_tag: f.opt()?,    // 18
            alert_color: f.opt()?,   // 19
            alert_pattern: f.opt()?, // 20
            vol_offset: f.opt()?,    // 21
        })
    }
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

impl TgidRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("TIN", reply)?;
        let name = Name(f.text()?); // 1
        let tgid = f.opt()?; // 2
        let lockout = f.flag()?; // 3
        let priority = f.flag()?; // 4
        let alert_tone = f.opt()?; // 5
        let alert_level = f.opt()?; // 6
        let links = f.links()?; // 7-8
        let system = f.link()?; // 9
        let group = f.link()?; // 10
        f.skip(1)?; // 11 reserved
        Ok(TgidRecord {
            name,
            tgid,
            lockout,
            priority,
            alert_tone,
            alert_level,
            links,
            system,
            group,
            audio_type: f.opt()?,    // 12
            number_tag: f.opt()?,    // 13
            alert_color: f.opt()?,   // 14
            alert_pattern: f.opt()?, // 15
            vol_offset: f.opt()?,    // 16
            tdma_slot: f.opt()?,     // 17
        })
    }
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

impl LocationAlertRecord {
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("LIN", reply)?;
        Ok(LocationAlertRecord {
            alert_type: f.req()?,  // 1
            name: Name(f.text()?), // 2
            lockout: f.flag()?,    // 3
            alert_tone: f.opt()?,  // 4
            alert_level: f.opt()?, // 5
            links: f.links()?,     // 6-7
            seq_no: f.opt()?,      // 8
            geofence: GeoFence {
                latitude: f.opt()?,  // 9
                longitude: f.opt()?, // 10
                range: f.opt()?,     // 11
                enabled: None,       // LIN has no GPS-enable field
            },
            speed_limit: f.opt()?,   // 12
            heading: f.opt()?,       // 13
            alert_color: f.opt()?,   // 14
            alert_pattern: f.opt()?, // 15
        })
    }
}

/// Everything read from the scanner's scan memory.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ScanDatabase {
    pub systems: Vec<System>, // in scan order (system list link order)
}

impl ScanDatabase {
    /// (systems, sites, channels incl. talkgroups) as counted in this database.
    pub fn counts(&self) -> (usize, usize, usize) {
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

    /// Make a new placeholder database 
    /// 
    /// The database is initialized with one system, one group, and one channel, all placeholder entries.
    pub fn new_placeholder() -> Self {
        Self {
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct ChannelGroup {
    pub index: Index,
    pub info: GroupRecord, // group_type == GroupType::Channel
    pub channels: Vec<Channel>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Channel {
    pub index: Index,
    pub info: ChannelRecord,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
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
pub struct TrunkFreq {
    pub index: Index,
    pub info: TrunkFreqRecord,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TgidGroup {
    pub index: Index,
    pub info: GroupRecord, // group_type == GroupType::Tgid
    pub tgids: Vec<TgidEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct TgidEntry {
    pub index: Index,
    pub info: TgidRecord,
}

/// Transport to the scanner. Implementations send `cmd` followed by "\r"
/// and return the reply line (without needing to strip the "\r").
pub trait Scanner {
    fn send(&mut self, cmd: &str) -> Result<String, Error>;
}

/// Run `f` in Program Mode, and always try to exit Program Mode afterward.
pub fn with_program_mode<S: Scanner, T>(
    sc: &mut S,
    f: impl FnOnce(&mut S) -> Result<T, Error>,
) -> Result<T, Error> {
    expect_ok("PRG", &sc.send("PRG")?)?;
    let result = f(sc);
    let exit = sc.send("EPG").and_then(|r| expect_ok("EPG", &r));
    match (result, exit) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), _) => Err(e), // report the original failure, not the EPG one
        (Ok(_), Err(e)) => Err(e),
    }
}

/// Follow a linked list from `head`. `read` fetches one record and returns
/// it with its FWD index.
fn walk_list<S: Scanner, T>(
    sc: &mut S,
    head: Option<Index>,
    mut read: impl FnMut(&mut S, Index) -> Result<(T, Option<Index>), Error>,
) -> Result<Vec<(Index, T)>, Error> {
    let mut out = Vec::new();
    let mut next = head;
    while let Some(idx) = next {
        if out.len() >= MAX_BLOCKS {
            return Err(Error::LinkLoop(idx)); // more records than memory blocks: a loop
        }
        let (item, fwd) = read(sc, idx)?;
        out.push((idx, item));
        next = fwd;
    }
    Ok(out)
}

fn read_group<S: Scanner>(sc: &mut S, idx: Index) -> Result<(GroupRecord, Option<Index>), Error> {
    let rec = GroupRecord::parse(&sc.send(&format!("GIN,{idx}"))?)?;
    let fwd = rec.links.fwd;
    Ok((rec, fwd))
}

/// Read the whole scan database. Must be called in Program Mode,
/// e.g. `with_program_mode(&mut port, read_database)`.
pub fn read_database<S: Scanner>(sc: &mut S) -> Result<ScanDatabase, Error> {
    read_database_with_progress(sc, |_, _| {})
}

/// Like [`read_database`], but calls `progress(done, total)` once the system
/// list is known and again after each system has been read.
pub fn read_database_with_progress<S: Scanner>(
    sc: &mut S,
    mut progress: impl FnMut(usize, usize),
) -> Result<ScanDatabase, Error> {
    let head = parse_index_reply("SIH", &sc.send("SIH")?)?;
    let records = walk_list(sc, head, |sc, idx| {
        let rec = SystemRecord::parse(&sc.send(&format!("SIN,{idx}"))?)?;
        let fwd = rec.links.fwd;
        Ok((rec, fwd))
    })?;

    let total = records.len();
    progress(0, total);
    let mut systems = Vec::with_capacity(total);
    for (index, info) in records {
        let kind = if info.sys_type.is_trunked() {
            read_trunked(sc, index, &info)?
        } else {
            SystemKind::Conventional { groups: read_channel_groups(sc, info.child_head)? }
        };
        systems.push(System { index, info, kind });
        progress(systems.len(), total);
    }
    Ok(ScanDatabase { systems })
}

/// Conventional system: groups, then channels in each group.
fn read_channel_groups<S: Scanner>(sc: &mut S, head: Option<Index>) -> Result<Vec<ChannelGroup>, Error> {
    let mut out = Vec::new();
    for (index, info) in walk_list(sc, head, |sc, i| read_group(sc, i))? {
        let channels = walk_list(sc, info.child_head, |sc, i| {
            let rec = ChannelRecord::parse(&sc.send(&format!("CIN,{i}"))?)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| Channel { index, info })
        .collect();
        out.push(ChannelGroup { index, info, channels });
    }
    Ok(out)
}

/// Trunked system: TRN, sites (with band plans and frequencies), then TGID groups.
fn read_trunked<S: Scanner>(sc: &mut S, sys: Index, info: &SystemRecord) -> Result<SystemKind, Error> {
    let trunk = TrunkRecord::parse(&sc.send(&format!("TRN,{sys}"))?)?;

    let site_records = walk_list(sc, info.child_head, |sc, i| {
        let rec = SiteRecord::parse(&sc.send(&format!("SIF,{i}"))?)?;
        let fwd = rec.links.fwd;
        Ok((rec, fwd))
    })?;
    let mut sites = Vec::with_capacity(site_records.len());
    for (index, site) in site_records {
        // Which band plan applies depends on the system type and site band setting
        let band_plan = if info.sys_type == SystemType::P25Standard {
            Some(BandPlan::P25(P25BandPlan::parse(&sc.send(&format!("ABP,{index}"))?)?))
        } else if site.mot_band_type == Some(MotBandType::Custom) {
            Some(BandPlan::Motorola(MotBandPlan::parse(&sc.send(&format!("MCP,{index}"))?)?))
        } else {
            None
        };
        let frequencies = walk_list(sc, site.freq_head, |sc, i| {
            let rec = TrunkFreqRecord::parse(&sc.send(&format!("TFQ,{i}"))?)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| TrunkFreq { index, info })
        .collect();
        sites.push(Site { index, info: site, band_plan, frequencies });
    }

    let mut tgid_groups = Vec::new();
    for (index, info) in walk_list(sc, trunk.tgid_group_head, |sc, i| read_group(sc, i))? {
        let tgids = walk_list(sc, info.child_head, |sc, i| {
            let rec = TgidRecord::parse(&sc.send(&format!("TIN,{i}"))?)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| TgidEntry { index, info })
        .collect();
        tgid_groups.push(TgidGroup { index, info, tgids });
    }

    Ok(SystemKind::Trunked { trunk, sites, tgid_groups })
}

/// Builder for comma-separated set commands.
pub struct SetCmd(Vec<String>);

impl SetCmd {
    pub fn new(cmd: &str) -> Self {
        SetCmd(vec![cmd.to_string()])
    }
    /// A present value.
    pub fn val(mut self, v: impl fmt::Display) -> Self {
        self.0.push(v.to_string());
        self
    }
    /// Optional value; None -> empty field (leave unchanged).
    pub fn opt<T: fmt::Display>(mut self, v: &Option<T>) -> Self {
        self.0.push(v.as_ref().map(|x| x.to_string()).unwrap_or_default());
        self
    }
    /// Optional 0/1 flag.
    pub fn flag(mut self, v: Option<bool>) -> Self {
        let s = match v {
            Some(true) => "1",
            Some(false) => "0",
            None => "",
        };
        self.0.push(s.to_string());
        self
    }
    /// n reserved (always empty) fields.
    pub fn rsv(mut self, n: usize) -> Self {
        self.0.extend(std::iter::repeat(String::new()).take(n));
        self
    }
    /// Optional latitude/longitude/range/enable quartet.
    pub fn geofence(self, g: &GeoFence) -> Self {
        self.opt(&g.latitude).opt(&g.longitude).opt(&g.range).flag(g.enabled)
    }
    pub fn build(self) -> String {
        self.0.join(",")
    }
}

/// Reserved fields between START_KEY and NUMBER_TAG in the SIN *set* layout.
/// The spec shows 6 here but 5 in the get reply; change this if testing shows 5.
pub const SIN_SET_RSV_AFTER_START_KEY: usize = 6;

impl SystemRecord {
    /// SIN set: index, name, qk, hold, lockout, delay, rsv x5, start key,
    /// rsv x6 (see constant), number tag, AGC analog, AGC digital, P25 waiting.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("SIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.quick_key)
            .opt(&self.hold_time)
            .flag(self.lockout)
            .opt(&self.delay)
            .rsv(5)
            .opt(&self.start_key)
            .rsv(SIN_SET_RSV_AFTER_START_KEY)
            .opt(&self.number_tag)
            .flag(self.agc_analog)
            .flag(self.agc_digital)
            .opt(&self.p25_waiting_ms)
            .build()
    }
}

impl GroupRecord {
    /// GIN set: index, name, qk, lockout, lat, lon, range, GPS enable.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("GIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.quick_key)
            .flag(self.lockout)
            .geofence(&self.geofence)
            .build()
    }
}

impl ChannelRecord {
    /// CIN set: the get layout minus the four index fields, plus the index up front.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("CIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.freq)
            .opt(&self.modulation)
            .opt(&self.tone)
            .flag(self.tone_lockout)
            .flag(self.lockout)
            .flag(self.priority)
            .flag(self.attenuator)
            .opt(&self.alert_tone)
            .opt(&self.alert_level)
            .rsv(1)
            .opt(&self.audio_type)
            .opt(&self.nac)
            .opt(&self.number_tag)
            .opt(&self.alert_color)
            .opt(&self.alert_pattern)
            .opt(&self.vol_offset)
            .build()
    }
}

impl TgidRecord {
    /// TIN set: index, name, TGID, lockout, priority, alert tone/level, rsv,
    /// audio type, number tag, alert color/pattern, volume offset, TDMA slot.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("TIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.tgid)
            .flag(self.lockout)
            .flag(self.priority)
            .opt(&self.alert_tone)
            .opt(&self.alert_level)
            .rsv(1)
            .opt(&self.audio_type)
            .opt(&self.number_tag)
            .opt(&self.alert_color)
            .opt(&self.alert_pattern)
            .opt(&self.vol_offset)
            .opt(&self.tdma_slot)
            .build()
    }
}

/// Run a create/append command and return the new index (-1 -> OutOfMemory).
fn alloc<S: Scanner>(sc: &mut S, cmd: &'static str, line: &str) -> Result<Index, Error> {
    parse_index_reply(cmd, &sc.send(line)?)?.ok_or(Error::OutOfMemory(cmd))
}

/// Recreate a conventional system on the scanner. Call in Program Mode.
/// Returns the new system index.
pub fn write_conventional<S: Scanner>(sc: &mut S, sys: &System) -> Result<Index, Error> {
    let groups = match &sys.kind {
        SystemKind::Conventional { groups } => groups,
        SystemKind::Trunked { .. } => return Err(Error::WrongSystemKind),
    };
    // The protect bit can only be set at creation time
    let protect = if sys.info.protect == Some(true) { 1 } else { 0 };
    let new_sys = alloc(sc, "CSY", &format!("CSY,{},{protect}", sys.info.sys_type))?;
    expect_ok("SIN", &sc.send(&sys.info.to_set(new_sys))?)?;

    for group in groups {
        let new_grp = alloc(sc, "AGC", &format!("AGC,{new_sys}"))?;
        expect_ok("GIN", &sc.send(&group.info.to_set(new_grp))?)?;
        for ch in &group.channels {
            let new_ch = alloc(sc, "ACC", &format!("ACC,{new_grp}"))?;
            expect_ok("CIN", &sc.send(&ch.info.to_set(new_ch))?)?;
        }
    }
    Ok(new_sys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Fake scanner: canned replies keyed by command; unknown commands get ERR.
    struct Mock(HashMap<&'static str, &'static str>);

    impl Scanner for Mock {
        fn send(&mut self, cmd: &str) -> Result<String, Error> {
            Ok(self.0.get(cmd).copied().unwrap_or("ERR").to_string())
        }
    }

    #[test]
    fn frequency_format() {
        // Spec example: 08510125 = 851.0125 MHz
        assert_eq!(Freq(8_510_125).to_string(), "08510125");
        assert_eq!(Freq::from_hz(851_012_500), Freq(8_510_125));
        assert_eq!("08510125".parse::<Freq>().unwrap().hz(), 851_012_500);
    }

    #[test]
    fn coordinates_round_trip() {
        // Spec examples: 40°42'51.12" N and 74°00'23.05" W
        let lat: Latitude = "40425112N".parse().unwrap();
        assert_eq!(lat.to_string(), "40425112N");
        assert!((lat.0.to_decimal() - 40.7142).abs() < 1e-6);
        let lon: Longitude = "074002305W".parse().unwrap();
        assert_eq!(lon.to_string(), "074002305W");
        assert!(lon.0.to_decimal() < 0.0); // west is negative
    }

    #[test]
    fn tone_codes() {
        assert_eq!(ToneCode::from_wire(76).unwrap().tone(), Tone::Ctcss { tenths_hz: 1000 });
        assert_eq!(ToneCode::from_wire(128).unwrap().tone(), Tone::Dcs { code: 23 });
        assert_eq!(ToneCode::from_wire(239).unwrap().tone(), Tone::Dcs { code: 214 });
        assert_eq!(ToneCode::from_tone(Tone::Dcs { code: 6 }).unwrap().wire(), 232);
        assert!(ToneCode::from_wire(120).is_none()); // undefined gap
    }

    #[test]
    fn digital_codes() {
        assert_eq!("293".parse::<DigitalCode>(), Ok(DigitalCode::Nac(0x293)));
        assert_eq!("100A".parse::<DigitalCode>(), Ok(DigitalCode::ColorCode(10)));
        assert_eq!(DigitalCode::ColorCode(10).to_string(), "100A");
    }

    #[test]
    fn p25_band_plan_hex() {
        // Spec example: base 851.00625 MHz = 0xA2510A2, spacing 6.25 kHz = 0x32
        let mut reply = String::from("ABP,A2510A2,32");
        for _ in 1..16 {
            reply.push_str(",0,0"); // fifteen unused entries
        }
        let plan = P25BandPlan::parse(&reply).unwrap();
        assert_eq!(plan.bands[0], Some(P25Band { base_hz: 851_006_250, spacing_hz: 6_250 }));
        assert!(plan.bands[1..].iter().all(Option::is_none));
    }

    #[test]
    fn channel_get_to_set() {
        let reply = "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,123,125,10,11,,0,,NONE,OFF,0,0";
        let ch = ChannelRecord::parse(reply).unwrap();
        assert_eq!(ch.freq, Some(Freq(1_545_500)));
        assert_eq!(ch.links, Links { rev: Some(Index(123)), fwd: Some(Index(125)) });
        assert_eq!(ch.nac, None); // empty field
        // Link fields drop out and the index goes up front
        assert_eq!(
            ch.to_set(Index(789)),
            "CIN,789,Dispatch,01545500,NFM,0,0,0,0,0,0,0,,0,,NONE,OFF,0,0"
        );
    }

    #[test]
    fn scanner_errors_are_detected() {
        assert!(matches!(SystemRecord::parse("NG"), Err(Error::Scanner { .. })));
        assert!(matches!(SystemRecord::parse("SIN,ERR"), Err(Error::Scanner { .. })));
        assert!(matches!(SystemRecord::parse("CIN,x"), Err(Error::UnexpectedReply { .. })));
    }

    #[test]
    fn short_reply_reports_position() {
        let err = GroupRecord::parse("GIN,C,Fire,1,0").unwrap_err();
        assert!(matches!(err, Error::MissingField { cmd: "GIN", pos: 5 }));
    }

    #[test]
    fn read_small_conventional_database() {
        let mut sc = Mock(HashMap::from([
            ("PRG", "PRG,OK"),
            ("EPG", "EPG,OK"),
            ("SIH", "SIH,10"),
            // System 10: one group (11), no neighbours
            ("SIN,10", "SIN,CNV,County,1,0,0,2,,,,,,-1,-1,11,11,1,.,,,,,,NONE,0,0,0,0,"),
            // Group 11: one channel (12)
            ("GIN,11", "GIN,C,Fireground,1,0,-1,-1,10,12,12,1,,,,"),
            ("CIN,12", "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,-1,-1,10,11,,0,,NONE,OFF,0,0"),
        ]));
        let db = with_program_mode(&mut sc, read_database).unwrap();

        assert_eq!(db.systems.len(), 1);
        let sys = &db.systems[0];
        assert_eq!(sys.info.name.as_str(), "County");
        assert_eq!(sys.info.delay, DelayTime::new(2));
        assert_eq!(sys.info.start_key, Some(KeyAssignment::Unassigned));
        match &sys.kind {
            SystemKind::Conventional { groups } => {
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].channels.len(), 1);
                assert_eq!(groups[0].channels[0].info.name.as_str(), "Dispatch");
            }
            SystemKind::Trunked { .. } => panic!("expected conventional"),
        }
    }

    #[test]
    fn database_round_trips_through_ron() {
        let mut sc = Mock(HashMap::from([
            ("PRG", "PRG,OK"),
            ("EPG", "EPG,OK"),
            ("SIH", "SIH,10"),
            ("SIN,10", "SIN,CNV,County,1,0,0,2,,,,,,-1,-1,11,11,1,.,,,,,,NONE,0,0,0,0,"),
            ("GIN,11", "GIN,C,Fireground,1,0,-1,-1,10,12,12,1,,,,"),
            ("CIN,12", "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,-1,-1,10,11,,0,,NONE,OFF,0,0"),
        ]));
        let db = with_program_mode(&mut sc, read_database).unwrap();
        let text = ron::ser::to_string_pretty(&db, ron::ser::PrettyConfig::default()).unwrap();
        let back: ScanDatabase = ron::from_str(&text).unwrap();
        assert_eq!(db, back);
    }

    #[test]
    fn name_validation() {
        assert!(Name::new("Fireground").is_ok());
        assert!(Name::new("This name is too long").is_err()); // over 16 chars
        assert!(Name::new("A,B").is_err()); // comma would break the field layout
    }
}
