//! Global message queue: warnings, cautions, and notes the user can acknowledge.

use dioxus::prelude::*;

/// Severity shown beside a queued note, caution, or warning.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MessageKind {
    Warning,
    Caution,
    Note,
}

impl MessageKind {
    /// Returns the text displayed in the message list.
    fn label(self) -> &'static str {
        match self {
            Self::Warning => "Warning",
            Self::Caution => "Caution",
            Self::Note => "Note",
        }
    }

    /// Selects the matching visual style for this severity.
    fn class(self) -> &'static str {
        match self {
            Self::Warning => "message-warning",
            Self::Caution => "message-caution",
            Self::Note => "message-note",
        }
    }
}

/// One queued message, including its source and whether the user has acknowledged it.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub id: u64,
    pub kind: MessageKind,
    pub text: String,
    /// Groups messages so a producer can replace its previous batch.
    pub source: &'static str,
    pub acknowledged: bool,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Messages waiting to be acknowledged or dismissed.
pub static MESSAGES: GlobalSignal<Vec<Message>> = Signal::global(Vec::new);
/// Whether the floating message panel is currently expanded.
static MESSAGES_OPEN: GlobalSignal<bool> = Signal::global(|| false);
/// Monotonic IDs keep each message row distinct as the queue changes.
static NEXT_ID: GlobalSignal<u64> = Signal::global(|| 0);

/// Adds a fresh message and assigns it a unique ID and local timestamp.
pub fn push_message(kind: MessageKind, source: &'static str, text: impl Into<String>) {
    let id = {
        let mut next = NEXT_ID.write();
        *next += 1;
        *next
    };
    MESSAGES.write().push(Message {
        id,
        kind,
        text: text.into(),
        source,
        acknowledged: false,
        timestamp: chrono::Local::now(),
    });
}

/// Marks one queued message as seen without removing it.
fn acknowledge(id: u64) {
    if let Some(m) = MESSAGES.write().iter_mut().find(|m| m.id == id) {
        m.acknowledged = true;
    }
}

/// Removes one message from the queue.
fn dismiss(id: u64) {
    MESSAGES.write().retain(|m| m.id != id);
}

/// Floating "Messages" button and the queue panel it toggles.
#[component]
pub fn MessageCenter() -> Element {
    // Read the queue once to drive both the badge and the newest-first list.
    let messages = MESSAGES.read();
    let unacknowledged = messages.iter().filter(|m| !m.acknowledged).count();
    let open = *MESSAGES_OPEN.read();

    rsx! {
        button {
            id: "messages-toggle",
            title: "Show or hide messages",
            onclick: move |_| {
                let now = !*MESSAGES_OPEN.peek();
                *MESSAGES_OPEN.write() = now;
            },
            "Messages"
            if unacknowledged > 0 {
                span { class: "messages-badge", "{unacknowledged}" }
            }
        }
        // Keep acknowledged messages visible until the user clears or dismisses them.
        if open {
            div { id: "messages-panel",
                div { class: "messages-header",
                    span { "Messages" }
                    button {
                        title: "Acknowledge all messages",
                        disabled: unacknowledged == 0,
                        onclick: move |_| {
                            for m in MESSAGES.write().iter_mut() {
                                m.acknowledged = true;
                            }
                        },
                        "Acknowledge all"
                    }
                    button {
                        title: "Remove acknowledged messages",
                        onclick: move |_| MESSAGES.write().retain(|m| !m.acknowledged),
                        "Clear acknowledged"
                    }
                }
                if messages.is_empty() {
                    p { class: "messages-empty", "No messages." }
                }
                for m in messages.iter().rev() {
                    div {
                        key: "{m.id}",
                        class: if m.acknowledged {
                            "message {m.kind.class()} acknowledged"
                        } else {
                            "message {m.kind.class()}"
                        },
                        span { class: "message-kind", "{m.kind.label()}" }
                        span { class: "message-time", {m.timestamp.format("%H:%M:%S").to_string()} }
                        span { class: "message-text", "{m.text}" }
                        if m.acknowledged {
                            button {
                                title: "Remove this message",
                                onclick: {
                                    let id = m.id;
                                    move |_| dismiss(id)
                                },
                                "Dismiss"
                            }
                        } else {
                            button {
                                title: "Acknowledge this message",
                                onclick: {
                                    let id = m.id;
                                    move |_| acknowledge(id)
                                },
                                "Acknowledge"
                            }
                        }
                    }
                }
            }
        }
    }
}
