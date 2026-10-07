use dioxus::prelude::*;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::scanner_db::{self, Error, Scanner, ScanDatabase};

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

/// The most recently downloaded scan database.
pub static SCAN_DATABASE: GlobalSignal<Option<ScanDatabase>> = Signal::global(|| None);

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
) -> Result<ScanDatabase, Error> {
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_secs(3))
        .open()
        .map_err(|e| Error::Io(e.into()))?;
    let mut scanner = SerialScanner(port);
    scanner_db::with_program_mode(&mut scanner, |sc| {
        scanner_db::read_database_with_progress(sc, &mut progress)
    })
}

/// Downloads the scan database from the detected scanner, updating
/// [`DOWNLOAD_STATUS`] and [`SCAN_DATABASE`].
pub async fn download_database() {
    let Some(port_name) = SCANNER_INFO.peek().as_ref().map(|s| s.port.clone()) else {
        *DOWNLOAD_STATUS.write() = DownloadStatus::Failed("No scanner detected".into());
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
        Ok(Ok(db)) => {
            *SCAN_DATABASE.write() = Some(db);
            *DOWNLOAD_STATUS.write() = DownloadStatus::Completed;
        }
        Ok(Err(e)) => *DOWNLOAD_STATUS.write() = DownloadStatus::Failed(e.to_string()),
        Err(e) => *DOWNLOAD_STATUS.write() = DownloadStatus::Failed(e.to_string()),
    }
    DOWNLOAD_ACTIVE.store(false, Ordering::SeqCst);
}
