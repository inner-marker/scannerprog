//! The Scanner page. Everything here is driven by the connected model's capabilities: controls the
//! model can't use stay visible but disabled, with an explanation (plan §7).
//!
//! PLAN: Phases 1-4 support the A/B/C/D live-control families. Deviations are marked `PLAN:`.
//! PLAN: no manual theme override, resizable drawer, tabbed medium layout, auto-reconnect or port
//! picker (the app auto-detects scanners).

mod advanced;
mod capability;
mod panels;
mod quick_keys;
mod tools;

use dioxus::prelude::*;

use crate::control::{perform, stop_remote_control};
use crate::models::live::{Action, Capability, Family};
use crate::scanner_interaction::connected_model;
use capability::gate;
use panels::{Audio, ConnectionBar, ModeBar, NowPlaying, StatusStrip, Transport};
use tools::{ActivityLog, BottomDrawer, JumpToTag, QuickTune, TOGGLE_DRAWER_REQUEST, Typing};

/// The shortcut's action and the capability it needs, if `e` is a shortcut.
fn shortcut(e: &KeyboardEvent) -> Option<(Capability, Action)> {
    // PLAN: J/T focus shortcuts are deferred; the controls remain keyboard-focusable by tab.
    Some(match e.key() {
        Key::ArrowLeft => (Capability::StepPrev, Action::StepPrev),
        Key::ArrowRight => (Capability::StepNext, Action::StepNext),
        Key::Character(c) => match c.as_str() {
            " " => (Capability::Hold, Action::Hold),
            "a" | "A" => (Capability::Avoid, Action::Avoid),
            "s" | "S" => (Capability::ResumeScan, Action::ResumeScan),
            "m" | "M" => (
                Capability::Mute,
                Action::SetMute(!crate::control::LIVE.peek().muted),
            ),
            _ => return None,
        },
        _ => return None,
    })
}

#[component]
pub fn ScannerPage() -> Element {
    // Leaving the page gives the serial port back for the Database page.
    use_drop(stop_remote_control);
    let typing: Typing = use_context_provider(|| Signal::new(false));

    rsx! {
        main {
            id: "scanner-page",
            tabindex: "0",
            onkeydown: move |e| {
                if typing() || e.modifiers().intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META) {
                    return;
                }
                if let Key::Character(character) = e.key() {
                    if character == "`" {
                        e.prevent_default();
                        *TOGGLE_DRAWER_REQUEST.write() += 1;
                        return;
                    }
                    if character.len() == 1 {
                        if let Some(digit) = character.chars().next().and_then(|c| c.to_digit(10)) {
                            let family = connected_model()
                                .and_then(|model| model.control_profile())
                                .map(|profile| profile.family);
                            if family == Some(Family::Legacy) {
                                if (1..=5).contains(&digit)
                                    && gate(Capability::QuickKeysBanks).is_enabled()
                                {
                                    spawn(crate::control::toggle_bct8_bank(digit as usize - 1));
                                }
                            } else {
                                let key = quick_keys::page_key(
                                    *quick_keys::QUICK_KEY_PAGE.peek(),
                                    digit as u8,
                                );
                                if family.is_some_and(|family| {
                                    matches!(family, Family::HomePatrol | Family::Sds)
                                }) {
                                    if gate(Capability::QuickKeysFavorites).is_enabled() {
                                        spawn(crate::control::toggle_hp_quick_key(
                                            crate::control::HpQuickKeyLevel::Favorite,
                                            key,
                                        ));
                                    }
                                } else if gate(Capability::QuickKeysSystems).is_enabled() {
                                    spawn(crate::control::toggle_system_key(key));
                                }
                            }
                            return;
                        }
                    }
                }
                if let Some((cap, action)) = shortcut(&e) {
                    e.prevent_default();
                    if gate(cap).is_enabled() {
                        spawn(perform(action));
                    }
                }
            },
            ConnectionBar {}
            ModeBar {}
            div { id: "scanner-columns",
                div { class: "scanner-col",
                    NowPlaying {}
                    Transport {}
                    Audio {}
                }
                div { class: "scanner-col",
                    QuickTune {}
                    JumpToTag {}
                    quick_keys::QuickKeys {}
                }
                div { class: "scanner-col",
                    ActivityLog {}
                }
            }
            BottomDrawer {}
            StatusStrip {}
        }
    }
}
