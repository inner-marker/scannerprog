//! Scanner models. Everything specific to one scanner model (wire protocol,
//! limits, supported features) lives under this module; the rest of the app talks
//! to a model only through [`ScannerModel`].

use std::fmt;

use crate::database::{
    Index, ScanDatabase, ScannerMemory, SystemKind, SystemType, ValidationError,
};

mod bcd325p2;
mod bc346xt;
mod bcd396xt;
mod uniden;

pub use uniden::Scanner;

/// Progress callback: `(systems done, systems total)`.
pub type Progress<'a> = &'a mut dyn FnMut(usize, usize);

/// What a model can store. Frequencies are in the wire unit of 100 Hz.
#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    /// Total records that fit in the scanner's memory pool.
    pub max_blocks: usize,
    /// Maximum number of systems.
    pub max_systems: usize,
    /// Maximum total number of trunked sites.
    pub max_sites: usize,
    /// Maximum combined number of conventional channels and talkgroups.
    pub max_channels: usize,
    /// System types the model can store.
    pub system_types: &'static [SystemType],
    /// Inclusive `(low, high)` ranges the scanner can receive.
    pub freq_ranges: &'static [(u32, u32)],
}

/// Everything the app needs to know about one scanner model.
pub trait ScannerModel: Sync {
    /// Display name, e.g. `BCD325P2`.
    fn name(&self) -> &'static str;
    /// True if the model string from the scanner's `MDL` reply is this model.
    fn matches(&self, mdl: &str) -> bool;
    /// Serial connection speed in bits per second.
    fn baud_rate(&self) -> u32;
    /// Memory limits, system types, and frequency coverage for this model.
    fn capabilities(&self) -> Capabilities;
    /// Reads the whole database (and, if available, memory statistics).
    fn download(
        &self,
        link: &mut dyn Scanner,
        progress: Progress,
    ) -> Result<(ScanDatabase, Option<ScannerMemory>), Error>;
    /// Replaces the scanner's contents with `db`.
    fn upload(&self, link: &mut dyn Scanner, db: &ScanDatabase, progress: Progress) -> Result<(), Error>;

    /// Generic database rules plus this model's limits.
    fn validate(&self, db: &ScanDatabase) -> Vec<ValidationError> {
        // Start with rules shared by all scanners, then apply this model's limits.
        let caps = self.capabilities();
        let mut errors = db.validate();

        // Record counts and block use are separate limits on many scanners.
        let (systems, sites, channels) = db.counts();
        for (what, count, max) in [
            ("systems", systems, caps.max_systems),
            ("sites", sites, caps.max_sites),
            ("channels", channels, caps.max_channels),
        ] {
            if count > max {
                errors.push(ValidationError::TooMany { what, count, max });
            }
        }
        let blocks = db.blocks_used();
        if blocks > caps.max_blocks {
            errors.push(ValidationError::OutOfMemory { used: blocks, max: caps.max_blocks });
        }

        for (i, sys) in db.systems.iter().enumerate() {
            // Capability checks keep unsupported protocols and out-of-band frequencies out of uploads.
            if !caps.system_types.contains(&sys.info.sys_type) {
                errors.push(ValidationError::UnsupportedSystemType {
                    system: i,
                    sys_type: sys.info.sys_type.to_string(),
                    model: self.name(),
                });
            }
            // The stored frequency value and the model ranges both use 100 Hz units.
            let mut check = |freq: Option<crate::database::Freq>, place: String| {
                if let Some(f) = freq {
                    if !caps.freq_ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&f.0)) {
                        errors.push(ValidationError::FrequencyOutOfRange {
                            system: i,
                            place,
                            freq: f.to_string(),
                            model: self.name(),
                        });
                    }
                }
            };
            match &sys.kind {
                SystemKind::Conventional { groups } => {
                    for (g, group) in groups.iter().enumerate() {
                        for (c, ch) in group.channels.iter().enumerate() {
                            check(ch.info.freq, format!("channel {} in group {}", c + 1, g + 1));
                        }
                    }
                }
                SystemKind::Trunked { sites, .. } => {
                    for (s, site) in sites.iter().enumerate() {
                        for (f, tf) in site.frequencies.iter().enumerate() {
                            check(tf.info.freq, format!("frequency {} in site {}", f + 1, s + 1));
                        }
                    }
                }
            }
        }
        errors
    }
}

/// The supported scanners, in the order used for the default model.
static MODELS: [&dyn ScannerModel; 3] = [&bcd325p2::Bcd325p2, &bcd396xt::Bcd396xt, &bc346xt::Bc346xt];

/// All supported models.
pub fn all() -> &'static [&'static dyn ScannerModel] {
    &MODELS
}

/// The model matching an `MDL` reply, if supported.
pub fn find(mdl: &str) -> Option<&'static dyn ScannerModel> {
    // The model implementation owns the case-insensitive check, so the registry stays generic.
    all().iter().copied().find(|m| m.matches(mdl))
}

/// The model with the given name (as stored in a database file).
pub fn by_name(name: &str) -> Option<&'static dyn ScannerModel> {
    // Database model names use the same lookup path as the scanner's MDL reply.
    find(name)
}

/// Model assumed when no scanner is connected.
pub fn default_model() -> &'static dyn ScannerModel {
    MODELS[0]
}

/// Adapter so the generic protocol code can run over a `&mut dyn Scanner`.
struct DynLink<'a>(&'a mut dyn Scanner);

impl Scanner for DynLink<'_> {
    /// Forward each command to the wrapped scanner connection.
    fn send(&mut self, cmd: &str) -> Result<String, Error> {
        self.0.send(cmd)
    }
}

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
    /// Format the scanner or parsing problem for display to the user.
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
    /// Wrap lower-level I/O failures so callers can use one error type.
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::SystemType;

    /// Checks that the registry resolves supported names and rejects an unknown model.
    #[test]
    fn registry_finds_models() {
        assert_eq!(find("BCD325P2").map(|m| m.name()), Some("BCD325P2"));
        assert_eq!(find("BCD396XT").map(|m| m.name()), Some("BCD396XT"));
        assert!(find("BC125AT").is_none());
    }

    /// Checks that the BC346XT does not advertise digital system types.
    #[test]
    fn bc346xt_has_no_digital_types() {
        let c = find("BC346XT").unwrap().capabilities();
        assert!(!c.system_types.contains(&SystemType::P25Standard));
        assert!(!c.system_types.contains(&SystemType::MotoTrbo));
        assert!(c.system_types.contains(&SystemType::Ltr));
    }

    /// Checks the BCD396XT excludes DMR while the BCD325P2 includes it.
    #[test]
    fn bcd396xt_has_no_dmr() {
        let c = find("BCD396XT").unwrap().capabilities();
        assert!(!c.system_types.contains(&SystemType::MotoTrbo));
        assert!(find("BCD325P2").unwrap().capabilities().system_types.contains(&SystemType::MotoTrbo));
    }
}
