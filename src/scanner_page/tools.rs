//! Tools below the main panels: quick tune, jump to tag, the activity log, and the bottom drawer.

use dioxus::prelude::*;
use std::time::Duration;

use super::capability::CapButton;
use super::panels::run;
use crate::control::{clear_activity, ACTIVITY, LIVE, TXRX_LOG};
use crate::messages::{push_message, MessageKind};
use crate::models::live::{Action, Capability};
use crate::scanner_interaction::default_save_dir;

/// Set while a text box has focus, so keyboard shortcuts don't fire while typing.
pub type Typing = Signal<bool>;

/// Modulations accepted by `QSH`.
const MODULATIONS: [&str; 6] = ["AUTO", "AM", "FM", "NFM", "WFM", "FMB"];

/// Parses a MHz entry such as `162.55` into hertz.
fn parse_mhz(text: &str) -> Option<u64> {
    let mhz: f64 = text.trim().parse().ok()?;
    (mhz > 0.0 && mhz < 10_000.0).then(|| (mhz * 1_000_000.0).round() as u64)
}

/// Tune straight to a frequency.
#[component]
pub fn QuickTune() -> Element {
    let mut typing = use_context::<Typing>();
    let mut text = use_signal(String::new);
    let mut modulation = use_signal(|| "AUTO".to_owned());
    let mut recent = use_signal(Vec::<u64>::new);
    let freq = parse_mhz(&text());
    let blocked = match (text().trim().is_empty(), freq) {
        (true, _) => Some("Enter a frequency in MHz.".to_owned()),
        (false, None) => Some("That is not a valid frequency.".to_owned()),
        _ => None,
    };

    let mut go = move |hz: u64| {
        run(Action::QuickTune { freq_hz: hz, modulation: Some(modulation()) });
        let mut r = recent.write();
        r.retain(|f| *f != hz);
        r.insert(0, hz);
        r.truncate(5);
    };

    rsx! {
        section { id: "quick-tune", class: "panel",
            h2 { "Quick tune" }
            div { class: "tune-row",
                input {
                    r#type: "text",
                    inputmode: "decimal",
                    placeholder: "MHz, e.g. 162.550",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                    onfocus: move |_| typing.set(true),
                    onblur: move |_| typing.set(false),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            if let Some(hz) = parse_mhz(&text()) {
                                go(hz);
                            }
                        }
                    },
                }
                select { onchange: move |e| modulation.set(e.value()),
                    for m in MODULATIONS {
                        option { value: m, selected: modulation() == m, "{m}" }
                    }
                }
                CapButton {
                    cap: Capability::QuickTune,
                    blocked,
                    onclick: move |()| {
                        if let Some(hz) = freq {
                            go(hz);
                        }
                    },
                    "Tune"
                }
            }
            if !recent().is_empty() {
                div { class: "tune-recent",
                    for hz in recent() {
                        CapButton {
                            cap: Capability::QuickTune,
                            onclick: move |()| go(hz),
                            "{hz as f64 / 1e6:.4}"
                        }
                    }
                }
            }
        }
    }
}

/// Jump to a system or channel by its number tag.
#[component]
pub fn JumpToTag() -> Element {
    let mut typing = use_context::<Typing>();
    let mut sys = use_signal(String::new);
    let mut chan = use_signal(String::new);
    let parse = |s: &str| s.trim().parse::<u16>().ok();
    let (s, c) = (parse(&sys()), parse(&chan()));
    let bad = |t: &str, v: Option<u16>| !t.trim().is_empty() && v.is_none();
    let blocked = if sys().trim().is_empty() && chan().trim().is_empty() {
        Some("Enter a system tag, a channel tag, or both.".to_owned())
    } else if bad(&sys(), s) || bad(&chan(), c) {
        Some("Tags are numbers from 0 to 999.".to_owned())
    } else {
        None
    };

    rsx! {
        section { id: "jump-tag", class: "panel",
            h2 { "Jump to tag" }
            div { class: "tune-row",
                input {
                    r#type: "text",
                    placeholder: "System tag",
                    value: "{sys}",
                    oninput: move |e| sys.set(e.value()),
                    onfocus: move |_| typing.set(true),
                    onblur: move |_| typing.set(false),
                }
                input {
                    r#type: "text",
                    placeholder: "Channel tag",
                    value: "{chan}",
                    oninput: move |e| chan.set(e.value()),
                    onfocus: move |_| typing.set(true),
                    onblur: move |_| typing.set(false),
                }
                CapButton {
                    cap: Capability::JumpToTag,
                    blocked,
                    onclick: move |()| run(Action::JumpToTag { sys: s, chan: c }),
                    "Jump"
                }
            }
        }
    }
}

/// Receptions seen while remote control was on, newest first.
#[component]
pub fn ActivityLog() -> Element {
    let log = ACTIVITY.read().clone();
    let rows: Vec<_> = log.entries.iter().rev().cloned().collect();
    let csv = log.to_csv();
    let empty = rows.is_empty();

    rsx! {
        section { id: "activity", class: "panel",
            h2 { "Activity" }
            div { class: "activity-tools",
                button {
                    disabled: empty,
                    onclick: move |_| {
                        let name = format!("activity-{}.csv", chrono::Local::now().format("%Y%m%d-%H%M%S"));
                        match default_save_dir().map(|d| d.join(name)) {
                            Some(path) => match std::fs::write(&path, &csv) {
                                Ok(()) => push_message(MessageKind::Note, "control", format!("Activity saved to {}", path.display())),
                                Err(e) => push_message(MessageKind::Warning, "control", format!("Could not save the activity log: {e}")),
                            },
                            None => push_message(MessageKind::Warning, "control", "No folder to save into".to_string()),
                        }
                    },
                    "Export CSV"
                }
                button {
                    disabled: empty,
                    onclick: move |_| {
                        clear_activity();
                    },
                    "Hide all"
                }
            }
            if empty {
                p { class: "control-hint", "Nothing received yet." }
            }
            table { class: "activity-table",
                for e in rows {
                    {
                        let place = [e.system.clone(), e.channel.clone()].into_iter().flatten().collect::<Vec<_>>().join(" · ");
                        let freq = e.freq_hz.map(|f| format!("{:.4}", f as f64 / 1e6)).or_else(|| e.tgid.clone().map(|t| format!("TGID {t}"))).unwrap_or_default();
                        let dur = e.duration.map_or_else(|| "…".to_owned(), |d: Duration| format!("{}s", d.as_secs()));
                        let hz = e.freq_hz;
                        rsx! {
                            tr {
                                td { "{e.clock}" }
                                td { "{place}" }
                                td { "{freq}" }
                                td { "{dur}" }
                                td {
                                    CapButton {
                                        cap: Capability::QuickTune,
                                        blocked: if hz.is_none() { Some("This reception has no frequency to tune to.".to_owned()) } else { None },
                                        onclick: move |()| {
                                            if let Some(freq_hz) = hz {
                                                run(Action::QuickTune { freq_hz, modulation: None });
                                            }
                                        },
                                        "Tune"
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

/// Tabs of the bottom drawer.
#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
enum Tab {
    #[default]
    Log,
    Spectrum,
    Remote,
}

/// Drawer state remembered between runs.
#[derive(Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, Default)]
struct DrawerState {
    open: bool,
    tab: Tab,
}

/// Where the drawer state is stored.
fn drawer_path() -> Option<std::path::PathBuf> {
    Some(dirs::data_local_dir()?.join("Scanner Programmer").join("scanner-drawer.ron"))
}

fn load_drawer() -> DrawerState {
    drawer_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| ron::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_drawer(state: DrawerState) {
    if let Some(path) = drawer_path() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = ron::ser::to_string(&state) {
            let _ = std::fs::write(path, text);
        }
    }
}

/// Collapsible drawer: the TX/RX log, and placeholders for the spectrum and virtual remote.
#[component]
pub fn BottomDrawer() -> Element {
    let mut state = use_signal(load_drawer);
    let s = state();
    let mut set = move |next: DrawerState| {
        state.set(next);
        save_drawer(next);
    };
    let lines = TXRX_LOG.read().iter().rev().take(200).map(|l| (if l.tx { "→" } else { "←" }, l.clone())).collect::<Vec<_>>();
    let live_summary = LIVE.read().freq_mhz().map_or_else(String::new, |m| format!("{m:.4} MHz"));

    rsx! {
        section { id: "drawer", class: if s.open { "open" } else { "" },
            div { class: "drawer-tabs",
                button { onclick: move |_| set(DrawerState { open: !s.open, tab: s.tab }), if s.open { "▼" } else { "▲" } }
                button { class: if s.tab == Tab::Log { "active" }, onclick: move |_| set(DrawerState { open: true, tab: Tab::Log }), "Log" }
                button { class: if s.tab == Tab::Spectrum { "active" }, onclick: move |_| set(DrawerState { open: true, tab: Tab::Spectrum }), "Spectrum" }
                button { class: if s.tab == Tab::Remote { "active" }, onclick: move |_| set(DrawerState { open: true, tab: Tab::Remote }), "Virtual remote" }
                span { class: "drawer-live", "{live_summary}" }
            }
            if s.open {
                div { class: "drawer-body",
                    match s.tab {
                        Tab::Log => rsx! {
                            div { class: "txrx-log",
                                if lines.is_empty() {
                                    p { class: "control-hint", "Commands appear here while remote control is on." }
                                }
                                for (arrow, l) in lines {
                                    div { class: if l.tx { "tx" } else { "rx" },
                                        span { class: "ts", "{l.clock} " }
                                        span { "{arrow} {l.text}" }
                                    }
                                }
                            }
                        },
                        Tab::Spectrum => rsx! {
                            p { class: "control-hint", "{super::capability::gate(Capability::Spectrum).reason().unwrap_or_default()}" }
                        },
                        Tab::Remote => rsx! {
                            p { class: "control-hint", "{super::capability::gate(Capability::Keypad).reason().unwrap_or_default()}" }
                        },
                    }
                }
            }
        }
    }
}
