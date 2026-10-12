//! Capability-aware control wrappers. Every control on the Scanner page goes through these so the
//! disabled states and messages are the same everywhere (plan §7). They never contain protocol code.

use dioxus::prelude::*;

use crate::control::{ConnState, CONN, REMOTE_CONTROL};
use crate::models::live::{Capability, Support};
use crate::scanner_interaction::{connected_model, SCANNER_INFO};

/// Whether a control can be used right now and, if not, why.
#[derive(Clone, Debug, PartialEq)]
pub enum Gate {
    Enabled,
    /// The model can't do this; shown dimmed with ⛔.
    Unsupported(String),
    /// Possible, but not at the moment (remote control off, still connecting, not built yet).
    NotNow(String),
    /// Works, but pauses scanning while the scanner is in Program Mode; shown with 🔒.
    NeedsProgramMode,
}

impl Gate {
    pub fn is_enabled(&self) -> bool {
        matches!(self, Gate::Enabled | Gate::NeedsProgramMode)
    }

    /// The explanation for a blocked control, if any.
    pub fn reason(&self) -> Option<String> {
        match self {
            Gate::Enabled => None,
            Gate::Unsupported(r) | Gate::NotNow(r) => Some(r.clone()),
            Gate::NeedsProgramMode => Some("Briefly pauses scanning while the scanner is in Program Mode.".into()),
        }
    }

    /// CSS class for the state.
    fn class(&self) -> &'static str {
        match self {
            Gate::Enabled => "",
            Gate::Unsupported(_) => "cap-unsupported",
            Gate::NotNow(_) => "cap-notnow",
            Gate::NeedsProgramMode => "cap-pgm",
        }
    }
}

/// Resolves the gate for `cap` from the connected model and connection state. Reactive.
pub fn gate(cap: Capability) -> Gate {
    let connected = SCANNER_INFO.read().is_some();
    let Some(profile) = connected_model().and_then(|m| m.control_profile()) else {
        return if connected {
            Gate::Unsupported("Remote control is not supported on this model yet.".into())
        } else {
            Gate::NotNow("Connect a scanner first.".into())
        };
    };
    match profile.support(cap) {
        Support::Unsupported(why) => return Gate::Unsupported(why.to_owned()),
        Support::Planned(why) => return Gate::NotNow(why.to_owned()),
        Support::Native | Support::Emulated | Support::NeedsProgramMode => {}
    }
    if !*REMOTE_CONTROL.read() {
        return Gate::NotNow("Turn on remote control first.".into());
    }
    match *CONN.read() {
        ConnState::Connected => {}
        ConnState::Degraded => return Gate::NotNow("The connection is unstable; waiting for the scanner.".into()),
        _ => return Gate::NotNow("Connecting to the scanner…".into()),
    }
    match profile.support(cap) {
        Support::NeedsProgramMode => Gate::NeedsProgramMode,
        _ => Gate::Enabled,
    }
}

/// A button bound to a capability. Blocked buttons stay visible; clicking one explains why.
#[component]
pub fn CapButton(
    cap: Capability,
    onclick: EventHandler<()>,
    #[props(default)] class: String,
    /// An extra reason the button can't be used right now, such as "nothing is playing".
    #[props(default)]
    blocked: Option<String>,
    #[props(default)] title: String,
    children: Element,
) -> Element {
    let mut explain = use_signal(|| None::<String>);
    let gate = match (gate(cap), blocked) {
        (Gate::Enabled | Gate::NeedsProgramMode, Some(why)) => Gate::NotNow(why),
        (g, _) => g,
    };
    let enabled = gate.is_enabled();
    let tip = gate.reason().unwrap_or(title);
    let state_class = gate.class();
    let glyph = match &gate {
        Gate::Unsupported(_) => " ⛔",
        Gate::NeedsProgramMode => " 🔒",
        _ => "",
    };

    rsx! {
        span { class: "cap-wrap",
            button {
                class: "cap-button {state_class} {class}",
                // aria-disabled (not `disabled`) keeps the button clickable so it can say why.
                "aria-disabled": "{!enabled}",
                title: "{tip}",
                onmouseleave: move |_| explain.set(None),
                onblur: move |_| explain.set(None),
                onclick: move |_| {
                    if enabled {
                        onclick.call(());
                    } else {
                        explain.set(gate.reason());
                    }
                },
                {children}
                "{glyph}"
            }
            if let Some(why) = explain() {
                span { class: "cap-popover", role: "status", "{why}" }
            }
        }
    }
}

/// A 0–`max` slider bound to a capability. Disabled sliders show the reason beside them.
#[component]
pub fn CapSlider(
    cap: Capability,
    label: String,
    min: u8,
    max: u8,
    value: u8,
    onchange: EventHandler<u8>,
) -> Element {
    let gate = gate(cap);
    let mut dragging = use_signal(|| None::<u8>);
    // Each change bumps the generation; only the last one within the delay is sent.
    let mut generation = use_signal(|| 0u32);
    let shown = dragging().unwrap_or(value);
    let state_class = gate.class();
    let enabled = gate.is_enabled();
    let reason = gate.reason().filter(|_| !enabled);

    rsx! {
        label { class: "cap-slider {state_class}",
            span { class: "cap-slider-label", "{label}" }
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                value: "{shown}",
                disabled: !enabled,
                oninput: move |e| {
                    let Ok(v) = e.value().parse::<u8>() else { return };
                    dragging.set(Some(v));
                    generation += 1;
                    let mine = generation();
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                        if generation() == mine {
                            onchange.call(v);
                            dragging.set(None);
                        }
                    });
                },
            }
            span { class: "cap-slider-value", "{shown}" }
            if let Some(why) = reason {
                span { class: "control-hint", "{why}" }
            }
        }
    }
}
