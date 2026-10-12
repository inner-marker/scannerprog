//! Live control of a connected scanner: a connection actor that owns the serial port while
//! Remote Control is on, polls the scanner's reception status, and runs queued commands.
//! All protocol knowledge lives in [`crate::models::live`]; this module only schedules I/O and
//! publishes the result as signals for the Scanner page.

use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::messages::{MessageKind, push_message};
use crate::models::bct8_live::{
    apply_channel as apply_bct8_channel, apply_unsolicited as apply_bct8_unsolicited,
    is_unsolicited as is_bct8_unsolicited, parse_banks as parse_bct8_banks,
    parse_lcd as parse_bct8_lcd, parse_lcd_frequency as parse_bct8_lcd_frequency,
    parse_mode as parse_bct8_mode,
};
use crate::models::homepatrol_live::{
    ChargeStatus, WaterfallStatus, parse_charge_status, parse_gst, parse_scanner_info,
    parse_waterfall,
};
use crate::models::live::{
    Action, ActivityLog, DisplayStatus, Family, LiveState, ScanMode, SpectrumPoint, commands,
    parse_csc, parse_sts,
};
use crate::models::{self, DynLink, Error, GroupSummary, Scanner, SystemSummary};
use crate::scanner_interaction::{DOWNLOAD_ACTIVE, SCANNER_INFO, connected_model, open_scanner};

/// How often the scanner's status is read while no command is waiting.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Volume and squelch are re-read every this many polls (about 5 s), in case the knobs are turned.
const LEVEL_EVERY: u32 = 25;

/// Consecutive failed polls before the connection is shown as degraded.
const DEGRADED_AFTER: u32 = 3;

/// Consecutive failed polls before the connection is given up.
const LOST_AFTER: u32 = 10;

/// Lines kept in the TX/RX log.
const LOG_LIMIT: usize = 1000;

/// True while the user has Remote Control switched on (the port is owned by the actor).
pub static REMOTE_CONTROL: GlobalSignal<bool> = Signal::global(|| false);

/// Connection health shown in the Scanner page's connection bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Off,
    Connecting,
    Connected,
    /// Polls are failing but the link may recover.
    Degraded,
    /// The scanner stopped answering and remote control was turned off.
    Lost,
}

pub static CONN: GlobalSignal<ConnState> = Signal::global(|| ConnState::Off);

/// What the scanner is doing, as far as the app can tell.
pub static LIVE: GlobalSignal<LiveState> = Signal::global(LiveState::default);

/// Poll health for the status strip.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Health {
    pub latency_ms: u32,
    pub errors: u32,
}

pub static HEALTH: GlobalSignal<Health> = Signal::global(Health::default);

/// One line of the TX/RX log.
#[derive(Debug, Clone, PartialEq)]
pub struct LogLine {
    pub clock: String,
    /// True for a command sent to the scanner, false for a reply (or an error).
    pub tx: bool,
    pub text: String,
}

/// The raw command log, newest last.
pub static TXRX_LOG: GlobalSignal<VecDeque<LogLine>> = Signal::global(VecDeque::new);

/// Receptions seen since remote control was turned on.
pub static ACTIVITY: GlobalSignal<ActivityLog> = Signal::global(ActivityLog::default);

/// LCD and indicator status from the latest `STS` poll.
pub static DISPLAY_STATUS: GlobalSignal<Option<DisplayStatus>> = Signal::global(|| None);

/// GPS fix state from `GGA`; `None` means unavailable or not polled yet.
pub static GPS_FIX: GlobalSignal<Option<bool>> = Signal::global(|| None);

/// Detailed charge information reported by SDS scanners.
pub static CHARGE_STATUS: GlobalSignal<Option<ChargeStatus>> = Signal::global(|| None);

/// Selected BCT8 scan banks A-E.
pub static BCT8_BANKS: GlobalSignal<Option<[bool; 5]>> = Signal::global(|| None);

/// Toggles one BCT8 scan bank immediately. At least one bank must remain selected.
pub async fn toggle_bct8_bank(index: usize) {
    let Some(mut banks) = *BCT8_BANKS.peek() else {
        return;
    };
    if index >= banks.len() {
        return;
    }
    if banks[index] && banks.iter().filter(|selected| **selected).count() == 1 {
        push_message(
            MessageKind::Warning,
            "control",
            "At least one BCT8 scan bank must remain enabled.".to_owned(),
        );
        return;
    }
    banks[index] = !banks[index];
    let selected = banks
        .iter()
        .enumerate()
        .filter_map(|(index, selected)| {
            selected.then_some(char::from(b'A' + u8::try_from(index).unwrap_or(0)))
        })
        .collect::<String>();
    match send_command(format!("SB {selected}")).await {
        Ok(reply) => match parse_bct8_banks(&reply) {
            Ok(banks) => *BCT8_BANKS.write() = Some(banks),
            Err(error) => push_message(
                MessageKind::Warning,
                "control",
                format!("Changing BCT8 scan banks returned an invalid reply: {error}"),
            ),
        },
        Err(error) => push_message(
            MessageKind::Warning,
            "control",
            format!("Changing BCT8 scan banks failed: {error}"),
        ),
    }
}

/// Recent samples from the active `CSC` sweep.
pub static SPECTRUM: GlobalSignal<VecDeque<SpectrumPoint>> = Signal::global(VecDeque::new);
pub static SPECTRUM_ACTIVE: GlobalSignal<bool> = Signal::global(|| false);

/// SDS waterfall metadata and bounded FFT history.
pub static WATERFALL_STATUS: GlobalSignal<Option<WaterfallStatus>> = Signal::global(|| None);
pub static WATERFALL: GlobalSignal<VecDeque<Vec<u16>>> = Signal::global(VecDeque::new);
pub static WATERFALL_ACTIVE: GlobalSignal<bool> = Signal::global(|| false);

/// True while a Program Mode job is running.
pub static PROGRAM_MODE: GlobalSignal<bool> = Signal::global(|| false);

/// Set by the Virtual Remote while its LCD is visible.
pub static DISPLAY_WANTED: AtomicBool = AtomicBool::new(false);

/// Controls the blocking CSC stream in the actor.
static SPECTRUM_WANTED: AtomicBool = AtomicBool::new(false);
static WATERFALL_WANTED: AtomicBool = AtomicBool::new(false);
static WATERFALL_RESTORE: Mutex<Option<ScanMode>> = Mutex::new(None);

/// The log being built; copied into [`ACTIVITY`] only when it changes, to avoid redrawing on every poll.
static ACTIVITY_STORE: Mutex<ActivityLog> = Mutex::new(ActivityLog {
    entries: Vec::new(),
    open: false,
});
static ACTIVITY_RADIO: Mutex<Option<String>> = Mutex::new(None);

fn activity_path(radio_id: &str) -> Result<std::path::PathBuf, String> {
    let root = dirs::data_local_dir()
        .ok_or("No local application-data folder is available for activity history.")?;
    let file_name = radio_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    Ok(root
        .join("Scanner Programmer")
        .join("activity")
        .join(format!("{file_name}.ron")))
}

fn load_activity(radio_id: &str) -> Result<ActivityLog, String> {
    let path = activity_path(radio_id)?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ActivityLog::default());
        }
        Err(error) => return Err(format!("Could not read {}: {error}", path.display())),
    };
    ron::from_str(&text).map_err(|error| format!("Could not parse {}: {error}", path.display()))
}

fn persist_activity(log: &ActivityLog) -> Result<(), String> {
    let radio_id = ACTIVITY_RADIO
        .lock()
        .expect("activity radio")
        .clone()
        .ok_or("No radio is selected for activity history.")?;
    let path = activity_path(&radio_id)?;
    let parent = path
        .parent()
        .ok_or("The activity-history path has no parent folder.")?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    let snapshot = log.persistence_snapshot(Instant::now());
    let text = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|error| format!("Could not serialize activity history: {error}"))?;
    std::fs::write(&path, text)
        .map_err(|error| format!("Could not save {}: {error}", path.display()))
}

fn publish_activity(log: &ActivityLog) {
    *ACTIVITY.write() = log.clone();
    if let Err(error) = persist_activity(log) {
        push_message(MessageKind::Warning, "control", error);
    }
}

fn persist_current_activity() {
    let store = ACTIVITY_STORE.lock().expect("activity");
    if let Err(error) = persist_activity(&store) {
        push_message(MessageKind::Warning, "control", error);
    }
}

/// Empties the activity log.
pub fn clear_activity() {
    let mut store = ACTIVITY_STORE.lock().expect("activity");
    *store = ActivityLog::default();
    publish_activity(&store);
}

/// The volume to return to when un-muting; mute is emulated by setting volume 0.
static PRE_MUTE_VOLUME: Mutex<u8> = Mutex::new(0);

/// Current local time for log lines.
fn clock() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Appends to the TX/RX log, dropping the oldest line past the limit.
fn log_line(tx: bool, text: String) {
    let mut log = TXRX_LOG.write();
    if log.len() >= LOG_LIMIT {
        log.pop_front();
    }
    log.push_back(LogLine {
        clock: clock(),
        tx,
        text,
    });
}

/// A request for the actor thread.
enum Request {
    /// One command and its reply line.
    Command {
        command: String,
        reply: tokio::sync::oneshot::Sender<Result<String, Error>>,
    },
    /// Several commands that must run back to back, such as a Program Mode edit.
    Job {
        program_mode: bool,
        job: Box<dyn FnOnce(&mut dyn Scanner) + Send>,
    },
    StartSpectrum {
        restore: Vec<String>,
    },
}

/// Message from the actor thread to the UI.
enum Event {
    /// A completed poll: the `GLG` reply and, every other poll, the `PWR` reply.
    Poll {
        glg: String,
        pwr: Option<String>,
        latency: Duration,
    },
    HpPoll {
        xml: String,
        latency: Duration,
    },
    Charge(String),
    LegacyPoll {
        mode: Option<String>,
        squelch: Option<String>,
        channel: Option<String>,
        lcd: Option<(String, String, String)>,
        banks: Option<String>,
        mute_control: Option<String>,
        mute: Option<String>,
        latency: Duration,
    },
    LegacyUnsolicited(String),
    WaterfallStatus(String),
    WaterfallFrame(String),
    /// A volume or squelch reading.
    Level {
        command: &'static str,
        reply: String,
    },
    Status(String),
    Gps(String),
    Spectrum(SpectrumPoint),
    SpectrumStopped(Option<String>),
    ProgramMode(bool),
    PollFailed {
        consecutive: u32,
    },
    Log {
        tx: bool,
        text: String,
    },
    /// The actor has stopped; the reason is `None` for a normal shutdown.
    Stopped(Option<String>),
}

/// Handle to the running actor. Dropping the sender stops the thread.
static LINK: Mutex<Option<mpsc::Sender<Request>>> = Mutex::new(None);

/// Queues a request for the actor.
fn submit(request: Request) -> Result<(), String> {
    let link = LINK
        .lock()
        .map_err(|_| "control link poisoned".to_owned())?;
    let tx = link.as_ref().ok_or("remote control is off")?;
    tx.send(request)
        .map_err(|_| "remote control stopped".to_owned())
}

/// Sends one command through the actor and returns the scanner's reply line.
pub async fn send_command(command: impl Into<String>) -> Result<String, Error> {
    let (reply, wait) = tokio::sync::oneshot::channel();
    submit(Request::Command {
        command: command.into(),
        reply,
    })
    .map_err(|e| Error::Io(std::io::Error::other(e)))?;
    wait.await
        .map_err(|_| Error::Io(std::io::Error::other("remote control stopped")))?
}

/// Runs `job` on the actor thread with exclusive use of the scanner, so polling pauses meanwhile.
async fn run_job<T: Send + 'static>(
    program_mode: bool,
    job: impl FnOnce(&mut DynLink) -> Result<T, Error> + Send + 'static,
) -> Result<T, String> {
    let (reply, wait) = tokio::sync::oneshot::channel();
    submit(Request::Job {
        program_mode,
        job: Box::new(move |link| {
            let _ = reply.send(job(&mut DynLink(link)).map_err(|e| e.to_string()));
        }),
    })?;
    wait.await
        .map_err(|_| "remote control stopped".to_owned())?
}

/// Starts a CSC sweep. It owns the serial link until [`stop_spectrum`] is called.
pub fn start_spectrum() {
    if SPECTRUM_WANTED.swap(true, Ordering::SeqCst) {
        return;
    }
    SPECTRUM.write().clear();
    *SPECTRUM_ACTIVE.write() = true;
    let restore = LIVE
        .peek()
        .mode
        .as_ref()
        .and_then(|mode| {
            connected_model()?
                .control_profile()
                .and_then(|p| commands(&Action::JumpMode(*mode), &p, &LIVE.peek()).ok())
        })
        .unwrap_or_else(|| vec!["KEY,S,P".to_owned()]);
    if let Err(e) = submit(Request::StartSpectrum { restore }) {
        SPECTRUM_WANTED.store(false, Ordering::SeqCst);
        *SPECTRUM_ACTIVE.write() = false;
        push_message(
            MessageKind::Warning,
            "control",
            format!("Starting spectrum failed: {e}"),
        );
    }
}

/// Stops the active CSC sweep; the actor restores the previous mode best-effort.
pub fn stop_spectrum() {
    SPECTRUM_WANTED.store(false, Ordering::SeqCst);
}

/// Enters SDS waterfall mode and enables GST/GWF polling.
pub async fn start_waterfall() {
    if WATERFALL_WANTED.swap(true, Ordering::SeqCst) {
        return;
    }
    *WATERFALL_RESTORE.lock().expect("waterfall restore") = LIVE.peek().mode;
    WATERFALL.write().clear();
    *WATERFALL_STATUS.write() = None;
    match send_command("JPM,WF_MODE,RESERVE").await {
        Ok(_) => *WATERFALL_ACTIVE.write() = true,
        Err(error) => {
            WATERFALL_WANTED.store(false, Ordering::SeqCst);
            push_message(
                MessageKind::Warning,
                "control",
                format!("Starting waterfall failed: {error}"),
            );
        }
    }
}

/// Stops SDS waterfall polling and restores the mode active before the view opened.
pub fn stop_waterfall() {
    if !WATERFALL_WANTED.swap(false, Ordering::SeqCst) {
        return;
    }
    *WATERFALL_ACTIVE.write() = false;
    let mode = WATERFALL_RESTORE.lock().expect("waterfall restore").take();
    if let Some(mode) = mode {
        spawn(async move {
            let Some(profile) = connected_model().and_then(|model| model.control_profile()) else {
                return;
            };
            let state = LIVE.peek().clone();
            if let Ok(commands) = commands(&Action::JumpMode(mode), &profile, &state) {
                for command in commands {
                    let _ = send_command(command).await;
                }
            }
        });
    }
}

/// Reads a level from the reply to `VOL` or `SQL`.
fn parse_level(command: &str, reply: &str) -> Option<u8> {
    reply
        .strip_prefix(command)?
        .strip_prefix(',')?
        .trim()
        .parse()
        .ok()
}

/// Reports a failed control action as a warning.
fn warn<T>(what: &str, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            push_message(
                MessageKind::Warning,
                "control",
                format!("{what} failed: {e}"),
            );
            None
        }
    }
}

/// Carries out a UI intent on the scanner, warning on failure. This is the only entry point the
/// Scanner page uses to change the scanner.
pub async fn perform(action: Action) {
    if let Err(e) = try_perform(&action).await {
        push_message(
            MessageKind::Warning,
            "control",
            format!("{} failed: {e}", action.label()),
        );
    }
}

/// Sends one validated virtual-keypad gesture.
pub async fn press_key(code: &'static str, mode: &'static str) {
    let result = async {
        let profile = connected_model()
            .and_then(|m| m.control_profile())
            .ok_or_else(|| "Remote control is not supported on this model".to_owned())?;
        if !profile.keymap.iter().any(|key| key.code == code) {
            return Err(format!("The {} has no {code} key", profile.model));
        }
        if !matches!(mode, "P" | "L" | "H" | "R") {
            return Err("Invalid keypad gesture".to_owned());
        }
        let command = if profile.family == Family::Legacy {
            let suffix = (mode == "L").then_some("H").unwrap_or("");
            format!("KEY{code}{suffix}")
        } else {
            format!("KEY,{code},{mode}")
        };
        send_command(command).await.map_err(|e| e.to_string())?;
        Ok(())
    }
    .await;
    if let Err(e) = result {
        push_message(
            MessageKind::Warning,
            "control",
            format!("Key press failed: {e}"),
        );
    }
}

/// [`perform`] without the warning, returning a message fit for the user.
async fn try_perform(action: &Action) -> Result<(), String> {
    if let Action::SetMute(mute) = action {
        return set_mute(*mute).await;
    }
    let model = connected_model().ok_or("No supported scanner is connected")?;
    let profile = model
        .control_profile()
        .ok_or("Remote control is not supported on this model")?;
    for cmd in commands(action, &profile, &LIVE.peek())? {
        let reply = send_command(cmd).await.map_err(|e| match (&e, action) {
            // SPEC: the scanner answers NG when a mode jump or quick search is not allowed right now.
            (
                Error::Scanner { .. },
                Action::JumpMode(_) | Action::QuickTune { .. } | Action::JumpToTag { .. },
            ) => "The scanner refused the command (it may be in a menu or have nothing to jump to)"
                .to_owned(),
            _ => e.to_string(),
        })?;
        if matches!(action, Action::SetRecord(_)) {
            let error = [
                ("0001", "the scanner could not access the recording file"),
                ("0002", "the battery is too low to record"),
                ("0003", "the recording session limit was reached"),
                ("0004", "the scanner's real-time clock was lost"),
            ]
            .into_iter()
            .find(|(code, _)| reply.contains(code))
            .map(|(_, message)| message);
            if let Some(error) = error {
                return Err(error.to_owned());
            }
        }
    }
    // The protocol cannot report the mode, so remember what we last asked for.
    let mut live = LIVE.write();
    match action {
        Action::JumpMode(mode) => live.mode = Some(*mode),
        Action::ResumeScan => live.mode = Some(ScanMode::Scan),
        Action::QuickTune { .. } => live.mode = Some(ScanMode::QuickSearch),
        Action::SetVolume(v) => {
            live.volume = Some(*v);
            live.muted = false;
        }
        Action::SetSquelch(v) => live.squelch = Some(*v),
        Action::SetMuteAutomatic => live.mute_automatic = true,
        Action::SetRecord(recording) => live.recording = *recording,
        _ => {}
    }
    Ok(())
}

/// Mutes by setting volume 0, remembering the old volume; un-mutes by restoring it.
async fn set_mute(mute: bool) -> Result<(), String> {
    let family = connected_model()
        .and_then(|model| model.control_profile())
        .map(|profile| profile.family)
        .ok_or("Remote control is not supported on this model")?;
    if family == Family::Legacy {
        send_command(if mute { "MUN" } else { "MUF" })
            .await
            .map_err(|error| error.to_string())?;
        LIVE.write().muted = mute;
        LIVE.write().mute_automatic = false;
        return Ok(());
    }
    let level = if mute {
        let volume = LIVE.peek().volume.unwrap_or(0);
        if volume > 0 {
            *PRE_MUTE_VOLUME.lock().map_err(|_| "mute state poisoned")? = volume;
        }
        0
    } else {
        *PRE_MUTE_VOLUME.lock().map_err(|_| "mute state poisoned")?
    };
    send_command(format!("VOL,{level}"))
        .await
        .map_err(|e| e.to_string())?;
    let mut live = LIVE.write();
    live.muted = mute;
    live.volume = Some(level);
    Ok(())
}

/// Sends `cmd`, reporting both sides to the UI's TX/RX log.
fn transfer(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    cmd: &str,
) -> Result<String, Error> {
    let _ = events.send(Event::Log {
        tx: true,
        text: cmd.to_owned(),
    });
    let result = link.send(cmd);
    let text = match &result {
        Ok(reply) => reply.clone(),
        Err(e) => format!("! {e}"),
    };
    let _ = events.send(Event::Log { tx: false, text });
    result
}

fn transfer_legacy(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    command: &str,
) -> Result<String, Error> {
    let _ = events.send(Event::Log {
        tx: true,
        text: command.to_owned(),
    });
    let mut on_unsolicited = |line: String| {
        let _ = events.send(Event::Log {
            tx: false,
            text: line.clone(),
        });
        let _ = events.send(Event::LegacyUnsolicited(line));
    };
    let predicate = |line: &str| is_bct8_unsolicited(command, line);
    let result = link.send_routed(command, &predicate, &mut on_unsolicited);
    let text = match &result {
        Ok(reply) => reply.clone(),
        Err(error) => format!("! {error}"),
    };
    let _ = events.send(Event::Log { tx: false, text });
    result
}

fn optional_legacy_poll(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    command: &str,
) -> Result<Option<String>, Error> {
    match transfer_legacy(link, events, command) {
        Ok(reply) => Ok(Some(reply)),
        Err(Error::Scanner { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}

fn transfer_xml(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    command: &str,
    closing_tag: &str,
) -> Result<String, Error> {
    let _ = events.send(Event::Log {
        tx: true,
        text: command.to_owned(),
    });
    let result = link.send_xml(command, closing_tag);
    let text = match &result {
        Ok(xml) => format!("<ScannerInfo XML: {} bytes>", xml.len()),
        Err(error) => format!("! {error}"),
    };
    let _ = events.send(Event::Log { tx: false, text });
    result
}

/// One status snapshot; `PWR` is read on every other poll to halve the traffic.
fn poll_once(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    tick: u32,
    gps: bool,
    pwr_status: bool,
    family: Family,
) -> Result<Event, Error> {
    let started = Instant::now();
    if family == Family::Legacy {
        let mode = optional_legacy_poll(link, events, "MD")?;
        if mode.is_none() {
            // SPEC: while the BCT8 is in its menu, only KEY commands are accepted.
            return Ok(Event::LegacyPoll {
                mode: None,
                squelch: None,
                channel: None,
                lcd: None,
                banks: None,
                mute_control: None,
                mute: None,
                latency: started.elapsed(),
            });
        }
        let squelch = optional_legacy_poll(link, events, "SQ")?;
        let channel = optional_legacy_poll(link, events, "MA")?;
        let status_every = if DISPLAY_WANTED.load(Ordering::Relaxed) {
            5
        } else {
            LEVEL_EVERY
        };
        let lcd = if tick % status_every == 0 {
            let banks = transfer_legacy(link, events, "LCD BNK");
            let channel = transfer_legacy(link, events, "LCD CHN");
            let frequency = transfer_legacy(link, events, "LCD FRQ");
            match (banks, channel, frequency) {
                (Ok(banks), Ok(channel), Ok(frequency)) => Some((banks, channel, frequency)),
                _ => None,
            }
        } else {
            None
        };
        let banks = (tick % LEVEL_EVERY == 0)
            .then(|| transfer_legacy(link, events, "SB").ok())
            .flatten();
        let mute = (tick % LEVEL_EVERY == 0)
            .then(|| transfer_legacy(link, events, "MU?").ok())
            .flatten();
        let mute_control = (tick % LEVEL_EVERY == 0)
            .then(|| transfer_legacy(link, events, "MU").ok())
            .flatten();
        return Ok(Event::LegacyPoll {
            mode,
            squelch,
            channel,
            lcd,
            banks,
            mute_control,
            mute,
            latency: started.elapsed(),
        });
    }
    if matches!(family, Family::HomePatrol | Family::Sds) {
        // PLAN: PSI push activation and interval units are undocumented; poll the equivalent
        // documented GSI snapshot until captured hardware transcripts can verify PSI.
        let xml = transfer_xml(link, events, "GSI", "</ScannerInfo>")?;
        if family == Family::Sds && tick % LEVEL_EVERY == 0 {
            if let Ok(reply) = transfer(link, events, "GCS") {
                let _ = events.send(Event::Charge(reply));
            }
        }
        if family == Family::Sds && WATERFALL_WANTED.load(Ordering::Relaxed) {
            if tick % 5 == 0 {
                if let Ok(reply) = transfer(link, events, "GST") {
                    let _ = events.send(Event::WaterfallStatus(reply));
                }
            }
            if let Ok(reply) = transfer(link, events, "GWF,1,1") {
                let _ = events.send(Event::WaterfallFrame(reply));
            }
        }
        return Ok(Event::HpPoll {
            xml,
            latency: started.elapsed(),
        });
    }
    let glg = transfer(link, events, "GLG")?;
    let pwr = if pwr_status && tick % 2 == 0 {
        Some(transfer(link, events, "PWR")?)
    } else {
        None
    };
    let status_every = if DISPLAY_WANTED.load(Ordering::Relaxed) {
        5
    } else {
        LEVEL_EVERY
    };
    if tick % status_every == 0 {
        if let Ok(status) = transfer(link, events, "STS") {
            let _ = events.send(Event::Status(status));
        }
        if gps && tick % LEVEL_EVERY == 0 {
            if let Ok(gga) = transfer(link, events, "GGA") {
                let _ = events.send(Event::Gps(gga));
            }
        }
    }
    Ok(Event::Poll {
        glg,
        pwr,
        latency: started.elapsed(),
    })
}

/// Thread body: opens the port, then alternates between queued commands and status polls
/// until the sender is dropped or the scanner stops answering.
fn run_actor(
    port: String,
    baud: u32,
    requests: mpsc::Receiver<Request>,
    events: tokio::sync::mpsc::UnboundedSender<Event>,
    gps: bool,
    pwr_status: bool,
    family: Family,
) {
    let outcome = (|| -> Result<(), Error> {
        let mut link = open_scanner(&port, baud, Duration::from_secs(2))?;
        if family == Family::Legacy {
            for command in ["IDN", "QUN", "RIN", "HAN", "HPN"] {
                let _ = transfer_legacy(&mut link, &events, command);
            }
        }
        let (mut tick, mut failures) = (0u32, 0u32);
        loop {
            match requests.recv_timeout(POLL_INTERVAL) {
                Ok(Request::Command { command, reply }) => {
                    let result = if family == Family::Legacy {
                        transfer_legacy(&mut link, &events, &command)
                    } else {
                        transfer(&mut link, &events, &command)
                    };
                    let _ = reply.send(result);
                }
                Ok(Request::Job { program_mode, job }) => {
                    if program_mode {
                        let _ = events.send(Event::ProgramMode(true));
                    }
                    job(&mut link);
                    if program_mode {
                        let _ = events.send(Event::ProgramMode(false));
                    }
                }
                Ok(Request::StartSpectrum { restore }) => {
                    let _ = events.send(Event::Log {
                        tx: true,
                        text: "CSC,ON".to_owned(),
                    });
                    let should_stop = || !SPECTRUM_WANTED.load(Ordering::SeqCst);
                    let mut on_line = |line: String| {
                        let _ = events.send(Event::Log {
                            tx: false,
                            text: line.clone(),
                        });
                        if let Ok(point) = parse_csc(&line) {
                            let _ = events.send(Event::Spectrum(point));
                        }
                    };
                    let stream_error = if let Err(e) =
                        link.stream("CSC,ON", "CSC,OFF", &should_stop, &mut on_line)
                    {
                        let _ = events.send(Event::Log {
                            tx: false,
                            text: format!("! spectrum: {e}"),
                        });
                        Some(e.to_string())
                    } else {
                        None
                    };
                    SPECTRUM_WANTED.store(false, Ordering::SeqCst);
                    for command in restore {
                        let _ = transfer(&mut link, &events, &command);
                    }
                    let _ = events.send(Event::SpectrumStopped(stream_error));
                }
                Err(RecvTimeoutError::Timeout) => {
                    tick = tick.wrapping_add(1);
                    // A missed poll is tolerated: the link only counts as lost after several in a row.
                    match poll_once(&mut link, &events, tick, gps, pwr_status, family) {
                        Ok(poll) => {
                            failures = 0;
                            if events.send(poll).is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            failures += 1;
                            let _ = events.send(Event::PollFailed {
                                consecutive: failures,
                            });
                            if failures >= LOST_AFTER {
                                return Err(e);
                            }
                        }
                    }
                    if tick % LEVEL_EVERY == 0
                        && !matches!(family, Family::HomePatrol | Family::Sds)
                    {
                        for command in ["VOL", "SQL"] {
                            if let Ok(reply) = transfer(&mut link, &events, command) {
                                let _ = events.send(Event::Level { command, reply });
                            }
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // Remote control is being turned off: put the quick keys back first.
                    if matches!(family, Family::EarlyComma | Family::Gen1 | Family::Xt)
                        && let Err(e) = restore_filter_now(&mut DynLink(&mut link))
                    {
                        let _ = events.send(Event::Log {
                            tx: false,
                            text: format!("! could not restore the scan filter: {e}"),
                        });
                    }
                    if family == Family::Legacy {
                        for command in ["IDF", "QUF", "RIF", "HAF", "HPF"] {
                            let _ = transfer_legacy(&mut link, &events, command);
                        }
                    }
                    return Ok(());
                }
            }
        }
    })();
    // Release the port for detection, download and upload before telling the UI.
    DOWNLOAD_ACTIVE.store(false, Ordering::SeqCst);
    let _ = events.send(Event::Stopped(outcome.err().map(|e| e.to_string())));
}

/// Folds one actor event into the UI signals. Returns false once the actor has stopped.
fn apply_event(event: Event) -> bool {
    match event {
        Event::Poll { glg, pwr, latency } => {
            let mut state = LIVE.peek().clone();
            if state.apply_glg(&glg).is_err() {
                HEALTH.write().errors += 1;
                return true;
            }
            if let Some(pwr) = pwr {
                state.apply_pwr(&pwr);
            }
            let mut store = ACTIVITY_STORE.lock().expect("activity");
            if store.observe(&state, Instant::now(), &clock()) {
                publish_activity(&store);
            }
            drop(store);
            if *LIVE.peek() != state {
                *LIVE.write() = state;
            }
            let latency_ms = u32::try_from(latency.as_millis()).unwrap_or(u32::MAX);
            let health = Health {
                latency_ms,
                errors: HEALTH.peek().errors,
            };
            if *HEALTH.peek() != health {
                *HEALTH.write() = health;
            }
            if *CONN.peek() != ConnState::Connected {
                *CONN.write() = ConnState::Connected;
            }
        }
        Event::HpPoll { xml, latency } => match parse_scanner_info(&xml) {
            Ok(snapshot) => {
                let state = snapshot.live;
                let mut store = ACTIVITY_STORE.lock().expect("activity");
                if store.observe(&state, Instant::now(), &clock()) {
                    publish_activity(&store);
                }
                drop(store);
                if *LIVE.peek() != state {
                    *LIVE.write() = state;
                }
                *DISPLAY_STATUS.write() = snapshot.display;
                let latency_ms = u32::try_from(latency.as_millis()).unwrap_or(u32::MAX);
                *HEALTH.write() = Health {
                    latency_ms,
                    errors: HEALTH.peek().errors,
                };
                *CONN.write() = ConnState::Connected;
            }
            Err(_) => HEALTH.write().errors += 1,
        },
        Event::Charge(reply) => match parse_charge_status(&reply) {
            Ok(status) => *CHARGE_STATUS.write() = Some(status),
            Err(_) => HEALTH.write().errors += 1,
        },
        Event::LegacyPoll {
            mode,
            squelch,
            channel,
            lcd,
            banks,
            mute_control,
            mute,
            latency,
        } => {
            let mut state = LIVE.peek().clone();
            if let Some(mode) = mode {
                if let Ok((mode, held)) = parse_bct8_mode(&mode) {
                    state.mode = Some(mode);
                    state.channel_held = held;
                } else {
                    HEALTH.write().errors += 1;
                }
            }
            if let Some(squelch) = squelch {
                state.squelch_open = squelch == "+";
            }
            if let Some(channel) = channel {
                if !(state.squelch_open && state.tgid.is_some()) {
                    let _ = apply_bct8_channel(&channel, &mut state);
                }
            }
            if let Some(mute) = mute {
                state.muted = mute == "MU ON";
            }
            if let Some(mute_control) = mute_control {
                state.mute_automatic = mute_control == "MUA";
            }
            if let Some(banks) = banks {
                match parse_bct8_banks(&banks) {
                    Ok(banks) => *BCT8_BANKS.write() = Some(banks),
                    Err(_) => HEALTH.write().errors += 1,
                }
            }
            if let Some((bank, channel, frequency)) = lcd {
                if let Some(freq_hz) = parse_bct8_lcd_frequency(&frequency) {
                    state.freq_hz = Some(freq_hz);
                }
                match parse_bct8_lcd(&bank, &channel, &frequency) {
                    Ok(mut display) => {
                        if let Some(previous) = DISPLAY_STATUS.peek().as_ref() {
                            display.weather_alert = previous.weather_alert;
                        }
                        display.squelch_open = state.squelch_open;
                        display.muted = state.muted;
                        *DISPLAY_STATUS.write() = Some(display);
                    }
                    Err(_) => HEALTH.write().errors += 1,
                }
            }
            let mut store = ACTIVITY_STORE.lock().expect("activity");
            if store.observe(&state, Instant::now(), &clock()) {
                publish_activity(&store);
            }
            drop(store);
            *LIVE.write() = state;
            *HEALTH.write() = Health {
                latency_ms: u32::try_from(latency.as_millis()).unwrap_or(u32::MAX),
                errors: HEALTH.peek().errors,
            };
            *CONN.write() = ConnState::Connected;
        }
        Event::LegacyUnsolicited(line) => {
            let mut state = LIVE.peek().clone();
            let mut display = DISPLAY_STATUS.peek().clone().unwrap_or_default();
            apply_bct8_unsolicited(&line, &mut state, &mut display);
            let mut store = ACTIVITY_STORE.lock().expect("activity");
            if store.observe(&state, Instant::now(), &clock()) {
                publish_activity(&store);
            }
            drop(store);
            if line.starts_with("ID E ") {
                state.tgid = None;
            }
            *LIVE.write() = state;
            *DISPLAY_STATUS.write() = Some(display);
        }
        Event::WaterfallStatus(reply) => match parse_gst(&reply) {
            Ok(status) => *WATERFALL_STATUS.write() = Some(status),
            Err(_) => HEALTH.write().errors += 1,
        },
        Event::WaterfallFrame(reply) => match parse_waterfall(&reply) {
            Ok(frame) => {
                let mut frames = WATERFALL.write();
                if frames.len() >= 80 {
                    frames.pop_front();
                }
                frames.push_back(frame);
            }
            Err(_) => HEALTH.write().errors += 1,
        },
        Event::Level { command, reply } => {
            let level = parse_level(command, &reply);
            let mut live = LIVE.write();
            match command {
                "VOL" => live.volume = level.or(live.volume),
                _ => live.squelch = level.or(live.squelch),
            }
        }
        Event::Status(reply) => {
            let battery = connected_model()
                .and_then(|m| m.control_profile())
                .is_some_and(|p| p.battery_status);
            match parse_sts(&reply, battery) {
                Ok(status) => {
                    let mut live = LIVE.write();
                    live.signal_bars = status.signal_bars.or(live.signal_bars);
                    live.squelch_open = status.squelch_open;
                    *DISPLAY_STATUS.write() = Some(status);
                }
                Err(_) => HEALTH.write().errors += 1,
            }
        }
        Event::Gps(reply) => {
            let quality = reply
                .split(',')
                .nth(7)
                .and_then(|field| field.parse::<u8>().ok());
            *GPS_FIX.write() = quality.map(|quality| quality > 0);
        }
        Event::Spectrum(point) => {
            let mut points = SPECTRUM.write();
            if points.len() >= 512 {
                points.pop_front();
            }
            points.push_back(point);
        }
        Event::SpectrumStopped(error) => {
            *SPECTRUM_ACTIVE.write() = false;
            if let Some(error) = error {
                push_message(
                    MessageKind::Warning,
                    "control",
                    format!("Spectrum sweep stopped: {error}"),
                );
            }
        }
        Event::ProgramMode(active) => *PROGRAM_MODE.write() = active,
        Event::PollFailed { consecutive } => {
            HEALTH.write().errors += 1;
            if consecutive >= DEGRADED_AFTER {
                *CONN.write() = ConnState::Degraded;
            }
        }
        Event::Log { tx, text } => log_line(tx, text),
        Event::Stopped(_) => {
            persist_current_activity();
            return false;
        }
    }
    true
}

/// Turns Remote Control on: claims the serial port and starts the actor and its UI relay.
pub async fn start_remote_control() {
    let Some(info) = SCANNER_INFO.peek().clone() else {
        push_message(
            MessageKind::Warning,
            "control",
            "No scanner detected".to_string(),
        );
        return;
    };
    let Some(model) = connected_model() else {
        push_message(
            MessageKind::Warning,
            "control",
            "Unsupported scanner model".to_string(),
        );
        return;
    };
    if model.control_profile().is_none() {
        push_message(
            MessageKind::Warning,
            "control",
            format!(
                "Remote control is not supported for the {} yet",
                model.name()
            ),
        );
        return;
    }
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        push_message(
            MessageKind::Warning,
            "control",
            "The scanner is busy with another operation".to_string(),
        );
        return;
    }

    let (tx, requests) = mpsc::channel();
    let (events_tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    *LINK.lock().expect("control link") = Some(tx);
    *LIVE.write() = LiveState::default();
    *ACTIVITY_RADIO.lock().expect("activity radio") = Some(info.id.clone());
    let activity = match load_activity(&info.id) {
        Ok(activity) => activity,
        Err(error) => {
            push_message(MessageKind::Warning, "control", error);
            ActivityLog::default()
        }
    };
    *ACTIVITY_STORE.lock().expect("activity") = activity.clone();
    *ACTIVITY.write() = activity;
    *HEALTH.write() = Health::default();
    *DISPLAY_STATUS.write() = None;
    *GPS_FIX.write() = None;
    *CHARGE_STATUS.write() = None;
    SPECTRUM.write().clear();
    *SPECTRUM_ACTIVE.write() = false;
    WATERFALL.write().clear();
    *WATERFALL_STATUS.write() = None;
    *WATERFALL_ACTIVE.write() = false;
    WATERFALL_WANTED.store(false, Ordering::SeqCst);
    *PROGRAM_MODE.write() = false;
    *HP_QUICK_KEYS.write() = None;
    *BCT8_BANKS.write() = None;
    *CONN.write() = ConnState::Connecting;
    *REMOTE_CONTROL.write() = true;

    let baud = info.baud;
    let gps = model
        .control_profile()
        .is_some_and(|profile| profile.gps_status);
    let pwr_status = model
        .control_profile()
        .is_some_and(|profile| profile.pwr_status);
    let family = model
        .control_profile()
        .map(|profile| profile.family)
        .unwrap_or(Family::Xt);
    std::thread::spawn(move || {
        run_actor(
            info.port, baud, requests, events_tx, gps, pwr_status, family,
        );
    });

    if matches!(family, Family::EarlyComma | Family::Gen1 | Family::Xt) && load_saved_snapshot() {
        *FILTER_ACTIVE.write() = true;
        push_message(
            MessageKind::Warning,
            "control",
            "A temporary scan filter from an earlier session is still stored. Restore it from the Scanner page.".to_string(),
        );
    }

    // Signals can only be written from the UI thread, so the actor's events are relayed here.
    let mut stop_reason = None;
    while let Some(event) = events.recv().await {
        if let Event::Stopped(reason) = &event {
            stop_reason.clone_from(reason);
        }
        if !apply_event(event) {
            break;
        }
    }
    *LINK.lock().expect("control link") = None;
    *LIVE.write() = LiveState::default();
    *DISPLAY_STATUS.write() = None;
    *GPS_FIX.write() = None;
    *CHARGE_STATUS.write() = None;
    SPECTRUM.write().clear();
    *SPECTRUM_ACTIVE.write() = false;
    WATERFALL.write().clear();
    *WATERFALL_STATUS.write() = None;
    *WATERFALL_ACTIVE.write() = false;
    *PROGRAM_MODE.write() = false;
    DISPLAY_WANTED.store(false, Ordering::SeqCst);
    SPECTRUM_WANTED.store(false, Ordering::SeqCst);
    WATERFALL_WANTED.store(false, Ordering::SeqCst);
    *QUICK_KEYS.write() = None;
    *GROUP_KEYS.write() = None;
    *HP_QUICK_KEYS.write() = None;
    *BCT8_BANKS.write() = None;
    *QUICK_KEY_DRAFT.write() = QuickKeyDraft::default();
    *FILTER_ACTIVE.write() = SNAPSHOT.lock().expect("snapshot").is_some();
    *CONN.write() = if stop_reason.is_some() {
        ConnState::Lost
    } else {
        ConnState::Off
    };
    if let Some(reason) = stop_reason {
        push_message(
            MessageKind::Warning,
            "control",
            format!("Remote control stopped: {reason}"),
        );
    }
    *REMOTE_CONTROL.write() = false;
}

/// Turns Remote Control off. The actor notices, releases the port and reports back.
pub fn stop_remote_control() {
    persist_current_activity();
    SPECTRUM_WANTED.store(false, Ordering::SeqCst);
    WATERFALL_WANTED.store(false, Ordering::SeqCst);
    *LINK.lock().expect("control link") = None;
}

/// Quick-key state read from the scanner.
#[derive(Debug, Clone, PartialEq)]
pub struct QuickKeys {
    /// 100 digits indexed by [`models::qsl_position`]: 0 unassigned, 1 on, 2 off.
    pub qsl: String,
    pub original_qsl: String,
    pub systems: Vec<SystemSummary>,
}

/// Group quick keys of one system.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupKeys {
    pub system: SystemSummary,
    /// 10 digits indexed by [`models::qgl_position`].
    pub qgl: String,
    pub original_qgl: String,
    pub groups: Vec<GroupSummary>,
}

/// System quick keys, once read from the scanner.
pub static QUICK_KEYS: GlobalSignal<Option<QuickKeys>> = Signal::global(|| None);

/// Group quick keys of the system whose groups are expanded.
pub static GROUP_KEYS: GlobalSignal<Option<GroupKeys>> = Signal::global(|| None);

/// Live Favorites/System/Department quick keys for HomePatrol and SDS scanners.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HpQuickKeys {
    pub favorites: Vec<u8>,
    pub systems: Vec<u8>,
    pub departments: Vec<u8>,
    pub favorite_context: Option<u8>,
    pub system_context: Option<u8>,
}

pub static HP_QUICK_KEYS: GlobalSignal<Option<HpQuickKeys>> = Signal::global(|| None);

/// Staged quick-key writes. Nothing here reaches the scanner until Apply is clicked.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuickKeyDraft {
    pub qsl: Option<String>,
    pub qgl: Vec<(u16, String)>,
}

impl QuickKeyDraft {
    pub fn count(&self) -> usize {
        usize::from(self.qsl.is_some()) + self.qgl.len()
    }
}

pub static QUICK_KEY_DRAFT: GlobalSignal<QuickKeyDraft> = Signal::global(QuickKeyDraft::default);

/// True while a temporary scan filter is in place and can be restored.
pub static FILTER_ACTIVE: GlobalSignal<bool> = Signal::global(|| false);

fn parse_hp_quick_keys(command: &'static str, reply: &str) -> Result<Vec<u8>, Error> {
    let fields: Vec<_> = reply.trim_end_matches(['\r', '\n']).split(',').collect();
    if fields.first() != Some(&command) {
        return Err(Error::UnexpectedReply {
            expected: command,
            reply: reply.to_owned(),
        });
    }
    if fields.len() < 101 {
        return Err(Error::MissingField {
            cmd: command,
            pos: 100,
        });
    }
    fields[fields.len() - 100..]
        .iter()
        .enumerate()
        .map(|(position, value)| {
            value
                .parse::<u8>()
                .ok()
                .filter(|state| *state <= 2)
                .ok_or_else(|| Error::BadValue {
                    cmd: command,
                    pos: position,
                    value: (*value).to_owned(),
                })
        })
        .collect()
}

/// Reads live Favorites, System, and Department quick-key state in the current XML context.
pub async fn refresh_hp_quick_keys() {
    let favorite = LIVE.peek().favorite_qk;
    let system = LIVE.peek().system_qk;
    let result = async {
        let favorites = parse_hp_quick_keys("FQK", &send_command("FQK").await?)?;
        let systems = match favorite {
            Some(favorite) => {
                parse_hp_quick_keys("SQK", &send_command(format!("SQK,{favorite}")).await?)?
            }
            None => Vec::new(),
        };
        let departments = match (favorite, system) {
            (Some(favorite), Some(system)) => parse_hp_quick_keys(
                "DQK",
                &send_command(format!("DQK,{favorite},{system}")).await?,
            )?,
            _ => Vec::new(),
        };
        Ok::<_, Error>(HpQuickKeys {
            favorites,
            systems,
            departments,
            favorite_context: favorite,
            system_context: system,
        })
    }
    .await;
    if let Some(keys) = warn(
        "Reading live quick keys",
        result.map_err(|error| error.to_string()),
    ) {
        *HP_QUICK_KEYS.write() = Some(keys);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum HpQuickKeyLevel {
    Favorite,
    System,
    Department,
}

/// Toggles one HomePatrol/SDS quick key immediately; no Program Mode is used.
pub async fn toggle_hp_quick_key(level: HpQuickKeyLevel, key: u8) {
    let Some(mut keys) = HP_QUICK_KEYS.peek().clone() else {
        return;
    };
    let (command, states) = match level {
        HpQuickKeyLevel::Favorite => ("FQK".to_owned(), &mut keys.favorites),
        HpQuickKeyLevel::System => {
            let Some(favorite) = keys.favorite_context else {
                return;
            };
            (format!("SQK,{favorite}"), &mut keys.systems)
        }
        HpQuickKeyLevel::Department => {
            let (Some(favorite), Some(system)) = (keys.favorite_context, keys.system_context)
            else {
                return;
            };
            (format!("DQK,{favorite},{system}"), &mut keys.departments)
        }
    };
    let position = usize::from(key);
    let Some(state) = states.get_mut(position) else {
        return;
    };
    if *state == 0 {
        return;
    }
    *state = if *state == 2 { 1 } else { 2 };
    let values = states
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",");
    match send_command(format!("{command},{values}")).await {
        Ok(_) => *HP_QUICK_KEYS.write() = Some(keys),
        Err(error) => push_message(
            MessageKind::Warning,
            "control",
            format!("Changing live quick key failed: {error}"),
        ),
    }
}

/// Quick-key settings saved before a temporary scan filter was applied.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FilterSnapshot {
    pub qsl: String,
    /// `(system index, QGL digits)` for each system whose group keys were changed.
    pub qgl: Vec<(u16, String)>,
    /// Original system lockout states, including systems with no quick key or a duplicate key.
    #[serde(default)]
    pub system_lockouts: Vec<(u16, Option<bool>)>,
}

/// The snapshot of the active filter, mirrored on disk so a crash or unplug can be recovered from.
static SNAPSHOT: Mutex<Option<FilterSnapshot>> = Mutex::new(None);

/// Where the snapshot is stored between sessions.
fn snapshot_path() -> Option<std::path::PathBuf> {
    let dir = dirs::data_local_dir()?.join("Scanner Programmer");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("scan-filter-snapshot.ron"))
}

/// Stores the snapshot in memory and on disk, or removes it when `None`.
fn store_snapshot(snapshot: Option<FilterSnapshot>) {
    if let Some(path) = snapshot_path() {
        match &snapshot {
            Some(s) => {
                if let Ok(text) = ron::ser::to_string_pretty(s, ron::ser::PrettyConfig::default()) {
                    let _ = std::fs::write(path, text);
                }
            }
            None => {
                let _ = std::fs::remove_file(path);
            }
        }
    }
    *SNAPSHOT.lock().expect("snapshot") = snapshot;
}

/// Loads a snapshot left behind by an earlier session; true if there is one.
pub fn load_saved_snapshot() -> bool {
    let saved = snapshot_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| ron::from_str::<FilterSnapshot>(&t).ok());
    let found = saved.is_some();
    if found {
        *SNAPSHOT.lock().expect("snapshot") = saved;
    }
    found
}

/// `digits` with every assigned key (1 or 2) turned off except `position`, which is turned on.
pub fn only_key(digits: &str, position: usize) -> String {
    digits
        .bytes()
        .enumerate()
        .map(|(i, b)| match b {
            b'1' | b'2' if i == position => '1',
            b'1' | b'2' => '2',
            other => other as char,
        })
        .collect()
}

/// `digits` with the key at `position` flipped between on and off; unassigned keys stay as they are.
pub fn toggled(digits: &str, position: usize) -> String {
    digits
        .bytes()
        .enumerate()
        .map(|(i, b)| match b {
            b'1' if i == position => '2',
            b'2' if i == position => '1',
            other => other as char,
        })
        .collect()
}

fn exclusive_system_lockouts(
    lockouts: &[(crate::database::Index, Option<bool>)],
    selected: crate::database::Index,
) -> Vec<(crate::database::Index, Option<bool>)> {
    lockouts
        .iter()
        .map(|(index, _)| (*index, Some(*index != selected)))
        .collect()
}

/// Leaves Program Mode's "scan hold" state by pressing Scan, ignoring a failure.
fn resume_scanning(link: &mut DynLink) {
    let _ = link.send("KEY,S,P");
}

/// Writes the saved quick keys back, forgets the snapshot, and resumes scanning.
fn restore_filter_now(link: &mut DynLink) -> Result<(), Error> {
    let Some(snapshot) = SNAPSHOT.lock().expect("snapshot").clone() else {
        return Ok(());
    };
    models::with_program_mode(link, |link| {
        models::write_qsl(link, &snapshot.qsl)?;
        for (sys, digits) in &snapshot.qgl {
            models::write_qgl(link, crate::database::Index(*sys), digits)?;
        }
        let lockouts = snapshot
            .system_lockouts
            .iter()
            .map(|(index, lockout)| (crate::database::Index(*index), *lockout))
            .collect::<Vec<_>>();
        models::write_system_lockouts(link, &lockouts)?;
        Ok(())
    })?;
    store_snapshot(None);
    resume_scanning(link);
    Ok(())
}

/// Reads the system quick keys from the scanner. This briefly leaves scan mode.
pub async fn refresh_quick_keys() {
    let result = run_job(true, |link| {
        let keys = models::with_program_mode(link, |link| {
            let qsl = models::read_qsl(link)?;
            Ok(QuickKeys {
                original_qsl: qsl.clone(),
                qsl,
                systems: models::read_system_summaries(link)?,
            })
        })?;
        resume_scanning(link);
        Ok(keys)
    })
    .await;
    if let Some(keys) = warn("Reading quick keys", result) {
        // A group list from before the refresh may no longer match the scanner.
        *GROUP_KEYS.write() = None;
        *QUICK_KEY_DRAFT.write() = QuickKeyDraft::default();
        *QUICK_KEYS.write() = Some(keys);
    }
}

/// Reads the groups and group quick keys of one system.
pub async fn load_group_keys(system: SystemSummary) {
    let sys = system.clone();
    let result = run_job(true, move |link| {
        let keys = models::with_program_mode(link, |link| {
            let original_qgl = models::read_qgl(link, sys.index)?;
            Ok(GroupKeys {
                qgl: original_qgl.clone(),
                original_qgl,
                groups: models::read_group_summaries(link, sys.index)?,
                system: sys,
            })
        })?;
        resume_scanning(link);
        Ok(keys)
    })
    .await;
    if let Some(mut keys) = warn("Reading groups", result) {
        if let Some((_, staged)) = QUICK_KEY_DRAFT
            .peek()
            .qgl
            .iter()
            .find(|(i, _)| *i == keys.system.index.0)
        {
            keys.qgl.clone_from(staged);
        }
        *GROUP_KEYS.write() = Some(keys);
    }
}

/// Stages a system quick-key change locally.
pub fn toggle_system_key(key: u8) {
    let position = models::qsl_position(key);
    if let Some(keys) = QUICK_KEYS.write().as_mut() {
        keys.qsl = toggled(&keys.qsl, position);
        QUICK_KEY_DRAFT.write().qsl = (keys.qsl != keys.original_qsl).then(|| keys.qsl.clone());
    }
}

/// Stages a group quick-key change locally.
pub fn toggle_group_key(system: crate::database::Index, key: u8) {
    let position = models::qgl_position(key);
    let mut group_keys = GROUP_KEYS.write();
    let Some(keys) = group_keys.as_mut() else {
        return;
    };
    keys.qgl = toggled(&keys.qgl, position);
    let mut draft = QUICK_KEY_DRAFT.write();
    draft.qgl.retain(|(i, _)| *i != system.0);
    if keys.qgl != keys.original_qgl {
        draft.qgl.push((system.0, keys.qgl.clone()));
    }
}

/// Writes every staged quick-key change in one Program Mode session.
pub async fn apply_quick_keys() {
    let draft = QUICK_KEY_DRAFT.peek().clone();
    if draft.count() == 0 {
        return;
    }
    let applied = draft.clone();
    let result = run_job(true, move |link| {
        models::with_program_mode(link, |link| {
            if let Some(qsl) = &applied.qsl {
                models::write_qsl(link, qsl)?;
            }
            for (system, qgl) in &applied.qgl {
                models::write_qgl(link, crate::database::Index(*system), qgl)?;
            }
            Ok(())
        })?;
        resume_scanning(link);
        Ok(())
    })
    .await;
    if warn("Applying quick keys", result).is_some() {
        if let Some(keys) = QUICK_KEYS.write().as_mut() {
            keys.original_qsl.clone_from(&keys.qsl);
        }
        if let Some(keys) = GROUP_KEYS.write().as_mut() {
            keys.original_qgl.clone_from(&keys.qgl);
        }
        *QUICK_KEY_DRAFT.write() = QuickKeyDraft::default();
    }
}

/// Drops staged quick-key changes without touching the scanner.
pub fn discard_quick_key_changes() {
    if let Some(keys) = QUICK_KEYS.write().as_mut() {
        keys.qsl.clone_from(&keys.original_qsl);
    }
    if let Some(keys) = GROUP_KEYS.write().as_mut() {
        keys.qgl.clone_from(&keys.original_qgl);
    }
    *QUICK_KEY_DRAFT.write() = QuickKeyDraft::default();
}

/// Temporarily scans only system quick key `key` — and, if `group` is given, only that group
/// quick key within it. The previous settings are saved first so they can be restored.
pub async fn scan_only(system: SystemSummary, group: Option<u8>) {
    let Some(key) = system.quick_key else { return };
    let sys_index = system.index;
    let result = run_job(true, move |link| {
        let qsl = models::with_program_mode(link, |link| {
            let before_qsl = models::read_qsl(link)?;
            let before_lockouts = models::read_system_lockouts(link)?;
            let before_qgl = match group {
                Some(_) => Some(models::read_qgl(link, sys_index)?),
                None => None,
            };
            // Keep the earliest settings if a filter is already active, so Restore goes back to the real ones.
            let mut snapshot =
                SNAPSHOT
                    .lock()
                    .expect("snapshot")
                    .clone()
                    .unwrap_or(FilterSnapshot {
                        qsl: before_qsl.clone(),
                        qgl: Vec::new(),
                        system_lockouts: before_lockouts
                            .iter()
                            .map(|(index, lockout)| (index.0, *lockout))
                            .collect(),
                    });
            if snapshot.system_lockouts.is_empty() {
                snapshot.system_lockouts = before_lockouts
                    .iter()
                    .map(|(index, lockout)| (index.0, *lockout))
                    .collect();
            }
            if let Some(q) = &before_qgl {
                if !snapshot.qgl.iter().any(|(i, _)| *i == sys_index.0) {
                    snapshot.qgl.push((sys_index.0, q.clone()));
                }
            }
            store_snapshot(Some(snapshot));

            let digits = only_key(&before_qsl, models::qsl_position(key));
            models::write_qsl(link, &digits)?;
            let exclusive_lockouts = exclusive_system_lockouts(&before_lockouts, sys_index);
            models::write_system_lockouts(link, &exclusive_lockouts)?;
            if let (Some(g), Some(q)) = (group, before_qgl) {
                models::write_qgl(link, sys_index, &only_key(&q, models::qgl_position(g)))?;
            }
            Ok(digits)
        })?;
        resume_scanning(link);
        Ok(qsl)
    })
    .await;
    if let Some(digits) = warn("Scan only", result) {
        *FILTER_ACTIVE.write() = true;
        if let Some(keys) = QUICK_KEYS.write().as_mut() {
            keys.qsl = digits;
        }
        push_message(
            MessageKind::Note,
            "control",
            "Temporary scan filter applied. Only the selected system is unlocked; use Restore to undo.".to_string(),
        );
        *GROUP_KEYS.write() = None;
    }
}

/// Restores the quick keys saved before the temporary scan filter.
pub async fn restore_filter() {
    let result = run_job(true, |link| restore_filter_now(link)).await;
    if warn("Restore", result).is_some() {
        *FILTER_ACTIVE.write() = false;
        *GROUP_KEYS.write() = None;
        refresh_quick_keys().await;
    }
}

/// Forgets a saved filter without writing it to the scanner.
pub fn discard_filter() {
    store_snapshot(None);
    *FILTER_ACTIVE.write() = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scan-only keeps one assigned key on, and toggling leaves unassigned keys alone.
    #[test]
    fn filter_digits() {
        assert_eq!(only_key("1201", 0), "1202");
        assert_eq!(only_key("1201", 1), "2102");
        assert_eq!(toggled("1201", 0), "2201");
        assert_eq!(toggled("1201", 2), "1201");
    }

    #[test]
    fn scan_only_locks_every_other_system_regardless_of_previous_state() {
        let systems = [
            (crate::database::Index(10), None),
            (crate::database::Index(20), Some(true)),
            (crate::database::Index(30), Some(false)),
        ];
        assert_eq!(
            exclusive_system_lockouts(&systems, crate::database::Index(20)),
            vec![
                (crate::database::Index(10), Some(true)),
                (crate::database::Index(20), Some(false)),
                (crate::database::Index(30), Some(true)),
            ]
        );
    }

    /// VOL and SQL replies yield their level; mismatched replies do not.
    #[test]
    fn parses_levels() {
        assert_eq!(parse_level("VOL", "VOL,7"), Some(7));
        assert_eq!(parse_level("SQL", "VOL,7"), None);
    }

    #[test]
    fn parses_homepatrol_quick_key_states_after_context_fields() {
        let states = (0..100)
            .map(|index| (index % 3).to_string())
            .collect::<Vec<_>>()
            .join(",");
        let parsed = parse_hp_quick_keys("SQK", &format!("SQK,4,7,{states}")).unwrap();
        assert_eq!(parsed.len(), 100);
        assert_eq!(&parsed[..6], &[0, 1, 2, 0, 1, 2]);
        assert!(parse_hp_quick_keys("FQK", "FQK,0,1").is_err());
    }
}
