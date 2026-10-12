//! B1/B2 and remaining C-family models added for Scanner-page live control in Phase 2.
//!
//! Their older database record layouts are not wire-compatible with the XT/P2 database driver.
//! Registering them for control must not silently risk corrupting scanner memory, so database
//! download/upload return an explicit error until a family-specific database codec is implemented.

use super::live::{self, Family};
use super::{Capabilities, Error, Progress, Scanner, ScannerModel};
use crate::database::{MAX_BLOCKS, ScanDatabase, ScannerMemory, SystemType};

const ANALOG_TYPES: &[SystemType] = &[
    SystemType::Conventional,
    SystemType::Motorola,
    SystemType::Edacs,
    SystemType::EdacsScat,
    SystemType::Ltr,
];
const DIGITAL_TYPES: &[SystemType] = &[
    SystemType::Conventional,
    SystemType::Motorola,
    SystemType::Edacs,
    SystemType::EdacsScat,
    SystemType::Ltr,
    SystemType::P25Standard,
];
const COMMON_RANGES: &[(u32, u32)] = &[
    (250_000, 540_000),
    (1_080_000, 1_740_000),
    (2_160_000, 5_120_000),
    (7_580_000, 8_240_000),
    (8_490_000, 8_690_000),
    (8_940_000, 9_600_000),
    (12_400_000, 13_000_000),
];
const DATABASE_MESSAGE: &str = "Database transfer is not implemented for this scanner's older memory-command dialect; live control is available.";

macro_rules! control_model {
    (
        $type:ident, $name:literal, $family:expr, $handheld:expr, $spectrum:expr,
        $power:expr, $types:expr, $sites:expr, $banks:expr
    ) => {
        pub struct $type;

        impl ScannerModel for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn matches(&self, mdl: &str) -> bool {
                mdl.eq_ignore_ascii_case($name)
            }

            fn baud_rate(&self) -> u32 {
                115_200
            }

            fn capabilities(&self) -> Capabilities {
                Capabilities {
                    max_blocks: MAX_BLOCKS,
                    max_systems: 500,
                    max_sites: $sites,
                    max_channels: 25_000,
                    system_types: $types,
                    dmr_fields: false,
                    freq_ranges: COMMON_RANGES,
                }
            }

            fn control_profile(&self) -> Option<live::ControlProfile> {
                Some(live::comma_profile(
                    self.name(),
                    &self.capabilities(),
                    $family,
                    $handheld,
                    $spectrum,
                    $power,
                    $banks,
                ))
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

control_model!(
    Br330t,
    "BR330T",
    Family::EarlyComma,
    true,
    false,
    true,
    ANALOG_TYPES,
    0,
    &[]
);
control_model!(
    Bcd996t,
    "BCD996T",
    Family::Gen1,
    false,
    true,
    false,
    DIGITAL_TYPES,
    1_000,
    &live::BANKS_WITH_MILITARY
);
control_model!(
    Bct15,
    "BCT15",
    Family::Gen1,
    false,
    true,
    false,
    ANALOG_TYPES,
    1_000,
    &live::BANKS_WITHOUT_MILITARY
);
control_model!(
    Bct15x,
    "BCT15X",
    Family::Xt,
    false,
    true,
    false,
    ANALOG_TYPES,
    1_000,
    &live::BANKS_WITHOUT_MILITARY
);
