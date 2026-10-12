//! Quick keys: staged system/group edits and the temporary scan filter.

use dioxus::prelude::*;

use super::capability::gate;
use crate::control::{
    FILTER_ACTIVE, GROUP_KEYS, HP_QUICK_KEYS, HpQuickKeyLevel, QUICK_KEY_DRAFT, QUICK_KEYS,
    apply_quick_keys, discard_filter, discard_quick_key_changes, load_group_keys,
    refresh_hp_quick_keys, refresh_quick_keys, restore_filter, scan_only, toggle_group_key,
    toggle_hp_quick_key, toggle_system_key,
};
use crate::models::live::{Capability, Family};
use crate::scanner_interaction::connected_model;

/// Visible system-key page (0–9), shared with keyboard shortcuts.
pub static QUICK_KEY_PAGE: GlobalSignal<u8> = Signal::global(|| 0);

/// Scanner quick-key number selected by a keyboard digit on `page`.
pub fn page_key(page: u8, digit: u8) -> u8 {
    if digit == 0 {
        page * 10
    } else {
        page * 10 + digit
    }
}

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
    let hp = connected_model()
        .and_then(|model| model.control_profile())
        .is_some_and(|profile| matches!(profile.family, Family::HomePatrol | Family::Sds));
    if hp {
        return rsx! { HpQuickKeysPanel {} };
    }
    let gate = gate(Capability::QuickKeysSystems);
    let enabled = gate.is_enabled();
    let keys = QUICK_KEYS.read().clone();
    let groups = GROUP_KEYS.read().clone();
    let filter = *FILTER_ACTIVE.read();
    let pending = QUICK_KEY_DRAFT.read().count();
    let page = *QUICK_KEY_PAGE.read();
    let editing_enabled = enabled && !filter;
    let scan_only_enabled = enabled && pending == 0;

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
                        disabled: !editing_enabled,
                        title: "Put the quick keys back the way they were.",
                        onclick: move |_| {
                            spawn(restore_filter());
                        },
                        "Restore"
                    }
                    if pending > 0 {
                        div { class: "qk-pending", role: "status",
                            strong { "{pending} pending quick-key change(s)." }
                            span { " Applying briefly pauses scanning." }
                            button {
                                disabled: !enabled || pending > 0,
                                title: if pending > 0 { "Apply or discard pending changes before reading again." },
                                onclick: move |_| {
                                    spawn(apply_quick_keys());
                                },
                                "Apply"
                            }
                            button { onclick: move |_| discard_quick_key_changes(), "Discard changes" }
                        }
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
                div { class: "qk-pages",
                    button {
                        disabled: page == 0,
                        onclick: move |_| *QUICK_KEY_PAGE.write() = page.saturating_sub(1),
                        "◀"
                    }
                    span { "Page {page} ({page * 10}–{page * 10 + 9})" }
                    button {
                        disabled: page == 9,
                        onclick: move |_| *QUICK_KEY_PAGE.write() = (page + 1).min(9),
                        "▶"
                    }
                }
                div { class: "qk-grid",
                    for offset in 0..10usize {
                        {
                            let pos = usize::from(page) * 10 + offset;
                            let d = offset;
                            let key = (if d == 9 { usize::from(page) * 10 } else { usize::from(page) * 10 + d + 1 }) as u8;
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
                                    disabled: !editing_enabled || digit == b'0',
                                    title: "{name}",
                                    onclick: move |_| {
                                        toggle_system_key(key);
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
                                        disabled: !scan_only_enabled || system.quick_key.is_none(),
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
                                            GroupList { keys: g, enabled: editing_enabled, scan_only_enabled }
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

fn hp_key_class(state: u8) -> &'static str {
    match state {
        2 => "qk on",
        1 => "qk off",
        _ => "qk none",
    }
}

#[component]
fn HpQuickKeysPanel() -> Element {
    let gate = gate(Capability::QuickKeysFavorites);
    let enabled = gate.is_enabled();
    let keys = HP_QUICK_KEYS.read().clone();
    let page = *QUICK_KEY_PAGE.read();

    rsx! {
        section { id: "quick-keys", class: "panel",
            h2 { "Live quick keys" }
            p { class: "control-hint",
                "Favorites, System, and Department quick keys apply immediately without entering Program Mode."
            }
            button {
                disabled: !enabled,
                onclick: move |_| {
                    spawn(refresh_hp_quick_keys());
                },
                "Read from scanner"
            }
            if let Some(keys) = keys {
                div { class: "qk-pages",
                    button {
                        disabled: page == 0,
                        onclick: move |_| *QUICK_KEY_PAGE.write() = page.saturating_sub(1),
                        "◀"
                    }
                    span { "Page {page} ({page * 10}–{page * 10 + 9})" }
                    button {
                        disabled: page == 9,
                        onclick: move |_| *QUICK_KEY_PAGE.write() = (page + 1).min(9),
                        "▶"
                    }
                }
                HpQuickKeyGrid {
                    label: "Favorites".to_owned(),
                    states: keys.favorites,
                    level: HpQuickKeyLevel::Favorite,
                    page,
                    enabled,
                }
                if !keys.systems.is_empty() {
                    HpQuickKeyGrid {
                        label: format!(
                            "Systems in Favorites QK {}",
                            keys.favorite_context.unwrap_or(0)
                        ),
                        states: keys.systems,
                        level: HpQuickKeyLevel::System,
                        page,
                        enabled,
                    }
                }
                if !keys.departments.is_empty() {
                    HpQuickKeyGrid {
                        label: format!(
                            "Departments in Favorites QK {} / System QK {}",
                            keys.favorite_context.unwrap_or(0),
                            keys.system_context.unwrap_or(0)
                        ),
                        states: keys.departments,
                        level: HpQuickKeyLevel::Department,
                        page,
                        enabled,
                    }
                }
            }
        }
    }
}

#[component]
fn HpQuickKeyGrid(
    label: String,
    states: Vec<u8>,
    level: HpQuickKeyLevel,
    page: u8,
    enabled: bool,
) -> Element {
    rsx! {
        h3 { "{label}" }
        div { class: "qk-grid",
            for offset in 0..10usize {
                {
                    let key = usize::from(page) * 10 + offset;
                    let state = states.get(key).copied().unwrap_or(0);
                    rsx! {
                        button {
                            class: hp_key_class(state),
                            disabled: !enabled || state == 0,
                            onclick: move |_| {
                                spawn(toggle_hp_quick_key(level, key as u8));
                            },
                            "{key}"
                        }
                    }
                }
            }
        }
    }
}

/// Groups of one system with their quick keys.
#[component]
fn GroupList(keys: crate::control::GroupKeys, enabled: bool, scan_only_enabled: bool) -> Element {
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
                                        toggle_group_key(index, k);
                                    }
                                },
                                "On/Off"
                            }
                            button {
                                disabled: !scan_only_enabled || key.is_none() || system.quick_key.is_none(),
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
