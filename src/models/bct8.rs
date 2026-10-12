//! BCT8 legacy live-control model.

use super::live;
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{MAX_BLOCKS, ScanDatabase, ScannerMemory, SystemType};

const TYPES: &[SystemType] = &[
    SystemType::Conventional,
    SystemType::Motorola,
    SystemType::Edacs,
    SystemType::EdacsScat,
    SystemType::Ltr,
];
const RANGES: &[(u32, u32)] = &[
    (250_000, 540_000),
    (1_080_000, 1_740_000),
    (4_000_000, 5_120_000),
    (8_060_000, 9_560_000),
];
const DATABASE_MESSAGE: &str = "BCT8 database transfer is not implemented; its fixed 250-channel legacy memory format is not compatible with the common database driver.";

pub struct Bct8;

impl ScannerModel for Bct8 {
    fn name(&self) -> &'static str {
        "BCT8"
    }

    fn matches(&self, mdl: &str) -> bool {
        mdl.eq_ignore_ascii_case("BCT8")
    }

    fn baud_rate(&self) -> u32 {
        57_600
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            max_blocks: MAX_BLOCKS,
            max_systems: 5,
            max_sites: 0,
            max_channels: 250,
            system_types: TYPES,
            dmr_fields: false,
            freq_ranges: RANGES,
        }
    }

    fn control_profile(&self) -> Option<live::ControlProfile> {
        Some(live::bct8_profile(self.name(), &self.capabilities()))
    }

    fn download(
        &self,
        _link: &mut dyn Scanner,
        _progress: Progress,
    ) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
        Err(Error::UnsupportedOperation(DATABASE_MESSAGE))
    }

    fn upload(
        &self,
        _link: &mut dyn Scanner,
        _db: &ScanDatabase,
        _progress: Progress,
    ) -> Result<(), Error> {
        Err(Error::UnsupportedOperation(DATABASE_MESSAGE))
    }
}
