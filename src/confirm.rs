use dioxus::prelude::*;
use std::rc::Rc;

/// Details needed to show one confirmation prompt and run its action if accepted.
#[derive(Clone)]
pub struct PendingConfirm {
    message: String,
    details: Option<String>,
    confirm_label: String,
    action: Rc<dyn Fn()>,
}

/// The single confirmation prompt currently shown by the app, if any.
static CONFIRM: GlobalSignal<Option<PendingConfirm>> = Signal::global(|| None);

/// Ask "are you sure?" and run `action` only if the user confirms.
pub fn confirm(message: impl Into<String>, confirm_label: impl Into<String>, action: impl Fn() + 'static) {
    open(message.into(), None, confirm_label.into(), action);
}

/// Like [`confirm`], with a longer explanation. Paragraphs are separated by blank lines.
pub fn confirm_with_details(
    message: impl Into<String>,
    details: impl Into<String>,
    confirm_label: impl Into<String>,
    action: impl Fn() + 'static,
) {
    open(message.into(), Some(details.into()), confirm_label.into(), action);
}

/// Replaces the current prompt with a new message and a one-shot confirmation action.
fn open(message: String, details: Option<String>, confirm_label: String, action: impl Fn() + 'static) {
    *CONFIRM.write() = Some(PendingConfirm { message, details, confirm_label, action: Rc::new(action) });
}

/// The modal itself. Render once, near the root.
#[component]
pub fn ConfirmDialog() -> Element {
    // Clone the callback before building event handlers so the signal borrow is short-lived.
    let Some(pending) = CONFIRM.read().clone() else {
        return rsx! {};
    };
    let action = std::rc::Rc::<dyn std::ops::Fn()>::clone(&pending.action);
    rsx! {
        div {
            id: "confirm-overlay",
            tabindex: "0",
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            onclick: move |_| *CONFIRM.write() = None,
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    *CONFIRM.write() = None;
                }
            },
            div {
                id: "confirm-box",
                onclick: move |e| e.stop_propagation(),
                // The overlay closes on outside click or Escape; clicks in the box stay put.
                p { class: "confirm-message", "{pending.message}" }
                if let Some(details) = pending.details.clone() {
                    for para in details.split("\n\n").map(str::to_string).collect::<Vec<_>>() {
                        p { class: "confirm-details", "{para}" }
                    }
                }
                div { class: "confirm-buttons",
                    button { onclick: move |_| *CONFIRM.write() = None, "Cancel" }
                    button {
                        onclick: move |_| {
                            *CONFIRM.write() = None;
                            action();
                        },
                        "{pending.confirm_label}"
                    }
                }
            }
        }
    }
}
