//! Merging systems from an import (a scanner download, later RadioReference) into the database.

use crate::database::{Provenance, ScanDatabase, Source, System, normalize_channel_modulations};

/// What a merge did, for the message shown to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergeReport {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    /// Systems that were edited locally and so were not overwritten by the import.
    pub conflicts: usize,
}

impl MergeReport {
    /// One line for the message list.
    pub fn summary(&self) -> String {
        let mut text = format!(
            "{} system(s) added, {} updated, {} unchanged",
            self.added, self.updated, self.unchanged
        );
        if self.conflicts > 0 {
            text.push_str(&format!(
                "; {} kept as edited because they were changed locally",
                self.conflicts
            ));
        }
        text.push('.');
        text
    }
}

/// True if `incoming` describes the same system as `existing`.
fn same_system(existing: &System, incoming: &System) -> bool {
    let (a, b) = (&existing.provenance, &incoming.provenance);
    if a.external_id.is_some() && a.external_id == b.external_id && a.source == b.source {
        return true;
    }
    existing.info.sys_type == incoming.info.sys_type
        && existing
            .info
            .name
            .as_str()
            .trim()
            .eq_ignore_ascii_case(incoming.info.name.as_str().trim())
}

/// Merges freshly imported systems into `db`.
///
/// - A system that matches nothing is added at the end.
/// - A match that has not been edited since it was imported is replaced by the import.
/// - A match that was edited locally is kept, and counted as a conflict.
///
/// Systems in `db` that are absent from the import are never removed. A database that is still
/// just the starting placeholder is replaced outright.
pub fn merge_import(
    db: &mut ScanDatabase,
    source: &Source,
    mut incoming: Vec<System>,
) -> MergeReport {
    let mut report = MergeReport::default();
    if db.is_placeholder() {
        db.systems.clear();
    }
    normalize_channel_modulations(&mut incoming);
    // Each existing system can be matched by at most one incoming system.
    let mut claimed = vec![false; db.systems.len()];

    for mut new in incoming {
        new.provenance = Provenance {
            source: source.clone(),
            external_id: new.provenance.external_id.clone(),
            baseline: Some(new.content_hash()),
        };
        let found =
            (0..db.systems.len()).find(|&i| !claimed[i] && same_system(&db.systems[i], &new));
        let Some(i) = found else {
            db.systems.push(new);
            claimed.push(true);
            report.added += 1;
            continue;
        };
        claimed[i] = true;
        let old = &db.systems[i];
        if old.content() == new.content() {
            // Same data: only refresh where it came from, so it counts as unmodified.
            db.systems[i].provenance = new.provenance;
            report.unchanged += 1;
        } else if old.is_locally_modified() {
            report.conflicts += 1;
        } else {
            new.uid = old.uid;
            db.systems[i] = new;
            report.updated += 1;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{
        Channel, ChannelGroup, ChannelRecord, GroupRecord, Modulation, Name, SystemKind,
        SystemRecord, SystemType, Uid,
    };

    fn system(name: &str) -> System {
        System {
            uid: Uid::new(),
            provenance: Provenance::default(),
            index: Default::default(),
            info: SystemRecord {
                name: Name::new(name).unwrap(),
                sys_type: SystemType::Conventional,
                ..Default::default()
            },
            kind: SystemKind::Conventional { groups: Vec::new() },
        }
    }

    fn scanner() -> Source {
        Source::Scanner {
            model: "BCD325P2".into(),
        }
    }

    #[test]
    fn first_import_replaces_placeholder() {
        let mut db = ScanDatabase::new_placeholder();
        let r = merge_import(&mut db, &scanner(), vec![system("Fire")]);
        assert_eq!(r.added, 1);
        assert_eq!(db.systems.len(), 1);
        assert_eq!(db.systems[0].info.name.as_str(), "Fire");
    }

    #[test]
    fn imported_blank_channel_modulation_becomes_auto() {
        let mut incoming = system("Fire");
        let mut channel = ChannelRecord::default();
        channel.modulation = None;
        incoming.kind = SystemKind::Conventional {
            groups: vec![ChannelGroup {
                index: Default::default(),
                info: GroupRecord::default(),
                channels: vec![Channel {
                    index: Default::default(),
                    info: channel,
                }],
            }],
        };
        let mut db = ScanDatabase::default();

        merge_import(&mut db, &scanner(), vec![incoming]);

        let SystemKind::Conventional { groups } = &db.systems[0].kind else {
            panic!("imported system should be conventional");
        };
        assert_eq!(
            groups[0].channels[0].info.modulation,
            Some(Modulation::Auto)
        );
    }

    #[test]
    fn repeat_import_is_idempotent() {
        let mut db = ScanDatabase::default();
        merge_import(&mut db, &scanner(), vec![system("Fire"), system("Police")]);
        let uids: Vec<_> = db.systems.iter().map(|s| s.uid).collect();
        let r = merge_import(&mut db, &scanner(), vec![system("Fire"), system("Police")]);
        assert_eq!((r.added, r.updated, r.unchanged, r.conflicts), (0, 0, 2, 0));
        assert_eq!(uids, db.systems.iter().map(|s| s.uid).collect::<Vec<_>>());
    }

    #[test]
    fn unmodified_system_is_updated_and_keeps_identity() {
        let mut db = ScanDatabase::default();
        merge_import(&mut db, &scanner(), vec![system("Fire")]);
        let uid = db.systems[0].uid;
        let mut changed = system("Fire");
        changed.info.hold_time = Some(5);
        let r = merge_import(&mut db, &scanner(), vec![changed]);
        assert_eq!((r.updated, r.conflicts), (1, 0));
        assert_eq!(db.systems[0].info.hold_time, Some(5));
        assert_eq!(db.systems[0].uid, uid);
    }

    #[test]
    fn locally_edited_system_is_kept() {
        let mut db = ScanDatabase::default();
        merge_import(&mut db, &scanner(), vec![system("Fire")]);
        db.systems[0].info.hold_time = Some(9);
        let mut changed = system("Fire");
        changed.info.hold_time = Some(5);
        let r = merge_import(&mut db, &scanner(), vec![changed]);
        assert_eq!((r.updated, r.conflicts), (0, 1));
        assert_eq!(db.systems[0].info.hold_time, Some(9));
    }

    #[test]
    fn user_systems_are_never_overwritten_and_absent_ones_stay() {
        let mut db = ScanDatabase::default();
        db.systems.push(system("Mine"));
        merge_import(&mut db, &scanner(), vec![system("Fire")]);
        let mut theirs = system("Mine");
        theirs.info.hold_time = Some(1);
        let r = merge_import(&mut db, &scanner(), vec![theirs]);
        assert_eq!(r.conflicts, 1);
        assert_eq!(db.systems.len(), 2);
    }
}
