use crate::messages::{MessageKind, push_message};
use dioxus::prelude::*;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::database::{ScanDatabase, ScannerMemory, Source};
use crate::merge::merge_import;
use crate::models::{self, Error, Projection, Scanner, ScannerModel};

/// Serial connection and identity details for a supported scanner.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScannerInfo {
    /// Stable per-radio identity used by persisted session data.
    pub id: String,
    pub port: String,
    pub model: String,
    pub firmware: String,
    pub baud: u32,
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
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConnectionPreference {
    port: String,
    model: String,
    baud: u32,
}

fn connection_preference_path() -> Option<std::path::PathBuf> {
    Some(
        dirs::data_local_dir()?
            .join("Scanner Programmer")
            .join("last-connection.ron"),
    )
}

fn load_connection_preference() -> Result<Option<ConnectionPreference>, String> {
    let Some(path) = connection_preference_path() else {
        return Ok(None);
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!("Could not read {}: {error}", path.display()));
        }
    };
    ron::from_str(&text)
        .map(Some)
        .map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn save_connection_preference(info: &ScannerInfo) -> Result<(), String> {
    let path = connection_preference_path()
        .ok_or("No local application-data folder is available for connection settings.")?;
    let parent = path
        .parent()
        .ok_or("The connection-settings path has no parent folder.")?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    let text = ron::ser::to_string(&ConnectionPreference {
        port: info.port.clone(),
        model: info.model.clone(),
        baud: info.baud,
    })
    .map_err(|error| format!("Could not serialize connection settings: {error}"))?;
    std::fs::write(&path, text)
        .map_err(|error| format!("Could not save {}: {error}", path.display()))
}

fn connection_preference_matches(
    preference: Option<&ConnectionPreference>,
    info: &ScannerInfo,
) -> bool {
    preference.is_some_and(|preference| {
        preference.port == info.port
            && preference.model == info.model
            && preference.baud == info.baud
    })
}

fn radio_id(model: &str, port: &str, usb: &UsbDeviceInfo) -> String {
    usb.serial_number
        .as_deref()
        .filter(|serial| !serial.trim().is_empty())
        .map_or_else(
            || format!("{model}@{port}"),
            |serial| format!("{model}#{serial}"),
        )
}

// Global signal for the currently connected scanner (UsbDeviceInfo)
/// USB device seen during port detection, whether supported or not.
pub static SCANNER_USB_DEVICE: GlobalSignal<Option<UsbDeviceInfo>> = Signal::global(|| None);

/// Preserve scanner-specific single-byte LCD codes in Unicode's private-use area.
fn decode_scanner_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii() {
                char::from(*byte)
            } else {
                char::from_u32(0xE000 + u32::from(*byte)).unwrap_or('\u{FFFD}')
            }
        })
        .collect()
}

/// Sends one `\r`-terminated command and returns the response without its `CMD,` prefix.
fn query(port: &mut Box<dyn serialport::SerialPort>, cmd: &str) -> Option<String> {
    let _ = port.clear(serialport::ClearBuffer::Input);
    port.write_all(format!("{cmd}\r").as_bytes()).ok()?;
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while port.read_exact(&mut byte).is_ok() {
        if byte[0] == b'\r' {
            let line = decode_scanner_bytes(&buf);
            return Some(line);
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
    let mut warnings = Vec::new();
    let preference = match load_connection_preference() {
        Ok(preference) => preference,
        Err(error) => {
            warnings.push(error);
            None
        }
    };

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
        let mut baud_rates = vec![115_200, 57_600, 38_400, 19_200, 9_600, 4_800, 2_400];
        if let Some(preference) = preference
            .as_ref()
            .filter(|saved| saved.port == p.port_name)
        {
            baud_rates.retain(|baud| *baud != preference.baud);
            baud_rates.insert(0, preference.baud);
        }
        for baud in baud_rates {
            let mut port = match serialport::new(&p.port_name, baud)
                .timeout(Duration::from_millis(500))
                .open()
            {
                Ok(port) => port,
                Err(error) => {
                    eprintln!(
                        "Failed to open serial port {} at {baud} baud: {error}",
                        p.port_name
                    );
                    continue;
                }
            };

            if let Some(reply) = query(&mut port, "MDL")
                && let Some(model) = reply.strip_prefix("MDL,")
                && models::find(model).is_some()
            {
                let firmware = query(&mut port, "VER")
                    .and_then(|reply| reply.strip_prefix("VER,").map(str::to_owned))
                    .unwrap_or_else(|| "unknown".into());
                let scanner = ScannerInfo {
                    id: radio_id(model, &p.port_name, &device),
                    port: p.port_name.clone(),
                    model: model.to_owned(),
                    firmware,
                    baud,
                };
                if !connection_preference_matches(preference.as_ref(), &scanner) {
                    if let Err(error) = save_connection_preference(&scanner) {
                        warnings.push(error);
                    }
                }
                return Ok(ScannerDetection {
                    scanner: Some(scanner),
                    usb_device: Some(device),
                    warnings,
                });
            }

            if let Some(reply) = query(&mut port, "SI")
                && let Some(identity) = reply.strip_prefix("SI ")
            {
                let model = identity.split(',').next().unwrap_or_default();
                if models::find(model).is_some() {
                    let firmware = query(&mut port, "VR")
                        .and_then(|reply| reply.strip_prefix("VR").map(str::to_owned))
                        .unwrap_or_else(|| "unknown".into());
                    let scanner = ScannerInfo {
                        id: radio_id(model, &p.port_name, &device),
                        port: p.port_name.clone(),
                        model: model.to_owned(),
                        firmware,
                        baud,
                    };
                    if !connection_preference_matches(preference.as_ref(), &scanner) {
                        if let Err(error) = save_connection_preference(&scanner) {
                            warnings.push(error);
                        }
                    }
                    return Ok(ScannerDetection {
                        scanner: Some(scanner),
                        usb_device: Some(device),
                        warnings,
                    });
                }
            }
        }
    }

    // Keep USB identity for the home page even when no compatible scanner answered.
    // No compatible scanner was found.
    Ok(ScannerDetection {
        scanner: None,
        usb_device,
        warnings,
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
pub static DOWNLOAD_STATUS: GlobalSignal<DownloadStatus> =
    Signal::global(|| DownloadStatus::NotStarted);

/// Validation errors for the loaded database; `None` means it needs checking again.
/// The database may be uploaded only when this is `Some` and empty.
pub static VALIDATION: GlobalSignal<Option<Vec<String>>> = Signal::global(|| None);

/// Database currently shown in the editor, if one is loaded.
pub static SCAN_DATABASE: GlobalSignal<Option<ScanDatabase>> =
    Signal::global(|| Some(ScanDatabase::new_placeholder()));

/// Memory report returned by the scanner with the most recent successful download.
pub static SCANNER_MEMORY: GlobalSignal<Option<ScannerMemory>> = Signal::global(|| None);

/// Supported scanner currently detected on a serial port.
pub static SCANNER_INFO: GlobalSignal<Option<ScannerInfo>> = Signal::global(|| None);

/// Guards the serial port so the background detector waits during transfers.
pub static DOWNLOAD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Model of the connected scanner, if one is detected and supported.
pub fn connected_model() -> Option<&'static dyn ScannerModel> {
    SCANNER_INFO
        .read()
        .as_ref()
        .and_then(|info| models::find(&info.model))
}

/// Scanner model the user picked while none is connected (by name); `None` means the default.
pub static CHOSEN_MODEL: GlobalSignal<Option<String>> = Signal::global(|| None);

/// Model the database is checked and uploaded against: the connected scanner if any, else the
/// user's choice, else the default. It never limits what can be edited, only what is sent.
pub fn target_model() -> &'static dyn ScannerModel {
    connected_model()
        .or_else(|| CHOSEN_MODEL.read().as_deref().and_then(models::by_name))
        .unwrap_or_else(models::default_model)
}

/// [`Scanner`] transport over a serial port.
struct SerialScanner(Box<dyn serialport::SerialPort>);

impl Scanner for SerialScanner {
    /// Sends one model command and turns the scanner rejection codes into errors.
    fn send(&mut self, cmd: &str) -> Result<String, Error> {
        // Drop stale bytes (e.g. a reply that arrived after a timeout) so replies stay paired with commands.
        let _ = self.0.clear(serialport::ClearBuffer::Input);
        self.0
            .write_all(format!("{cmd}\r").as_bytes())
            .map_err(Error::Io)?;
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            self.0.read_exact(&mut byte).map_err(Error::Io)?;
            if matches!(byte[0], b'\r' | b'\n') {
                break;
            }
            buf.push(byte[0]);
        }
        let reply = decode_scanner_bytes(&buf);
        let name = cmd.split(',').next().unwrap_or(cmd);
        let body = reply.strip_prefix(&format!("{name},")).unwrap_or(&reply);
        if matches!(body, "ERR" | "NG" | "FER" | "ORER") {
            return Err(Error::Scanner {
                cmd: name.to_owned(),
                reply,
            });
        }
        Ok(reply)
    }

    fn send_routed(
        &mut self,
        cmd: &str,
        is_unsolicited: &dyn Fn(&str) -> bool,
        on_unsolicited: &mut dyn FnMut(String),
    ) -> Result<String, Error> {
        self.0
            .write_all(format!("{cmd}\r").as_bytes())
            .map_err(Error::Io)?;
        loop {
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            loop {
                self.0.read_exact(&mut byte).map_err(Error::Io)?;
                if byte[0] == b'\r' {
                    break;
                }
                if byte[0] != b'\n' {
                    buf.push(byte[0]);
                }
            }
            let reply = decode_scanner_bytes(&buf);
            if is_unsolicited(&reply) {
                on_unsolicited(reply);
                continue;
            }
            if matches!(reply.as_str(), "ERR" | "NG" | "FER" | "ORER") {
                return Err(Error::Scanner {
                    cmd: cmd.to_owned(),
                    reply,
                });
            }
            return Ok(reply);
        }
    }

    fn send_xml(&mut self, cmd: &str, closing_tag: &str) -> Result<String, Error> {
        let _ = self.0.clear(serialport::ClearBuffer::Input);
        self.0
            .write_all(format!("{cmd}\r").as_bytes())
            .map_err(Error::Io)?;
        let mut xml = String::new();
        let mut line = Vec::new();
        let mut byte = [0u8; 1];
        loop {
            self.0.read_exact(&mut byte).map_err(Error::Io)?;
            if matches!(byte[0], b'\r' | b'\n') {
                if line.is_empty() {
                    continue;
                }
                let text = decode_scanner_bytes(&line);
                line.clear();
                let body = text
                    .strip_prefix(&format!("{cmd},"))
                    .unwrap_or(text.as_str());
                if matches!(body, "ERR" | "NG" | "FER" | "ORER") {
                    return Err(Error::Scanner {
                        cmd: cmd.to_owned(),
                        reply: text,
                    });
                }
                if text.starts_with(&format!("{cmd},")) && text.contains("<XML>") {
                    continue;
                }
                xml.push_str(&text);
                xml.push('\n');
                if text.contains(closing_tag) {
                    break;
                }
                if xml.len() > 1_048_576 {
                    return Err(Error::BadValue {
                        cmd: "XML",
                        pos: xml.len(),
                        value: "scanner XML response exceeded 1 MiB".to_owned(),
                    });
                }
            } else {
                line.push(byte[0]);
            }
        }
        Ok(xml)
    }

    fn stream(
        &mut self,
        start: &str,
        stop: &str,
        should_stop: &dyn Fn() -> bool,
        on_line: &mut dyn FnMut(String),
    ) -> Result<(), Error> {
        let old_timeout = self.0.timeout();
        self.0
            .clear(serialport::ClearBuffer::Input)
            .map_err(|e| Error::Io(e.into()))?;
        self.0
            .write_all(format!("{start}\r").as_bytes())
            .map_err(Error::Io)?;
        self.0
            .set_timeout(Duration::from_millis(250))
            .map_err(|e| Error::Io(e.into()))?;
        let mut line = Vec::new();
        let mut byte = [0u8; 1];
        while !should_stop() {
            match self.0.read_exact(&mut byte) {
                Ok(()) if byte[0] == b'\r' => {
                    let received = decode_scanner_bytes(&line);
                    if matches!(received.as_str(), "ERR" | "NG" | "FER" | "ORER" | "CSC,NG") {
                        let _ = self.0.set_timeout(old_timeout);
                        return Err(Error::Scanner {
                            cmd: "CSC".to_owned(),
                            reply: received,
                        });
                    }
                    on_line(received);
                    line.clear();
                }
                Ok(()) => line.push(byte[0]),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) => {}
                Err(e) => {
                    let _ = self.0.set_timeout(old_timeout);
                    return Err(Error::Io(e));
                }
            }
        }
        self.0
            .write_all(format!("{stop}\r").as_bytes())
            .map_err(Error::Io)?;
        line.clear();
        loop {
            self.0.read_exact(&mut byte).map_err(Error::Io)?;
            if byte[0] == b'\r' {
                break;
            }
            line.push(byte[0]);
        }
        self.0
            .set_timeout(old_timeout)
            .map_err(|e| Error::Io(e.into()))?;
        let reply = decode_scanner_bytes(&line);
        if reply != "CSC,OK" {
            return Err(Error::UnexpectedReply {
                expected: "CSC",
                reply,
            });
        }
        Ok(())
    }
}

/// Opens `port_name` as a [`Scanner`] with the given read timeout.
pub fn open_scanner(port_name: &str, baud: u32, timeout: Duration) -> Result<impl Scanner, Error> {
    let port = serialport::new(port_name, baud)
        .timeout(timeout)
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    Ok(SerialScanner(port))
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
    push_message(
        MessageKind::Warning,
        "download",
        format!("Download failed: {error}"),
    );
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
            // Merge rather than replace, so edits and systems from other sources are kept.
            let source = Source::Scanner {
                model: model.name().to_owned(),
            };
            let report = {
                let mut library = SCAN_DATABASE.write();
                merge_import(
                    library.get_or_insert_with(ScanDatabase::default),
                    &source,
                    db.systems,
                )
            };
            *VALIDATION.write() = None;
            *DOWNLOAD_STATUS.write() = DownloadStatus::Completed;
            push_message(
                MessageKind::Note,
                "download",
                format!("Download completed: {}", report.summary()),
            );
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
    push_message(
        MessageKind::Warning,
        "upload",
        format!("Upload failed: {error}"),
    );
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
    // Only the part the scanner can use is sent, so only that part has to pass validation.
    let Projection { db, .. } = model.project(&db);
    let errors = db.validate();
    if !errors.is_empty() {
        let problems: Vec<String> = errors.iter().map(ToString::to_string).collect();
        fail_upload(format!(
            "Database cannot be uploaded to a {}: {}",
            model.name(),
            problems.join("; ")
        ));
        return;
    }
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }
    *UPLOAD_PROGRESS.write() = Some((0, db.systems.len()));
    push_message(
        MessageKind::Note,
        "upload",
        format!("Uploading database to {}...", model.name()),
    );

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
        Ok(Ok(())) => push_message(
            MessageKind::Note,
            "upload",
            "Upload completed successfully.",
        ),
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
pub fn save_database_ron(
    db: &ScanDatabase,
    path: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
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
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    let mut db: ScanDatabase =
        ron::from_str(&text).map_err(|e| format!("Could not parse {}: {e}", path.display()))?;
    db.migrate();
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::decode_scanner_bytes;

    #[test]
    fn scanner_byte_decoder_preserves_ascii_utf8_and_raw_glyphs() {
        assert_eq!(decode_scanner_bytes(b"SCAN"), "SCAN");
        assert_eq!(decode_scanner_bytes("Temp °".as_bytes()), "Temp °");
        assert_eq!(decode_scanner_bytes(&[b'A', 0x81, b'B']), "A\u{e081}B");
    }
}
