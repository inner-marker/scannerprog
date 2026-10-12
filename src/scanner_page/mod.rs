//! The Scanner page. Everything here is driven by the connected model's capabilities: controls the
//! model can't use stay visible but disabled, with an explanation (plan §7).
//!
//! PLAN: this implements Phase 1 of `Roadmaps/0-2-1 SCANNER_PAGE_PLAN.md` for the XT/P2 scanner family
//! (BCD325P2, BCD396XT, BC346XT). Deviations are marked `PLAN:` where they occur.
//! PLAN: no manual theme override, resizable drawer, tabbed medium layout, quick-key digit shortcuts,
//! auto-reconnect or port picker (the app auto-detects scanners); the drawer's Spectrum and
//! Virtual Remote tabs are placeholders.

mod capability;
mod panels;
mod quick_keys;
mod tools;

use dioxus::prelude::*;

use crate::control::{stop_remote_control, perform};
use crate::models::live::{Action, Capability};
use capability::gate;
use panels::{Audio, ConnectionBar, ModeBar, NowPlaying, StatusStrip, Transport};
use tools::{ActivityLog, BottomDrawer, JumpToTag, QuickTune, Typing};

/// The shortcut's action and the capability it needs, if `e` is a shortcut.
fn shortcut(e: &KeyboardEvent) -> Option<(Capability, Action)> {
    // PLAN: shortcuts are Space, ←, →, A, S and M; the plan's J, T and backtick are not wired up yet.
    Some(match e.key() {
        Key::ArrowLeft => (Capability::StepPrev, Action::StepPrev),
        Key::ArrowRight => (Capability::StepNext, Action::StepNext),
        Key::Character(c) => match c.as_str() {
            " " => (Capability::Hold, Action::Hold),
            "a" | "A" => (Capability::Avoid, Action::Avoid),
            "s" | "S" => (Capability::ResumeScan, Action::ResumeScan),
            "m" | "M" => (Capability::Mute, Action::SetMute(!crate::control::LIVE.peek().muted)),
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
