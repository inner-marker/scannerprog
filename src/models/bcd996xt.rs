//! Uniden BCD996XT (Operation Specification, section 7.13 Remote Command).
//!
//! The mobile counterpart of the BCD996XT. The database commands are the same except that
//! several reserved fields are `[RECORD]` (tape-out); this app leaves those at the scanner's default.
//! There is no DMR/MotoTRBO support.

use super::uniden::{self, Dialect};
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{MAX_BLOCKS, ScanDatabase, ScannerMemory, SystemType};

/// Uniden BCD996XT model definition and capabilities.
pub struct Bcd996xt;

/// Wire dialect used by models without DMR/MotoTRBO fields.
const DIALECT: Dialect = Dialect::NO_DMR;

impl ScannerModel for Bcd996xt {
    /// Return the model name used in the UI and database files.
    fn name(&self) -> &'static str {
        "BCD996XT"
    }

    /// Match the scanner's model reply without regard to letter case.
    fn matches(&self, mdl: &str) -> bool {
        mdl.eq_ignore_ascii_case("BCD996XT")
    }

    /// Return the serial speed required by this scanner.
    fn baud_rate(&self) -> u32 {
        115_200
    }

    /// Describe the storage and frequency limits used during validation.
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            max_blocks: MAX_BLOCKS,
            max_systems: 500,
            max_sites: 1_000,
            max_channels: 25_000,
            system_types: &[
                SystemType::Conventional,
                SystemType::Motorola,
                SystemType::Edacs,
                SystemType::EdacsScat,
                SystemType::Ltr,
                SystemType::P25Standard,
                SystemType::P25OneFreq,
            ],
            // The spec lists no coverage; these are the receive ranges assumed to match the BCD396XT
            // (the spec's base-frequency field allows 25–1300 MHz).
            // TODO: confirm against the BCD996XT manual.
            dmr_fields: false,
            freq_ranges: &[
                (250_000, 540_000),
                (1_080_000, 1_740_000),
                (2_160_000, 5_120_000),
                (7_580_000, 8_240_000),
                (8_490_000, 8_690_000),
                (8_940_000, 9_600_000),
                (12_400_000, 13_000_000),
            ],
        }
    }

    /// Live-control profile (the XT/P2 command set; VOL 0–29, SQL 0–19, no power-off).
    fn control_profile(&self) -> Option<super::live::ControlProfile> {
        Some(super::live::xt_profile(
            self.name(),
            &self.capabilities(),
            &super::live::XtVariant::MOBILE,
        ))
    }

    /// Read this model's database using its selected wire dialect.
    fn download(
        &self,
        link: &mut dyn Scanner,
        progress: Progress,
    ) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
        uniden::download(link, DIALECT, self.name(), progress)
    }

    /// Write the database to this model using its selected wire dialect.
    fn upload(
        &self,
        link: &mut dyn Scanner,
        db: &ScanDatabase,
        progress: Progress,
    ) -> Result<(), Error> {
        uniden::upload(link, DIALECT, db, progress)
    }
}
