//! Live control of a connected scanner: a connection actor that owns the serial port while
//! Remote Control is on, polls the scanner's reception status, and runs queued commands.
//! All protocol knowledge lives in [`crate::models::live`]; this module only schedules I/O and
//! publishes the result as signals for the Scanner page.

use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::messages::{push_message, MessageKind};
use crate::models::live::{commands, Action, ActivityLog, LiveState, ScanMode};
use crate::models::{self, DynLink, Error, GroupSummary, Scanner, SystemSummary};
use crate::scanner_interaction::{
    connected_model, open_scanner, DOWNLOAD_ACTIVE, SCANNER_INFO,
};

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

/// The log being built; copied into [`ACTIVITY`] only when it changes, to avoid redrawing on every poll.
static ACTIVITY_STORE: Mutex<ActivityLog> = Mutex::new(ActivityLog { entries: Vec::new(), open: false });

/// Empties the activity log.
pub fn clear_activity() {
    *ACTIVITY_STORE.lock().expect("activity") = ActivityLog::default();
    *ACTIVITY.write() = ActivityLog::default();
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
    log.push_back(LogLine { clock: clock(), tx, text });
}

/// A request for the actor thread.
enum Request {
    /// One command and its reply line.
    Command { command: String, reply: tokio::sync::oneshot::Sender<Result<String, Error>> },
    /// Several commands that must run back to back, such as a Program Mode edit.
    Job(Box<dyn FnOnce(&mut dyn Scanner) + Send>),
}

/// Message from the actor thread to the UI.
enum Event {
    /// A completed poll: the `GLG` reply and, every other poll, the `PWR` reply.
    Poll { glg: String, pwr: Option<String>, latency: Duration },
    /// A volume or squelch reading.
    Level { command: &'static str, reply: String },
    PollFailed { consecutive: u32 },
    Log { tx: bool, text: String },
    /// The actor has stopped; the reason is `None` for a normal shutdown.
    Stopped(Option<String>),
}

/// Handle to the running actor. Dropping the sender stops the thread.
static LINK: Mutex<Option<mpsc::Sender<Request>>> = Mutex::new(None);

/// Queues a request for the actor.
fn submit(request: Request) -> Result<(), String> {
    let link = LINK.lock().map_err(|_| "control link poisoned".to_owned())?;
    let tx = link.as_ref().ok_or("remote control is off")?;
    tx.send(request).map_err(|_| "remote control stopped".to_owned())
}

/// Sends one command through the actor and returns the scanner's reply line.
pub async fn send_command(command: impl Into<String>) -> Result<String, Error> {
    let (reply, wait) = tokio::sync::oneshot::channel();
    submit(Request::Command { command: command.into(), reply }).map_err(|e| Error::Io(std::io::Error::other(e)))?;
    wait.await.map_err(|_| Error::Io(std::io::Error::other("remote control stopped")))?
}

/// Runs `job` on the actor thread with exclusive use of the scanner, so polling pauses meanwhile.
async fn run_job<T: Send + 'static>(
    job: impl FnOnce(&mut DynLink) -> Result<T, Error> + Send + 'static,
) -> Result<T, String> {
    let (reply, wait) = tokio::sync::oneshot::channel();
    submit(Request::Job(Box::new(move |link| {
        let _ = reply.send(job(&mut DynLink(link)).map_err(|e| e.to_string()));
    })))?;
    wait.await.map_err(|_| "remote control stopped".to_owned())?
}

/// Reads a level from the reply to `VOL` or `SQL`.
fn parse_level(command: &str, reply: &str) -> Option<u8> {
    reply.strip_prefix(command)?.strip_prefix(',')?.trim().parse().ok()
}

/// Reports a failed control action as a warning.
fn warn<T>(what: &str, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(v) => Some(v),
        Err(e) => {
            push_message(MessageKind::Warning, "control", format!("{what} failed: {e}"));
            None
        }
    }
}

/// Carries out a UI intent on the scanner, warning on failure. This is the only entry point the
/// Scanner page uses to change the scanner.
pub async fn perform(action: Action) {
    if let Err(e) = try_perform(&action).await {
        push_message(MessageKind::Warning, "control", format!("{} failed: {e}", action.label()));
    }
}

/// [`perform`] without the warning, returning a message fit for the user.
async fn try_perform(action: &Action) -> Result<(), String> {
    if let Action::SetMute(mute) = action {
        return set_mute(*mute).await;
    }
    let model = connected_model().ok_or("No supported scanner is connected")?;
    let profile = model.control_profile().ok_or("Remote control is not supported on this model")?;
    for cmd in commands(action, &profile)? {
        send_command(cmd).await.map_err(|e| match (&e, action) {
            // SPEC: the scanner answers NG when a mode jump or quick search is not allowed right now.
            (Error::Scanner { .. }, Action::JumpMode(_) | Action::QuickTune { .. } | Action::JumpToTag { .. }) => {
                "The scanner refused the command (it may be in a menu or have nothing to jump to)".to_owned()
            }
            _ => e.to_string(),
        })?;
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
        _ => {}
    }
    Ok(())
}

/// Mutes by setting volume 0, remembering the old volume; un-mutes by restoring it.
async fn set_mute(mute: bool) -> Result<(), String> {
    let level = if mute {
        let volume = LIVE.peek().volume.unwrap_or(0);
        if volume > 0 {
            *PRE_MUTE_VOLUME.lock().map_err(|_| "mute state poisoned")? = volume;
        }
        0
    } else {
        *PRE_MUTE_VOLUME.lock().map_err(|_| "mute state poisoned")?
    };
    send_command(format!("VOL,{level}")).await.map_err(|e| e.to_string())?;
    let mut live = LIVE.write();
    live.muted = mute;
    live.volume = Some(level);
    Ok(())
}

/// Sends `cmd`, reporting both sides to the UI's TX/RX log.
fn transfer(link: &mut dyn Scanner, events: &tokio::sync::mpsc::UnboundedSender<Event>, cmd: &str) -> Result<String, Error> {
    let _ = events.send(Event::Log { tx: true, text: cmd.to_owned() });
    let result = link.send(cmd);
    let text = match &result {
        Ok(reply) => reply.clone(),
        Err(e) => format!("! {e}"),
    };
    let _ = events.send(Event::Log { tx: false, text });
    result
}

/// One status snapshot; `PWR` is read on every other poll to halve the traffic.
fn poll_once(
    link: &mut dyn Scanner,
    events: &tokio::sync::mpsc::UnboundedSender<Event>,
    tick: u32,
) -> Result<Event, Error> {
    let started = Instant::now();
    let glg = transfer(link, events, "GLG")?;
    let pwr = if tick % 2 == 0 { Some(transfer(link, events, "PWR")?) } else { None };
    Ok(Event::Poll { glg, pwr, latency: started.elapsed() })
}

/// Thread body: opens the port, then alternates between queued commands and status polls
/// until the sender is dropped or the scanner stops answering.
fn run_actor(
    port: String,
    baud: u32,
    requests: mpsc::Receiver<Request>,
    events: tokio::sync::mpsc::UnboundedSender<Event>,
) {
    let outcome = (|| -> Result<(), Error> {
        let mut link = open_scanner(&port, baud, Duration::from_secs(2))?;
        let (mut tick, mut failures) = (0u32, 0u32);
        loop {
            match requests.recv_timeout(POLL_INTERVAL) {
                Ok(Request::Command { command, reply }) => {
                    let _ = reply.send(transfer(&mut link, &events, &command));
                }
                Ok(Request::Job(job)) => job(&mut link),
                Err(RecvTimeoutError::Timeout) => {
                    tick = tick.wrapping_add(1);
                    // A missed poll is tolerated: the link only counts as lost after several in a row.
                    match poll_once(&mut link, &events, tick) {
                        Ok(poll) => {
                            failures = 0;
                            if events.send(poll).is_err() {
                                return Ok(());
                            }
                        }
                        Err(e) => {
                            failures += 1;
                            let _ = events.send(Event::PollFailed { consecutive: failures });
                            if failures >= LOST_AFTER {
                                return Err(e);
                            }
                        }
                    }
                    if tick % LEVEL_EVERY == 0 {
                        for command in ["VOL", "SQL"] {
                            if let Ok(reply) = transfer(&mut link, &events, command) {
                                let _ = events.send(Event::Level { command, reply });
                            }
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // Remote control is being turned off: put the quick keys back first.
                    if let Err(e) = restore_filter_now(&mut DynLink(&mut link)) {
                        let _ = events.send(Event::Log { tx: false, text: format!("! could not restore the scan filter: {e}") });
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
                *ACTIVITY.write() = store.clone();
            }
            drop(store);
            if *LIVE.peek() != state {
                *LIVE.write() = state;
            }
            let latency_ms = u32::try_from(latency.as_millis()).unwrap_or(u32::MAX);
            let health = Health { latency_ms, errors: HEALTH.peek().errors };
            if *HEALTH.peek() != health {
                *HEALTH.write() = health;
            }
            if *CONN.peek() != ConnState::Connected {
                *CONN.write() = ConnState::Connected;
            }
        }
        Event::Level { command, reply } => {
            let level = parse_level(command, &reply);
            let mut live = LIVE.write();
            match command {
                "VOL" => live.volume = level.or(live.volume),
                _ => live.squelch = level.or(live.squelch),
            }
        }
        Event::PollFailed { consecutive } => {
            HEALTH.write().errors += 1;
            if consecutive >= DEGRADED_AFTER {
                *CONN.write() = ConnState::Degraded;
            }
        }
        Event::Log { tx, text } => log_line(tx, text),
        Event::Stopped(_) => return false,
    }
    true
}

/// Turns Remote Control on: claims the serial port and starts the actor and its UI relay.
pub async fn start_remote_control() {
    let Some(info) = SCANNER_INFO.peek().clone() else {
        push_message(MessageKind::Warning, "control", "No scanner detected".to_string());
        return;
    };
    let Some(model) = connected_model() else {
        push_message(MessageKind::Warning, "control", "Unsupported scanner model".to_string());
        return;
    };
    if model.control_profile().is_none() {
        push_message(
            MessageKind::Warning,
            "control",
            format!("Remote control is not supported for the {} yet", model.name()),
        );
        return;
    }
    if DOWNLOAD_ACTIVE.swap(true, Ordering::SeqCst) {
        push_message(MessageKind::Warning, "control", "The scanner is busy with another operation".to_string());
        return;
    }

    let (tx, requests) = mpsc::channel();
    let (events_tx, mut events) = tokio::sync::mpsc::unbounded_channel();
    *LINK.lock().expect("control link") = Some(tx);
    *LIVE.write() = LiveState::default();
    *ACTIVITY_STORE.lock().expect("activity") = ActivityLog::default();
    *ACTIVITY.write() = ActivityLog::default();
    *HEALTH.write() = Health::default();
    *CONN.write() = ConnState::Connecting;
    *REMOTE_CONTROL.write() = true;

    let baud = model.baud_rate();
    std::thread::spawn(move || run_actor(info.port, baud, requests, events_tx));

    if load_saved_snapshot() {
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
    *QUICK_KEYS.write() = None;
    *GROUP_KEYS.write() = None;
    *FILTER_ACTIVE.write() = SNAPSHOT.lock().expect("snapshot").is_some();
    *CONN.write() = if stop_reason.is_some() { ConnState::Lost } else { ConnState::Off };
    if let Some(reason) = stop_reason {
        push_message(MessageKind::Warning, "control", format!("Remote control stopped: {reason}"));
    }
    *REMOTE_CONTROL.write() = false;
}

/// Turns Remote Control off. The actor notices, releases the port and reports back.
pub fn stop_remote_control() {
    *LINK.lock().expect("control link") = None;
}

/// Quick-key state read from the scanner.
#[derive(Debug, Clone, PartialEq)]
pub struct QuickKeys {
    /// 100 digits indexed by [`models::qsl_position`]: 0 unassigned, 1 on, 2 off.
    pub qsl: String,
    pub systems: Vec<SystemSummary>,
}

/// Group quick keys of one system.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupKeys {
    pub system: SystemSummary,
    /// 10 digits indexed by [`models::qgl_position`].
    pub qgl: String,
    pub groups: Vec<GroupSummary>,
}

/// System quick keys, once read from the scanner.
pub static QUICK_KEYS: GlobalSignal<Option<QuickKeys>> = Signal::global(|| None);

/// Group quick keys of the system whose groups are expanded.
pub static GROUP_KEYS: GlobalSignal<Option<GroupKeys>> = Signal::global(|| None);

/// True while a temporary scan filter is in place and can be restored.
pub static FILTER_ACTIVE: GlobalSignal<bool> = Signal::global(|| false);

/// Quick-key settings saved before a temporary scan filter was applied.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FilterSnapshot {
    pub qsl: String,
    /// `(system index, QGL digits)` for each system whose group keys were changed.
    pub qgl: Vec<(u16, String)>,
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
        Ok(())
    })?;
    store_snapshot(None);
    resume_scanning(link);
    Ok(())
}

/// Reads the system quick keys from the scanner. This briefly leaves scan mode.
pub async fn refresh_quick_keys() {
    let result = run_job(|link| {
        let keys = models::with_program_mode(link, |link| {
            Ok(QuickKeys { qsl: models::read_qsl(link)?, systems: models::read_system_summaries(link)? })
        })?;
        resume_scanning(link);
        Ok(keys)
    })
    .await;
    if let Some(keys) = warn("Reading quick keys", result) {
        // A group list from before the refresh may no longer match the scanner.
        *GROUP_KEYS.write() = None;
        *QUICK_KEYS.write() = Some(keys);
    }
}

/// Reads the groups and group quick keys of one system.
pub async fn load_group_keys(system: SystemSummary) {
    let sys = system.clone();
    let result = run_job(move |link| {
        let keys = models::with_program_mode(link, |link| {
            Ok(GroupKeys {
                qgl: models::read_qgl(link, sys.index)?,
                groups: models::read_group_summaries(link, sys.index)?,
                system: sys,
            })
        })?;
        resume_scanning(link);
        Ok(keys)
    })
    .await;
    if let Some(keys) = warn("Reading groups", result) {
        *GROUP_KEYS.write() = Some(keys);
    }
}

/// Turns a system quick key on or off.
pub async fn toggle_system_key(key: u8) {
    let position = models::qsl_position(key);
    let result = run_job(move |link| {
        let digits = models::with_program_mode(link, |link| {
            let digits = toggled(&models::read_qsl(link)?, position);
            models::write_qsl(link, &digits)?;
            Ok(digits)
        })?;
        resume_scanning(link);
        Ok(digits)
    })
    .await;
    if let Some(digits) = warn("Changing quick key", result) {
        if let Some(keys) = QUICK_KEYS.write().as_mut() {
            keys.qsl = digits;
        }
    }
}

/// Turns a group quick key of the expanded system on or off.
pub async fn toggle_group_key(system: crate::database::Index, key: u8) {
    let position = models::qgl_position(key);
    let result = run_job(move |link| {
        let digits = models::with_program_mode(link, |link| {
            let digits = toggled(&models::read_qgl(link, system)?, position);
            models::write_qgl(link, system, &digits)?;
            Ok(digits)
        })?;
        resume_scanning(link);
        Ok(digits)
    })
    .await;
    if let Some(digits) = warn("Changing group quick key", result) {
        if let Some(keys) = GROUP_KEYS.write().as_mut() {
            keys.qgl = digits;
        }
    }
}

/// Temporarily scans only system quick key `key` — and, if `group` is given, only that group
/// quick key within it. The previous settings are saved first so they can be restored.
pub async fn scan_only(system: SystemSummary, group: Option<u8>) {
    let Some(key) = system.quick_key else { return };
    let sys_index = system.index;
    let result = run_job(move |link| {
        let (qsl, unaffected) = models::with_program_mode(link, |link| {
            let before_qsl = models::read_qsl(link)?;
            let before_qgl = match group {
                Some(_) => Some(models::read_qgl(link, sys_index)?),
                None => None,
            };
            // Keep the earliest settings if a filter is already active, so Restore goes back to the real ones.
            let mut snapshot = SNAPSHOT.lock().expect("snapshot").clone().unwrap_or(FilterSnapshot {
                qsl: before_qsl.clone(),
                qgl: Vec::new(),
            });
            if let Some(q) = &before_qgl {
                if !snapshot.qgl.iter().any(|(i, _)| *i == sys_index.0) {
                    snapshot.qgl.push((sys_index.0, q.clone()));
                }
            }
            store_snapshot(Some(snapshot));

            let digits = only_key(&before_qsl, models::qsl_position(key));
            models::write_qsl(link, &digits)?;
            if let (Some(g), Some(q)) = (group, before_qgl) {
                models::write_qgl(link, sys_index, &only_key(&q, models::qgl_position(g)))?;
            }
            // Systems without a quick key cannot be switched off, so they keep scanning.
            let unaffected = digits.bytes().filter(|&b| b == b'0').count();
            Ok((digits, unaffected))
        })?;
        resume_scanning(link);
        Ok((qsl, unaffected))
    })
    .await;
    if let Some((digits, _)) = warn("Scan only", result) {
        *FILTER_ACTIVE.write() = true;
        if let Some(keys) = QUICK_KEYS.write().as_mut() {
            keys.qsl = digits;
        }
        push_message(
            MessageKind::Note,
            "control",
            "Temporary scan filter applied. Systems without a quick key still scan; use Restore to undo.".to_string(),
        );
        *GROUP_KEYS.write() = None;
    }
}

/// Restores the quick keys saved before the temporary scan filter.
pub async fn restore_filter() {
    let result = run_job(|link| restore_filter_now(link)).await;
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

    /// VOL and SQL replies yield their level; mismatched replies do not.
    #[test]
    fn parses_levels() {
        assert_eq!(parse_level("VOL", "VOL,7"), Some(7));
        assert_eq!(parse_level("SQL", "VOL,7"), None);
    }
}
