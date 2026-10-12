//! HomePatrol-style live-control models (BCD436HP/536HP and SDS series).
//!
//! These radios do not expose the program-mode memory commands used by the common database
//! driver. Their Favorites List database is addressed through transient XML handles, so database
//! transfer remains explicitly unavailable while Scanner-page live control is supported.

use super::live::{self, Family};
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{MAX_BLOCKS, ScanDatabase, ScannerMemory, SystemType};

const TYPES: &[SystemType] = &[
    SystemType::Conventional,
    SystemType::Motorola,
    SystemType::Edacs,
    SystemType::EdacsScat,
    SystemType::Ltr,
    SystemType::P25Standard,
    SystemType::P25OneFreq,
    SystemType::MotoTrbo,
    SystemType::DmrOneFreq,
];
const RANGES: &[(u32, u32)] = &[
    (250_000, 540_000),
    (1_080_000, 1_740_000),
    (2_160_000, 5_120_000),
    (7_580_000, 8_240_000),
    (8_490_000, 8_690_000),
    (8_940_000, 9_600_000),
    (12_400_000, 13_000_000),
];
const DATABASE_MESSAGE: &str = "This HomePatrol-style scanner manages Favorites Lists internally; common-database transfer is not implemented.";

macro_rules! hp_model {
    ($type:ident, $name:literal, $family:expr, $mobile:expr $(, $alias:literal)?) => {
        pub struct $type;

        impl ScannerModel for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn matches(&self, mdl: &str) -> bool {
                mdl.eq_ignore_ascii_case($name) $(|| mdl.eq_ignore_ascii_case($alias))?
            }

            fn baud_rate(&self) -> u32 {
                115_200
            }

            fn capabilities(&self) -> Capabilities {
                Capabilities {
                    max_blocks: MAX_BLOCKS,
                    max_systems: 1_000,
                    max_sites: 1_000,
                    max_channels: 1_000_000,
                    system_types: TYPES,
                    dmr_fields: true,
                    freq_ranges: RANGES,
                }
            }

            fn control_profile(&self) -> Option<live::ControlProfile> {
                Some(live::hp_profile(self.name(), &self.capabilities(), $family, $mobile))
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
    };
}

hp_model!(Bcd436hp, "BCD436HP", Family::HomePatrol, false);
hp_model!(Bcd536hp, "BCD536HP", Family::HomePatrol, true);
hp_model!(Sds100, "SDS100", Family::Sds, false);
hp_model!(Sds150, "SDS150", Family::Sds, false, "SDS150GBT");
hp_model!(Sds200, "SDS200", Family::Sds, true);
