//! Protocol shared by the Uniden BCD/BC "remote command" family: reply parsing,
//! set-command building, and reading/writing a whole database. Differences between
//! models are expressed with [`Dialect`].

use std::fmt;
use std::str::FromStr;

use super::Error;
use crate::database::*;

/// The few places where models in the family differ on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dialect {
    /// TIN carries a trailing TDMA slot field.
    pub tdma_slot: bool,
    /// TFQ carries a trailing color code field.
    pub color_code: bool,
}

impl Dialect {
    /// Models with DMR/MotoTRBO support (BCD325P2).
    pub const DIGITAL_DMR: Dialect = Dialect { tdma_slot: true, color_code: true };
    /// Models without DMR support (BCD396XT).
    pub const NO_DMR: Dialect = Dialect { tdma_slot: false, color_code: false };
}

/// Sequential reader over the comma-separated fields of one reply.
pub struct Fields<'a> {
    cmd: &'static str,
    parts: Vec<&'a str>,
    pos: usize, // next field to read; field 0 is the command name
}

impl<'a> Fields<'a> {
    /// Split a reply and check it starts with `cmd` and is not an error.
    pub fn new(cmd: &'static str, reply: &'a str) -> Result<Self, Error> {
        let reply = reply.trim_end_matches(|c| c == '\r' || c == '\n');
        let parts: Vec<&str> = reply.split(',').collect();
        let is_err = |s: &str| matches!(s, "ERR" | "NG" | "FER" | "ORER");
        // The scanner uses both bare errors and command-prefixed errors.
        if is_err(parts[0]) || (parts.len() == 2 && is_err(parts[1])) {
            return Err(Error::Scanner { cmd: cmd.to_string(), reply: reply.to_string() });
        }
        if parts[0] != cmd {
            return Err(Error::UnexpectedReply { expected: cmd, reply: reply.to_string() });
        }
        Ok(Fields { cmd, parts, pos: 1 })
    }

    /// Next field as raw text; error if the reply ran out.
    fn raw(&mut self) -> Result<&'a str, Error> {
        let pos = self.pos;
        self.pos += 1;
        self.parts.get(pos).copied().ok_or(Error::MissingField { cmd: self.cmd, pos })
    }

    /// Build a field-value error using this reply's command name.
    fn bad(&self, pos: usize, value: &str) -> Error {
        Error::BadValue { cmd: self.cmd, pos, value: value.to_string() }
    }

    /// Skip n fields (reserved fields).
    pub fn skip(&mut self, n: usize) -> Result<(), Error> {
        for _ in 0..n {
            self.raw()?;
        }
        Ok(())
    }

    /// Raw text, possibly empty (used for names).
    pub fn text(&mut self) -> Result<String, Error> {
        Ok(self.raw()?.to_string())
    }

    /// Empty field -> None, otherwise parse with FromStr.
    pub fn opt<T: FromStr>(&mut self) -> Result<Option<T>, Error> {
        let pos = self.pos;
        let s = self.raw()?;
        if s.is_empty() {
            return Ok(None);
        }
        s.parse().map(Some).map_err(|_| self.bad(pos, s))
    }

    /// Like opt, but an empty field is an error.
    pub fn req<T: FromStr>(&mut self) -> Result<T, Error> {
        let pos = self.pos;
        self.opt()?.ok_or(Error::MissingField { cmd: self.cmd, pos })
    }

    /// "0"/"1" flag; empty -> None.
    pub fn flag(&mut self) -> Result<Option<bool>, Error> {
        let pos = self.pos;
        match self.raw()? {
            "" => Ok(None),
            "0" => Ok(Some(false)),
            "1" => Ok(Some(true)),
            other => Err(self.bad(pos, other)),
        }
    }

    /// Index field; "-1" or empty -> None.
    pub fn link(&mut self) -> Result<Option<Index>, Error> {
        let pos = self.pos;
        let s = self.raw()?;
        if s.is_empty() || s == "-1" {
            return Ok(None);
        }
        match s.parse::<u16>() {
            Ok(v) if v > 0 => Ok(Some(Index(v))), // valid indexes start at 1
            _ => Err(self.bad(pos, s)),
        }
    }

    /// Pair of REV/FWD index fields.
    pub fn links(&mut self) -> Result<Links, Error> {
        Ok(Links { rev: self.link()?, fwd: self.link()? })
    }

    /// Latitude, longitude, range, GPS-enable in that order.
    pub fn geofence(&mut self) -> Result<GeoFence, Error> {
        Ok(GeoFence {
            latitude: self.opt()?,
            longitude: self.opt()?,
            range: self.opt()?,
            enabled: self.flag()?,
        })
    }
}

/// Parse a single-index reply such as "SIH,123", "CSY,-1", or "FWD,456".
pub fn parse_index_reply(cmd: &'static str, reply: &str) -> Result<Option<Index>, Error> {
    Fields::new(cmd, reply)?.link()
}

/// Ask the scanner how many memory blocks are still free (RMB).
pub fn read_free_blocks<S: Scanner>(sc: &mut S) -> Result<usize, Error> {
    let reply = sc.send("RMB")?;
    let value = Fields::new("RMB", &reply)?.raw()?.to_string();
    value
        .parse()
        .map_err(|_| Error::BadValue { cmd: "RMB", pos: 1, value })
}

/// Read the scanner's own memory statistics. Must be called in Program Mode.
pub fn read_scanner_memory<S: Scanner>(sc: &mut S) -> Result<ScannerMemory, Error> {
    let free_blocks = read_free_blocks(sc)?;
    let reply = sc.send("MEM")?;
    let mut f = Fields::new("MEM", &reply)?;
    Ok(ScannerMemory {
        free_blocks,
        percent_used: f.req()?,
        systems: f.req()?,
        sites: f.req()?,
        channels: f.req()?,
        location_alerts: f.req()?,
    })
}

/// Check a "CMD,OK" reply.
pub fn expect_ok(cmd: &'static str, reply: &str) -> Result<(), Error> {
    match Fields::new(cmd, reply)?.raw()? {
        "OK" => Ok(()),
        _ => Err(Error::UnexpectedReply { expected: cmd, reply: reply.to_string() }),
    }
}

impl SystemRecord {
    /// Parse a SIN reply in the field order defined by the scanner protocol.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("SIN", reply)?;
        let sys_type = f.req()?; // 1
        let name = Name(f.text()?); // 2
        let quick_key = f.opt()?; // 3
        let hold_time = f.opt()?; // 4
        let lockout = f.flag()?; // 5
        let delay = f.opt()?; // 6
        f.skip(5)?; // 7-11 reserved
        let links = f.links()?; // 12-13
        let child_head = f.link()?; // 14
        let child_tail = f.link()?; // 15
        let seq_no = f.opt()?; // 16
        let start_key = f.opt()?; // 17
        f.skip(5)?; // 18-22 reserved
        Ok(SystemRecord {
            sys_type,
            name,
            quick_key,
            hold_time,
            lockout,
            delay,
            links,
            child_head,
            child_tail,
            seq_no,
            start_key,
            number_tag: f.opt()?,     // 23
            agc_analog: f.flag()?,    // 24
            agc_digital: f.flag()?,   // 25
            p25_waiting_ms: f.opt()?, // 26
            protect: f.flag()?,       // 27 (28 is reserved, ignored)
        })
    }
}

impl TrunkRecord {
    /// Parse a TRN reply, including the group-list heads and tails at the end.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("TRN", reply)?;
        let id_search = f.flag()?; // 1
        let status_bit = f.flag()?; // 2
        let end_code = f.opt()?; // 3
        let afs = f.flag()?; // 4
        f.skip(2)?; // 5-6 reserved
        let emergency_alert = f.opt()?; // 7
        let emergency_level = f.opt()?; // 8
        let fleet_map = f.opt()?; // 9
        let custom_fleet_map = Some(f.text()?).filter(|s| !s.is_empty()); // 10
        f.skip(10)?; // 11-20 reserved
        Ok(TrunkRecord {
            id_search,
            status_bit,
            end_code,
            afs,
            emergency_alert,
            emergency_level,
            fleet_map,
            custom_fleet_map,
            tgid_group_head: f.link()?,    // 21
            tgid_group_tail: f.link()?,    // 22
            lockout_group_head: f.link()?, // 23
            lockout_group_tail: f.link()?, // 24
            hex_ids: f.flag()?,            // 25
            emergency_color: f.opt()?,     // 26
            emergency_pattern: f.opt()?,   // 27
            nac: f.opt()?,                 // 28
            priority_id_scan: f.flag()?,   // 29
        })
    }
}

impl SiteRecord {
    /// Parse a SIF site reply, including its linked frequency list and location settings.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("SIF", reply)?;
        f.skip(1)?; // 1 reserved
        let name = Name(f.text()?); // 2
        let quick_key = f.opt()?; // 3
        let hold_time = f.opt()?; // 4
        let lockout = f.flag()?; // 5
        let modulation = f.opt()?; // 6
        let attenuator = f.flag()?; // 7
        let control_channel_only = f.flag()?; // 8
        f.skip(2)?; // 9-10 reserved
        let links = f.links()?; // 11-12
        let system = f.link()?; // 13
        let freq_head = f.link()?; // 14
        let freq_tail = f.link()?; // 15
        let seq_no = f.opt()?; // 16
        let start_key = f.opt()?; // 17
        let geofence = f.geofence()?; // 18-21
        f.skip(1)?; // 22 reserved
        Ok(SiteRecord {
            name,
            quick_key,
            hold_time,
            lockout,
            modulation,
            attenuator,
            control_channel_only,
            links,
            system,
            freq_head,
            freq_tail,
            seq_no,
            start_key,
            geofence,
            mot_band_type: f.opt()?,  // 23
            edacs_type: f.opt()?,     // 24
            p25_waiting_ms: f.opt()?, // 25 (26 reserved, ignored)
        })
    }
}

impl MotBandPlan {
    /// Parse the six four-field Motorola band-plan entries returned by MCP.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("MCP", reply)?;
        let mut bands = [None; 6];
        for band in bands.iter_mut() {
            // Four fields per entry: lower, upper, step, offset
            let parts = (f.opt()?, f.opt()?, f.opt()?, f.opt()?);
            if let (Some(lower), Some(upper), Some(step_code), Some(offset)) = parts {
                *band = Some(MotBand { lower, upper, step_code, offset });
            }
        }
        Ok(MotBandPlan { bands })
    }
}

impl P25BandPlan {
    /// Parse the sixteen hexadecimal base/spacing pairs returned by ABP.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("ABP", reply)?;
        let mut bands = [None; 16];
        for (i, band) in bands.iter_mut().enumerate() {
            let pos = 1 + i * 2; // base field position for error reports
            let base = f.text()?;
            let spacing = f.text()?;
            if base.is_empty() || base == "0" {
                continue; // unused entry, or hidden by the protect bit
            }
            let base = u64::from_str_radix(&base, 16).map_err(|_| f.bad(pos, &base))?;
            let spacing = u32::from_str_radix(&spacing, 16).map_err(|_| f.bad(pos + 1, &spacing))?;
            *band = Some(P25Band { base_hz: base * 5, spacing_hz: spacing * 125 });
        }
        Ok(P25BandPlan { bands })
    }
}

impl TrunkFreqRecord {
    /// Parse a TFQ reply, reading its final color-code field only when the model has it.
    pub fn parse(reply: &str, d: Dialect) -> Result<Self, Error> {
        let mut f = Fields::new("TFQ", reply)?;
        // The 396XT reply ends before the DMR-only color-code field.
        let freq = f.opt()?; // 1
        let lcn = f.opt()?; // 2
        let lockout = f.flag()?; // 3
        let links = f.links()?; // 4-5
        let system = f.link()?; // 6
        let site = f.link()?; // 7
        f.skip(1)?; // 8 reserved
        let number_tag = f.opt()?; // 9
        let vol_offset = f.opt()?; // 10
        f.skip(1)?; // 11 reserved (spec omits the comma after it; see section 11)
        Ok(TrunkFreqRecord {
            freq,
            lcn,
            lockout,
            links,
            system,
            site,
            number_tag,
            vol_offset,
            color_code: if d.color_code { f.opt()? } else { None }, // 12
        })
    }
}

impl GroupRecord {
    /// Parse a GIN reply for either a channel group or a talkgroup group.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("GIN", reply)?;
        Ok(GroupRecord {
            group_type: f.req()?,  // 1
            name: Name(f.text()?), // 2
            quick_key: f.opt()?,   // 3
            lockout: f.flag()?,    // 4
            links: f.links()?,     // 5-6
            system: f.link()?,     // 7
            child_head: f.link()?, // 8
            child_tail: f.link()?, // 9
            seq_no: f.opt()?,      // 10
            geofence: f.geofence()?, // 11-14
        })
    }
}

impl ChannelRecord {
    /// Parse the CIN reply for one conventional channel.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("CIN", reply)?;
        let name = Name(f.text()?); // 1
        let freq = f.opt()?; // 2
        let modulation = f.opt()?; // 3
        let tone = f.opt()?; // 4
        let tone_lockout = f.flag()?; // 5
        let lockout = f.flag()?; // 6
        let priority = f.flag()?; // 7
        let attenuator = f.flag()?; // 8
        let alert_tone = f.opt()?; // 9
        let alert_level = f.opt()?; // 10
        let links = f.links()?; // 11-12
        let system = f.link()?; // 13
        let group = f.link()?; // 14
        f.skip(1)?; // 15 reserved
        Ok(ChannelRecord {
            name,
            freq,
            modulation,
            tone,
            tone_lockout,
            lockout,
            priority,
            attenuator,
            alert_tone,
            alert_level,
            links,
            system,
            group,
            audio_type: f.opt()?,    // 16
            nac: f.opt()?,           // 17
            number_tag: f.opt()?,    // 18
            alert_color: f.opt()?,   // 19
            alert_pattern: f.opt()?, // 20
            vol_offset: f.opt()?,    // 21
        })
    }
}

impl TgidRecord {
    /// Parse a TIN reply, accounting for whether this model sends the TDMA slot field.
    pub fn parse(reply: &str, d: Dialect) -> Result<Self, Error> {
        let mut f = Fields::new("TIN", reply)?;
        // The 396XT reply has no TDMA slot at the end.
        let name = Name(f.text()?); // 1
        let tgid = f.opt()?; // 2
        let lockout = f.flag()?; // 3
        let priority = f.flag()?; // 4
        let alert_tone = f.opt()?; // 5
        let alert_level = f.opt()?; // 6
        let links = f.links()?; // 7-8
        let system = f.link()?; // 9
        let group = f.link()?; // 10
        f.skip(1)?; // 11 reserved
        Ok(TgidRecord {
            name,
            tgid,
            lockout,
            priority,
            alert_tone,
            alert_level,
            links,
            system,
            group,
            audio_type: f.opt()?,    // 12
            number_tag: f.opt()?,    // 13
            alert_color: f.opt()?,   // 14
            alert_pattern: f.opt()?, // 15
            vol_offset: f.opt()?,    // 16
            tdma_slot: if d.tdma_slot { f.opt()? } else { None }, // 17
        })
    }
}

impl LocationAlertRecord {
    /// Parse a LIN location-alert reply and its geofence coordinates.
    pub fn parse(reply: &str) -> Result<Self, Error> {
        let mut f = Fields::new("LIN", reply)?;
        Ok(LocationAlertRecord {
            alert_type: f.req()?,  // 1
            name: Name(f.text()?), // 2
            lockout: f.flag()?,    // 3
            alert_tone: f.opt()?,  // 4
            alert_level: f.opt()?, // 5
            links: f.links()?,     // 6-7
            seq_no: f.opt()?,      // 8
            geofence: GeoFence {
                latitude: f.opt()?,  // 9
                longitude: f.opt()?, // 10
                range: f.opt()?,     // 11
                enabled: None,       // LIN has no GPS-enable field
            },
            speed_limit: f.opt()?,   // 12
            heading: f.opt()?,       // 13
            alert_color: f.opt()?,   // 14
            alert_pattern: f.opt()?, // 15
        })
    }
}

/// Transport to the scanner. Implementations send `cmd` followed by "\r"
/// and return the reply line (without needing to strip the "\r").
pub trait Scanner {
    /// Send one command and return its reply line.
    fn send(&mut self, cmd: &str) -> Result<String, Error>;
}

/// Run `f` in Program Mode, and always try to exit Program Mode afterward.
pub fn with_program_mode<S: Scanner, T>(
    sc: &mut S,
    f: impl FnOnce(&mut S) -> Result<T, Error>,
) -> Result<T, Error> {
    // Program Mode brackets database commands; try EPG even if the operation fails.
    expect_ok("PRG", &sc.send("PRG")?)?;
    let result = f(sc);
    let exit = sc.send("EPG").and_then(|r| expect_ok("EPG", &r));
    match (result, exit) {
        (Ok(v), Ok(())) => Ok(v),
        (Err(e), _) => Err(e), // report the original failure, not the EPG one
        (Ok(_), Err(e)) => Err(e),
    }
}

/// Follow a linked list from `head`. `read` fetches one record and returns
/// it with its FWD index.
fn walk_list<S: Scanner, T>(
    sc: &mut S,
    head: Option<Index>,
    mut read: impl FnMut(&mut S, Index) -> Result<(T, Option<Index>), Error>,
) -> Result<Vec<(Index, T)>, Error> {
    let mut out = Vec::new();
    let mut next = head;
    while let Some(idx) = next {
        if out.len() >= MAX_BLOCKS {
            return Err(Error::LinkLoop(idx)); // more records than memory blocks: a loop
        }
        let (item, fwd) = read(sc, idx)?;
        out.push((idx, item));
        // Follow FWD rather than assuming list entries have adjacent indices.
        next = fwd;
    }
    Ok(out)
}

/// Read one GIN record and return the forward link used to continue its list.
fn read_group<S: Scanner>(sc: &mut S, idx: Index) -> Result<(GroupRecord, Option<Index>), Error> {
    let rec = GroupRecord::parse(&sc.send(&format!("GIN,{idx}"))?)?;
    let fwd = rec.links.fwd;
    Ok((rec, fwd))
}

/// Read the whole scan database. Must be called in Program Mode,
/// e.g. `with_program_mode(&mut port, read_database)`.
pub fn read_database<S: Scanner>(sc: &mut S, d: Dialect) -> Result<ScanDatabase, Error> {
    read_database_with_progress(sc, d, |_, _| {})
}

/// Like [`read_database`], but calls `progress(done, total)` once the system
/// list is known and again after each system has been read.
pub fn read_database_with_progress<S: Scanner>(
    sc: &mut S,
    d: Dialect,
    mut progress: impl FnMut(usize, usize),
) -> Result<ScanDatabase, Error> {
    let head = parse_index_reply("SIH", &sc.send("SIH")?)?;
    let records = walk_list(sc, head, |sc, idx| {
        let rec = SystemRecord::parse(&sc.send(&format!("SIN,{idx}"))?)?;
        let fwd = rec.links.fwd;
        Ok((rec, fwd))
    })?;

    let total = records.len();
    progress(0, total);
    let mut systems = Vec::with_capacity(total);
    // Headers identify each system and its child list; read each system's contents afterward.
    for (index, info) in records {
        let kind = if info.sys_type.is_trunked() {
            read_trunked(sc, d, index, &info)?
        } else {
            SystemKind::Conventional { groups: read_channel_groups(sc, info.child_head)? }
        };
        systems.push(System { index, info, kind });
        progress(systems.len(), total);
    }
    Ok(ScanDatabase { model: None, systems })
}

/// Conventional system: groups, then channels in each group.
fn read_channel_groups<S: Scanner>(sc: &mut S, head: Option<Index>) -> Result<Vec<ChannelGroup>, Error> {
    let mut out = Vec::new();
    for (index, info) in walk_list(sc, head, |sc, i| read_group(sc, i))? {
        // A conventional group owns a separate linked list of channels.
        let channels = walk_list(sc, info.child_head, |sc, i| {
            let rec = ChannelRecord::parse(&sc.send(&format!("CIN,{i}"))?)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| Channel { index, info })
        .collect();
        out.push(ChannelGroup { index, info, channels });
    }
    Ok(out)
}

/// Trunked system: TRN, sites (with band plans and frequencies), then TGID groups.
fn read_trunked<S: Scanner>(sc: &mut S, d: Dialect, sys: Index, info: &SystemRecord) -> Result<SystemKind, Error> {
    let trunk = TrunkRecord::parse(&sc.send(&format!("TRN,{sys}"))?)?;

    let site_records = walk_list(sc, info.child_head, |sc, i| {
        let rec = SiteRecord::parse(&sc.send(&format!("SIF,{i}"))?)?;
        let fwd = rec.links.fwd;
        Ok((rec, fwd))
    })?;
    let mut sites = Vec::with_capacity(site_records.len());
    for (index, site) in site_records {
        // P25 standard systems use ABP; Motorola sites use MCP only for custom band plans.
        let band_plan = if info.sys_type == SystemType::P25Standard {
            Some(BandPlan::P25(P25BandPlan::parse(&sc.send(&format!("ABP,{index}"))?)?))
        } else if site.mot_band_type == Some(MotBandType::Custom) {
            Some(BandPlan::Motorola(MotBandPlan::parse(&sc.send(&format!("MCP,{index}"))?)?))
        } else {
            None
        };
        let frequencies = walk_list(sc, site.freq_head, |sc, i| {
            let rec = TrunkFreqRecord::parse(&sc.send(&format!("TFQ,{i}"))?, d)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| TrunkFreq { index, info })
        .collect();
        sites.push(Site { index, info: site, band_plan, frequencies });
    }

    let mut tgid_groups = Vec::new();
    for (index, info) in walk_list(sc, trunk.tgid_group_head, |sc, i| read_group(sc, i))? {
        // Talkgroups form a separate linked list under each talkgroup group.
        let tgids = walk_list(sc, info.child_head, |sc, i| {
            let rec = TgidRecord::parse(&sc.send(&format!("TIN,{i}"))?, d)?;
            let fwd = rec.links.fwd;
            Ok((rec, fwd))
        })?
        .into_iter()
        .map(|(index, info)| TgidEntry { index, info })
        .collect();
        tgid_groups.push(TgidGroup { index, info, tgids });
    }

    Ok(SystemKind::Trunked { trunk, sites, tgid_groups })
}

/// Builder for comma-separated set commands.
pub struct SetCmd(Vec<String>);

impl SetCmd {
    /// Start a command with its protocol name as the first field.
    pub fn new(cmd: &str) -> Self {
        SetCmd(vec![cmd.to_string()])
    }
    /// A present value.
    pub fn val(mut self, v: impl fmt::Display) -> Self {
        self.0.push(v.to_string());
        self
    }
    /// Optional value; None -> empty field (leave unchanged).
    pub fn opt<T: fmt::Display>(mut self, v: &Option<T>) -> Self {
        self.0.push(v.as_ref().map(|x| x.to_string()).unwrap_or_default());
        self
    }
    /// Optional 0/1 flag.
    pub fn flag(mut self, v: Option<bool>) -> Self {
        let s = match v {
            Some(true) => "1",
            Some(false) => "0",
            None => "",
        };
        self.0.push(s.to_string());
        self
    }
    /// n reserved (always empty) fields.
    pub fn rsv(mut self, n: usize) -> Self {
        self.0.extend(std::iter::repeat(String::new()).take(n));
        self
    }
    /// Optional latitude/longitude/range/enable quartet.
    pub fn geofence(self, g: &GeoFence) -> Self {
        self.opt(&g.latitude).opt(&g.longitude).opt(&g.range).flag(g.enabled)
    }
    /// Join the fields with commas while retaining empty placeholders.
    pub fn build(self) -> String {
        self.0.join(",")
    }
}

/// Reserved fields between START_KEY and NUMBER_TAG in the SIN *set* layout.
/// The spec shows 6 here but 5 in the get reply; change this if testing shows 5.
pub const SIN_SET_RSV_AFTER_START_KEY: usize = 6;

impl SystemRecord {
    /// SIN set: index, name, qk, hold, lockout, delay, rsv x5, start key,
    /// rsv x6 (see constant), number tag, AGC analog, AGC digital, P25 waiting.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("SIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.quick_key)
            .opt(&self.hold_time)
            .flag(self.lockout)
            .opt(&self.delay)
            .rsv(5)
            .opt(&self.start_key)
            .rsv(SIN_SET_RSV_AFTER_START_KEY)
            .opt(&self.number_tag)
            .flag(self.agc_analog)
            .flag(self.agc_digital)
            .opt(&self.p25_waiting_ms)
            .build()
    }
}

impl GroupRecord {
    /// GIN set: index, name, qk, lockout, lat, lon, range, GPS enable.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("GIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.quick_key)
            .flag(self.lockout)
            .geofence(&self.geofence)
            .build()
    }
}

impl ChannelRecord {
    /// CIN set: the get layout minus the four index fields, plus the index up front.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("CIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.freq)
            .opt(&self.modulation)
            .opt(&self.tone)
            .flag(self.tone_lockout)
            .flag(self.lockout)
            .flag(self.priority)
            .flag(self.attenuator)
            .opt(&self.alert_tone)
            .opt(&self.alert_level)
            .rsv(1)
            .opt(&self.audio_type)
            .opt(&self.nac)
            .opt(&self.number_tag)
            .opt(&self.alert_color)
            .opt(&self.alert_pattern)
            .opt(&self.vol_offset)
            .build()
    }
}

impl TgidRecord {
    /// TIN set: index, name, TGID, lockout, priority, alert tone/level, rsv,
    /// audio type, number tag, alert color/pattern, volume offset, TDMA slot.
    pub fn to_set(&self, idx: Index, d: Dialect) -> String {
        let cmd = SetCmd::new("TIN")
            .val(idx)
            .val(&self.name)
            .opt(&self.tgid)
            .flag(self.lockout)
            .flag(self.priority)
            .opt(&self.alert_tone)
            .opt(&self.alert_level)
            .rsv(1)
            .opt(&self.audio_type)
            .opt(&self.number_tag)
            .opt(&self.alert_color)
            .opt(&self.alert_pattern)
            .opt(&self.vol_offset);
        // Do not append an empty placeholder on models whose TIN format ends earlier.
        if d.tdma_slot { cmd.opt(&self.tdma_slot) } else { cmd }.build()
    }
}

impl TrunkRecord {
    /// TRN set: index, id search, status bit, end code, AFS, rsv x2, emergency alert/level,
    /// fleet map, custom fleet map, rsv x10, hex IDs, emergency color/pattern, NAC, priority ID scan.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("TRN")
            .val(idx)
            .flag(self.id_search)
            .flag(self.status_bit)
            .opt(&self.end_code)
            .flag(self.afs)
            .rsv(2)
            .opt(&self.emergency_alert)
            .opt(&self.emergency_level)
            .opt(&self.fleet_map)
            .opt(&self.custom_fleet_map)
            .rsv(10)
            .flag(self.hex_ids)
            .opt(&self.emergency_color)
            .opt(&self.emergency_pattern)
            .opt(&self.nac)
            .flag(self.priority_id_scan)
            .build()
    }
}

impl SiteRecord {
    /// SIF set: index, name, qk, hold, lockout, modulation, attenuator, control channel only,
    /// rsv x2, start key, geofence, rsv, Motorola band type, EDACS type, P25 waiting, rsv.
    pub fn to_set(&self, idx: Index) -> String {
        SetCmd::new("SIF")
            .val(idx)
            .val(&self.name)
            .opt(&self.quick_key)
            .opt(&self.hold_time)
            .flag(self.lockout)
            .opt(&self.modulation)
            .flag(self.attenuator)
            .flag(self.control_channel_only)
            .rsv(2)
            .opt(&self.start_key)
            .geofence(&self.geofence)
            .rsv(1)
            .opt(&self.mot_band_type)
            .opt(&self.edacs_type)
            .opt(&self.p25_waiting_ms)
            .rsv(1)
            .build()
    }
}

impl TrunkFreqRecord {
    /// TFQ set: index, frequency, LCN, lockout, rsv, number tag, volume offset, rsv, color code.
    pub fn to_set(&self, idx: Index, d: Dialect) -> String {
        let cmd = SetCmd::new("TFQ")
            .val(idx)
            .opt(&self.freq)
            .opt(&self.lcn)
            .flag(self.lockout)
            .rsv(1)
            .opt(&self.number_tag)
            .opt(&self.vol_offset)
            .rsv(1);
        // Do not append a color-code placeholder for models whose TFQ format ends earlier.
        if d.color_code { cmd.opt(&self.color_code) } else { cmd }.build()
    }
}

impl MotBandPlan {
    /// MCP set: site index, then lower/upper/step/offset for each of the six entries.
    /// Unused entries are sent empty, which leaves them unchanged.
    pub fn to_set(&self, site: Index) -> String {
        let mut cmd = SetCmd::new("MCP").val(site);
        for band in &self.bands {
            cmd = match band {
                Some(b) => cmd.val(b.lower).val(b.upper).val(b.step_code).val(b.offset),
                None => cmd.rsv(4),
            };
        }
        cmd.build()
    }
}

impl P25BandPlan {
    /// ABP set: site index, then hex base and spacing for each of the sixteen entries.
    /// Unused entries are sent empty, which leaves them unchanged.
    pub fn to_set(&self, site: Index) -> String {
        let mut cmd = SetCmd::new("ABP").val(site);
        for band in &self.bands {
            cmd = match band {
                Some(b) => cmd.val(format!("{:X}", b.base_hz / 5)).val(format!("{:X}", b.spacing_hz / 125)),
                None => cmd.rsv(2),
            };
        }
        cmd.build()
    }
}

/// Run a create/append command and return the new index (-1 -> OutOfMemory).
fn alloc<S: Scanner>(sc: &mut S, cmd: &'static str, line: &str) -> Result<Index, Error> {
    parse_index_reply(cmd, &sc.send(line)?)?.ok_or(Error::OutOfMemory(cmd))
}

/// Recreate a system on the scanner, conventional or trunked. Call in Program Mode.
/// Returns the new system index.
pub fn write_system<S: Scanner>(sc: &mut S, d: Dialect, sys: &System) -> Result<Index, Error> {
    // The protect bit can only be set at creation time
    let protect = if sys.info.protect == Some(true) { 1 } else { 0 };
    let new_sys = alloc(sc, "CSY", &format!("CSY,{},{protect}", sys.info.sys_type))?;
    expect_ok("SIN", &sc.send(&sys.info.to_set(new_sys))?)?;

    match &sys.kind {
        SystemKind::Conventional { groups } => {
            // Create each parent before its children so new links target the right records.
            for group in groups {
                let new_grp = alloc(sc, "AGC", &format!("AGC,{new_sys}"))?;
                expect_ok("GIN", &sc.send(&group.info.to_set(new_grp))?)?;
                for ch in &group.channels {
                    let new_ch = alloc(sc, "ACC", &format!("ACC,{new_grp}"))?;
                    expect_ok("CIN", &sc.send(&ch.info.to_set(new_ch))?)?;
                }
            }
        }
        SystemKind::Trunked { trunk, sites, tgid_groups } => {
            expect_ok("TRN", &sc.send(&trunk.to_set(new_sys))?)?;
            // Build the parent site before adding its frequencies.
            for site in sites {
                let new_site = alloc(sc, "AST", &format!("AST,{new_sys},"))?;
                // The band type must be set before a custom band plan is accepted.
                expect_ok("SIF", &sc.send(&site.info.to_set(new_site))?)?;
                match &site.band_plan {
                    Some(BandPlan::Motorola(plan)) => expect_ok("MCP", &sc.send(&plan.to_set(new_site))?)?,
                    Some(BandPlan::P25(plan)) => expect_ok("ABP", &sc.send(&plan.to_set(new_site))?)?,
                    None => {}
                }
                for freq in &site.frequencies {
                    let new_freq = alloc(sc, "ACC", &format!("ACC,{new_site}"))?;
                    expect_ok("TFQ", &sc.send(&freq.info.to_set(new_freq, d))?)?;
                }
            }
            // These groups are parent records too, so create them before their talkgroups.
            for group in tgid_groups {
                let new_grp = alloc(sc, "AGT", &format!("AGT,{new_sys}"))?;
                expect_ok("GIN", &sc.send(&group.info.to_set(new_grp))?)?;
                for tgid in &group.tgids {
                    let new_tgid = alloc(sc, "ACT", &format!("ACT,{new_grp}"))?;
                    expect_ok("TIN", &sc.send(&tgid.info.to_set(new_tgid, d))?)?;
                }
            }
        }
    }
    Ok(new_sys)
}

/// Recreate a conventional system on the scanner. Call in Program Mode.
/// Returns the new system index.
pub fn write_conventional<S: Scanner>(sc: &mut S, d: Dialect, sys: &System) -> Result<Index, Error> {
    match sys.kind {
        SystemKind::Conventional { .. } => write_system(sc, d, sys),
        SystemKind::Trunked { .. } => Err(Error::WrongSystemKind),
    }
}

/// Delete every system (and everything under it) from the scanner. Call in Program Mode.
pub fn delete_all_systems<S: Scanner>(sc: &mut S) -> Result<(), Error> {
    let head = parse_index_reply("SIH", &sc.send("SIH")?)?;
    // Deleting a top-level system also removes all records attached beneath it.
    let systems = walk_list(sc, head, |sc, idx| {
        let rec = SystemRecord::parse(&sc.send(&format!("SIN,{idx}"))?)?;
        let fwd = rec.links.fwd;
        Ok(((), fwd))
    })?;
    for (idx, ()) in systems {
        expect_ok("DSY", &sc.send(&format!("DSY,{idx}"))?)?;
    }
    Ok(())
}

/// Replace the scanner's systems with `db`: deletes every existing system, then writes each
/// system in order. Call in Program Mode. Calls `progress(done, total)` before the first
/// system is written and after each one. Settings outside the scan database are untouched.
///
/// The old systems are gone once this starts, so a failure part-way leaves the scanner
/// with only the systems written so far.
pub fn write_database_with_progress<S: Scanner>(
    sc: &mut S,
    d: Dialect,
    db: &ScanDatabase,
    mut progress: impl FnMut(usize, usize),
) -> Result<(), Error> {
    // Refuse before deleting anything if it cannot possibly fit.
    if db.blocks_used() > MAX_BLOCKS {
        return Err(Error::OutOfMemory("upload"));
    }
    delete_all_systems(sc)?;
    let total = db.systems.len();
    progress(0, total);
    for (i, sys) in db.systems.iter().enumerate() {
        write_system(sc, d, sys)?;
        progress(i + 1, total);
    }
    Ok(())
}

/// Like [`write_database_with_progress`], without progress reporting.
pub fn write_database<S: Scanner>(sc: &mut S, d: Dialect, db: &ScanDatabase) -> Result<(), Error> {
    write_database_with_progress(sc, d, db, |_, _| {})
}

/// Shared [`ScannerModel::download`](super::ScannerModel::download) for the family.
pub fn download(
    link: &mut dyn Scanner,
    d: Dialect,
    model: &str,
    progress: super::Progress,
) -> Result<(ScanDatabase, Option<ScannerMemory>), Error> {
    with_program_mode(&mut super::DynLink(link), |sc| {
        let mut db = read_database_with_progress(sc, d, progress)?;
        db.model = Some(model.to_owned());
        // The cross-check is optional: a failed RMB/MEM must not fail the download.
        Ok((db, read_scanner_memory(sc).ok()))
    })
}

/// Shared [`ScannerModel::upload`](super::ScannerModel::upload) for the family.
pub fn upload(link: &mut dyn Scanner, d: Dialect, db: &ScanDatabase, progress: super::Progress) -> Result<(), Error> {
    with_program_mode(&mut super::DynLink(link), |sc| write_database_with_progress(sc, d, db, progress))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Fake scanner: canned replies keyed by command; unknown commands get ERR.
    struct Mock(HashMap<&'static str, &'static str>);

    impl Scanner for Mock {
        fn send(&mut self, cmd: &str) -> Result<String, Error> {
            Ok(self.0.get(cmd).copied().unwrap_or("ERR").to_string())
        }
    }

    /// Records every command; hands out increasing indices for create commands.
    struct Recorder {
        sent: Vec<String>,
        next: u32,
    }

    impl Scanner for Recorder {
        fn send(&mut self, cmd: &str) -> Result<String, Error> {
            self.sent.push(cmd.to_string());
            let name = cmd.split(',').next().unwrap();
            Ok(match name {
                "SIH" => "SIH,-1".to_string(),
                "CSY" | "AGC" | "AGT" | "AST" | "ACC" | "ACT" => {
                    self.next += 1;
                    format!("{name},{}", self.next)
                }
                _ => format!("{name},OK"),
            })
        }
    }

    #[test]
    /// Checks an upload writes the expected systems and reports final progress.
    fn write_database_creates_every_system() {
        let db = ScanDatabase::new_placeholder();
        let mut sc = Recorder { sent: vec![], next: 0 };
        let mut last = (0, 0);
        write_database_with_progress(&mut sc, Dialect::DIGITAL_DMR, &db, |d, t| last = (d, t)).unwrap();
        assert_eq!(last, (db.systems.len(), db.systems.len()));
        assert_eq!(sc.sent[0], "SIH");
        assert!(sc.sent.iter().any(|c| c.starts_with("CSY,")));
        assert!(sc.sent.iter().any(|c| c.starts_with("SIN,")));
        assert!(sc.sent.iter().any(|c| c.starts_with("CIN,")));
    }

    #[test]
    /// Checks quick-key uniqueness across systems and groups.
    fn validate_duplicate_keys() {
        let mut db = ScanDatabase::new_placeholder();
        db.systems[0].info.name = Name("System".to_string());
        if let SystemKind::Conventional { groups } = &mut db.systems[0].kind {
            groups[0].info.name = Name("Group".to_string());
            groups[0].channels[0].info.name = Name("Channel".to_string());
        }
        assert!(db.validate().is_empty());
        let mut second = db.systems[0].clone();
        db.systems[0].info.quick_key = Some(KeyAssignment::Key(1));
        db.systems[0].info.start_key = Some(KeyAssignment::Key(2));
        second.info.quick_key = Some(KeyAssignment::Key(1));
        second.info.start_key = Some(KeyAssignment::Key(2));
        db.systems.push(second);
        assert_eq!(db.validate().len(), 2);
        db.systems[1].info.quick_key = Some(KeyAssignment::Unassigned);
        db.systems[1].info.start_key = None;
        assert!(db.validate().is_empty());
        if let SystemKind::Conventional { groups } = &mut db.systems[0].kind {
            let mut g = groups[0].clone();
            groups[0].info.quick_key = Some(KeyAssignment::Key(3));
            g.info.quick_key = Some(KeyAssignment::Key(3));
            groups.push(g);
        }
        assert_eq!(
            db.validate(),
            vec![ValidationError::DuplicateGroupQuickKey { system: 0, key: 3, first: 0, duplicate: 1 }]
        );
    }

    #[test]
    /// Checks name validation covers conventional and trunked records.
    fn validate_all_names_and_include_invalid_name_in_error() {
        let mut db = ScanDatabase::new_placeholder();
        db.systems[0].info.name = Name("Bad_System".to_string());
        if let SystemKind::Conventional { groups } = &mut db.systems[0].kind {
            groups[0].info.name = Name("Bad_Group".to_string());
            groups[0].channels[0].info.name = Name("Bad_Channel".to_string());
        }

        let mut trunked = db.systems[0].clone();
        trunked.info.name = Name("Trunked".to_string());
        trunked.kind = SystemKind::Trunked {
            trunk: TrunkRecord::default(),
            sites: vec![Site {
                index: Index::default(),
                info: SiteRecord { name: Name("Bad_Site".to_string()), ..Default::default() },
                band_plan: None,
                frequencies: Vec::new(),
            }],
            tgid_groups: vec![TgidGroup {
                index: Index::default(),
                info: GroupRecord { name: Name("Bad_TGID_Group".to_string()), ..Default::default() },
                tgids: vec![TgidEntry {
                    index: Index::default(),
                    info: TgidRecord { name: Name("Bad_Talkgroup".to_string()), ..Default::default() },
                }],
            }],
        };
        db.systems.push(trunked);

        let errors = db.validate();
        assert_eq!(errors.len(), 6);
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        for invalid_name in [
            "Bad_System",
            "Bad_Group",
            "Bad_Channel",
            "Bad_Site",
            "Bad_TGID_Group",
            "Bad_Talkgroup",
        ] {
            assert!(messages.iter().any(|message| message.contains(invalid_name)));
        }
    }

    #[test]
    /// Checks frequency values round-trip through the scanner wire format.
    fn frequency_format() {
        // Spec example: 08510125 = 851.0125 MHz
        assert_eq!(Freq(8_510_125).to_string(), "08510125");
        assert_eq!(Freq::from_hz(851_012_500), Freq(8_510_125));
        assert_eq!("08510125".parse::<Freq>().unwrap().hz(), 851_012_500);
    }

    #[test]
    /// Checks coordinate formatting and west-longitude sign handling.
    fn coordinates_round_trip() {
        // Spec examples: 40°42'51.12" N and 74°00'23.05" W
        let lat: Latitude = "40425112N".parse().unwrap();
        assert_eq!(lat.to_string(), "40425112N");
        assert!((lat.0.to_decimal() - 40.7142).abs() < 1e-6);
        let lon: Longitude = "074002305W".parse().unwrap();
        assert_eq!(lon.to_string(), "074002305W");
        assert!(lon.0.to_decimal() < 0.0); // west is negative
    }

    #[test]
    /// Checks supported tone codes and rejects undefined wire values.
    fn tone_codes() {
        assert_eq!(ToneCode::from_wire(76).unwrap().tone(), Tone::Ctcss { tenths_hz: 1000 });
        assert_eq!(ToneCode::from_wire(128).unwrap().tone(), Tone::Dcs { code: 23 });
        assert_eq!(ToneCode::from_wire(239).unwrap().tone(), Tone::Dcs { code: 214 });
        assert_eq!(ToneCode::from_tone(Tone::Dcs { code: 6 }).unwrap().wire(), 232);
        assert!(ToneCode::from_wire(120).is_none()); // undefined gap
    }

    #[test]
    /// Checks NAC and color-code text representations.
    fn digital_codes() {
        assert_eq!("293".parse::<DigitalCode>(), Ok(DigitalCode::Nac(0x293)));
        assert_eq!("100A".parse::<DigitalCode>(), Ok(DigitalCode::ColorCode(10)));
        assert_eq!(DigitalCode::ColorCode(10).to_string(), "100A");
    }

    #[test]
    /// Checks P25 band-plan hexadecimal decoding and empty entries.
    fn p25_band_plan_hex() {
        // Spec example: base 851.00625 MHz = 0xA2510A2, spacing 6.25 kHz = 0x32
        let mut reply = String::from("ABP,A2510A2,32");
        for _ in 1..16 {
            reply.push_str(",0,0"); // fifteen unused entries
        }
        let plan = P25BandPlan::parse(&reply).unwrap();
        assert_eq!(plan.bands[0], Some(P25Band { base_hz: 851_006_250, spacing_hz: 6_250 }));
        assert!(plan.bands[1..].iter().all(Option::is_none));
    }

    #[test]
    /// Checks channel reply parsing and set-command field ordering.
    fn channel_get_to_set() {
        let reply = "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,123,125,10,11,,0,,NONE,OFF,0,0";
        let ch = ChannelRecord::parse(reply).unwrap();
        assert_eq!(ch.freq, Some(Freq(1_545_500)));
        assert_eq!(ch.links, Links { rev: Some(Index(123)), fwd: Some(Index(125)) });
        assert_eq!(ch.nac, None); // empty field
        // Link fields drop out and the index goes up front
        assert_eq!(
            ch.to_set(Index(789)),
            "CIN,789,Dispatch,01545500,NFM,0,0,0,0,0,0,0,,0,,NONE,OFF,0,0"
        );
    }

    #[test]
    /// Checks scanner and unexpected-reply errors are reported distinctly.
    fn scanner_errors_are_detected() {
        assert!(matches!(SystemRecord::parse("NG"), Err(Error::Scanner { .. })));
        assert!(matches!(SystemRecord::parse("SIN,ERR"), Err(Error::Scanner { .. })));
        assert!(matches!(SystemRecord::parse("CIN,x"), Err(Error::UnexpectedReply { .. })));
    }

    #[test]
    /// Checks a truncated reply identifies the missing field position.
    fn short_reply_reports_position() {
        let err = GroupRecord::parse("GIN,C,Fire,1,0").unwrap_err();
        assert!(matches!(err, Error::MissingField { cmd: "GIN", pos: 5 }));
    }

    #[test]
    /// Checks a small linked conventional database can be read from Program Mode.
    fn read_small_conventional_database() {
        let mut sc = Mock(HashMap::from([
            ("PRG", "PRG,OK"),
            ("EPG", "EPG,OK"),
            ("SIH", "SIH,10"),
            // System 10: one group (11), no neighbours
            ("SIN,10", "SIN,CNV,County,1,0,0,2,,,,,,-1,-1,11,11,1,.,,,,,,NONE,0,0,0,0,"),
            // Group 11: one channel (12)
            ("GIN,11", "GIN,C,Fireground,1,0,-1,-1,10,12,12,1,,,,"),
            ("CIN,12", "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,-1,-1,10,11,,0,,NONE,OFF,0,0"),
        ]));
        let db = with_program_mode(&mut sc, |sc| read_database(sc, Dialect::DIGITAL_DMR)).unwrap();

        assert_eq!(db.systems.len(), 1);
        let sys = &db.systems[0];
        assert_eq!(sys.info.name.as_str(), "County");
        assert_eq!(sys.info.delay, DelayTime::new(2));
        assert_eq!(sys.info.start_key, Some(KeyAssignment::Unassigned));
        match &sys.kind {
            SystemKind::Conventional { groups } => {
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].channels.len(), 1);
                assert_eq!(groups[0].channels[0].info.name.as_str(), "Dispatch");
            }
            SystemKind::Trunked { .. } => panic!("expected conventional"),
        }
    }

    #[test]
    /// Checks a downloaded database survives serialization and deserialization.
    fn database_round_trips_through_ron() {
        let mut sc = Mock(HashMap::from([
            ("PRG", "PRG,OK"),
            ("EPG", "EPG,OK"),
            ("SIH", "SIH,10"),
            ("SIN,10", "SIN,CNV,County,1,0,0,2,,,,,,-1,-1,11,11,1,.,,,,,,NONE,0,0,0,0,"),
            ("GIN,11", "GIN,C,Fireground,1,0,-1,-1,10,12,12,1,,,,"),
            ("CIN,12", "CIN,Dispatch,01545500,NFM,0,0,0,0,0,0,0,-1,-1,10,11,,0,,NONE,OFF,0,0"),
        ]));
        let db = with_program_mode(&mut sc, |sc| read_database(sc, Dialect::DIGITAL_DMR)).unwrap();
        let text = ron::ser::to_string_pretty(&db, ron::ser::PrettyConfig::default()).unwrap();
        let back: ScanDatabase = ron::from_str(&text).unwrap();
        assert_eq!(db, back);
    }

    #[test]
    /// Checks scanner name length and comma restrictions.
    fn name_validation() {
        assert!(Name::new("Fireground").is_ok());
        assert!(Name::new("This name is too long").is_err()); // over 16 chars
        assert!(Name::new("A,B").is_err()); // comma would break the field layout
    }

    #[test]
    /// Checks the non-DMR dialect omits the DMR-only set-command fields.
    fn no_dmr_dialect_drops_trailing_fields() {
        let tgid = TgidRecord { tdma_slot: Some(TdmaSlot::Slot1), ..Default::default() };
        let full = tgid.to_set(Index(5), Dialect::DIGITAL_DMR);
        let short = tgid.to_set(Index(5), Dialect::NO_DMR);
        assert_eq!(full.split(',').count(), short.split(',').count() + 1);
        assert!(full.ends_with(",1") && !short.ends_with(",1"));

        let freq = TrunkFreqRecord { color_code: Some(ColorCodeSetting::Search), ..Default::default() };
        let full = freq.to_set(Index(5), Dialect::DIGITAL_DMR);
        let short = freq.to_set(Index(5), Dialect::NO_DMR);
        assert_eq!(full.split(',').count(), short.split(',').count() + 1);
    }

    #[test]
    /// Checks the non-DMR dialect accepts replies without DMR-only trailing fields.
    fn no_dmr_dialect_parses_shorter_replies() {
        let tin = "TIN,Fire,100,0,0,0,0,1,2,3,4,,0,NONE,OFF,0,0";
        let rec = TgidRecord::parse(tin, Dialect::NO_DMR).unwrap();
        assert_eq!(rec.tdma_slot, None);
        assert!(TgidRecord::parse(tin, Dialect::DIGITAL_DMR).is_err());

        let tfq = "TFQ,08510125,,0,1,2,3,4,,NONE,0,";
        let rec = TrunkFreqRecord::parse(tfq, Dialect::NO_DMR).unwrap();
        assert_eq!(rec.color_code, None);
    }
}
