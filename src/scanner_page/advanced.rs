//! Phase 2's demand-driven LCD/keypad remote and CSC sweep view.

use dioxus::prelude::*;
use std::sync::atomic::Ordering;

use super::capability::{CapButton, gate};
use crate::control::{
    DISPLAY_STATUS, DISPLAY_WANTED, SPECTRUM, SPECTRUM_ACTIVE, press_key, start_spectrum,
    stop_spectrum,
};
use crate::models::live::{Capability, DisplayLine};
use crate::scanner_interaction::connected_model;

#[component]
fn LcdLine(line: DisplayLine, large: bool) -> Element {
    let modes: Vec<char> = line.modes.chars().collect();
    rsx! {
        div { class: if large { "lcd-line large" } else { "lcd-line" },
            for (index, character) in line.text.chars().enumerate() {
                {
                    let mode = modes.get(index).copied().unwrap_or(' ');
                    rsx! {
                        span {
                            class: match mode {
                                '*' => "reverse",
                                '_' => "underline",
                                _ => "",
                            },
                            "{character}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn RemoteKey(code: &'static str, label: &'static str) -> Element {
    let mut generation = use_signal(|| 0u32);
    let mut long_sent = use_signal(|| false);
    rsx! {
        button {
            class: "remote-key",
            onpointerdown: move |_| {
                generation += 1;
                let mine = generation();
                long_sent.set(false);
                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                    if generation() == mine {
                        long_sent.set(true);
                        press_key(code, "L").await;
                    }
                });
            },
            onpointerup: move |_| {
                generation += 1;
                if !long_sent() {
                    spawn(press_key(code, "P"));
                }
            },
            onpointerleave: move |_| generation += 1,
            title: "Tap to press; hold for a long press.",
            "{label}"
        }
    }
}

/// STS LCD mirror and model-specific virtual keypad.
#[component]
pub fn VirtualRemote() -> Element {
    use_effect(|| DISPLAY_WANTED.store(true, Ordering::SeqCst));
    use_drop(|| DISPLAY_WANTED.store(false, Ordering::SeqCst));
    let status = DISPLAY_STATUS.read().clone();
    let profile = connected_model().and_then(|model| model.control_profile());

    rsx! {
        div { class: "virtual-remote",
            if gate(Capability::LcdMirror).is_enabled() {
                div { class: "lcd-mirror", role: "img", "aria-label": "Scanner LCD display",
                    if let Some(status) = status {
                        for (index, line) in status.lines.into_iter().enumerate() {
                            LcdLine { line, large: status.large.get(index).copied().unwrap_or(false) }
                        }
                    } else {
                        p { class: "control-hint", "Waiting for the next STS display poll…" }
                    }
                }
            } else {
                p { class: "control-hint", "{gate(Capability::LcdMirror).reason().unwrap_or_default()}" }
            }
            div { class: "remote-keypad",
                if let Some(profile) = profile {
                    for key in profile.keymap {
                        RemoteKey { code: key.code, label: key.label }
                    }
                }
            }
        }
    }
}

/// CSC sweep display. SVG is used instead of a canvas because Dioxus can update it without JS.
#[component]
pub fn SpectrumView() -> Element {
    // PLAN: SVG replaces the suggested canvas. CSC is low-rate serial data, so a 30 fps canvas
    // adds an interop dependency without improving fidelity.
    use_drop(stop_spectrum);
    let capability = gate(Capability::Spectrum);
    let mut confirmed = use_signal(|| false);
    let active = *SPECTRUM_ACTIVE.read();
    let samples: Vec<_> = SPECTRUM.read().iter().copied().collect();
    let (low, high) = samples.iter().fold((u64::MAX, 0u64), |(low, high), point| {
        (low.min(point.freq_hz), high.max(point.freq_hz))
    });
    let span = high.saturating_sub(low).max(1);
    let points = samples
        .iter()
        .map(|point| {
            let x = (point.freq_hz.saturating_sub(low) as f64 / span as f64) * 1000.0;
            let y = 200.0 - (f64::from(point.rssi.min(1023)) / 1023.0) * 190.0;
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let low_label = if low == u64::MAX {
        "Waiting for CSC data...".to_owned()
    } else {
        format!("{:.4} MHz", low as f64 / 1e6)
    };
    let high_label = if high == 0 {
        String::new()
    } else {
        format!("{:.4} MHz", high as f64 / 1e6)
    };

    rsx! {
        div { class: "spectrum-view",
            if !capability.is_enabled() {
                p { class: "control-hint", "{capability.reason().unwrap_or_default()}" }
            } else if !confirmed() {
                p { "Starting a sweep changes the scanner to Custom Search until the Spectrum tab closes." }
                CapButton {
                    cap: Capability::Spectrum,
                    onclick: move |()| {
                        confirmed.set(true);
                        start_spectrum();
                    },
                    "Start sweep"
                }
            } else if active {
                div { class: "spectrum-labels",
                    span { "{low_label}" }
                    span { "{high_label}" }
                }
                svg { class: "spectrum-chart", view_box: "0 0 1000 200", preserve_aspect_ratio: "none",
                    polyline { points, fill: "none", stroke: "#59d779", stroke_width: "3" }
                }
                button {
                    onclick: move |_| {
                        stop_spectrum();
                        confirmed.set(false);
                    },
                    "Stop sweep"
                }
            } else {
                p { class: "control-hint", "The sweep is stopped." }
                button { onclick: move |_| start_spectrum(), "Restart sweep" }
            }
        }
    }
}
