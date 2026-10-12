//! Which parts of the common database a scanner model can use.
//!
//! [`project`] never changes the database it is given: it returns a copy holding only what the
//! model can store, plus a [`CompatReport`] describing everything that was left out or altered.

use std::fmt;

use super::Capabilities;
use crate::database::{Freq, ScanDatabase, System, SystemKind};

/// How an item is affected by a model's limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Left out of what is sent to the scanner.
    Skipped,
    /// Sent, but with some settings the model cannot store removed.
    Degraded,
}

/// One incompatibility between a system and a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatIssue {
    /// Position of the system in the database that was checked.
    pub system: usize,
    pub system_name: String,
    pub severity: Severity,
    pub message: String,
}

impl fmt::Display for CompatIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = match self.severity {
            Severity::Skipped => "skipped",
            Severity::Degraded => "changed",
        };
        write!(
            f,
            "System \"{}\" ({verb}): {}",
            self.system_name, self.message
        )
    }
}

/// Everything a model cannot take from a database as it is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompatReport {
    pub issues: Vec<CompatIssue>,
}

impl CompatReport {
    /// True if the model can use the whole database unchanged.
    pub fn is_compatible(&self) -> bool {
        self.issues.is_empty()
    }

    /// Issues of one severity.
    fn count(&self, severity: Severity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Number of skipped items.
    pub fn skipped(&self) -> usize {
        self.count(Severity::Skipped)
    }

    /// Number of items that lose settings.
    pub fn degraded(&self) -> usize {
        self.count(Severity::Degraded)
    }

    /// Issues that affect the system at this position.
    pub fn for_system(&self, system: usize) -> impl Iterator<Item = &CompatIssue> {
        self.issues.iter().filter(move |i| i.system == system)
    }

    /// True if the whole system is skipped, or something inside it is.
    pub fn system_has_skips(&self, system: usize) -> bool {
        self.for_system(system)
            .any(|i| i.severity == Severity::Skipped)
    }

    /// One line for banners and messages.
    pub fn summary(&self, model: &str) -> String {
        format!(
            "The {model} cannot use everything in this database: {} item(s) will be skipped and {} changed on upload.",
            self.skipped(),
            self.degraded()
        )
    }
}

/// The part of a database a model can use, and what had to be left out.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    pub db: ScanDatabase,
    pub report: CompatReport,
}

/// Copy of `db` limited to what a model with these capabilities can store.
pub fn project(model: &str, caps: &Capabilities, db: &ScanDatabase) -> Projection {
    let mut issues = Vec::new();
    // Original positions travel with the systems so issues point into the caller's database.
    let mut kept: Vec<(usize, System)> = Vec::new();

    for (i, sys) in db.systems.iter().enumerate() {
        let name = sys.info.name.to_string();
        let mut issue = |severity, message: String| {
            issues.push(CompatIssue {
                system: i,
                system_name: name.clone(),
                severity,
                message,
            });
        };
        if !caps.system_types.contains(&sys.info.sys_type) {
            issue(
                Severity::Skipped,
                format!(
                    "system type {} is not supported by the {model}",
                    sys.info.sys_type
                ),
            );
            continue;
        }

        let mut sys = sys.clone();
        let in_range = |f: Option<Freq>| {
            f.is_none_or(|f| {
                caps.freq_ranges
                    .iter()
                    .any(|&(lo, hi)| (lo..=hi).contains(&f.0))
            })
        };
        let mut dropped = Vec::new();
        let mut degraded = 0usize;
        match &mut sys.kind {
            SystemKind::Conventional { groups } => {
                for (g, group) in groups.iter_mut().enumerate() {
                    let mut c = 0;
                    group.channels.retain(|ch| {
                        c += 1;
                        let ok = in_range(ch.info.freq);
                        if !ok {
                            dropped.push(format!(
                                "channel {c} in group {} ({}) is outside the {model}'s frequency coverage",
                                g + 1,
                                ch.info.freq.map(|f| f.to_string()).unwrap_or_default()
                            ));
                        }
                        ok
                    });
                }
            }
            SystemKind::Trunked {
                sites, tgid_groups, ..
            } => {
                for (s, site) in sites.iter_mut().enumerate() {
                    let mut f = 0;
                    site.frequencies.retain(|tf| {
                        f += 1;
                        let ok = in_range(tf.info.freq);
                        if !ok {
                            dropped.push(format!(
                                "frequency {f} in site {} ({}) is outside the {model}'s frequency coverage",
                                s + 1,
                                tf.info.freq.map(|f| f.to_string()).unwrap_or_default()
                            ));
                        }
                        ok
                    });
                    if !caps.dmr_fields {
                        for tf in &mut site.frequencies {
                            degraded += usize::from(tf.info.color_code.take().is_some());
                        }
                    }
                }
                if !caps.dmr_fields {
                    for group in tgid_groups {
                        for t in &mut group.tgids {
                            degraded += usize::from(t.info.tdma_slot.take().is_some());
                        }
                    }
                }
            }
        }
        for message in dropped {
            issue(Severity::Skipped, message);
        }
        if degraded > 0 {
            issue(
                Severity::Degraded,
                format!(
                    "{degraded} DMR setting(s) (TDMA slot / color code) are not supported by the {model}"
                ),
            );
        }
        kept.push((i, sys));
    }

    // Drop systems from the end of the scan order until the model's limits are met.
    loop {
        let trial = ScanDatabase {
            systems: kept.iter().map(|(_, s)| s.clone()).collect(),
            ..db.clone()
        };
        let (systems, sites, channels) = trial.counts();
        let over = systems > caps.max_systems
            || sites > caps.max_sites
            || channels > caps.max_channels
            || trial.blocks_used() > caps.max_blocks;
        if !over {
            break;
        }
        let Some((i, sys)) = kept.pop() else { break };
        issues.push(CompatIssue {
            system: i,
            system_name: sys.info.name.to_string(),
            severity: Severity::Skipped,
            message: format!("does not fit in the {model}'s memory"),
        });
    }

    issues.sort_by_key(|i| i.system);
    Projection {
        db: ScanDatabase {
            systems: kept.into_iter().map(|(_, s)| s).collect(),
            ..db.clone()
        },
        report: CompatReport { issues },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{
        Channel, ChannelGroup, ChannelRecord, FORMAT_VERSION, Name, Provenance, SystemKind,
        SystemRecord, SystemType, TdmaSlot, TgidEntry, TgidGroup, TgidRecord, Uid,
    };
    use crate::models;

    fn conventional(name: &str, freqs: &[u32]) -> System {
        let channels = freqs
            .iter()
            .map(|&f| Channel {
                index: Default::default(),
                info: ChannelRecord {
                    name: Name::new("Ch").unwrap(),
                    freq: Some(Freq(f)),
                    ..Default::default()
                },
            })
            .collect();
        System {
            uid: Uid::new(),
            provenance: Provenance::default(),
            index: Default::default(),
            info: SystemRecord {
                name: Name::new(name).unwrap(),
                sys_type: SystemType::Conventional,
                ..Default::default()
            },
            kind: SystemKind::Conventional {
                groups: vec![ChannelGroup {
                    index: Default::default(),
                    info: Default::default(),
                    channels,
                }],
            },
        }
    }

    fn db(systems: Vec<System>) -> ScanDatabase {
        ScanDatabase {
            format_version: FORMAT_VERSION,
            model: None,
            systems,
        }
    }

    fn model(name: &str) -> &'static dyn models::ScannerModel {
        models::by_name(name).unwrap()
    }

    #[test]
    fn compatible_database_is_unchanged() {
        let d = db(vec![conventional("Fire", &[1_545_000])]);
        let p = model("BCD325P2").project(&d);
        assert!(p.report.is_compatible());
        assert_eq!(p.db, d);
    }

    #[test]
    fn out_of_range_channel_is_skipped_and_source_untouched() {
        // 100 MHz is outside every supported range.
        let d = db(vec![conventional("Mixed", &[1_545_000, 1_000_000_000])]);
        let p = model("BCD325P2").project(&d);
        assert_eq!(p.report.skipped(), 1);
        assert!(p.report.system_has_skips(0));
        let SystemKind::Conventional { groups } = &p.db.systems[0].kind else {
            unreachable!()
        };
        assert_eq!(groups[0].channels.len(), 1);
        let SystemKind::Conventional { groups } = &d.systems[0].kind else {
            unreachable!()
        };
        assert_eq!(groups[0].channels.len(), 2);
    }

    #[test]
    fn unsupported_system_type_is_skipped_on_bc346xt_only() {
        let mut sys = conventional("Digital", &[1_545_000]);
        sys.info.sys_type = SystemType::MotoTrbo;
        sys.kind = SystemKind::Trunked {
            trunk: Default::default(),
            sites: Vec::new(),
            tgid_groups: vec![TgidGroup {
                index: Default::default(),
                info: Default::default(),
                tgids: vec![TgidEntry {
                    index: Default::default(),
                    info: TgidRecord {
                        tdma_slot: Some(TdmaSlot::Slot1),
                        ..Default::default()
                    },
                }],
            }],
        };
        let d = db(vec![conventional("Plain", &[1_545_000]), sys]);
        let p = model("BC346XT").project(&d);
        assert_eq!(p.db.systems.len(), 1);
        assert_eq!(p.report.issues[0].system, 1);
        assert!(model("BCD325P2").project(&d).report.is_compatible());
    }

    #[test]
    fn dmr_settings_are_dropped_where_unsupported() {
        let mut sys = conventional("Trunk", &[]);
        sys.info.sys_type = SystemType::Ltr;
        sys.kind = SystemKind::Trunked {
            trunk: Default::default(),
            sites: Vec::new(),
            tgid_groups: vec![TgidGroup {
                index: Default::default(),
                info: Default::default(),
                tgids: vec![TgidEntry {
                    index: Default::default(),
                    info: TgidRecord {
                        tdma_slot: Some(TdmaSlot::Slot1),
                        ..Default::default()
                    },
                }],
            }],
        };
        let d = db(vec![sys]);
        let p = model("BCD396XT").project(&d);
        assert_eq!(p.report.degraded(), 1);
        let SystemKind::Trunked { tgid_groups, .. } = &p.db.systems[0].kind else {
            unreachable!()
        };
        assert_eq!(tgid_groups[0].tgids[0].info.tdma_slot, None);
    }

    #[test]
    fn systems_over_the_limit_are_dropped_from_the_end() {
        let max = model("BCD325P2").capabilities().max_systems;
        let d = db((0..=max)
            .map(|i| conventional(&format!("S{i}"), &[]))
            .collect());
        let p = model("BCD325P2").project(&d);
        assert_eq!(p.db.systems.len(), max);
        assert_eq!(p.report.issues.len(), 1);
        assert_eq!(p.report.issues[0].system, max);
    }
}
