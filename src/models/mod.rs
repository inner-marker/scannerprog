//! Scanner models. Everything specific to one scanner model (wire protocol,
//! limits, supported features) lives under this module; the rest of the app talks
//! to a model only through [`ScannerModel`].

use std::fmt;

use crate::database::{Index, ScanDatabase, ScannerMemory, SystemType};

mod bc346xt;
mod bcd325p2;
mod bcd396xt;
mod bcd996p2;
mod bcd996xt;
mod compat;
pub mod live;
mod phase2_models;
mod uniden;

pub use compat::{CompatReport, Projection};
pub use uniden::{
    GroupSummary, Scanner, SystemSummary, qgl_position, qsl_position, read_group_summaries,
    read_qgl, read_qsl, read_system_summaries, with_program_mode, write_qgl, write_qsl,
};

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
    /// True if the model stores DMR settings (TDMA slot, color code).
    pub dmr_fields: bool,
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
    /// What the Scanner page can do with this model; `None` if live control isn't supported.
    fn control_profile(&self) -> Option<live::ControlProfile> {
        None
    }
    /// Reads the whole database (and, if available, memory statistics).
    fn download(
        &self,
        link: &mut dyn Scanner,
        progress: Progress,
    ) -> Result<(ScanDatabase, Option<ScannerMemory>), Error>;
    /// Replaces the scanner's contents with `db`.
    fn upload(
        &self,
        link: &mut dyn Scanner,
        db: &ScanDatabase,
        progress: Progress,
    ) -> Result<(), Error>;

    /// The part of `db` this model can store, and what had to be left out.
    fn project(&self, db: &ScanDatabase) -> Projection {
        compat::project(self.name(), &self.capabilities(), db)
    }
}

/// The supported scanners, in the order used for the default model.
static MODELS: [&dyn ScannerModel; 9] = [
    &bcd325p2::Bcd325p2,
    &bcd396xt::Bcd396xt,
    &bc346xt::Bc346xt,
    &bcd996xt::Bcd996xt,
    &bcd996p2::Bcd996p2,
    &phase2_models::Br330t,
    &phase2_models::Bcd996t,
    &phase2_models::Bct15,
    &phase2_models::Bct15x,
];

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
pub struct DynLink<'a>(pub &'a mut dyn Scanner);

impl Scanner for DynLink<'_> {
    /// Forward each command to the wrapped scanner connection.
    fn send(&mut self, cmd: &str) -> Result<String, Error> {
        self.0.send(cmd)
    }

    fn stream(
        &mut self,
        start: &str,
        stop: &str,
        should_stop: &dyn Fn() -> bool,
        on_line: &mut dyn FnMut(String),
    ) -> Result<(), Error> {
        self.0.stream(start, stop, should_stop, on_line)
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
    UnexpectedReply {
        expected: &'static str,
        reply: String,
    },
    /// A required field was empty, or the reply ended before this position.
    MissingField { cmd: &'static str, pos: usize },
    /// A field could not be parsed as the expected type.
    BadValue {
        cmd: &'static str,
        pos: usize,
        value: String,
    },
    /// A create/append command returned -1 (no free memory blocks).
    OutOfMemory(&'static str),
    /// Following FWD links never reached -1, so the chain is looping.
    LinkLoop(Index),
    /// A name was too long, contained a comma, or used non-printable characters.
    InvalidName(String),
    /// A write function was handed the wrong kind of system.
    WrongSystemKind,
    /// The model is available for live control but its database dialect is not implemented.
    UnsupportedOperation(&'static str),
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
            Error::UnsupportedOperation(message) => f.write_str(message),
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
        assert_eq!(find("BR330T").map(|m| m.name()), Some("BR330T"));
        assert_eq!(find("BCT15X").map(|m| m.name()), Some("BCT15X"));
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
        assert!(
            find("BCD325P2")
                .unwrap()
                .capabilities()
                .system_types
                .contains(&SystemType::MotoTrbo)
        );
    }
}
