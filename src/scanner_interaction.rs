use dioxus::prelude::*;
use crate::messages::{push_message, MessageKind};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::database::{ScanDatabase, ScannerMemory};
use crate::models::{self, Error, Scanner, ScannerModel};

/// Serial connection and identity details for a supported scanner.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannerInfo {
    pub port: String,
    pub model: String,
    pub firmware: String,
}



/// USB vendor, product, and descriptive information reported by the operating system.
#[derive(Debug, Clone, PartialEq)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

/// Results from one port scan, including any USB device found.
pub struct ScannerDetection {
    pub scanner: Option<ScannerInfo>,
    pub usb_device: Option<UsbDeviceInfo>,
}

// Global signal for the currently connected scanner (UsbDeviceInfo)
/// USB device seen during port detection, whether supported or not.
pub static SCANNER_USB_DEVICE: GlobalSignal<Option<UsbDeviceInfo>> = Signal::global( || None );

/// Sends one `\r`-terminated command and returns the response without its `CMD,` prefix.
fn query(port: &mut Box<dyn serialport::SerialPort>, cmd: &str) -> Option<String> {
    port.write_all(format!("{cmd}\r").as_bytes()).ok()?;
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while port.read_exact(&mut byte).is_ok() {
        if byte[0] == b'\r' {
            let line = String::from_utf8_lossy(&buf).into_owned();
            return line.strip_prefix(&format!("{cmd},")).map(str::to_owned);
        }
        buf.push(byte[0]);
    }
    None
}

/// Looks for a scanner
/// 
/// Iterates over all available USB serial ports to find a compatible scanner.
pub fn detect_scanner() -> Result<ScannerDetection, serialport::Error> {
    // Get a list of all available serial ports.
    let ports = serialport::available_ports()?;
    let mut usb_device: Option<UsbDeviceInfo> = None;

    // for each port ...
    for p in ports {
        // USB serial ports can expose scanner identity commands; unrelated port types cannot.
        // Only consider USB ports.
        let serialport::SerialPortType::UsbPort(info) = &p.port_type else {
            continue;
        };

        // Keep USB details even if the device turns out not to be a supported scanner.
        let device = UsbDeviceInfo {
            vendor_id: info.vid,
            product_id: info.pid,
            serial_number: info.serial_number.clone(),
            manufacturer: info.manufacturer.clone(),
            product: info.product.clone(),
        };

        usb_device = Some(device.clone());

        // Keep this probe short so an unrelated or busy USB serial device does not stall detection.
        // Try to open the serial port with a 500ms timeout.
        let mut port = match serialport::new(&p.port_name, 115_200)
            .timeout(Duration::from_millis(500))
            .open()
        {
            Ok(port) => port,
            Err(error) => {
                eprintln!("Failed to open serial port {}: {error}", p.port_name);
                continue;
            }
        };


        // Query the scanner for its model and firmware information.
        let Some(model) = query(&mut port, "MDL") else {
            continue;
        };
        // A model response is useful only if this app knows how to program that model.
        if models::find(&model).is_none() {
            continue; // a Uniden scanner, but not a model we support
        }
        // println!("Detected scanner model: {}", model);
        // Query the scanner for its firmware information.
        let firmware = query(&mut port, "VER").unwrap_or_else(|| "unknown".into());
        // println!("Detected scanner firmware: {}", firmware);
        
        // Return the scanner information if both model and firmware were successfully queried.
        return Ok(ScannerDetection {
            scanner: Some(ScannerInfo {
                port: p.port_name.clone(),
                model,
                firmware,
            }),
            usb_device: Some(device),
        });
    }

    // Keep USB identity for the home page even when no compatible scanner answered.
    // No compatible scanner was found.
    Ok(ScannerDetection {
        scanner: None,
        usb_device,
    })
}



/// Current state and per-system progress of a database download.
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    NotStarted,
    InProgress { done: usize, total: usize },
    Completed,
    Failed(String),
}

/// State and per-system progress for the current database download.
pub static DOWNLOAD_STATUS: GlobalSignal<DownloadStatus> = Signal::global(|| DownloadStatus::NotStarted);

/// Validation errors for the loaded database; `None` means it needs checking again.
/// The database may be uploaded only when this is `Some` and empty.
pub static VALIDATION: GlobalSignal<Option<Vec<String>>> = Signal::global(|| None);

/// Database currently shown in the editor, if one is loaded.
pub static SCAN_DATABASE: GlobalSignal<Option<ScanDatabase>> = Signal::global(|| Some(ScanDatabase::new_placeholder()));

/// Memory report returned by the scanner with the most recent successful download.
pub static SCANNER_MEMORY: GlobalSignal<Option<ScannerMemory>> = Signal::global(|| None);

/// Supported scanner currently detected on a serial port.
pub static SCANNER_INFO: GlobalSignal<Option<ScannerInfo>> = Signal::global(|| None);

/// Guards the serial port so the background detector waits during transfers.
pub static DOWNLOAD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Model of the connected scanner, if one is detected and supported.
pub fn connected_model() -> Option<&'static dyn ScannerModel> {
    SCANNER_INFO.read().as_ref().and_then(|info| models::find(&info.model))
}

/// Model the database is validated against: the connected scanner if any, so one
/// database can be checked for each scanner it is sent to.
pub fn validation_model() -> &'static dyn ScannerModel {
    connected_model().unwrap_or_else(active_model)
}

/// Model governing the editor: the loaded database's model, else the connected
/// scanner's, else the default model.
pub fn active_model() -> &'static dyn ScannerModel {
    SCAN_DATABASE
        .read()
        .as_ref()
        .and_then(|db| db.model.as_deref())
        .and_then(models::by_name)
        .or_else(connected_model)
        .unwrap_or_else(models::default_model)
}

/// [`Scanner`] transport over a serial port.
struct SerialScanner(Box<dyn serialport::SerialPort>);

impl Scanner for SerialScanner {
    /// Sends one model command and turns the scanner rejection codes into errors.
    fn send(&mut self, cmd: &str) -> Result<String, Error> {
        self.0.write_all(format!("{cmd}\r").as_bytes()).map_err(Error::Io)?;
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            self.0.read_exact(&mut byte).map_err(Error::Io)?;
            if byte[0] == b'\r' {
                break;
            }
            buf.push(byte[0]);
        }
        let reply = String::from_utf8_lossy(&buf).into_owned();
        let name = cmd.split(',').next().unwrap_or(cmd);
        let body = reply.strip_prefix(&format!("{name},")).unwrap_or(&reply);
        if matches!(body, "ERR" | "NG" | "FER" | "ORER") {
            return Err(Error::Scanner { cmd: name.to_owned(), reply });
        }
        Ok(reply)
    }
}

/// Blocking: reads the whole scan database over `port_name`, reporting `(done, total)` systems.
fn download_blocking(
    model: &dyn ScannerModel,
    port_name: &str,
    mut progress: impl FnMut(usize, usize),
) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
    let port = serialport::new(port_name, model.baud_rate())
        .timeout(Duration::from_secs(3))
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    let mut scanner = SerialScanner(port);
    model.download(&mut scanner, &mut progress)
}

/// Records a failed download in both its status signal and the message queue.
fn fail_download(error: String) {
    push_message(MessageKind::Warning, "download", format!("Download failed: {error}"));
    *DOWNLOAD_STATUS.write() = DownloadStatus::Failed(error);
}

/// Downloads the scan database from the detected scanner, updating
/// [`DOWNLOAD_STATUS`] and [`SCAN_DATABASE`].
pub async fn download_database() {
    let Some(port_name) = SCANNER_INFO.peek().as_ref().map(|s| s.port.clone()) else {
        fail_download("No scanner detected".into());
        return;
    };
    let Some(model) = connected_model() else {
        fail_download("Unsupported scanner model".into());
        return;
    };
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }
    *DOWNLOAD_STATUS.write() = DownloadStatus::InProgress { done: 0, total: 0 };

    // Serial I/O is blocking, so run it off the UI task and send progress back over a channel.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let task = tokio::task::spawn_blocking(move || {
        download_blocking(model, &port_name, |done, total| {
            let _ = tx.send((done, total));
        })
    });
    // Signals can only be written from the UI thread, so progress is relayed through a channel.
    while let Some((done, total)) = rx.recv().await {
        *DOWNLOAD_STATUS.write() = DownloadStatus::InProgress { done, total };
    }

    match task.await {
        Ok(Ok((db, memory))) => {
            *SCANNER_MEMORY.write() = memory;
            *SCAN_DATABASE.write() = Some(db);
            *VALIDATION.write() = None;
            *DOWNLOAD_STATUS.write() = DownloadStatus::Completed;
            push_message(MessageKind::Note, "download", "Download completed successfully.");
        }
        Ok(Err(Error::Scanner { cmd, reply })) if cmd == "PRG" => {
            fail_download(format!(
                "scanner rejected PRG: {reply}. The scanner refuses Program Mode while it is in a menu, \
                 direct entry, or Quick Save. Return it to the normal scan/hold screen and try again."
            ));
        }
        Ok(Err(e)) => fail_download(e.to_string()),
        Err(e) => fail_download(e.to_string()),
    }
    DOWNLOAD_ACTIVE.store(false, Ordering::SeqCst);
}

/// `Some((done, total))` systems while an upload to the scanner is running.
pub static UPLOAD_PROGRESS: GlobalSignal<Option<(usize, usize)>> = Signal::global(|| None);

/// Blocking: replaces the scanner's systems with `db`, reporting `(done, total)` systems.
fn upload_blocking(
    model: &dyn ScannerModel,
    port_name: &str,
    db: &ScanDatabase,
    mut progress: impl FnMut(usize, usize),
) -> Result<(), Error> {
    let port = serialport::new(port_name, model.baud_rate())
        .timeout(Duration::from_secs(10))
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    let mut scanner = SerialScanner(port);
    model.upload(&mut scanner, db, &mut progress)
}

/// Adds a warning for a failed upload without hiding the current database.
fn fail_upload(error: String) {
    push_message(MessageKind::Warning, "upload", format!("Upload failed: {error}"));
}

/// Writes [`SCAN_DATABASE`] to the detected scanner, replacing its existing systems.
pub async fn upload_database() {
    let Some(port_name) = SCANNER_INFO.peek().as_ref().map(|s| s.port.clone()) else {
        fail_upload("No scanner detected".into());
        return;
    };
    let Some(db) = SCAN_DATABASE.peek().clone() else {
        fail_upload("No database to upload".into());
        return;
    };
    let Some(model) = connected_model() else {
        fail_upload("Unsupported scanner model".into());
        return;
    };
    let errors = model.validate(&db);
    if !errors.is_empty() {
        let problems: Vec<String> = errors.iter().map(ToString::to_string).collect();
        fail_upload(format!("Database cannot be uploaded to a {}: {}", model.name(), problems.join("; ")));
        return;
    }
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }
    *UPLOAD_PROGRESS.write() = Some((0, db.systems.len()));
    push_message(MessageKind::Note, "upload", format!("Uploading database to {}...", model.name()));

    // Serial I/O is blocking, so run it off the UI task and send progress back over a channel.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let task = tokio::task::spawn_blocking(move || {
        upload_blocking(model, &port_name, &db, |done, total| {
            let _ = tx.send((done, total));
        })
    });
    // Only this async side updates the UI signal while the worker handles serial I/O.
    while let Some(p) = rx.recv().await {
        *UPLOAD_PROGRESS.write() = Some(p);
    }

    match task.await {
        Ok(Ok(())) => push_message(MessageKind::Note, "upload", "Upload completed successfully."),
        Ok(Err(Error::Scanner { cmd, reply })) if cmd == "PRG" => fail_upload(format!(
            "scanner rejected PRG: {reply}. Return the scanner to the normal scan/hold screen and try again."
        )),
        Ok(Err(e)) => fail_upload(e.to_string()),
        Err(e) => fail_upload(e.to_string()),
    }
    *UPLOAD_PROGRESS.write() = None;
    DOWNLOAD_ACTIVE.store(false, Ordering::SeqCst);
}

/// Suggested file name for saving a database: `<timestamp> <model> Download.ron`.
pub fn default_save_name(model: &str) -> String {
    // Keep path separators in the model name out of the file name.
    let model = model.replace(['/', '\\'], "-");
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    format!("{stamp} {model} Download.ron")
}

/// Folder the save dialog opens in: `~/Documents/Scanner Programmer`, if it can be found.
pub fn default_save_dir() -> Option<std::path::PathBuf> {
    let dir = dirs::document_dir()?.join("Scanner Programmer");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Saves `db` as RON to `path`, giving it a `.ron` extension if it has none.
pub fn save_database_ron(db: &ScanDatabase, path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let mut path = path.to_path_buf();
    if path.extension().is_none() {
        path.set_extension("ron");
    }
    let text = ron::ser::to_string_pretty(db, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Could not serialize the database: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(path)
}

/// Reads a RON database previously written by [`save_database_ron`].
pub fn load_database_ron(path: &std::path::Path) -> Result<ScanDatabase, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("Could not parse {}: {e}", path.display()))
}
