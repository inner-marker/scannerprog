//! Quick keys: reading and toggling system and group quick keys, and the temporary scan filter.
//! PLAN: edits apply immediately (with the 🔒 note) instead of being staged and applied in one batch.

use dioxus::prelude::*;

use super::capability::gate;
use crate::control::{
    discard_filter, load_group_keys, refresh_quick_keys, restore_filter, scan_only, toggle_group_key,
    toggle_system_key, FILTER_ACTIVE, GROUP_KEYS, QUICK_KEYS,
};
use crate::models::live::Capability;

/// Digit state of a quick key: 1 on, 2 off, anything else unassigned.
fn key_class(digit: u8) -> &'static str {
    match digit {
        b'1' => "qk on",
        b'2' => "qk off",
        _ => "qk none",
    }
}

/// System quick-key grid, the scanner's systems, and the temporary scan filter.
#[component]
pub fn QuickKeys() -> Element {
    let gate = gate(Capability::QuickKeysSystems);
    let enabled = gate.is_enabled();
    let keys = QUICK_KEYS.read().clone();
    let groups = GROUP_KEYS.read().clone();
    let filter = *FILTER_ACTIVE.read();

    rsx! {
        section { id: "quick-keys", class: "panel",
            h2 { "Quick keys" }
            if let Some(why) = gate.reason().filter(|_| !enabled) {
                p { class: "control-hint", "{why}" }
            }
            if filter {
                div { class: "filter-banner",
                    "Temporary scan filter active. "
                    button {
                        disabled: !enabled,
                        title: "Put the quick keys back the way they were.",
                        onclick: move |_| {
                            spawn(restore_filter());
                        },
                        "Restore"
                    }
                    button {
                        title: "Forget the saved settings without writing them to the scanner.",
                        onclick: move |_| discard_filter(),
                        "Discard"
                    }
                }
            }
            p { class: "control-hint",
                "Reading or changing quick keys briefly pauses scanning; the scanner is set back to Scan afterwards."
            }
            button {
                disabled: !enabled,
                onclick: move |_| {
                    spawn(refresh_quick_keys());
                },
                "Read from scanner"
            }
            if let Some(keys) = keys {
                div { class: "qk-grid",
                    for pos in 0..100usize {
                        {
                            let page = pos / 10;
                            let d = pos % 10;
                            let key = (if d == 9 { page * 10 } else { page * 10 + d + 1 }) as u8;
                            let digit = keys.qsl.as_bytes()[pos];
                            let name = keys
                                .systems
                                .iter()
                                .find(|s| s.quick_key == Some(key))
                                .map(|s| s.name.clone())
                                .unwrap_or_default();
                            rsx! {
                                button {
                                    class: key_class(digit),
                                    disabled: !enabled || digit == b'0',
                                    title: "{name}",
                                    onclick: move |_| {
                                        spawn(toggle_system_key(key));
                                    },
                                    "{key}"
                                }
                            }
                        }
                    }
                }
                div { class: "qk-systems",
                    for system in keys.systems.clone() {
                        {
                            let digit = system
                                .quick_key
                                .map(|k| keys.qsl.as_bytes()[crate::models::qsl_position(k)])
                                .unwrap_or(b'0');
                            let for_scan = system.clone();
                            let for_groups = system.clone();
                            let open = groups.as_ref().is_some_and(|g| g.system.index == system.index);
                            let label = match system.quick_key {
                                Some(k) => format!("[{k}] {}", system.name),
                                None => format!("[–] {}", system.name),
                            };
                            rsx! {
                                div { class: "qk-system",
                                    span { class: key_class(digit), "{label}" }
                                    button {
                                        disabled: !enabled || system.quick_key.is_none(),
                                        title: "Temporarily scan only this system.",
                                        onclick: move |_| {
                                            spawn(scan_only(for_scan.clone(), None));
                                        },
                                        "Scan only this"
                                    }
                                    button {
                                        disabled: !enabled,
                                        onclick: move |_| {
                                            spawn(load_group_keys(for_groups.clone()));
                                        },
                                        "Groups"
                                    }
                                    if open {
                                        if let Some(g) = groups.clone() {
                                            GroupList { keys: g, enabled }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Groups of one system with their quick keys.
#[component]
fn GroupList(keys: crate::control::GroupKeys, enabled: bool) -> Element {
    rsx! {
        ul { class: "qk-groups",
            for group in keys.groups.clone() {
                {
                    let digit = group
                        .quick_key
                        .map(|k| keys.qgl.as_bytes()[crate::models::qgl_position(k)])
                        .unwrap_or(b'0');
                    let system = keys.system.clone();
                    let index = system.index;
                    let key = group.quick_key;
                    rsx! {
                        li {
                            span { class: key_class(digit),
                                if let Some(k) = key { "[{k}] " } else { "[–] " }
                                "{group.name}"
                            }
                            button {
                                disabled: !enabled || key.is_none(),
                                onclick: move |_| {
                                    if let Some(k) = key {
                                        spawn(toggle_group_key(index, k));
                                    }
                                },
                                "On/Off"
                            }
                            button {
                                disabled: !enabled || key.is_none() || system.quick_key.is_none(),
                                title: "Temporarily scan only this group.",
                                onclick: move |_| {
                                    spawn(scan_only(system.clone(), key));
                                },
                                "Scan only this"
                            }
                        }
                    }
                }
            }
        }
    }
}
