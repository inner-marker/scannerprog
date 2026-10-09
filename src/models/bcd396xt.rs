//! Uniden BCD396XT (Operation Specification, section 7.13 Remote Command).
//!
//! The database commands match the BCD325P2 except that there is no DMR/MotoTRBO
//! support: no TRBO/DMR system types, no TDMA slot on TIN and no color code on TFQ.

use super::uniden::{self, Dialect};
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{ScanDatabase, ScannerMemory, SystemType, MAX_BLOCKS};

/// Uniden BCD396XT model definition and capabilities.
pub struct Bcd396xt;

/// Wire dialect used by models without DMR/MotoTRBO fields.
const DIALECT: Dialect = Dialect::NO_DMR;

impl ScannerModel for Bcd396xt {
    /// Return the model name used in the UI and database files.
    fn name(&self) -> &'static str {
        "BCD396XT"
    }

    /// Match the scanner's model reply without regard to letter case.
    fn matches(&self, mdl: &str) -> bool {
        mdl.eq_ignore_ascii_case("BCD396XT")
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
            // The spec lists no coverage; these are the published receive ranges
            // (25-54, 108-174, 216-512, 758-824, 849-869, 894-960, 1240-1300 MHz).
            // TODO: confirm against the BCD396XT manual.
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

    /// Read this model's database using its selected wire dialect.
    fn download(
        &self,
        link: &mut dyn Scanner,
        progress: Progress,
    ) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
        uniden::download(link, DIALECT, self.name(), progress)
    }

    /// Write the database to this model using its selected wire dialect.
    fn upload(&self, link: &mut dyn Scanner, db: &ScanDatabase, progress: Progress) -> Result<(), Error> {
        uniden::upload(link, DIALECT, db, progress)
    }
}
