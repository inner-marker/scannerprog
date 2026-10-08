use dioxus::prelude::*;
use crate::messages::{push_message, MessageKind};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::scanner_db::{self, Error, ScanDatabase, Scanner, ScannerMemory};

#[derive(Debug, Clone, PartialEq)]
pub struct ScannerInfo {
    pub port: String,
    pub model: String,
    pub firmware: String,
}


/// USB Device Info
#[derive(Debug, Clone, PartialEq)]
pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

pub struct ScannerDetection {
    pub scanner: Option<ScannerInfo>,
    pub usb_device: Option<UsbDeviceInfo>,
}

// Global signal for the currently connected scanner (UsbDeviceInfo)
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
        // Only consider USB ports.
        let serialport::SerialPortType::UsbPort(info) = &p.port_type else {
            continue;
        };

        // make a UsbDeviceInfo instance for this port
        let device = UsbDeviceInfo {
            vendor_id: info.vid,
            product_id: info.pid,
            serial_number: info.serial_number.clone(),
            manufacturer: info.manufacturer.clone(),
            product: info.product.clone(),
        };

        usb_device = Some(device.clone());

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

    // No compatible scanner was found.
    Ok(ScannerDetection {
        scanner: None,
        usb_device,
    })
}


/// Progress of a download from the scanner.
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    NotStarted,
    InProgress { done: usize, total: usize },
    Completed,
    Failed(String),
}

pub static DOWNLOAD_STATUS: GlobalSignal<DownloadStatus> = Signal::global(|| DownloadStatus::NotStarted);

/// Result of the last validation: `None` if not validated (or edited since),
/// `Some(errors)` otherwise. The database may be uploaded only when this is `Some` and empty.
pub static VALIDATION: GlobalSignal<Option<Vec<String>>> = Signal::global(|| None);

/// The most recently downloaded scan database.
pub static SCAN_DATABASE: GlobalSignal<Option<ScanDatabase>> = Signal::global(|| Some(ScanDatabase::new_placeholder()));

/// Memory statistics reported by the scanner (RMB, MEM) at the last download, if it answered.
pub static SCANNER_MEMORY: GlobalSignal<Option<ScannerMemory>> = Signal::global(|| None);

/// The scanner currently detected on a serial port.
pub static SCANNER_INFO: GlobalSignal<Option<ScannerInfo>> = Signal::global(|| None);

/// Set while a download is running so scanner detection doesn't touch the port.
pub static DOWNLOAD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// [`Scanner`] transport over a serial port.
struct SerialScanner(Box<dyn serialport::SerialPort>);

impl Scanner for SerialScanner {
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
    port_name: &str,
    mut progress: impl FnMut(usize, usize),
) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_secs(3))
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    let mut scanner = SerialScanner(port);
    scanner_db::with_program_mode(&mut scanner, |sc| {
        let db = scanner_db::read_database_with_progress(sc, &mut progress)?;
        // The cross-check is optional: a failed RMB/MEM must not fail the download.
        Ok((db, scanner_db::read_scanner_memory(sc).ok()))
    })
}

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
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }
    *DOWNLOAD_STATUS.write() = DownloadStatus::InProgress { done: 0, total: 0 };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let task = tokio::task::spawn_blocking(move || {
        download_blocking(&port_name, |done, total| {
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
    port_name: &str,
    db: &ScanDatabase,
    mut progress: impl FnMut(usize, usize),
) -> Result<(), Error> {
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_secs(10))
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    let mut scanner = SerialScanner(port);
    scanner_db::with_program_mode(&mut scanner, |sc| {
        scanner_db::write_database_with_progress(sc, db, &mut progress)
    })
}

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
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }
    *UPLOAD_PROGRESS.write() = Some((0, db.systems.len()));
    push_message(MessageKind::Note, "upload", "Uploading database to scanner...");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let task = tokio::task::spawn_blocking(move || {
        upload_blocking(&port_name, &db, |done, total| {
            let _ = tx.send((done, total));
        })
    });
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
