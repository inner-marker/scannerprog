use dioxus::html::g::prevent_default;
use dioxus::prelude::*;
use std::time::Duration;

use crate::database_view::{DatabaseView, MemoryUsage};
use crate::messages::{push_message, MessageCenter, MessageKind};
use crate::scanner_interaction::{
    default_save_dir, default_save_name, detect_scanner, download_database, load_database_ron, save_database_ron, DownloadStatus, DOWNLOAD_ACTIVE, DOWNLOAD_STATUS, SCANNER_INFO, SCANNER_MEMORY, SCAN_DATABASE, VALIDATION,
    SCANNER_USB_DEVICE,
};

const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Clone, Copy, PartialEq)]
enum ConnectionStatus {
    Connected,
    Searching,
    Error,
}

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(NavBar)]
        #[route("/")]
        Home {},
        #[route("/database")]
        DatabasePage {},
}


#[component]
pub fn App() -> Element {
    let mut ports_seen = use_signal(String::new);
    let mut connection_status = use_signal(|| ConnectionStatus::Searching);
    use_context_provider(|| connection_status);

    // A loop to continuously check for connected scanners and update the UI accordingly.
    use_future(move || async move {
        // do the loop
        loop {
            // The port is busy while downloading.
            if DOWNLOAD_ACTIVE.load(std::sync::atomic::Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(500)).await;
                continue;
            }
            let detection = match tokio::task::spawn_blocking(detect_scanner).await {
                Ok(Ok(detection)) => {
                    connection_status.set(if detection.scanner.is_some() {
                        ConnectionStatus::Connected
                    } else {
                        ConnectionStatus::Searching
                    });
                    Some(detection)
                }
                Ok(Err(error)) => {
                    connection_status.set(ConnectionStatus::Error);
                    eprintln!("Failed to scan serial ports: {error}");
                    None
                }
                Err(error) => {
                    connection_status.set(ConnectionStatus::Error);
                    eprintln!("Scanner detection task failed: {error}");
                    None
                }
            };
            let listing = serialport::available_ports()
                .map(|p| {
                    p.iter()
                        .map(|p| p.port_name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|e| e.to_string());
            ports_seen.set(listing);
            if let Some(detection) = detection {
                if *SCANNER_INFO.peek() != detection.scanner {
                    *SCANNER_INFO.write() = detection.scanner;
                }
                if *SCANNER_USB_DEVICE.read() != detection.usb_device {
                    *SCANNER_USB_DEVICE.write() = detection.usb_device;
                }
            }

            // Sleep for 0.5 seconds before checking again.
            let secs = 0.5; // <- change here
            tokio::time::sleep(Duration::from_secs_f32(secs)).await;
        }
    });

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}

#[component]
fn NavBar() -> Element {
    let connection_status = use_context::<Signal<ConnectionStatus>>();
    let (status_label, status_class) = match connection_status() {
        ConnectionStatus::Connected => ("Connected", "connection-connected"),
        ConnectionStatus::Searching => ("Searching...", "connection-searching"),
        ConnectionStatus::Error => ("Error", "connection-error"),
    };

    rsx! {
        nav { id: "navbar",
            Link { to: Route::Home {}, active_class: "active", "Home" }
            Link { to: Route::DatabasePage {}, active_class: "active", "Database" }
            span {
                class: "connection-indicator {status_class}",
                title: "Connection status",
                "Connection: {status_label}"
            }
        }
        Outlet::<Route> {}
        MessageCenter {}
    }
}

#[component]
fn Home() -> Element {
    let usb_device = SCANNER_USB_DEVICE.read().clone();
    let scanner = SCANNER_INFO.read().clone();

    rsx! {
        main { id: "home",
            h1 { "Scanner Information" }
            if usb_device.is_some() || scanner.is_some() {
                table {
                    thead {
                        tr {
                            th { scope: "col", "Property" }
                            th { scope: "col", "Value" }
                        }
                    }
                    tbody {
                        if let Some(device) = usb_device {
                            tr { class: "section-heading",
                                th { scope: "rowgroup", colspan: "2", "USB Device" }
                            }
                            tr {
                                th { scope: "row", "Vendor ID" }
                                td { "{device.vendor_id:#06x}" }
                            }
                            tr {
                                th { scope: "row", "Product ID" }
                                td { "{device.product_id:#06x}" }
                            }
                            tr {
                                th { scope: "row", "Manufacturer" }
                                td { {device.manufacturer.as_deref().unwrap_or("Not reported")} }
                            }
                            tr {
                                th { scope: "row", "Product" }
                                td { {device.product.as_deref().unwrap_or("Not reported")} }
                            }
                            tr {
                                th { scope: "row", "Serial Number" }
                                td { {device.serial_number.as_deref().unwrap_or("Not reported")} }
                            }
                        }
                        if let Some(info) = scanner {
                            tr { class: "section-heading",
                                th { scope: "rowgroup", colspan: "2", "Scanner" }
                            }
                            tr {
                                th { scope: "row", "Model" }
                                td { "{info.model}" }
                            }
                            tr {
                                th { scope: "row", "Firmware" }
                                td { "{info.firmware}" }
                            }
                        }
                    }
                }
            } else {
                p { "No USB scanner detected. Searching for scanners..." }
            }
        }
    }
}

/// global signal for tracking "Upload to Scanner" enablement
/// 
/// Defaults to `false`. Only becomes `true` after a successful validation. Any changes to the database will reset it to `false`.
pub static UPLOAD_ENABLED: GlobalSignal<bool> = GlobalSignal::new( || false );


#[component]
fn Toolbar() -> Element {



    rsx! {
        div { id: "toolbar",
            button {
                id: "download-from-scanner",
                disabled: matches!(*DOWNLOAD_STATUS.read(), DownloadStatus::InProgress { .. }),
                onclick: move |_| {
                    *UPLOAD_ENABLED.write() = false;
                    spawn(download_database());
                },
                title: "Download the database from the connected scanner.",
                "Download From Scanner"
            }
            button {
                id: "validate-database",
                disabled: SCAN_DATABASE.read().is_none(),
                onclick: move |_| {
                    *UPLOAD_ENABLED.write() = false;
                    let errors: Vec<String> = match SCAN_DATABASE.peek().as_ref() {
                        Some(db) => db.validate().iter().map(|e| e.to_string()).collect(),
                        None => return,
                    };
                    if errors.is_empty() {
                        push_message(MessageKind::Note, "validation", "Database is valid.");
                        *UPLOAD_ENABLED.write() = true;
                    } else {
                        for error in &errors {
                            push_message(MessageKind::Warning, "validation", error.clone());
                        }
                    }
                    *VALIDATION.write() = Some(errors);
                },
                title: "Check the database for errors before uploading.",
                "Validate"
            }
            button {
                id: "upload-to-scanner",
                disabled: !*UPLOAD_ENABLED.read(),
                title: "Upload the database to the scanner (requires a successful validation).",
                onclick: move |_| {
                    // send a note to the message queue
                    push_message(MessageKind::Note, "upload", "Uploading database to scanner...");
                },
                "Upload To Scanner"
            }
            button {
                id: "save-ron",
                disabled: SCAN_DATABASE.read().is_none(),
                onclick: move |_| {
                    spawn(async move {
                        let model = SCANNER_INFO
                            .peek()
                            .as_ref()
                            .map(|s| s.model.clone())
                            .unwrap_or_else(|| "Scanner".into());
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title("Save Database")
                            .add_filter("RON file", &["ron"])
                            .set_file_name(default_save_name(&model));
                        if let Some(dir) = default_save_dir() {
                            dialog = dialog.set_directory(dir);
                        }
                        // None means the user cancelled.
                        let Some(file) = dialog.save_file().await else {
                            return;
                        };
                        let result = match SCAN_DATABASE.peek().as_ref() {
                            Some(db) => save_database_ron(db, file.path()),
                            None => Err("Nothing to save".to_string()),
                        };
                        match result {
                            Ok(path) => push_message(
                                MessageKind::Note,
                                "file",
                                format!("Saved to {}", path.display()),
                            ),
                            Err(error) => push_message(
                                MessageKind::Warning,
                                "file",
                                format!("Save failed: {error}"),
                            ),
                        }
                    });
                },
                title: "Save the database to a file.",
                "Save to File"
            }
            button {
                id: "load-ron",
                disabled: matches!(*DOWNLOAD_STATUS.read(), DownloadStatus::InProgress { .. }),
                onclick: move |_| {
                    *UPLOAD_ENABLED.write() = false;
                    spawn(async move {
                        let mut dialog = rfd::AsyncFileDialog::new()
                            .set_title("Load Database")
                            .add_filter("RON file", &["ron"]);
                        if let Some(dir) = default_save_dir() {
                            dialog = dialog.set_directory(dir);
                        }
                        // None means the user cancelled.
                        let Some(file) = dialog.pick_file().await else {
                            return;
                        };
                        match load_database_ron(file.path()) {
                            Ok(db) => {
                                *SCAN_DATABASE.write() = Some(db);
                                *VALIDATION.write() = None;
                                *SCANNER_MEMORY.write() = None;
                                *DOWNLOAD_STATUS.write() = DownloadStatus::NotStarted;
                                push_message(
                                    MessageKind::Note,
                                    "file",
                                    format!("Loaded {}", file.path().display()),
                                );
                            }
                            Err(error) => push_message(
                                MessageKind::Warning,
                                "file",
                                format!("Load failed: {error}"),
                            ),
                        }
                    });
                },
                title: "Load a database from a file.",
                "Load from File"
            }
            button {
                id: "clear-database",
                onclick: move |_| {
                    *UPLOAD_ENABLED.write() = false;
                    *SCAN_DATABASE.write() = None;
                    *VALIDATION.write() = None;
                    *SCANNER_MEMORY.write() = None;
                    *DOWNLOAD_STATUS.write() = DownloadStatus::NotStarted;
                    push_message(MessageKind::Note, "file", "Database cleared.");
                },
                "Reset"
            }
            MemoryUsage {}
        }
    }
}

#[component]
fn DatabasePage() -> Element {
    rsx! {
        Toolbar {}
        main {
            match *DOWNLOAD_STATUS.read() {
                DownloadStatus::InProgress { done, total } => rsx! {
                    p { "Download in progress... ({done}/{total} systems)" }
                },
                _ => rsx! {}
            }
            DatabaseView {}
        }
    }
}
