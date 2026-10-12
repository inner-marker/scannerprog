//! Tools below the main panels: quick tune, jump to tag, the activity log, and the bottom drawer.

use dioxus::prelude::*;
use std::time::Duration;

use super::advanced::{SpectrumView, VirtualRemote};
use super::capability::CapButton;
use super::panels::run;
use crate::control::{ACTIVITY, LIVE, TXRX_LOG, clear_activity};
use crate::messages::{MessageKind, push_message};
use crate::models::live::{Action, Capability, Family};
use crate::scanner_interaction::{connected_model, default_save_dir};

/// Incremented by the status strip to open the connection log.
pub static OPEN_LOG_REQUEST: GlobalSignal<u32> = Signal::global(|| 0);
pub static OPEN_SPECTRUM_REQUEST: GlobalSignal<u32> = Signal::global(|| 0);
pub static TOGGLE_DRAWER_REQUEST: GlobalSignal<u32> = Signal::global(|| 0);

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
        run(Action::QuickTune {
            freq_hz: hz,
            modulation: Some(modulation()),
        });
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
    let mut favorite = use_signal(String::new);
    let mut sys = use_signal(String::new);
    let mut chan = use_signal(String::new);
    let hp = connected_model()
        .and_then(|model| model.control_profile())
        .is_some_and(|profile| matches!(profile.family, Family::HomePatrol | Family::Sds));
    let parse = |s: &str| s.trim().parse::<u16>().ok();
    let (f, s, c) = (parse(&favorite()), parse(&sys()), parse(&chan()));
    let bad = |t: &str, v: Option<u16>| !t.trim().is_empty() && v.is_none();
    let blocked =
        if favorite().trim().is_empty() && sys().trim().is_empty() && chan().trim().is_empty() {
            Some("Enter a system tag, a channel tag, or both.".to_owned())
        } else if bad(&favorite(), f) || bad(&sys(), s) || bad(&chan(), c) {
            Some("Tags are numbers from 0 to 999.".to_owned())
        } else {
            None
        };

    rsx! {
        section { id: "jump-tag", class: "panel",
            h2 { "Jump to tag" }
            div { class: "tune-row",
                if hp {
                    input {
                        r#type: "text",
                        placeholder: "Favorites tag",
                        value: "{favorite}",
                        oninput: move |e| favorite.set(e.value()),
                        onfocus: move |_| typing.set(true),
                        onblur: move |_| typing.set(false),
                    }
                }
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
                    onclick: move |()| run(Action::JumpToTag {
                        favorite: f,
                        sys: s,
                        chan: c,
                    }),
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
                        let csv = csv.clone();
                        spawn(async move {
                            let mut dialog = rfd::AsyncFileDialog::new()
                                .set_title("Export Activity")
                                .add_filter("CSV file", &["csv"])
                                .set_file_name(name);
                            if let Some(directory) = default_save_dir() {
                                dialog = dialog.set_directory(directory);
                            }
                            let Some(file) = dialog.save_file().await else {
                                return;
                            };
                            match std::fs::write(file.path(), csv) {
                                Ok(()) => push_message(MessageKind::Note, "control", format!("Activity saved to {}", file.path().display())),
                                Err(error) => push_message(MessageKind::Warning, "control", format!("Could not save the activity log: {error}")),
                            }
                        });
                    },
                    "Export CSV"
                }
                button {
                    disabled: empty,
                    onclick: move |_| {
                        clear_activity();
                    },
                    "Clear history"
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
    Some(
        dirs::data_local_dir()?
            .join("Scanner Programmer")
            .join("scanner-drawer.ron"),
    )
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

/// Collapsible drawer: the TX/RX log, spectrum sweep, and virtual remote.
#[component]
pub fn BottomDrawer() -> Element {
    let mut state = use_signal(load_drawer);
    let open_log = *OPEN_LOG_REQUEST.read();
    let open_spectrum = *OPEN_SPECTRUM_REQUEST.read();
    let toggle_drawer = *TOGGLE_DRAWER_REQUEST.read();
    use_effect(move || {
        if open_log > 0 {
            let next = DrawerState {
                open: true,
                tab: Tab::Log,
            };
            state.set(next);
            save_drawer(next);
        }
    });
    use_effect(move || {
        if open_spectrum > 0 {
            let next = DrawerState {
                open: true,
                tab: Tab::Spectrum,
            };
            state.set(next);
            save_drawer(next);
        }
    });
    use_effect(move || {
        if toggle_drawer > 0 {
            let current = state();
            let next = DrawerState {
                open: !current.open,
                tab: current.tab,
            };
            state.set(next);
            save_drawer(next);
        }
    });
    let s = state();
    let mut set = move |next: DrawerState| {
        state.set(next);
        save_drawer(next);
    };
    let lines = TXRX_LOG
        .read()
        .iter()
        .rev()
        .take(200)
        .map(|l| (if l.tx { "→" } else { "←" }, l.clone()))
        .collect::<Vec<_>>();
    let live_summary = LIVE
        .read()
        .freq_mhz()
        .map_or_else(String::new, |m| format!("{m:.4} MHz"));

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
                            SpectrumView {}
                        },
                        Tab::Remote => rsx! {
                            VirtualRemote {}
                        },
                    }
                }
            }
        }
    }
}
