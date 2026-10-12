//! Live-control types shared by every model, plus the wire translation for the "XT / P2"
//! family (BCD325P2, BCD396XT, BC346XT, BCD996XT, BCD996P2). Nothing here depends on the UI, so it can be unit-tested.
//!
//! PLAN: the plan's `Family`/`Driver` split is not needed yet because the app supports one
//! family; `ControlProfile` + [`commands`] play the driver role and a second family can add its own.

use std::time::{Duration, Instant};

use super::Capabilities;

/// Every feature the Scanner page can bind a control to. Some are only listed so the support table is complete.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    SignalMeter,
    Hold,
    HoldAtLevel,
    StepNext,
    StepPrev,
    Avoid,
    ResumeScan,
    Volume,
    Squelch,
    Mute,
    ModeSwitch,
    ToneOut,
    Discovery,
    Waterfall,
    Highway,
    QuickKeysSystems,
    QuickKeysGroups,
    QuickKeysFavorites,
    QuickKeysBanks,
    QuickTune,
    JumpToTag,
    Spectrum,
    LcdMirror,
    Keypad,
    Battery,
    WeatherAlert,
    PowerOff,
}

/// How (or whether) a model supports a capability.
#[derive(Clone, Debug, PartialEq)]
pub enum Support {
    /// A dedicated command with an acknowledgment.
    Native,
    /// Achieved with key presses; there is no confirmation.
    Emulated,
    /// Needs Program Mode, which pauses scanning.
    NeedsProgramMode,
    /// The model cannot do this; the reason is shown to the user.
    Unsupported(&'static str),
    /// The model can, but the app does not do it yet.
    Planned(&'static str),
}

/// Static per-model data used by the Scanner page.
#[derive(Clone, Debug)]
pub struct ControlProfile {
    /// Model name, e.g. `BCD325P2`.
    pub model: &'static str,
    /// `None` means no remote volume.
    pub volume_range: Option<(u8, u8)>,
    /// `None` means no remote squelch.
    pub squelch_range: Option<(u8, u8)>,
    /// Receivable ranges in the wire unit of 100 Hz.
    pub freq_ranges: &'static [(u32, u32)],
    /// Service search banks as `(label, JPM index)`.
    pub service_banks: &'static [(&'static str, &'static str)],
    /// True if the model accepts `POF`.
    pub power_off: bool,
}

impl ControlProfile {
    /// How this model supports `cap`.
    pub fn support(&self, cap: Capability) -> Support {
        use Capability::*;
        use Support::*;
        match cap {
            SignalMeter | Volume | Squelch | ModeSwitch | ToneOut | QuickTune | JumpToTag => Native,
            Hold | StepNext | StepPrev | Avoid | ResumeScan | Mute => Emulated,
            QuickKeysSystems | QuickKeysGroups => NeedsProgramMode,
            HoldAtLevel => Unsupported("Only HomePatrol and SDS scanners can hold at a chosen level."),
            Discovery => Unsupported("Discovery modes exist only on HomePatrol and SDS scanners."),
            Waterfall => Unsupported("A waterfall is only available on SDS scanners."),
            Highway => Unsupported("Highway Patrol mode exists only on the BCT8."),
            QuickKeysFavorites => Unsupported("Favorites Lists exist only on HomePatrol and SDS scanners."),
            QuickKeysBanks => Unsupported("Quick-key banks A–E exist only on the BCT8."),
            PowerOff if self.power_off => Native,
            PowerOff => Unsupported("This model has no remote power-off command."),
            Spectrum => Planned("Spectrum sweep (CSC) is not implemented yet."),
            LcdMirror => Planned("The LCD mirror is not implemented yet."),
            Keypad => Planned("The virtual keypad is not implemented yet."),
            Battery => Planned("Battery status is not implemented yet."),
            WeatherAlert => Planned("Weather alert status is not implemented yet."),
        }
    }
}

/// Service banks of the BCD325P2 and BCD396XT.
const BANKS_WITH_MILITARY: [(&str, &str); 12] = [
    ("Public Safety", "PublicSafety"),
    ("News", "News"),
    ("HAM", "HAM"),
    ("Marine", "Marine"),
    ("Railroad", "Railroad"),
    ("Air", "Air"),
    ("CB", "CB"),
    ("FRS/GMRS/MURS", "FRS/GMRS/MURS"),
    ("Racing", "Racing"),
    ("FM Broadcast", "FM"),
    ("Special", "Special"),
    ("Military Air", "Military"),
];

/// The BC346XT has no Military Air bank.
const BANKS_WITHOUT_MILITARY: [(&str, &str); 11] = [
    ("Public Safety", "PublicSafety"),
    ("News", "News"),
    ("HAM", "HAM"),
    ("Marine", "Marine"),
    ("Railroad", "Railroad"),
    ("Air", "Air"),
    ("CB", "CB"),
    ("FRS/GMRS/MURS", "FRS/GMRS/MURS"),
    ("Racing", "Racing"),
    ("FM Broadcast", "FM"),
    ("Special", "Special"),
];

/// Differences between models of the XT/P2 family that the Scanner page cares about.
pub struct XtVariant {
    /// Highest `VOL` level.
    pub volume_max: u8,
    /// Highest (closed) `SQL` level.
    pub squelch_max: u8,
    pub military_air: bool,
    pub power_off: bool,
}

impl XtVariant {
    /// BCD325P2, BCD396XT: VOL 0–15, SQL 0–15, with `POF`.
    pub const HANDHELD: XtVariant = XtVariant { volume_max: 15, squelch_max: 15, military_air: true, power_off: true };
    /// BCD996XT, BCD996P2: VOL 0–29, SQL 0–19, no `POF`.
    pub const MOBILE: XtVariant = XtVariant { volume_max: 29, squelch_max: 19, military_air: true, power_off: false };
}

/// Profile for the XT/P2 family. These models share one command set and differ only in data.
pub fn xt_profile(model: &'static str, caps: &Capabilities, v: &XtVariant) -> ControlProfile {
    ControlProfile {
        model,
        // SPEC: VOL and SQL ranges differ per model (handhelds 0–15; BCD996XT/P2 VOL 0–29, SQL 0–19).
        volume_range: Some((0, v.volume_max)),
        squelch_range: Some((0, v.squelch_max)),
        freq_ranges: caps.freq_ranges,
        service_banks: if v.military_air { &BANKS_WITH_MILITARY } else { &BANKS_WITHOUT_MILITARY },
        // SPEC: POF is listed for the handhelds only; the BCD996 specs have no power-off command.
        power_off: v.power_off,
    }
}

/// The scanner's weather scan modes: `(label, JPM index)`.
pub const WEATHER_MODES: [(&str, &str); 8] = [
    ("Normal", "NORMAL"),
    ("Alert only", "A_ONLY"),
    ("SAME 1", "SAME_1"),
    ("SAME 2", "SAME_2"),
    ("SAME 3", "SAME_3"),
    ("SAME 4", "SAME_4"),
    ("SAME 5", "SAME_5"),
    ("All FIPS", "ALL_FIPS"),
];

/// NOAA weather channels, for display only: `(label, MHz)`.
pub const NOAA_CHANNELS: [(&str, f64); 7] = [
    ("WX1", 162.400),
    ("WX2", 162.425),
    ("WX3", 162.450),
    ("WX4", 162.475),
    ("WX5", 162.500),
    ("WX6", 162.525),
    ("WX7", 162.550),
];

/// The NOAA channel label for a frequency in MHz, if it is one of the seven.
pub fn noaa_label(mhz: f64) -> Option<&'static str> {
    NOAA_CHANNELS.iter().find(|(_, f)| (f - mhz).abs() < 0.0005).map(|(l, _)| *l)
}

/// What the scanner is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanMode {
    Scan,
    /// Service search bank (the `JPM` index).
    Service(&'static str),
    Custom,
    CloseCall,
    QuickSearch,
    /// Weather mode (the `JPM` index).
    Weather(&'static str),
    ToneOut,
}

/// The state of the scanner as far as the app can tell. `None` means the model cannot say.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LiveState {
    /// App-tracked: the XT/P2 protocol cannot report the mode, so this is the last one we set.
    pub mode: Option<ScanMode>,
    pub system: Option<String>,
    pub group: Option<String>,
    pub channel: Option<String>,
    pub freq_hz: Option<u64>,
    /// Talkgroup ID when the scanner reports one instead of a frequency.
    pub tgid: Option<String>,
    pub modulation: Option<String>,
    /// CTCSS/DCS code or P25 NAC, as text.
    pub sub_audio: Option<String>,
    pub system_tag: Option<String>,
    pub channel_tag: Option<String>,
    /// Signal strength 0–5.
    pub signal_bars: Option<u8>,
    pub squelch_open: bool,
    pub volume: Option<u8>,
    pub squelch: Option<u8>,
    /// App-tracked: mute is emulated with volume 0.
    pub muted: bool,
}

/// Non-empty trimmed text, or `None`.
fn text(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_owned())
}

impl LiveState {
    /// True when the scanner reports no reception details at all.
    pub fn is_idle(&self) -> bool {
        self.freq_hz.is_none() && self.tgid.is_none() && self.system.is_none() && self.channel.is_none()
    }

    /// The reception frequency in MHz.
    pub fn freq_mhz(&self) -> Option<f64> {
        self.freq_hz.map(|f| f as f64 / 1_000_000.0)
    }

    /// Updates the reception fields from a `GLG` reply (the full line, including `GLG,`).
    pub fn apply_glg(&mut self, reply: &str) -> Result<(), super::Error> {
        let mut f = reply.trim_end().split(',');
        if f.next() != Some("GLG") {
            return Err(super::Error::UnexpectedReply { expected: "GLG", reply: reply.to_owned() });
        }
        // SPEC: the empty reply is printed with different comma counts, so missing fields are empty.
        let mut next = || f.next().unwrap_or("");
        let frq = next().trim().to_owned();
        let modulation = next();
        let _attenuation = next();
        let tone = next();
        let (system, group, channel) = (next(), next(), next());
        let squelch = next();
        let _mute = next(); // the scanner's own mute flag, not the app's emulated mute
        let (system_tag, channel_tag) = (next(), next());
        let nac = next();

        // Observed on hardware: a decimal point means MHz; a bare number of 6+ digits is in 100 Hz
        // units; anything shorter is a talkgroup ID.
        self.freq_hz = None;
        self.tgid = None;
        if frq.contains('.') {
            self.freq_hz = frq.parse::<f64>().ok().map(|m| (m * 1_000_000.0).round() as u64);
        } else if frq.len() >= 6 && frq.bytes().all(|b| b.is_ascii_digit()) {
            self.freq_hz = frq.parse::<u64>().ok().map(|v| v * 100);
        } else if !frq.is_empty() {
            self.tgid = Some(frq);
        }
        self.modulation = text(modulation);
        self.system = text(system);
        self.group = text(group);
        self.channel = text(channel);
        self.squelch_open = squelch.trim() == "1";
        let tag = |s: &str| text(s).filter(|t| t != "NONE");
        self.system_tag = tag(system_tag);
        self.channel_tag = tag(channel_tag);

        // A tone code of 0 means none; a NAC of NONE means none.
        let mut sub = Vec::new();
        if let Some(t) = text(tone).filter(|t| t != "0") {
            sub.push(format!("Tone {t}"));
        }
        if let Some(n) = tag(nac) {
            sub.push(format!("NAC {n}"));
        }
        self.sub_audio = (!sub.is_empty()).then(|| sub.join(" · "));
        Ok(())
    }

    /// Updates the signal meter from a `PWR` reply (RSSI 0–1023).
    pub fn apply_pwr(&mut self, reply: &str) {
        let rssi = reply
            .strip_prefix("PWR,")
            .and_then(|r| r.split(',').next())
            .and_then(|r| r.trim().parse::<u16>().ok());
        self.signal_bars = rssi.map(bars_from_rssi);
    }
}

/// Maps RSSI 0–1023 to 0–5 bars.
pub fn bars_from_rssi(rssi: u16) -> u8 {
    (rssi.min(1023) / 205).min(5) as u8
}

/// Universal UI intents. The driver turns these into wire commands.
#[allow(dead_code)] // PowerOff has no button yet
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Hold,
    ResumeScan,
    StepNext,
    StepPrev,
    Avoid,
    SetVolume(u8),
    SetSquelch(u8),
    /// Handled by the session (it needs the volume to restore); never reaches [`commands`].
    SetMute(bool),
    JumpMode(ScanMode),
    QuickTune { freq_hz: u64, modulation: Option<String> },
    JumpToTag { sys: Option<u16>, chan: Option<u16> },
    PowerOff,
}

impl Action {
    /// Short name for messages.
    pub fn label(&self) -> &'static str {
        match self {
            Action::Hold => "Hold",
            Action::ResumeScan => "Scan",
            Action::StepNext => "Next",
            Action::StepPrev => "Previous",
            Action::Avoid => "Avoid",
            Action::SetVolume(_) => "Volume",
            Action::SetSquelch(_) => "Squelch",
            Action::SetMute(_) => "Mute",
            Action::JumpMode(_) => "Mode change",
            Action::QuickTune { .. } => "Quick tune",
            Action::JumpToTag { .. } => "Jump to tag",
            Action::PowerOff => "Power off",
        }
    }
}

/// The wire commands (in order) that carry out `action` on this model, or a user-facing reason it can't.
pub fn commands(action: &Action, profile: &ControlProfile) -> Result<Vec<String>, String> {
    // SPEC: KEY,<code>,<mode>; H hold, S scan, L lockout, < and > step (VFO left/right).
    let key = |c: char| vec![format!("KEY,{c},P")];
    Ok(match action {
        Action::Hold => key('H'),
        Action::ResumeScan | Action::JumpMode(ScanMode::Scan) => key('S'),
        Action::Avoid => key('L'),
        Action::StepNext => key('>'),
        Action::StepPrev => key('<'),
        Action::SetVolume(v) => {
            let (lo, hi) = profile.volume_range.ok_or("This model has no remote volume.")?;
            vec![format!("VOL,{}", (*v).clamp(lo, hi))]
        }
        Action::SetSquelch(v) => {
            let (lo, hi) = profile.squelch_range.ok_or("This model has no remote squelch.")?;
            vec![format!("SQL,{}", (*v).clamp(lo, hi))]
        }
        Action::SetMute(_) => return Err("Mute is handled by the session.".into()),
        Action::JumpMode(ScanMode::Service(bank)) => vec![format!("JPM,SVC_MODE,{bank}")],
        Action::JumpMode(ScanMode::Custom) => vec!["JPM,CTM_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::CloseCall) => vec!["JPM,CC_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::Weather(mode)) => vec![format!("JPM,WX_MODE,{mode}")],
        Action::JumpMode(ScanMode::ToneOut) => vec!["JPM,FTO_MODE,RESERVE".into()],
        Action::JumpMode(ScanMode::QuickSearch) => return Err("Use Quick Tune to enter Quick Search.".into()),
        Action::QuickTune { freq_hz, modulation } => {
            let wire = u32::try_from(freq_hz / 100).map_err(|_| "Frequency is out of range.")?;
            if !profile.freq_ranges.iter().any(|(lo, hi)| (*lo..=*hi).contains(&wire)) {
                return Err(format!(
                    "{:.4} MHz is outside the {} receive range.",
                    *freq_hz as f64 / 1e6,
                    profile.model
                ));
            }
            // SPEC: QSH,<FRQ>,<RSV>,<MOD>; empty fields keep the scanner's current options.
            let modulation = modulation.as_deref().filter(|m| *m != "AUTO").unwrap_or("");
            vec![format!("QSH,{wire:08},,{modulation}")]
        }
        Action::JumpToTag { sys, chan } => {
            if sys.is_none() && chan.is_none() {
                return Err("Enter a system tag, a channel tag, or both.".into());
            }
            if sys.is_some_and(|t| t > 999) || chan.is_some_and(|t| t > 999) {
                return Err("Number tags are 0–999.".into());
            }
            let field = |t: &Option<u16>| t.map(|t| t.to_string()).unwrap_or_default();
            vec![format!("JNT,{},{}", field(sys), field(chan))]
        }
        Action::PowerOff => {
            if !profile.power_off {
                return Err("This model has no remote power-off command.".into());
            }
            vec!["POF".into()]
        }
    })
}

/// One reception in the app-side activity log.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityEntry {
    /// Wall-clock time the reception started, e.g. `14:02:11`.
    pub clock: String,
    pub system: Option<String>,
    pub group: Option<String>,
    pub channel: Option<String>,
    pub freq_hz: Option<u64>,
    pub tgid: Option<String>,
    /// `None` while the reception is still going.
    pub duration: Option<Duration>,
    pub peak_bars: u8,
    started: Instant,
}

impl ActivityEntry {
    /// True if `s` is the same reception (same place on the same system) as this entry.
    fn same_as(&self, s: &LiveState) -> bool {
        self.channel == s.channel && self.freq_hz == s.freq_hz && self.tgid == s.tgid && self.system == s.system
    }
}

/// The most recent receptions, newest last.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActivityLog {
    pub entries: Vec<ActivityEntry>,
    /// True while the last entry is still open.
    pub(crate) open: bool,
}

/// Entries kept in memory.
const ACTIVITY_LIMIT: usize = 500;

impl ActivityLog {
    /// Feeds one status update in; returns true if the log changed.
    /// A reception starts when squelch opens and ends when it closes or the channel changes.
    pub fn observe(&mut self, s: &LiveState, now: Instant, clock: &str) -> bool {
        let mut changed = false;
        if self.open {
            let last = self.entries.last_mut().expect("open entry");
            if s.squelch_open && last.same_as(s) {
                let bars = s.signal_bars.unwrap_or(0);
                if bars > last.peak_bars {
                    last.peak_bars = bars;
                    changed = true;
                }
                return changed;
            }
            last.duration = Some(now.duration_since(last.started));
            self.open = false;
            changed = true;
        }
        if s.squelch_open {
            if self.entries.len() >= ACTIVITY_LIMIT {
                self.entries.remove(0);
            }
            self.entries.push(ActivityEntry {
                clock: clock.to_owned(),
                system: s.system.clone(),
                group: s.group.clone(),
                channel: s.channel.clone(),
                freq_hz: s.freq_hz,
                tgid: s.tgid.clone(),
                duration: None,
                peak_bars: s.signal_bars.unwrap_or(0),
                started: now,
            });
            self.open = true;
            changed = true;
        }
        changed
    }

    /// The log as CSV (oldest first).
    pub fn to_csv(&self) -> String {
        let q = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        let mut out = String::from("Time,System,Group,Channel,Frequency MHz,TGID,Duration s,Peak bars\n");
        for e in &self.entries {
            out.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                q(&e.clock),
                q(e.system.as_deref().unwrap_or("")),
                q(e.group.as_deref().unwrap_or("")),
                q(e.channel.as_deref().unwrap_or("")),
                e.freq_hz.map(|f| format!("{:.4}", f as f64 / 1e6)).unwrap_or_default(),
                q(e.tgid.as_deref().unwrap_or("")),
                e.duration.map(|d| d.as_secs().to_string()).unwrap_or_default(),
                e.peak_bars,
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    /// The BCD325P2 profile.
    fn profile() -> ControlProfile {
        models::find("BCD325P2").unwrap().control_profile().unwrap()
    }

    /// A reception reply fills names, tags, squelch and the frequency.
    #[test]
    fn parses_glg() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,4625000,FM,0,0,Fire,Dispatch,Main,1,0,12,3,NONE").unwrap();
        assert_eq!((s.system.as_deref(), s.group.as_deref(), s.channel.as_deref()), (Some("Fire"), Some("Dispatch"), Some("Main")));
        assert!(s.squelch_open);
        assert_eq!(s.freq_hz, Some(462_500_000));
        assert_eq!((s.system_tag.as_deref(), s.channel_tag.as_deref()), (Some("12"), Some("3")));
        assert_eq!(s.sub_audio, None);
    }

    /// Tone codes and NAC are combined; a decimal frequency is MHz.
    #[test]
    fn glg_sub_audio_and_decimal_frequency() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,162.5500,NFM,0,23,WX,,,1,0,NONE,NONE,293").unwrap();
        assert_eq!(s.freq_hz, Some(162_550_000));
        assert_eq!(s.sub_audio.as_deref(), Some("Tone 23 · NAC 293"));
    }

    /// Both spellings of the empty reply are idle, and a talkgroup ID is not a frequency.
    #[test]
    fn idle_and_talkgroup() {
        let mut s = LiveState::default();
        s.apply_glg("GLG,,,,,,,,,,,,").unwrap();
        assert!(s.is_idle());
        s.apply_glg("GLG,,,,,,,,,,").unwrap();
        assert!(s.is_idle());
        s.apply_glg("GLG,12345,,,,Sys,Grp,Chan,1,0,,,").unwrap();
        assert_eq!((s.freq_hz, s.tgid.as_deref()), (None, Some("12345")));
    }

    /// Other replies are rejected, and PWR maps to 0–5 bars.
    #[test]
    fn rejects_wrong_reply_and_maps_bars() {
        assert!(LiveState::default().apply_glg("ERR").is_err());
        let mut s = LiveState::default();
        s.apply_pwr("PWR,1023,4625000");
        assert_eq!(s.signal_bars, Some(4));
        s.apply_pwr("PWR,0,0");
        assert_eq!(s.signal_bars, Some(0));
        assert_eq!(bars_from_rssi(5000), 4);
    }

    /// Weather frequencies map to their WX channel and others do not.
    #[test]
    fn noaa_labels() {
        assert_eq!(noaa_label(162.4), Some("WX1"));
        assert_eq!(noaa_label(162.55), Some("WX7"));
        assert_eq!(noaa_label(162.6), None);
    }

    /// Actions translate to the documented wire commands.
    #[test]
    fn translates_actions() {
        let p = profile();
        let one = |a: Action| commands(&a, &p).unwrap();
        assert_eq!(one(Action::Hold), ["KEY,H,P"]);
        assert_eq!(one(Action::StepNext), ["KEY,>,P"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Scan)), ["KEY,S,P"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Service("HAM"))), ["JPM,SVC_MODE,HAM"]);
        assert_eq!(one(Action::JumpMode(ScanMode::Weather("SAME_2"))), ["JPM,WX_MODE,SAME_2"]);
        assert_eq!(one(Action::SetVolume(99)), ["VOL,15"]);
        assert_eq!(one(Action::JumpToTag { sys: Some(12), chan: None }), ["JNT,12,"]);
        assert_eq!(one(Action::JumpToTag { sys: None, chan: Some(3) }), ["JNT,,3"]);
        assert_eq!(
            one(Action::QuickTune { freq_hz: 462_562_500, modulation: Some("AUTO".into()) }),
            ["QSH,04625625,,"]
        );
        assert_eq!(
            one(Action::QuickTune { freq_hz: 162_400_000, modulation: Some("NFM".into()) }),
            ["QSH,01624000,,NFM"]
        );
    }

    /// Bad input is refused with a reason instead of being sent.
    #[test]
    fn refuses_bad_input() {
        let p = profile();
        assert!(commands(&Action::QuickTune { freq_hz: 600_000_000, modulation: None }, &p).is_err());
        assert!(commands(&Action::JumpToTag { sys: None, chan: None }, &p).is_err());
        assert!(commands(&Action::JumpToTag { sys: Some(1000), chan: None }, &p).is_err());
        assert!(commands(&Action::SetMute(true), &p).is_err());
    }

    /// Each model has a profile, and the BC346XT lacks the Military Air bank.
    #[test]
    fn profiles_per_model() {
        for m in models::all() {
            assert!(m.control_profile().is_some(), "{}", m.name());
        }
        let banks = |n: &str| models::find(n).unwrap().control_profile().unwrap().service_banks.len();
        assert_eq!((banks("BCD325P2"), banks("BC346XT")), (12, 11));
        let ranges = |n: &str| models::find(n).unwrap().control_profile().unwrap().volume_range;
        assert_eq!((ranges("BCD396XT"), ranges("BCD996XT")), (Some((0, 15)), Some((0, 29))));
        assert!(commands(&Action::PowerOff, &models::find("BCD996P2").unwrap().control_profile().unwrap()).is_err());
        assert!(matches!(profile().support(Capability::HoldAtLevel), Support::Unsupported(_)));
        assert_eq!(profile().support(Capability::Mute), Support::Emulated);
    }

    /// Receptions are logged on squelch open, closed on squelch close or channel change.
    #[test]
    fn activity_log() {
        let t0 = Instant::now();
        let mut log = ActivityLog::default();
        let mut s = LiveState { squelch_open: true, channel: Some("A".into()), signal_bars: Some(2), ..Default::default() };
        assert!(log.observe(&s, t0, "10:00:00"));
        s.signal_bars = Some(4);
        assert!(log.observe(&s, t0, "10:00:01"));
        assert!(!log.observe(&s, t0, "10:00:02"));
        s.channel = Some("B".into());
        log.observe(&s, t0 + Duration::from_secs(3), "10:00:03");
        assert_eq!(log.entries.len(), 2);
        assert_eq!((log.entries[0].peak_bars, log.entries[0].duration), (4, Some(Duration::from_secs(3))));
        s.squelch_open = false;
        log.observe(&s, t0 + Duration::from_secs(5), "10:00:05");
        assert_eq!(log.entries[1].duration, Some(Duration::from_secs(2)));
        assert!(log.to_csv().lines().count() == 3);
    }
}
