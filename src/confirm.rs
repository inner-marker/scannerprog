use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Clone)]
pub struct PendingConfirm {
    message: String,
    details: Option<String>,
    confirm_label: String,
    action: Rc<dyn Fn()>,
}

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

fn open(message: String, details: Option<String>, confirm_label: String, action: impl Fn() + 'static) {
    *CONFIRM.write() = Some(PendingConfirm { message, details, confirm_label, action: Rc::new(action) });
}

/// The modal itself. Render once, near the root.
#[component]
pub fn ConfirmDialog() -> Element {
    let Some(pending) = CONFIRM.read().clone() else {
        return rsx! {};
    };
    let action = pending.action.clone();
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
