use dioxus::prelude::*;
use crate::confirm::confirm;
use std::fmt::Display;
use std::str::FromStr;

use crate::database::{
    NumberTag,
    BandPlan, Channel, ChannelGroup, Freq, GroupRecord, GroupType, KeyAssignment, Modulation, Name, ScanDatabase, Site, System,
    SystemKind, SystemRecord, SiteRecord, TgidRecord, ChannelRecord, SystemType, TgidEntry, TgidGroup, TrunkFreq, TrunkRecord
};
use crate::scanner_interaction::{active_model, SCAN_DATABASE, VALIDATION};

/// Pencil icon used by the inline edit controls.
const EDIT_ICON: Asset = asset!("/assets/icons/icons8-edit-pencil-24.svg");

// ---- Turning record values into editable text, and edited text back into values. ----
// Text that does not parse leaves the original value in place.

/// Optional value to string. Empty if `None`.
/// 
/// # Arguments
/// * `v` - The optional value to convert to a string.
///
/// # Returns
/// A string representation of the value, or an empty string if `None`.
/// 
/// # Examples
/// ```
/// let v: Option<i32> = Some(42);
/// assert_eq!(opt_string(&v), "42");
/// let v: Option<i32> = None;
/// assert_eq!(opt_string(&v), "");
/// ```
fn opt_string<T: Display>(v: &Option<T>) -> String {
    v.as_ref().map(|v| v.to_string()).unwrap_or_default()
}

/// Commits an optional value from a draft string.
/// 
/// In other words, it attempts to parse the draft string into a value of type `T`. 
/// If parsing fails, it falls back to the original value.
/// 
/// # Arguments
/// * `draft` - The draft string to parse.
/// * `orig` - The original optional value.
///
/// # Returns
/// The parsed value if successful, or the original value if parsing fails.
/// 
/// # Examples
/// ```
/// let orig: Option<i32> = Some(42);
/// assert_eq!(commit_opt("43", &orig), Some(43));
/// assert_eq!(commit_opt("", &orig), None);
/// ```
fn commit_opt<T: FromStr + Clone>(draft: &str, orig: &Option<T>) -> Option<T> {
    let draft = draft.trim();
    if draft.is_empty() {
        return None;
    }
    draft.parse().ok().or_else(|| orig.clone())
}

/// Converts an optional key assignment to a string.
/// 
/// # Arguments
/// * `v` - The optional key assignment to convert.
///
/// # Returns
/// A string representation of the key assignment, or an empty string if `None` or unassigned.
/// 
/// # Examples
/// ```
/// let v: Option<KeyAssignment> = Some(KeyAssignment::Key(42));
/// assert_eq!(key_string(&v), "42");
/// let v: Option<KeyAssignment> = None;
/// assert_eq!(key_string(&v), "");
/// ```
fn key_string(v: &Option<KeyAssignment>) -> String {
    match v {
        Some(KeyAssignment::Key(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// Rounds a numeric draft to `step`, clamps it into `0..=max`, or returns `None` if it is not a number.
fn clamp_int(draft: &str, max: u32, step: u32) -> Option<u32> {
    let n: f64 = draft.trim().parse().ok().filter(|n: &f64| n.is_finite())?;
    let step = step as f64;
    let n = (n / step).round() * step;
    Some(n.clamp(0.0, max as f64 / step * step) as u32)
}

/// Commits a key number up to `max`, preserving the distinction between blank and unassigned.
fn commit_key(draft: &str, orig: &Option<KeyAssignment>, max: u8) -> Option<KeyAssignment> {
    if draft.trim().is_empty() {
        return match orig {
            None | Some(KeyAssignment::Unassigned) => *orig,
            Some(KeyAssignment::Key(_)) => Some(KeyAssignment::Unassigned),
        };
    }
    clamp_int(draft, max as u32, 1).map(|n| KeyAssignment::Key(n as u8)).or(*orig)
}

/// Shows a numbered tag as text, leaving unassigned tags blank.
fn tag_string(v: &Option<NumberTag>) -> String {
    match v {
        Some(NumberTag::Tag(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// Commits a numbered tag, preserving an explicit unassigned value when the draft is blank.
fn commit_tag(draft: &str, orig: &Option<NumberTag>) -> Option<NumberTag> {
    if draft.trim().is_empty() {
        return match orig {
            None | Some(NumberTag::Unassigned) => *orig,
            Some(NumberTag::Tag(_)) => Some(NumberTag::Unassigned),
        };
    }
    clamp_int(draft, 999, 1).map(|n| NumberTag::Tag(n as u16)).or(*orig)
}

/// Integer rounded to a multiple of `step` and clamped to `0..=max`; unparsable text keeps the original.
fn commit_int<T: TryFrom<u32> + Copy>(draft: &str, orig: &Option<T>, max: u32, step: u32) -> Option<T> {
    if draft.trim().is_empty() {
        return None;
    }
    clamp_int(draft, max, step).and_then(|n| T::try_from(n).ok()).or(*orig)
}

/// Shows an optional frequency in MHz for the editor.
fn freq_string(v: &Option<Freq>) -> String {
    v.map(|f| f.mhz().to_string()).unwrap_or_default()
}

/// Parses a MHz frequency draft and stores it as rounded Hz, keeping the old value if invalid.
fn commit_freq(draft: &str, orig: &Option<Freq>) -> Option<Freq> {
    let draft = draft.trim();
    if draft.is_empty() {
        return None;
    }
    match draft.parse::<f64>() {
        Ok(mhz) if mhz.is_finite() && mhz >= 0.0 => Some(Freq::from_hz((mhz * 1e6).round() as u64)),
        _ => *orig,
    }
}

/// Maximum name length accepted by this scanner editor.
const NAME_MAX: usize = 16;

/// Reports whether a character is allowed in a scanner name.
fn name_char_ok(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !@#$%^&*()-/<>.?".contains(c)
}

/// Removes unsupported characters and truncates a draft to the scanner name limit.
fn sanitize_name(text: &str) -> String {
    text.chars().filter(|c| name_char_ok(*c)).take(NAME_MAX).collect()
}

/// Disallowed characters are dropped and the result is cut to 16 characters.
fn commit_name(draft: &str, orig: &Name) -> Name {
    Name::new(sanitize_name(draft).trim()).unwrap_or_else(|_| orig.clone())
}

/// Preserves an unset boolean unless the user changes its displayed value.
fn commit_bool(draft: bool, orig: &Option<bool>) -> Option<bool> {
    if draft == orig.unwrap_or(false) {
        *orig
    } else {
        Some(draft)
    }
}

// ---- Database lookups by position, used to write edits back. ----

/// Looks up a mutable system by its current position in the database.
fn system_mut(db: &mut ScanDatabase, si: usize) -> Option<&mut System> {
    db.systems.get_mut(si)
}

/// Gets the conventional channel groups for a system, if that system has them.
fn groups_mut(db: &mut ScanDatabase, si: usize) -> Option<&mut Vec<ChannelGroup>> {
    match &mut system_mut(db, si)?.kind {
        SystemKind::Conventional { groups } => Some(groups),
        _ => None,
    }
}

/// Gets the trunked sites for a system, if that system has them.
fn sites_mut(db: &mut ScanDatabase, si: usize) -> Option<&mut Vec<Site>> {
    match &mut system_mut(db, si)?.kind {
        SystemKind::Trunked { sites, .. } => Some(sites),
        _ => None,
    }
}

/// Gets the talkgroup groups for a trunked system, if present.
fn tgid_groups_mut(db: &mut ScanDatabase, si: usize) -> Option<&mut Vec<TgidGroup>> {
    match &mut system_mut(db, si)?.kind {
        SystemKind::Trunked { tgid_groups, .. } => Some(tgid_groups),
        _ => None,
    }
}

/// Applies an edit only when a database is currently loaded.
fn update_db(f: impl FnOnce(&mut ScanDatabase)) {
    if let Some(db) = SCAN_DATABASE.write().as_mut() {
        f(db);
    }
}

/// Per list (identified by a path string): the delete counter and the lowest index affected by it.
/// Rows share per-position edit state, so rows at or after a deleted index are remounted;
/// rows before it, and every other list, keep their state and expanded/collapsed status.
static DELETED: GlobalSignal<std::collections::HashMap<String, (u64, usize)>> =
    Signal::global(Default::default);

/// Builds a stable row key, remounting only rows shifted by a deletion in this list.
fn item_key(list: &str, i: usize) -> String {
    match DELETED.read().get(list) {
        Some(&(generation, from)) if i >= from => format!("{list}-{i}-{generation}"),
        _ => format!("{list}-{i}"),
    }
}

/// Builds the render key for this database row.
fn key_systems(i: usize) -> String {
    item_key("systems", i)
}
/// Builds the render key for this database row.
fn key_groups(si: usize, i: usize) -> String {
    item_key(&format!("s{si}/groups"), i)
}
/// Builds the render key for this database row.
fn key_sites(si: usize, i: usize) -> String {
    item_key(&format!("s{si}/sites"), i)
}
/// Builds the render key for this database row.
fn key_tgid_groups(si: usize, i: usize) -> String {
    item_key(&format!("s{si}/tgroups"), i)
}
/// Builds the render key for this database row.
fn key_channels(si: usize, gi: usize, i: usize) -> String {
    item_key(&format!("s{si}/g{gi}/channels"), i)
}
/// Builds the render key for this database row.
fn key_freqs(si: usize, sti: usize, i: usize) -> String {
    item_key(&format!("s{si}/t{sti}/freqs"), i)
}
/// Builds the render key for this database row.
fn key_tgids(si: usize, gi: usize, i: usize) -> String {
    item_key(&format!("s{si}/tg{gi}/tgids"), i)
}

/// Deletes an item and invalidates editor state for rows whose positions may have shifted.
fn delete_item(list: String, i: usize, f: impl FnOnce(&mut ScanDatabase)) {
    update_db(f);
    let mut deleted = DELETED.write();
    let entry = deleted.entry(list).or_insert((0, i));
    *entry = (entry.0 + 1, i.min(entry.1));
    *crate::components::UPLOAD_ENABLED.write() = false;
    *VALIDATION.write() = None;
}

// ---- Adding new items. New items get a placeholder name and default settings. ----


/// Returns the user-facing name for a selectable system type.
fn system_type_label(t: SystemType) -> &'static str {
    match t {
        SystemType::Conventional => "Conventional",
        SystemType::Motorola => "Motorola",
        SystemType::Edacs => "EDCS Wide/Narrow",
        SystemType::EdacsScat => "EDCS SCAT",
        SystemType::Ltr => "LTR",
        SystemType::P25Standard => "P25 Standard",
        SystemType::P25OneFreq => "P25 One Frequency",
        SystemType::MotoTrbo => "MotoTRBO",
        SystemType::DmrOneFreq => "DMR One Frequency",
    }
}

/// Creates a valid scanner name, falling back to an empty default if needed.
fn new_name(text: &str) -> Name {
    Name::new(text).unwrap_or_default()
}

/// Makes a new system with the correct conventional or trunked shape for its type.
fn new_system(sys_type: SystemType) -> System {
    let info = SystemRecord { name: new_name("New System"), sys_type, ..Default::default() };
    let kind = if sys_type.is_trunked() {
        SystemKind::Trunked { trunk: TrunkRecord::default(), sites: Vec::new(), tgid_groups: Vec::new() }
    } else {
        SystemKind::Conventional { groups: Vec::new() }
    };
    System { index: Default::default(), info, kind }
}

/// Makes an empty conventional group with a visible placeholder name.
/// Makes a channel with a placeholder name and default settings.
fn new_channel_group() -> ChannelGroup {
    ChannelGroup {
        index: Default::default(),
        info: GroupRecord { name: new_name("New Group"), ..Default::default() },
        channels: Vec::new(),
    }
}

/// Makes an empty talkgroup group with a visible placeholder name.
/// Makes a talkgroup entry with a placeholder name.
fn new_tgid_group() -> TgidGroup {
    TgidGroup {
        index: Default::default(),
        info: GroupRecord { name: new_name("New TG Group"), group_type: GroupType::Tgid, ..Default::default() },
        tgids: Vec::new(),
    }
}

/// Makes an empty trunked site ready for its band plan and frequencies.
fn new_site() -> Site {
    Site {
        index: Default::default(),
        info: SiteRecord { name: new_name("New Site"), ..Default::default() },
        band_plan: None,
        frequencies: Vec::new(),
    }
}

fn new_channel() -> Channel {
    Channel { index: Default::default(), info: ChannelRecord { name: new_name("New Channel"), ..Default::default() } }
}

/// Makes an empty trunking frequency entry.
fn new_trunk_freq() -> TrunkFreq {
    TrunkFreq { index: Default::default(), info: Default::default() }
}

fn new_tgid() -> TgidEntry {
    TgidEntry { index: Default::default(), info: TgidRecord { name: new_name("New TGID"), ..Default::default() } }
}

/// Button that asks the parent to add a new item.
#[component]
fn AddButton(label: &'static str, on_add: EventHandler<()>) -> Element {
    rsx! {
        button { class: "add-button", onclick: move |_| on_add.call(()), "+ {label}" }
    }
}

// ---- Input widgets: show the draft while editing, the stored value otherwise. ----

/// Text input that edits a temporary draft and otherwise displays the stored value.
fn text_field(kind: &'static str, editing: bool, shown: String, mut draft: Signal<String>) -> Element {
    let value = if editing { draft() } else { shown };
    rsx! {
        input {
            r#type: kind,
            value: "{value}",
            disabled: !editing,
            oninput: move |e| draft.set(e.value()),
        }
    }
}

/// Name input that filters unsupported characters as the user types.
fn name_field(editing: bool, shown: String, mut draft: Signal<String>) -> Element {
    let value = if editing { draft() } else { shown };
    rsx! {
        input {
            r#type: "text",
            maxlength: "{NAME_MAX}",
            value: "{value}",
            disabled: !editing,
            oninput: move |e| draft.set(sanitize_name(&e.value())),
        }
    }
}

/// Numeric input with the scanner editor range and step shown to the browser.
fn num_field(editing: bool, shown: String, mut draft: Signal<String>, max: u32, step: u32) -> Element {
    let value = if editing { draft() } else { shown };
    rsx! {
        input {
            r#type: "number",
            min: "0",
            max: "{max}",
            step: "{step}",
            value: "{value}",
            disabled: !editing,
            oninput: move |e| draft.set(e.value()),
        }
    }
}

/// Frequency input displayed in MHz and committed to the database as Hz.
fn freq_field(editing: bool, shown: &Option<Freq>, draft: Signal<String>) -> Element {
    let value = if editing { draft() } else { freq_string(shown) };
    let mut draft = draft;
    rsx! {
        input {
            r#type: "number",
            step: "0.0001",
            min: "0",
            value: "{value}",
            disabled: !editing,
            oninput: move |e| draft.set(e.value()),
        }
        " MHz"
    }
}

/// Checkbox that keeps an unset boolean distinct until the user changes it.
fn check_field(editing: bool, shown: &Option<bool>, mut draft: Signal<bool>) -> Element {
    let checked = if editing { draft() } else { shown.unwrap_or(false) };
    rsx! {
        input {
            r#type: "checkbox",
            checked,
            disabled: !editing,
            onchange: move |e| draft.set(e.checked()),
        }
    }
}

/// Selection input that switches between the current value and an editable draft.
fn select_field(options: Vec<String>, editing: bool, shown: String, mut draft: Signal<String>) -> Element {
    let current = if editing { draft() } else { shown };
    rsx! {
        select {
            disabled: !editing,
            value: "{current}",
            onchange: move |e| draft.set(e.value()),
            for o in options {
                option { value: "{o}", selected: o == current, if o.is_empty() { "-" } else { "{o}" } }
            }
        }
    }
}

/// Lists supported modulation choices, with a blank option for automatic selection.
fn modulation_options() -> Vec<String> {
    [Modulation::Auto, Modulation::Am, Modulation::Fm, Modulation::Nfm, Modulation::Wfm, Modulation::Fmb]
        .into_iter()
        .map(|m| m.to_string())
        .fold(vec![String::new()], |mut v, m| {
            v.push(m);
            v
        })
}

/// Toggles an item between view and edit mode. Shown at the end of each system, group and row.
/// `on_toggle` receives the new state: `true` on entering edit mode, `false` on leaving it.
#[component]
fn EditButton(mut editing: Signal<bool>, on_toggle: EventHandler<bool>) -> Element {
    let title = if editing() { "Finish editing" } else { "Edit" };
    rsx! {
        button {
            class: if editing() { "edit-button active" } else { "edit-button" },
            title,
            // Inside a <summary>, a click would otherwise toggle the <details>.
            onclick: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                *crate::components::UPLOAD_ENABLED.write() = false;
                let now = !editing();
                editing.set(now);
                *VALIDATION.write() = None;
                on_toggle.call(now);
            },
            img { src: EDIT_ICON, alt: "Edit" }
        }
    }
}

/// Removes an item. Shown next to each [`EditButton`].
#[component]
fn DeleteButton(what: String, on_delete: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "delete-button",
            title: "Delete",
            // Inside a <summary>, a click would otherwise toggle the <details>.
            onclick: move |evt| {
                evt.prevent_default();
                evt.stop_propagation();
                confirm(format!("Delete this {what}?"), "Delete", move || on_delete.call(()));
            },
            "X"
        }
    }
}

/// Memory used by the current database, shown at the right end of the toolbar.
#[component]
pub fn MemoryUsage() -> Element {
    let db = SCAN_DATABASE.read();
    let Some(db) = db.as_ref() else {
        return rsx! {};
    };
    let used = db.blocks_used();
    let max_blocks = active_model().capabilities().max_blocks;
    let percent = used as f64 / max_blocks as f64 * 100.0;
    rsx! {
        span { id: "memory-usage",
            "Memory used: {used} / {max_blocks} blocks ({percent:.1}%) "
            meter { min: 0, max: max_blocks as f64, value: used as f64 }
        }
    }
}

/// The downloaded scan database as a collapsible tree.
/// The <details> elements are not controlled by Dioxus, so open state is set on the DOM directly.
fn set_all_open(open: bool) {
    document::eval(&format!(
        "document.querySelectorAll('#database-view details').forEach(d => d.open = {open});"
    ));
}

/// Main database editor: systems can be expanded into settings, groups, sites, and rows.
#[component]
pub fn DatabaseView() -> Element {
    // This draft controls the type of the next system; it does not alter existing systems.
    let mut new_type = use_signal(|| SystemType::default().to_string());
    let db = SCAN_DATABASE.read();
    let systems = db.as_ref().map(|db| db.systems.clone()).unwrap_or_default();
    rsx! {
        div { id: "database-view",
            h2 { "Systems ({systems.len()})" }
            div { class: "expand-controls",
                button {
                    title: "Collapse every system, group and site",
                    onclick: move |_| set_all_open(false),
                    "Collapse All"
                }
                button {
                    title: "Expand every system, group and site",
                    onclick: move |_| set_all_open(true),
                    "Expand All"
                }
            }
            for (si, system) in systems.into_iter().enumerate() {
                SystemView { key: "{key_systems(si)}", si, system }
            }
            div { class: "add-row",
                AddButton {
                    label: "Add System",
                    on_add: move |_| {
                        let sys_type = new_type().parse().unwrap_or_default();
                        let system = new_system(sys_type);
                        let mut db = SCAN_DATABASE.write();
                        db.get_or_insert_with(ScanDatabase::default).systems.push(system);
                    },
                }
                select {
                    value: "{new_type}",
                    onchange: move |e| new_type.set(e.value()),
                    for &t in active_model().capabilities().system_types {
                        option { value: "{t}", selected: t.to_string() == new_type(), "{system_type_label(t)}" }
                    }
                }
            }
        }
    }
}

/// Shows one expandable system and the matching editor for its system type.
#[component]
fn SystemView(si: usize, system: System) -> Element {
    let editing = use_signal(|| false);
    // Each editor keeps a local draft until the user finishes the edit.
    let mut name = use_signal(String::new);
    let sys_type = format!("{:?}", system.info.sys_type);
    let initial_name = system.info.name.to_string();
    let settings = match (system.info.sys_type, &system.kind) {
        (SystemType::P25Standard | SystemType::P25OneFreq, SystemKind::Trunked { trunk, .. }) => rsx! {
            P25SystemSettingsView { si, info: system.info.clone(), trunk: trunk.clone() }
        },
        _ => rsx! { SystemSettingsView { si, info: system.info.clone() } },
    };
    rsx! {
        details {
            summary {
                {name_field(editing(), system.info.name.to_string(), name)}
                " ({sys_type})"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(initial_name.clone());
                        } else {
                            update_db(|db| {
                                if let Some(s) = system_mut(db, si) {
                                    s.info.name = commit_name(&name(), &s.info.name);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "system and everything in it".to_string(),
                    on_delete: move |_| delete_item("systems".to_string(), si, |db| {
                        if si < db.systems.len() {
                            db.systems.remove(si);
                        }
                    }),
                }
            }
            {settings}
            match system.kind {
                SystemKind::Conventional { groups } => rsx! {
                    for (gi, group) in groups.into_iter().enumerate() {
                        ChannelGroupView { key: "{key_groups(si, gi)}", si, gi, group }
                    }
                    AddButton {
                        label: "Add Group",
                        on_add: move |_| update_db(|db| {
                            if let Some(g) = groups_mut(db, si) {
                                g.push(new_channel_group());
                            }
                        }),
                    }
                },
                SystemKind::Trunked { sites, tgid_groups, .. } => rsx! {
                    for (sti, site) in sites.into_iter().enumerate() {
                        SiteView { key: "{key_sites(si, sti)}", si, sti, site }
                    }
                    AddButton {
                        label: "Add Site",
                        on_add: move |_| update_db(|db| {
                            if let Some(s) = sites_mut(db, si) {
                                s.push(new_site());
                            }
                        }),
                    }
                    for (gi, group) in tgid_groups.into_iter().enumerate() {
                        TgidGroupView { key: "{key_tgid_groups(si, gi)}", si, gi, group }
                    }
                    AddButton {
                        label: "Add Talkgroup Group",
                        on_add: move |_| update_db(|db| {
                            if let Some(g) = tgid_groups_mut(db, si) {
                                g.push(new_tgid_group());
                            }
                        }),
                    }
                },
            }
        }
    }
}

/// A system's quick key, startup key, lockout, hold/delay times, AGC and digital waiting settings.
#[component]
fn SystemSettingsView(si: usize, info: SystemRecord) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut quick_key = use_signal(String::new);
    let mut start_key = use_signal(String::new);
    let mut lockout = use_signal(|| false);
    let mut hold_time = use_signal(String::new);
    let mut delay = use_signal(String::new);
    let mut agc_analog = use_signal(|| false);
    let mut agc_digital = use_signal(|| false);
    let mut waiting = use_signal(String::new);
    let loaded = info.clone();
    rsx! {
        details {
            summary {
                "Settings"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            quick_key.set(key_string(&loaded.quick_key));
                            start_key.set(key_string(&loaded.start_key));
                            lockout.set(loaded.lockout.unwrap_or(false));
                            hold_time.set(opt_string(&loaded.hold_time));
                            delay.set(opt_string(&loaded.delay));
                            agc_analog.set(loaded.agc_analog.unwrap_or(false));
                            agc_digital.set(loaded.agc_digital.unwrap_or(false));
                            waiting.set(opt_string(&loaded.p25_waiting_ms));
                        } else {
                            update_db(|db| {
                                if let Some(s) = system_mut(db, si) {
                                    let i = &mut s.info;
                                    i.quick_key = commit_key(&quick_key(), &i.quick_key, 99);
                                    i.start_key = commit_key(&start_key(), &i.start_key, 9);
                                    i.lockout = commit_bool(lockout(), &i.lockout);
                                    i.hold_time = commit_int(&hold_time(), &i.hold_time, 255, 1);
                                    i.delay = commit_opt(&delay(), &i.delay);
                                    i.agc_analog = commit_bool(agc_analog(), &i.agc_analog);
                                    i.agc_digital = commit_bool(agc_digital(), &i.agc_digital);
                                    i.p25_waiting_ms = commit_int(&waiting(), &i.p25_waiting_ms, 1000, 100);
                                }
                            });
                        }
                    },
                }
            }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Setting" }
                        th { "Value" }
                    }
                }
                tbody {
                    tr {
                        td { "Quick Key" }
                        td { {num_field(ed, key_string(&info.quick_key), quick_key, 99, 1)} }
                    }
                    tr {
                        td { "Startup Key" }
                        td { {num_field(ed, key_string(&info.start_key), start_key, 9, 1)} }
                    }
                    tr {
                        td { "Lockout" }
                        td { {check_field(ed, &info.lockout, lockout)} }
                    }
                    tr {
                        td { "Hold Time" }
                        td {
                            {num_field(ed, opt_string(&info.hold_time), hold_time, 255, 1)}
                            " s"
                        }
                    }
                    tr {
                        td { "Delay Time" }
                        td {
                            {text_field("text", ed, opt_string(&info.delay), delay)}
                            " s"
                        }
                    }
                    tr {
                        td { "AGC Analog" }
                        td { {check_field(ed, &info.agc_analog, agc_analog)} }
                    }
                    tr {
                        td { "AGC Digital" }
                        td { {check_field(ed, &info.agc_digital, agc_digital)} }
                    }
                    tr {
                        td { "Digital Waiting" }
                        td {
                            {num_field(ed, opt_string(&info.p25_waiting_ms), waiting, 1000, 100)}
                            " ms"
                        }
                    }
                }
            }
        }
    }
}

/// Labels for the two P25 talkgroup scan modes.
const ID_MODES: [&str; 2] = ["ID Scan", "ID Search"];
/// Labels for decimal and hexadecimal talkgroup display.
const ID_FORMATS: [&str; 2] = ["Decimal", "Hex"];

/// P25 system settings: number tag, ID scan/search, delay, priority ID scan, ID format and AGC.
#[component]
fn P25SystemSettingsView(si: usize, info: SystemRecord, trunk: TrunkRecord) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut number_tag = use_signal(String::new);
    let mut id_mode = use_signal(String::new);
    let mut delay = use_signal(String::new);
    let mut priority = use_signal(|| false);
    let mut id_format = use_signal(String::new);
    let mut agc = use_signal(|| false);
    let shown_mode = ID_MODES[(trunk.id_search == Some(true)) as usize].to_string();
    let shown_format = ID_FORMATS[(trunk.hex_ids == Some(true)) as usize].to_string();
    let (loaded_info, loaded_trunk) = (info.clone(), trunk.clone());
    let (mode_now, format_now) = (shown_mode.clone(), shown_format.clone());
    rsx! {
        details {
            summary {
                "Settings (P25)"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            number_tag.set(tag_string(&loaded_info.number_tag));
                            id_mode.set(mode_now.clone());
                            delay.set(opt_string(&loaded_info.delay));
                            priority.set(loaded_trunk.priority_id_scan.unwrap_or(false));
                            id_format.set(format_now.clone());
                            agc.set(loaded_info.agc_digital.unwrap_or(false));
                        } else {
                            update_db(|db| {
                                if let Some(s) = system_mut(db, si) {
                                    s.info.number_tag = commit_tag(&number_tag(), &s.info.number_tag);
                                    s.info.delay = commit_opt(&delay(), &s.info.delay);
                                    s.info.agc_digital = commit_bool(agc(), &s.info.agc_digital);
                                    if let SystemKind::Trunked { trunk, .. } = &mut s.kind {
                                        trunk.id_search = commit_bool(id_mode() == ID_MODES[1], &trunk.id_search);
                                        trunk.priority_id_scan = commit_bool(priority(), &trunk.priority_id_scan);
                                        trunk.hex_ids = commit_bool(id_format() == ID_FORMATS[1], &trunk.hex_ids);
                                    }
                                }
                            });
                        }
                    },
                }
            }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Setting" }
                        th { "Value" }
                    }
                }
                tbody {
                    tr {
                        td { "Number Tag" }
                        td { {num_field(ed, tag_string(&info.number_tag), number_tag, 999, 1)} }
                    }
                    tr {
                        td { "ID Scan/Search" }
                        td { {select_field(ID_MODES.map(String::from).to_vec(), ed, shown_mode, id_mode)} }
                    }
                    tr {
                        td { "Delay Time" }
                        td {
                            {text_field("text", ed, opt_string(&info.delay), delay)}
                            " s"
                        }
                    }
                    tr {
                        td { "Priority ID Scan" }
                        td { {check_field(ed, &trunk.priority_id_scan, priority)} }
                    }
                    tr {
                        td { "ID Format" }
                        td { {select_field(ID_FORMATS.map(String::from).to_vec(), ed, shown_format, id_format)} }
                    }
                    tr {
                        td { "AGC" }
                        td { {check_field(ed, &info.agc_digital, agc)} }
                    }
                }
            }
        }
    }
}

/// A channel group's quick key, lockout and location (geofence) settings.
#[component]
fn GroupSettingsView(si: usize, gi: usize, info: GroupRecord) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut quick_key = use_signal(String::new);
    let mut lockout = use_signal(|| false);
    let mut latitude = use_signal(String::new);
    let mut longitude = use_signal(String::new);
    let mut range = use_signal(String::new);
    let mut gps = use_signal(|| false);
    let loaded = info.clone();
    rsx! {
        details {
            summary {
                "Settings"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            quick_key.set(key_string(&loaded.quick_key));
                            lockout.set(loaded.lockout.unwrap_or(false));
                            latitude.set(opt_string(&loaded.geofence.latitude));
                            longitude.set(opt_string(&loaded.geofence.longitude));
                            range.set(opt_string(&loaded.geofence.range));
                            gps.set(loaded.geofence.enabled.unwrap_or(false));
                        } else {
                            update_db(|db| {
                                if let Some(g) = groups_mut(db, si).and_then(|g| g.get_mut(gi)) {
                                    let i = &mut g.info;
                                    i.quick_key = commit_key(&quick_key(), &i.quick_key, 99);
                                    i.lockout = commit_bool(lockout(), &i.lockout);
                                    i.geofence.latitude = commit_opt(&latitude(), &i.geofence.latitude);
                                    i.geofence.longitude = commit_opt(&longitude(), &i.geofence.longitude);
                                    i.geofence.range = commit_opt(&range(), &i.geofence.range);
                                    i.geofence.enabled = commit_bool(gps(), &i.geofence.enabled);
                                }
                            });
                        }
                    },
                }
            }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Setting" }
                        th { "Value" }
                    }
                }
                tbody {
                    tr {
                        td { "Quick Key" }
                        td { {num_field(ed, key_string(&info.quick_key), quick_key, 99, 1)} }
                    }
                    tr {
                        td { "Lockout" }
                        td { {check_field(ed, &info.lockout, lockout)} }
                    }
                    tr {
                        td { "Latitude" }
                        td { {text_field("text", ed, opt_string(&info.geofence.latitude), latitude)} }
                    }
                    tr {
                        td { "Longitude" }
                        td { {text_field("text", ed, opt_string(&info.geofence.longitude), longitude)} }
                    }
                    tr {
                        td { "Range" }
                        td { {text_field("number", ed, opt_string(&info.geofence.range), range)} }
                    }
                    tr {
                        td { "GPS Enable" }
                        td { {check_field(ed, &info.geofence.enabled, gps)} }
                    }
                }
            }
        }
    }
}

/// Shows a conventional channel group, its settings, and its editable channels.
#[component]
fn ChannelGroupView(si: usize, gi: usize, group: ChannelGroup) -> Element {
    let editing = use_signal(|| false);
    let mut name = use_signal(String::new);
    let count = group.channels.len();
    let initial_name = group.info.name.to_string();
    rsx! {
        details {
            summary {
                "Group: "
                {name_field(editing(), group.info.name.to_string(), name)}
                " ({count} channels)"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(initial_name.clone());
                        } else {
                            update_db(|db| {
                                if let Some(g) = groups_mut(db, si).and_then(|g| g.get_mut(gi)) {
                                    g.info.name = commit_name(&name(), &g.info.name);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "channel group and its channels".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/groups"), gi, |db| {
                        if let Some(g) = groups_mut(db, si).filter(|g| gi < g.len()) {
                            g.remove(gi);
                        }
                    }),
                }
            }
            GroupSettingsView { si, gi, info: group.info.clone() }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Name" }
                        th { "Frequency" }
                        th { "Modulation" }
                        th { "Tone" }
                        th { "Lockout" }
                        th {}
                    }
                }
                tbody {
                    for (ci, channel) in group.channels.into_iter().enumerate() {
                        ChannelRow { key: "{key_channels(si, gi, ci)}", si, gi, ci, channel }
                    }
                }
            }
            AddButton {
                label: "Add Channel",
                on_add: move |_| update_db(|db| {
                    if let Some(g) = groups_mut(db, si).and_then(|g| g.get_mut(gi)) {
                        g.channels.push(new_channel());
                    }
                }),
            }
        }
    }
}

/// Shows and edits one channel within a conventional group.
#[component]
fn ChannelRow(si: usize, gi: usize, ci: usize, channel: Channel) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut name = use_signal(String::new);
    let mut freq = use_signal(String::new);
    let mut modulation = use_signal(String::new);
    let mut tone = use_signal(String::new);
    let mut lockout = use_signal(|| false);
    let loaded = channel.info.clone();
    rsx! {
        tr {
            td { {name_field(ed, channel.info.name.to_string(), name)} }
            td { {freq_field(ed, &channel.info.freq, freq)} }
            td { {select_field(modulation_options(), ed, opt_string(&channel.info.modulation), modulation)} }
            td { {text_field("text", ed, opt_string(&channel.info.tone), tone)} }
            td { {check_field(ed, &channel.info.lockout, lockout)} }
            td {
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(loaded.name.to_string());
                            freq.set(freq_string(&loaded.freq));
                            modulation.set(opt_string(&loaded.modulation));
                            tone.set(opt_string(&loaded.tone));
                            lockout.set(loaded.lockout.unwrap_or(false));
                        } else {
                            update_db(|db| {
                                let channel = groups_mut(db, si)
                                    .and_then(|g| g.get_mut(gi))
                                    .and_then(|g| g.channels.get_mut(ci));
                                if let Some(c) = channel {
                                    let i = &mut c.info;
                                    i.name = commit_name(&name(), &i.name);
                                    i.freq = commit_freq(&freq(), &i.freq);
                                    i.modulation = commit_opt(&modulation(), &i.modulation);
                                    i.tone = commit_opt(&tone(), &i.tone);
                                    i.lockout = commit_bool(lockout(), &i.lockout);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "channel".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/g{gi}/channels"), ci, |db| {
                        let channels = groups_mut(db, si)
                            .and_then(|g| g.get_mut(gi))
                            .map(|g| &mut g.channels)
                            .filter(|c| ci < c.len());
                        if let Some(c) = channels {
                            c.remove(ci);
                        }
                    }),
                }
            }
        }
    }
}

/// Shows a trunked site, its band plan, and the frequencies assigned to it.
#[component]
fn SiteView(si: usize, sti: usize, site: Site) -> Element {
    let editing = use_signal(|| false);
    let mut name = use_signal(String::new);
    let count = site.frequencies.len();
    let initial_name = site.info.name.to_string();
    let plan = match &site.band_plan {
        Some(BandPlan::Motorola(_)) => "Motorola custom band plan",
        Some(BandPlan::P25(_)) => "P25 band plan",
        None => "default band plan",
    };
    rsx! {
        details {
            summary {
                "Site: "
                {name_field(editing(), site.info.name.to_string(), name)}
                " ({count} frequencies, {plan})"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(initial_name.clone());
                        } else {
                            update_db(|db| {
                                if let Some(s) = sites_mut(db, si).and_then(|s| s.get_mut(sti)) {
                                    s.info.name = commit_name(&name(), &s.info.name);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "site and its frequencies".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/sites"), sti, |db| {
                        if let Some(s) = sites_mut(db, si).filter(|s| sti < s.len()) {
                            s.remove(sti);
                        }
                    }),
                }
            }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Frequency" }
                        th { "LCN" }
                        th { "Lockout" }
                        th {}
                    }
                }
                tbody {
                    for (fi, freq) in site.frequencies.into_iter().enumerate() {
                        TrunkFreqRow { key: "{key_freqs(si, sti, fi)}", si, sti, fi, freq }
                    }
                }
            }
            AddButton {
                label: "Add Frequency",
                on_add: move |_| update_db(|db| {
                    if let Some(s) = sites_mut(db, si).and_then(|s| s.get_mut(sti)) {
                        s.frequencies.push(new_trunk_freq());
                    }
                }),
            }
        }
    }
}

/// Shows and edits one control-channel frequency in a trunked site.
#[component]
fn TrunkFreqRow(si: usize, sti: usize, fi: usize, freq: TrunkFreq) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut frequency = use_signal(String::new);
    let mut lcn = use_signal(String::new);
    let mut lockout = use_signal(|| false);
    let loaded = freq.info.clone();
    rsx! {
        tr {
            td { {freq_field(ed, &freq.info.freq, frequency)} }
            td { {text_field("number", ed, opt_string(&freq.info.lcn), lcn)} }
            td { {check_field(ed, &freq.info.lockout, lockout)} }
            td {
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            frequency.set(freq_string(&loaded.freq));
                            lcn.set(opt_string(&loaded.lcn));
                            lockout.set(loaded.lockout.unwrap_or(false));
                        } else {
                            update_db(|db| {
                                let entry = sites_mut(db, si)
                                    .and_then(|s| s.get_mut(sti))
                                    .and_then(|s| s.frequencies.get_mut(fi));
                                if let Some(f) = entry {
                                    f.info.freq = commit_freq(&frequency(), &f.info.freq);
                                    f.info.lcn = commit_opt(&lcn(), &f.info.lcn);
                                    f.info.lockout = commit_bool(lockout(), &f.info.lockout);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "frequency".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/t{sti}/freqs"), fi, |db| {
                        let freqs = sites_mut(db, si)
                            .and_then(|s| s.get_mut(sti))
                            .map(|s| &mut s.frequencies)
                            .filter(|f| fi < f.len());
                        if let Some(f) = freqs {
                            f.remove(fi);
                        }
                    }),
                }
            }
        }
    }
}

/// Shows a talkgroup group and the talkgroups it contains.
#[component]
fn TgidGroupView(si: usize, gi: usize, group: TgidGroup) -> Element {
    let editing = use_signal(|| false);
    let mut name = use_signal(String::new);
    let count = group.tgids.len();
    let initial_name = group.info.name.to_string();
    rsx! {
        details {
            summary {
                "Talkgroups: "
                {name_field(editing(), group.info.name.to_string(), name)}
                " ({count})"
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(initial_name.clone());
                        } else {
                            update_db(|db| {
                                if let Some(g) = tgid_groups_mut(db, si).and_then(|g| g.get_mut(gi)) {
                                    g.info.name = commit_name(&name(), &g.info.name);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "talkgroup group and its talkgroups".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/tgroups"), gi, |db| {
                        if let Some(g) = tgid_groups_mut(db, si).filter(|g| gi < g.len()) {
                            g.remove(gi);
                        }
                    }),
                }
            }
            // The settings table keeps each field aligned with its current scanner value.
            table {
                thead {
                    tr {
                        th { "Name" }
                        th { "TGID" }
                        th { "Lockout" }
                        th { "Priority" }
                        th {}
                    }
                }
                tbody {
                    for (ti, tgid) in group.tgids.into_iter().enumerate() {
                        TgidRow { key: "{key_tgids(si, gi, ti)}", si, gi, ti, tgid }
                    }
                }
            }
            AddButton {
                label: "Add Talkgroup",
                on_add: move |_| update_db(|db| {
                    if let Some(g) = tgid_groups_mut(db, si).and_then(|g| g.get_mut(gi)) {
                        g.tgids.push(new_tgid());
                    }
                }),
            }
        }
    }
}

/// Shows and edits one talkgroup entry.
#[component]
fn TgidRow(si: usize, gi: usize, ti: usize, tgid: TgidEntry) -> Element {
    // Keep editable drafts local so an unfinished edit never changes the database.
    let editing = use_signal(|| false);
    let ed = editing();
    let mut name = use_signal(String::new);
    let mut id = use_signal(String::new);
    let mut lockout = use_signal(|| false);
    let mut priority = use_signal(|| false);
    let loaded = tgid.info.clone();
    rsx! {
        tr {
            td { {name_field(ed, tgid.info.name.to_string(), name)} }
            td { {text_field("text", ed, opt_string(&tgid.info.tgid), id)} }
            td { {check_field(ed, &tgid.info.lockout, lockout)} }
            td { {check_field(ed, &tgid.info.priority, priority)} }
            td {
                EditButton {
                    editing,
                    on_toggle: move |now: bool| {
                        if now {
                            name.set(loaded.name.to_string());
                            id.set(opt_string(&loaded.tgid));
                            lockout.set(loaded.lockout.unwrap_or(false));
                            priority.set(loaded.priority.unwrap_or(false));
                        } else {
                            update_db(|db| {
                                let entry = tgid_groups_mut(db, si)
                                    .and_then(|g| g.get_mut(gi))
                                    .and_then(|g| g.tgids.get_mut(ti));
                                if let Some(t) = entry {
                                    t.info.name = commit_name(&name(), &t.info.name);
                                    t.info.tgid = commit_opt(&id(), &t.info.tgid);
                                    t.info.lockout = commit_bool(lockout(), &t.info.lockout);
                                    t.info.priority = commit_bool(priority(), &t.info.priority);
                                }
                            });
                        }
                    },
                }
                DeleteButton {
                    what: "talkgroup".to_string(),
                    on_delete: move |_| delete_item(format!("s{si}/tg{gi}/tgids"), ti, |db| {
                        let tgids = tgid_groups_mut(db, si)
                            .and_then(|g| g.get_mut(gi))
                            .map(|g| &mut g.tgids)
                            .filter(|t| ti < t.len());
                        if let Some(t) = tgids {
                            t.remove(ti);
                        }
                    }),
                }
            }
        }
    }
}
