//! Uniden BCD325P2 (remote command protocol ver. 1.02).

use super::uniden::{self, Dialect};
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{ScanDatabase, ScannerMemory, SystemType, MAX_BLOCKS};

/// Uniden BCD325P2 model definition and capabilities.
pub struct Bcd325p2;

/// Wire dialect with the DMR and MotoTRBO fields enabled.
const DIALECT: Dialect = Dialect::DIGITAL_DMR;

impl ScannerModel for Bcd325p2 {
    /// Return the model name used in the UI and database files.
    fn name(&self) -> &'static str {
        "BCD325P2"
    }

    /// Match the scanner's model reply without regard to letter case.
    fn matches(&self, mdl: &str) -> bool {
        mdl.eq_ignore_ascii_case("BCD325P2")
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
                SystemType::MotoTrbo,
                SystemType::DmrOneFreq,
            ],
            // 25-512, 758-824, 849-869 and 894-960 MHz
            freq_ranges: &[
                (250_000, 5_120_000),
                (7_580_000, 8_240_000),
                (8_490_000, 8_690_000),
                (8_940_000, 9_600_000),
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
