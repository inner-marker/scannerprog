//! The Scanner page's status and control panels: connection bar, mode bar, now playing,
//! transport, audio and the status strip.

use dioxus::prelude::*;

use super::capability::{CapButton, CapSlider, gate};
use super::tools::{OPEN_LOG_REQUEST, OPEN_SPECTRUM_REQUEST};
use crate::control::{
    CHARGE_STATUS, CONN, ConnState, DISPLAY_STATUS, GPS_FIX, HEALTH, LIVE, PROGRAM_MODE,
    REMOTE_CONTROL, perform, start_remote_control, stop_remote_control,
};
use crate::models::live::{
    Action, Capability, Family, HoldLevel, NOAA_CHANNELS, ScanMode, WEATHER_MODES, noaa_label,
};
use crate::scanner_interaction::{DOWNLOAD_ACTIVE, SCANNER_INFO, connected_model};

/// Runs `action` without blocking the UI.
pub fn run(action: Action) {
    spawn(perform(action));
}

/// Model, port, connection health, and the Remote Control toggle.
#[component]
pub fn ConnectionBar() -> Element {
    let on = *REMOTE_CONTROL.read();
    let info = SCANNER_INFO.read().clone();
    let supported = connected_model().is_some_and(|m| m.control_profile().is_some());
    let busy = !on && DOWNLOAD_ACTIVE.load(std::sync::atomic::Ordering::Relaxed);
    let conn = *CONN.read();

    let hint = if info.is_none() {
        Some("Connect a scanner to use remote control.")
    } else if !supported {
        Some("Remote control is not supported for this scanner model yet.")
    } else if busy {
        Some("The scanner is busy with another operation.")
    } else {
        None
    };
    let (dot, text) = match conn {
        ConnState::Off => ("off", "Remote control off"),
        ConnState::Connecting => ("warn", "Connecting…"),
        ConnState::Connected => ("ok", "Connected"),
        ConnState::Degraded => ("warn", "Not responding"),
        ConnState::Lost => ("bad", "Connection lost"),
    };
    let identity = info.map_or_else(
        || "No scanner detected".to_owned(),
        |i| format!("{} on {}", i.model, i.port),
    );

    rsx! {
        header { id: "connection-bar",
            span { class: "conn-dot {dot}", "aria-hidden": "true" }
            span { class: "conn-identity", "{identity}" }
            span { class: "conn-text", "{text}" }
            button {
                id: "remote-toggle-button",
                class: if on { "remote-on" },
                disabled: !on && hint.is_some(),
                title: "Turning this off gives the serial port back for download and upload.",
                onclick: move |_| {
                    if on {
                        stop_remote_control();
                    } else {
                        // Forever-spawn so the port is released even if the page is left mid-start.
                        dioxus::core::spawn_forever(start_remote_control());
                    }
                },
                if on { "Remote control: On" } else { "Remote control: Off" }
            }
            if let Some(hint) = hint {
                span { class: "control-hint", "{hint}" }
            }
            if *PROGRAM_MODE.read() {
                span { class: "program-mode-badge", role: "status",
                    "Program Mode"
                    button { onclick: move |_| stop_remote_control(), "Leave" }
                }
            }
        }
    }
}

/// Which popover of the mode bar is open.
#[derive(Clone, Copy, PartialEq)]
enum Popover {
    Service,
    Weather,
}

/// Buttons that switch what the scanner is doing.
#[component]
pub fn ModeBar() -> Element {
    let mode = LIVE.read().mode;
    let live_freq = LIVE.read().freq_hz;
    let mut open = use_signal(|| None::<Popover>);
    let banks = connected_model()
        .and_then(|m| m.control_profile())
        .map(|p| p.service_banks)
        .unwrap_or(&[]);
    let hp = connected_model()
        .and_then(|model| model.control_profile())
        .is_some_and(|profile| matches!(profile.family, Family::HomePatrol | Family::Sds));
    let current = |m: fn(&ScanMode) -> bool| mode.as_ref().is_some_and(m);

    rsx! {
        nav { id: "mode-bar", "aria-label": "Scan mode",
            CapButton {
                cap: Capability::ModeSwitch,
                class: if current(|m| matches!(m, ScanMode::Scan)) { "active" } else { "" },
                onclick: move |()| run(Action::JumpMode(ScanMode::Scan)),
                "Scan"
            }
            CapButton {
                cap: Capability::ModeSwitch,
                class: if current(|m| matches!(m, ScanMode::Service(_))) { "active" } else { "" },
                blocked: hp.then(|| "HomePatrol/SDS scanners have no direct Service Scan jump command.".to_owned()),
                onclick: move |()| open.set(if open() == Some(Popover::Service) { None } else { Some(Popover::Service) }),
                "Service ▾"
            }
            CapButton {
                cap: Capability::ModeSwitch,
                class: if current(|m| matches!(m, ScanMode::Custom)) { "active" } else { "" },
                onclick: move |()| run(Action::JumpMode(ScanMode::Custom)),
                "Custom"
            }
            CapButton {
                cap: Capability::ModeSwitch,
                class: if current(|m| matches!(m, ScanMode::CloseCall)) { "active" } else { "" },
                onclick: move |()| run(Action::JumpMode(ScanMode::CloseCall)),
                "Close Call"
            }
            CapButton {
                cap: Capability::QuickTune,
                class: if current(|m| matches!(m, ScanMode::QuickSearch)) { "active" } else { "" },
                blocked: if live_freq.is_none() { Some("Quick Search starts from the current frequency; nothing is being received.".to_owned()) } else { None },
                onclick: move |()| {
                    if let Some(freq_hz) = live_freq {
                        run(Action::QuickTune { freq_hz, modulation: None });
                    }
                },
                "Quick Search"
            }
            CapButton {
                cap: Capability::ModeSwitch,
                class: if current(|m| matches!(m, ScanMode::Weather(_))) { "active" } else { "" },
                onclick: move |()| open.set(if open() == Some(Popover::Weather) { None } else { Some(Popover::Weather) }),
                "Weather ▾"
            }
            CapButton {
                cap: Capability::ToneOut,
                class: if current(|m| matches!(m, ScanMode::ToneOut)) { "active" } else { "" },
                onclick: move |()| run(Action::JumpMode(ScanMode::ToneOut)),
                "Tone-Out"
            }
            CapButton { cap: Capability::Discovery, onclick: move |()| {}, "Discovery" }
            CapButton {
                cap: Capability::Waterfall,
                onclick: move |()| *OPEN_SPECTRUM_REQUEST.write() += 1,
                "Waterfall"
            }
            CapButton { cap: Capability::Highway, onclick: move |()| {}, "Highway" }
        }
        match open() {
            Some(Popover::Service) => rsx! {
                div { class: "mode-popover",
                    for (label, index) in banks.iter().copied() {
                        CapButton {
                            cap: Capability::ModeSwitch,
                            onclick: move |()| {
                                run(Action::JumpMode(ScanMode::Service(index)));
                                open.set(None);
                            },
                            "{label}"
                        }
                    }
                }
            },
            Some(Popover::Weather) => rsx! {
                div { class: "mode-popover",
                    for (label, index) in WEATHER_MODES {
                        CapButton {
                            cap: Capability::ModeSwitch,
                            onclick: move |()| {
                                run(Action::JumpMode(ScanMode::Weather(index)));
                                open.set(None);
                            },
                            "{label}"
                        }
                    }
                    // The scanner picks the NOAA channel itself; the table only shows the frequencies.
                    table { class: "noaa-table",
                        tr {
                            for (label, mhz) in NOAA_CHANNELS {
                                td { "{label}" br {} "{mhz:.3}" }
                            }
                        }
                    }
                }
            },
            None => rsx! {},
        }
    }
}

/// Signal strength as five segments.
#[component]
fn SignalBars(bars: Option<u8>) -> Element {
    let lit = bars.unwrap_or(0);
    rsx! {
        span { class: "signal-bars", role: "img", "aria-label": "Signal {lit} of 5",
            for i in 1..=5u8 {
                span { class: if i <= lit { "bar lit" } else { "bar" } }
            }
        }
    }
}

/// What the scanner is receiving right now. Always shown; empty fields show a dash.
#[component]
pub fn NowPlaying() -> Element {
    let live = LIVE.read().clone();
    let on = *REMOTE_CONTROL.read();
    let freq = match (live.freq_mhz(), &live.tgid) {
        (Some(mhz), _) => format!("{mhz:.4} MHz"),
        (None, Some(tgid)) => format!("TGID {tgid}"),
        (None, None) => "—".to_owned(),
    };
    // The scanner's own NOAA mode picks the channel; we only add the matching WX label here.
    let wx = live.freq_mhz().and_then(noaa_label);
    let dash = |v: &Option<String>| v.clone().unwrap_or_else(|| "—".to_owned());
    let status = if !on {
        "Remote control is off."
    } else if live.is_idle() {
        "Scanning…"
    } else {
        ""
    };
    let tags = match (&live.system_tag, &live.channel_tag) {
        (None, None) => String::new(),
        (s, c) => format!(
            "Tags {} / {}",
            s.as_deref().unwrap_or("–"),
            c.as_deref().unwrap_or("–")
        ),
    };

    rsx! {
        section { id: "now-playing", class: if on { "panel" } else { "panel dimmed" },
            h2 { "Now playing" }
            div { class: "np-channel", "{dash(&live.channel)}" }
            div { class: "np-system", "{dash(&live.system)} · {dash(&live.group)}" }
            div { class: "frequency",
                "{freq}"
                if let Some(wx) = wx {
                    span { class: "wx-label", " {wx}" }
                }
            }
            div { class: "np-meta",
                span { "{dash(&live.modulation)}" }
                if let Some(sub) = live.sub_audio.clone() {
                    span { "{sub}" }
                }
                if !tags.is_empty() {
                    span { "{tags}" }
                }
            }
            div { class: "np-signal",
                SignalBars { bars: live.signal_bars }
                span { class: if live.squelch_open { "sql-led open" } else { "sql-led" },
                    if live.squelch_open { "Receiving" } else { "Squelched" }
                }
            }
            if !status.is_empty() {
                p { class: "control-hint", "{status}" }
            }
        }
    }
}

/// Hold, step, scan and avoid buttons.
#[component]
pub fn Transport() -> Element {
    let hp = connected_model()
        .and_then(|model| model.control_profile())
        .is_some_and(|profile| matches!(profile.family, Family::HomePatrol | Family::Sds));
    rsx! {
        section { id: "transport", class: "panel",
            h2 { "Transport" }
            div { class: "transport-buttons",
                CapButton { cap: Capability::StepPrev, title: "Previous (←)", onclick: move |()| run(Action::StepPrev), "◀ Prev" }
                CapButton { cap: Capability::Hold, title: "Hold on this channel (Space)", onclick: move |()| run(Action::Hold), "Hold" }
                CapButton { cap: Capability::ResumeScan, title: "Resume scanning (S)", onclick: move |()| run(Action::ResumeScan), "Scan" }
                CapButton { cap: Capability::StepNext, title: "Next (→)", onclick: move |()| run(Action::StepNext), "Next ▶" }
                CapButton { cap: Capability::Avoid, title: "Temporarily avoid this channel (A)", onclick: move |()| run(Action::Avoid), if hp { "Temp avoid" } else { "Avoid" } }
                if hp {
                    CapButton { cap: Capability::Avoid, title: "Permanently avoid this channel", onclick: move |()| run(Action::AvoidPermanent), "Permanent avoid" }
                }
            }
            div { class: "hold-levels", "Hold level:"
                CapButton { cap: Capability::HoldAtLevel, onclick: move |()| run(Action::HoldLevel(HoldLevel::System)), "System" }
                CapButton { cap: Capability::HoldAtLevel, onclick: move |()| run(Action::HoldLevel(HoldLevel::Department)), "Department" }
                CapButton { cap: Capability::HoldAtLevel, onclick: move |()| run(Action::HoldLevel(HoldLevel::Channel)), "Channel" }
            }
        }
    }
}

/// Volume, squelch and mute.
#[component]
pub fn Audio() -> Element {
    let live = LIVE.read().clone();
    let profile = connected_model().and_then(|m| m.control_profile());
    let (vmin, vmax) = profile
        .as_ref()
        .and_then(|p| p.volume_range)
        .unwrap_or((0, 15));
    let (smin, smax) = profile
        .as_ref()
        .and_then(|p| p.squelch_range)
        .unwrap_or((0, 15));

    rsx! {
        section { id: "audio", class: "panel",
            h2 { "Audio" }
            CapSlider {
                cap: Capability::Volume,
                label: "Volume",
                min: vmin,
                max: vmax,
                value: live.volume.unwrap_or(0),
                onchange: move |v| run(Action::SetVolume(v)),
            }
            CapSlider {
                cap: Capability::Squelch,
                label: "Squelch",
                min: smin,
                max: smax,
                value: live.squelch.unwrap_or(0),
                onchange: move |v| run(Action::SetSquelch(v)),
            }
            CapButton {
                cap: Capability::Mute,
                class: if live.muted { "active" } else { "" },
                title: "Mute (M). Mutes by setting the volume to 0.",
                onclick: move |()| run(Action::SetMute(!LIVE.peek().muted)),
                if live.muted { "Unmute" } else { "Mute" }
            }
            CapButton {
                cap: Capability::Record,
                class: if live.recording { "active record-active" } else { "" },
                onclick: move |()| run(Action::SetRecord(!LIVE.peek().recording)),
                if live.recording { "Stop recording" } else { "Start recording" }
            }
        }
    }
}

/// One-line summary of the link and the capabilities still to come.
#[component]
pub fn StatusStrip() -> Element {
    let health = *HEALTH.read();
    let on = *REMOTE_CONTROL.read();
    let status = DISPLAY_STATUS.read().clone();
    let profile = connected_model().and_then(|m| m.control_profile());
    let charge = CHARGE_STATUS.read().clone();

    rsx! {
        footer { id: "status-strip",
            if on {
                span { "Poll {health.latency_ms} ms" }
                button {
                    class: "status-link",
                    title: "Open connection log",
                    onclick: move |_| *OPEN_LOG_REQUEST.write() += 1,
                    "Errors {health.errors}"
                }
            } else {
                span { "Not polling" }
            }
            if let Some(profile) = profile {
                if profile.battery_status {
                    if let Some(charge) = charge {
                        span {
                            title: "{charge.millivolts} mV, {charge.current_ma} mA, {charge.temperature_c:.1} °C, charge state {charge.state}",
                            class: if charge.percent <= 10 { "alert" } else { "" },
                            "Battery: {charge.percent}%"
                        }
                    } else {
                        span {
                            class: if status.as_ref().and_then(|s| s.battery_low) == Some(true) { "alert" } else { "" },
                            "Battery: "
                            if status.as_ref().and_then(|s| s.battery_low) == Some(true) { "Low" } else { "OK" }
                        }
                    }
                } else {
                    span { class: "unavailable", title: "{gate(Capability::Battery).reason().unwrap_or_default()}", "Battery: n/a" }
                }
                span {
                    class: if status.as_ref().is_some_and(|s| s.weather_alert) { "alert" } else { "" },
                    "WX alert: "
                    if status.as_ref().is_some_and(|s| s.weather_alert) { "Active" } else { "None" }
                }
                if profile.gps_status {
                    span {
                        "GPS: "
                        match *GPS_FIX.read() {
                            Some(true) => "Fix",
                            Some(false) => "No fix",
                            None => "—",
                        }
                    }
                }
            }
        }
    }
}
