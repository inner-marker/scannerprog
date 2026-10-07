use dioxus::prelude::*;

use crate::scanner_db::{BandPlan, Freq, MAX_BLOCKS, Modulation, ChannelGroup, Site, System, SystemKind, TgidGroup};
use crate::scanner_interaction::{SCANNER_MEMORY, SCAN_DATABASE};

fn text(v: String) -> Element {
    rsx! { input { r#type: "text", value: "{v}", disabled: true } }
}

fn opt_text<T: std::fmt::Display>(v: &Option<T>) -> Element {
    text(v.as_ref().map(|v| v.to_string()).unwrap_or_default())
}

fn opt_number<T: std::fmt::Display>(v: &Option<T>) -> Element {
    let v = v.as_ref().map(|v| v.to_string()).unwrap_or_default();
    rsx! { input { r#type: "number", value: "{v}", disabled: true } }
}

fn checkbox(v: &Option<bool>) -> Element {
    rsx! { input { r#type: "checkbox", checked: v.unwrap_or(false), disabled: true } }
}

/// Frequency in MHz, e.g. `01237000` -> `123.7`.
fn freq_input(v: &Option<Freq>) -> Element {
    let value = v.map(|f| f.mhz().to_string()).unwrap_or_default();
    rsx! {
        input { r#type: "number", step: "0.0001", min: "0", value: "{value}", disabled: true }
        " MHz"
    }
}

fn modulation_select(v: &Option<Modulation>) -> Element {
    let current = v.map(|m| m.to_string()).unwrap_or_default();
    rsx! {
        select { disabled: true, value: "{current}",
            option { value: "", selected: v.is_none(), "-" }
            for m in [
                Modulation::Auto,
                Modulation::Am,
                Modulation::Fm,
                Modulation::Nfm,
                Modulation::Wfm,
                Modulation::Fmb,
            ] {
                option { value: "{m}", selected: *v == Some(m), "{m}" }
            }
        }
    }
}

/// The downloaded scan database as a collapsible tree.
#[component]
pub fn DatabaseView() -> Element {
    let db = SCAN_DATABASE.read();
    let Some(db) = db.as_ref() else {
        return rsx! {};
    };
    let used = db.blocks_used();
    let (systems, sites, channels) = db.counts();
    let percent = used as f64 / MAX_BLOCKS as f64 * 100.0;
    rsx! {
        div { id: "database-view",
            p { id: "memory-usage",
                "Memory used: {used} / {MAX_BLOCKS} blocks ({percent:.1}%) "
                meter { min: 0, max: MAX_BLOCKS as f64, value: used as f64 }
            }
            if let Some(mem) = SCANNER_MEMORY.read().as_ref() {
                p { id: "scanner-memory-usage",
                    "Scanner reports: {mem.percent_used}% memory used, {mem.free_blocks} blocks free "
                    "(systems {mem.systems}, sites {mem.sites}, channels {mem.channels}, location alerts {mem.location_alerts}); as of the last download"
                }
                p {
                    "Downloaded here: systems {systems}, sites {sites}, channels/talkgroups {channels}"
                }
            }
            h2 { "Systems ({db.systems.len()})" }
            for system in db.systems.iter() {
                SystemView { system: system.clone() }
            }
        }
    }
}

#[component]
fn SystemView(system: System) -> Element {
    let name = system.info.name.to_string();
    let sys_type = format!("{:?}", system.info.sys_type);
    rsx! {
        details {
            summary {
                {text(name)}
                " ({sys_type})"
            }
            match system.kind {
                SystemKind::Conventional { groups } => rsx! {
                    for group in groups {
                        ChannelGroupView { group }
                    }
                },
                SystemKind::Trunked { sites, tgid_groups, .. } => rsx! {
                    for site in sites {
                        SiteView { site }
                    }
                    for group in tgid_groups {
                        TgidGroupView { group }
                    }
                },
            }
        }
    }
}

#[component]
fn ChannelGroupView(group: ChannelGroup) -> Element {
    let name = group.info.name.to_string();
    let count = group.channels.len();
    rsx! {
        details {
            summary {
                "Group: "
                {text(name)}
                " ({count} channels)"
            }
            table {
                thead {
                    tr {
                        th { "Name" }
                        th { "Frequency" }
                        th { "Modulation" }
                        th { "Tone" }
                        th { "Lockout" }
                    }
                }
                tbody {
                    for channel in group.channels.iter() {
                        tr {
                            td { {text(channel.info.name.to_string())} }
                            td { {freq_input(&channel.info.freq)} }
                            td { {modulation_select(&channel.info.modulation)} }
                            td { {opt_text(&channel.info.tone)} }
                            td { {checkbox(&channel.info.lockout)} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SiteView(site: Site) -> Element {
    let name = site.info.name.to_string();
    let count = site.frequencies.len();
    let plan = match &site.band_plan {
        Some(BandPlan::Motorola(_)) => "Motorola custom band plan",
        Some(BandPlan::P25(_)) => "P25 band plan",
        None => "default band plan",
    };
    rsx! {
        details {
            summary {
                "Site: "
                {text(name)}
                " ({count} frequencies, {plan})"
            }
            table {
                thead {
                    tr {
                        th { "Frequency" }
                        th { "LCN" }
                        th { "Lockout" }
                    }
                }
                tbody {
                    for freq in site.frequencies.iter() {
                        tr {
                            td { {freq_input(&freq.info.freq)} }
                            td { {opt_number(&freq.info.lcn)} }
                            td { {checkbox(&freq.info.lockout)} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TgidGroupView(group: TgidGroup) -> Element {
    let name = group.info.name.to_string();
    let count = group.tgids.len();
    rsx! {
        details {
            summary {
                "Talkgroups: "
                {text(name)}
                " ({count})"
            }
            table {
                thead {
                    tr {
                        th { "Name" }
                        th { "TGID" }
                        th { "Lockout" }
                        th { "Priority" }
                    }
                }
                tbody {
                    for tgid in group.tgids.iter() {
                        tr {
                            td { {text(tgid.info.name.to_string())} }
                            td { {opt_text(&tgid.info.tgid)} }
                            td { {checkbox(&tgid.info.lockout)} }
                            td { {checkbox(&tgid.info.priority)} }
                        }
                    }
                }
            }
        }
    }
}
