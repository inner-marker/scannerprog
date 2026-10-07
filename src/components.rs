use dioxus::prelude::*;
use std::time::Duration;

use crate::database_view::DatabaseView;
use crate::scanner_interaction::{
    detect_scanner, download_database, save_database_ron, DownloadStatus, DOWNLOAD_ACTIVE, DOWNLOAD_STATUS, SCANNER_INFO, SCAN_DATABASE,
    SCANNER_USB_DEVICE,
};

const MAIN_CSS: Asset = asset!("/assets/main.css");

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(NavBar)]
        #[route("/")]
        Home {},
        #[route("/database")]
        Database {},
}


#[component]
pub fn App() -> Element {
    let mut ports_seen = use_signal(String::new);

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
                Ok(Ok(detection)) => Some(detection),
                Ok(Err(error)) => {
                    eprintln!("Failed to scan serial ports: {error}");
                    None
                }
                Err(error) => {
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
    let connected = SCANNER_INFO.read().is_some();
    let nav = navigator();

    // Leave the Database page as soon as the scanner goes away.
    use_effect(move || {
        let connected = SCANNER_INFO.read().is_some();
        if !connected && router().current::<Route>() == (Route::Database {}) {
            nav.replace(Route::Home {});
        }
    });

    rsx! {
        nav { id: "navbar",
            Link { to: Route::Home {}, active_class: "active", "Home" }
            if connected {
                Link { to: Route::Database {}, active_class: "active", "Database" }
            } else {
                span { class: "disabled", title: "No scanner connected", "Database" }
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        main {
            match *SCANNER_USB_DEVICE.read() {
                Some(_) => rsx! {
                    p { "USB Device Information" }
                    ul {
                        li { "Vendor ID: {SCANNER_USB_DEVICE.read().as_ref().map(|d| d.vendor_id.clone()).unwrap_or_default():#04x}" } // as hex
                        li { "Product ID: {SCANNER_USB_DEVICE.read().as_ref().map(|d| d.product_id.clone()).unwrap_or_default():#04x}" } // as hex
                        li { "Manufacturer: {SCANNER_USB_DEVICE.read().as_ref().map(|d| d.manufacturer.clone()).unwrap_or_default():?}" }
                        li { "Product: {SCANNER_USB_DEVICE.read().as_ref().map(|d| d.product.clone()).unwrap_or_default():?}" }
                        li { "Serial Number: {SCANNER_USB_DEVICE.read().as_ref().map(|d| d.serial_number.clone()).unwrap_or_default():?}" }
                    }

                    p { "Scanner information from the connected USB device." }
                    ul {
                        li { "Model: {SCANNER_INFO.read().as_ref().map(|s| s.model.clone()).unwrap_or_default():?}" }
                        li { "Firmware: {SCANNER_INFO.read().as_ref().map(|s| s.firmware.clone()).unwrap_or_default():?}" }
                    }
                },
                None => {
                    rsx! {
                        p { "No USB scanner detected. Searching for scanners..." }
                    }
                }
            }
        }
    }
}

#[component]
fn Database() -> Element {
    let mut save_message = use_signal(String::new);
    rsx! {
        main {
            h1 { "Database" }
            div {
                button {
                    id: "download-from-scanner",
                    disabled: matches!(*DOWNLOAD_STATUS.read(), DownloadStatus::InProgress { .. }),
                    onclick: move |_| {
                        spawn(download_database());
                    },
                    title: "Download the database from the connected scanner.",
                    "Download"
                }
            }

            if SCAN_DATABASE.read().is_some() {
                div {
                    button {
                        id: "save-ron",
                        onclick: move |_| {
                            let model = SCANNER_INFO
                                .peek()
                                .as_ref()
                                .map(|s| s.model.clone())
                                .unwrap_or_else(|| "Scanner".into());
                            let result = match SCAN_DATABASE.peek().as_ref() {
                                Some(db) => save_database_ron(db, &model),
                                None => Err("Nothing to save".to_string()),
                            };
                            save_message.set(match result {
                                Ok(path) => format!("Saved to {}", path.display()),
                                Err(error) => format!("Save failed: {error}"),
                            });
                        },
                        "Save as RON"
                    }
                    " {save_message}"
                }
            }

            div {
                match *DOWNLOAD_STATUS.read() {
                    DownloadStatus::NotStarted => rsx! { p { "Download not started." } },
                    DownloadStatus::InProgress { done, total } => rsx! {
                        p { "Download in progress... ({done}/{total} systems)" }
                    },
                    DownloadStatus::Completed => rsx! {
                        p { "Download completed successfully." }
                        DatabaseView {}
                    },
                    DownloadStatus::Failed(ref error) => rsx! { p { "Download failed: {error}" } },
                }
            }
        }
    }
}
