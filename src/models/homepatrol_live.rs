//! Parsers for HomePatrol/SDS XML status and SDS status extensions.

use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::Error;
use super::live::{DisplayLine, DisplayStatus, LiveState, ScanMode};

/// Normalized state produced by one GSI/PSI `ScannerInfo` document.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HpSnapshot {
    pub live: LiveState,
    pub display: Option<DisplayStatus>,
    pub weather_alert: bool,
    pub battery_volts: Option<f32>,
}

/// Detailed SDS charge status from `GCS`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChargeStatus {
    pub state: u8,
    pub millivolts: u16,
    pub percent: u8,
    pub current_ma: i16,
    pub temperature_c: f32,
}

/// SDS waterfall bounds and marker returned by `GST`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WaterfallStatus {
    pub display: DisplayStatus,
    pub waterfall_mode: bool,
    pub marker_hz: Option<u64>,
    pub center_hz: Option<u64>,
    pub lower_hz: Option<u64>,
    pub upper_hz: Option<u64>,
}

fn attrs(
    reader: &Reader<&[u8]>,
    element: &BytesStart<'_>,
) -> Result<HashMap<String, String>, Error> {
    let mut out = HashMap::new();
    for attr in element.attributes() {
        let attr = attr.map_err(|error| Error::BadValue {
            cmd: "GSI",
            pos: 0,
            value: error.to_string(),
        })?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).into_owned();
        let value = attr
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| Error::BadValue {
                cmd: "GSI",
                pos: 0,
                value: error.to_string(),
            })?
            .into_owned();
        out.insert(key, value);
    }
    Ok(out)
}

fn on(value: Option<&String>) -> bool {
    value.is_some_and(|value| value == "On")
}

fn number(value: Option<&String>) -> Option<u32> {
    let value = value?.trim();
    value
        .parse()
        .ok()
        .or_else(|| u32::from_str_radix(value.trim_start_matches("0x"), 16).ok())
        .filter(|value| *value != u32::MAX)
}

fn quick_key(value: Option<&String>) -> Option<u8> {
    value.filter(|value| value.as_str() != "None")?.parse().ok()
}

fn frequency(value: Option<&String>) -> Option<u64> {
    let text = value?.trim().trim_end_matches("MHz").trim();
    text.parse::<f64>()
        .ok()
        .map(|mhz| (mhz * 1_000_000.0).round() as u64)
}

fn mode(name: Option<&String>, screen: Option<&String>) -> Option<ScanMode> {
    let name = name.map(String::as_str).unwrap_or_default();
    let screen = screen.map(String::as_str).unwrap_or_default();
    if name.contains("Weather") || screen == "wx_alert" {
        Some(ScanMode::Weather("NORMAL"))
    } else if name.contains("Tone-Out") || screen == "tone_out" {
        Some(ScanMode::ToneOut)
    } else if name.contains("Quick Search") || screen == "quick_search" {
        Some(ScanMode::QuickSearch)
    } else if name.contains("Custom Search") || screen == "custom_search" {
        Some(ScanMode::Custom)
    } else if name.contains("Close Call") || matches!(screen, "close_call" | "cc_searching") {
        Some(ScanMode::CloseCall)
    } else if name.contains("Service") {
        Some(ScanMode::Service(""))
    } else if name.contains("Scan") || screen.ends_with("_scan") {
        Some(ScanMode::Scan)
    } else {
        None
    }
}

fn apply_element(snapshot: &mut HpSnapshot, name: &[u8], values: &HashMap<String, String>) {
    match name {
        b"ScannerInfo" => {
            snapshot.live.mode = mode(values.get("Mode"), values.get("V_Screen"));
        }
        b"MonitorList" => {
            snapshot.live.favorite = values.get("Name").cloned();
            snapshot.live.favorite_index = number(values.get("Index"));
            snapshot.live.favorite_qk = quick_key(values.get("Q_Key"));
        }
        b"System" => {
            snapshot.live.system = values.get("Name").cloned();
            snapshot.live.system_index = number(values.get("Index"));
            snapshot.live.system_qk = quick_key(values.get("Q_Key"));
            snapshot.live.system_held = on(values.get("Hold"));
        }
        b"Department" => {
            snapshot.live.group = values.get("Name").cloned();
            snapshot.live.department_index = number(values.get("Index"));
            snapshot.live.department_qk = quick_key(values.get("Q_Key"));
            snapshot.live.department_held = on(values.get("Hold"));
        }
        b"ConvFrequency" | b"TGID" | b"SrchFrequency" | b"CcHitsChannel" | b"ToneOutChannel"
        | b"WxChannel" => {
            snapshot.live.channel = values.get("Name").cloned();
            snapshot.live.channel_index = number(values.get("Index"));
            snapshot.live.channel_target = Some(match name {
                b"TGID" => "TGID",
                b"SrchFrequency" => "QS_FREQ",
                b"CcHitsChannel" => "CCHIT",
                b"ToneOutChannel" => "FTO",
                b"WxChannel" => "WX",
                _ => "CFREQ",
            });
            snapshot.live.freq_hz = frequency(values.get("Freq"));
            snapshot.live.modulation = values.get("Mod").cloned();
            snapshot.live.sub_audio = values
                .get("SAD")
                .filter(|value| value.as_str() != "None")
                .cloned();
            snapshot.live.channel_held = on(values.get("Hold"));
            snapshot.live.tgid = values
                .get("TGID")
                .filter(|value| value.as_str() != "None")
                .map(|value| {
                    value
                        .trim_start_matches("TGID:")
                        .trim_start_matches("TGID ")
                        .to_owned()
                });
        }
        b"Property" => {
            snapshot.live.volume = values.get("VOL").and_then(|value| value.parse().ok());
            snapshot.live.squelch = values.get("SQL").and_then(|value| value.parse().ok());
            snapshot.live.signal_bars = values
                .get("Sig")
                .and_then(|value| value.parse::<u8>().ok())
                .map(|value| value.min(5));
            snapshot.live.muted = values.get("Mute").is_some_and(|value| value == "Mute");
            snapshot.live.recording = on(values.get("Rec"));
            snapshot.battery_volts = values.get("Battery").and_then(|value| value.parse().ok());
        }
        b"WxMode" => {
            snapshot.weather_alert = values
                .get("Mode")
                .is_some_and(|value| value.contains("Alert"))
        }
        b"InfoArea1" | b"InfoArea2" | b"OverWrite" | b"PlainText" | b"PopupScreen" => {
            if let Some(text) = values.get("Text") {
                let display = snapshot.display.get_or_insert_with(DisplayStatus::default);
                display.lines.push(DisplayLine {
                    text: text.clone(),
                    modes: String::new(),
                });
                display.large.push(false);
            }
        }
        _ => {}
    }
}

/// Parse a complete `ScannerInfo` XML document returned by GSI or PSI.
pub fn parse_scanner_info(xml: &str) -> Result<HpSnapshot, Error> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut snapshot = HpSnapshot::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let values = attrs(&reader, &element)?;
                apply_element(&mut snapshot, element.name().as_ref(), &values);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(Error::BadValue {
                    cmd: "GSI",
                    pos: 0,
                    value: error.to_string(),
                });
            }
        }
    }
    snapshot.live.squelch_open = snapshot.live.signal_bars.unwrap_or(0) > 0;
    if let Some(display) = snapshot.display.as_mut() {
        display.battery_low = snapshot.battery_volts.map(|volts| volts < 3.3);
        display.weather_alert = snapshot.weather_alert;
        display.signal_bars = snapshot.live.signal_bars;
        display.squelch_open = snapshot.live.squelch_open;
        display.muted = snapshot.live.muted;
    }
    Ok(snapshot)
}

fn wire_frequency(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.contains('.') {
        value
            .parse::<f64>()
            .ok()
            .map(|mhz| (mhz * 1_000_000.0).round() as u64)
    } else {
        value.parse::<u64>().ok().map(|wire| wire * 100)
    }
}

/// Parse SDS `GST`, which uses the same line-pair display prefix as STS.
pub fn parse_gst(reply: &str) -> Result<WaterfallStatus, Error> {
    let fields: Vec<&str> = reply.trim_end_matches(['\r', '\n']).split(',').collect();
    if fields.first() != Some(&"GST") {
        return Err(Error::UnexpectedReply {
            expected: "GST",
            reply: reply.to_owned(),
        });
    }
    let form = fields.get(1).copied().unwrap_or_default();
    if !(5..=20).contains(&form.len()) {
        return Err(Error::BadValue {
            cmd: "GST",
            pos: 1,
            value: form.to_owned(),
        });
    }
    let tail = 2 + form.len() * 2;
    if fields.len() < tail + 12 {
        return Err(Error::MissingField {
            cmd: "GST",
            pos: tail + 11,
        });
    }
    let lines = (0..form.len())
        .map(|index| DisplayLine {
            text: fields[2 + index * 2]
                .replace('\t', ",")
                .trim_end()
                .to_owned(),
            modes: fields[3 + index * 2].to_owned(),
        })
        .collect();
    Ok(WaterfallStatus {
        display: DisplayStatus {
            large: form.bytes().map(|value| value == b'1').collect(),
            lines,
            muted: !matches!(fields[tail], "" | "0" | "Unmute"),
            ..DisplayStatus::default()
        },
        waterfall_mode: fields[tail + 3] == "1",
        marker_hz: wire_frequency(fields[tail + 4]),
        center_hz: wire_frequency(fields[tail + 7]),
        lower_hz: wire_frequency(fields[tail + 8]),
        upper_hz: wire_frequency(fields[tail + 9]),
    })
}

/// Parse one comma-separated 240-point SDS waterfall line.
pub fn parse_waterfall(reply: &str) -> Result<Vec<u16>, Error> {
    let mut fields = reply.trim_end_matches(['\r', '\n']).split(',');
    let command = fields.next();
    if !matches!(command, Some("GWF" | "PWF")) {
        return Err(Error::UnexpectedReply {
            expected: "GWF",
            reply: reply.to_owned(),
        });
    }
    let values = fields
        .filter(|field| !field.is_empty())
        .map(|field| {
            field.parse::<u16>().map_err(|_| Error::BadValue {
                cmd: "GWF",
                pos: 0,
                value: field.to_owned(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 240 {
        return Err(Error::BadValue {
            cmd: "GWF",
            pos: values.len(),
            value: "expected 240 FFT points".to_owned(),
        });
    }
    Ok(values)
}

/// Parse the SDS `GCS` charge-status line.
pub fn parse_charge_status(reply: &str) -> Result<ChargeStatus, Error> {
    let body = reply
        .trim_end_matches(['\r', '\n'])
        .strip_prefix("GCS,")
        .ok_or_else(|| Error::UnexpectedReply {
            expected: "GCS",
            reply: reply.to_owned(),
        })?;
    let mut status = ChargeStatus::default();
    for field in body.split(',') {
        if let Some(value) = field.strip_prefix("CST=") {
            status.state = value.parse().map_err(|_| Error::BadValue {
                cmd: "GCS",
                pos: 1,
                value: value.to_owned(),
            })?;
        } else if let Some(value) = field.strip_prefix("VOLT=") {
            let (millivolts, percent) = value.split_once("mV:").ok_or_else(|| Error::BadValue {
                cmd: "GCS",
                pos: 2,
                value: value.to_owned(),
            })?;
            status.millivolts = millivolts.parse().map_err(|_| Error::BadValue {
                cmd: "GCS",
                pos: 2,
                value: millivolts.to_owned(),
            })?;
            status.percent =
                percent
                    .trim_end_matches('%')
                    .parse()
                    .map_err(|_| Error::BadValue {
                        cmd: "GCS",
                        pos: 3,
                        value: percent.to_owned(),
                    })?;
        } else if let Some(value) = field.strip_prefix("CURR=") {
            status.current_ma =
                value
                    .trim_end_matches("mA")
                    .parse()
                    .map_err(|_| Error::BadValue {
                        cmd: "GCS",
                        pos: 4,
                        value: value.to_owned(),
                    })?;
        } else if let Some(value) = field.strip_prefix("TEMP=") {
            status.temperature_c =
                value
                    .trim_end_matches('C')
                    .trim()
                    .parse()
                    .map_err(|_| Error::BadValue {
                        cmd: "GCS",
                        pos: 5,
                        value: value.to_owned(),
                    })?;
        }
    }
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_info_normalizes_current_handles_and_properties() {
        let xml = r#"<?xml version="1.0"?>
<ScannerInfo Mode="Scan" V_Screen="conventional_scan">
  <MonitorList Name="Local" Index="00000001" Q_Key="3"/>
  <System Name="County" Index="0000000A" Q_Key="4" Hold="Off"/>
  <Department Name="Dispatch" Index="11" Q_Key="5" Hold="On"/>
  <ConvFrequency Name="Fire" Index="12" Q_Key="None" Freq="155.2500MHz" Mod="NFM" SAD="127.3" Hold="On"/>
  <Property VOL="8" SQL="2" Sig="4" Battery="4.01" Rec="On" Mute="Unmute"/>
</ScannerInfo>"#;
        let snapshot = parse_scanner_info(xml).unwrap();
        assert_eq!(snapshot.live.favorite.as_deref(), Some("Local"));
        assert_eq!(snapshot.live.system_index, Some(10));
        assert_eq!(snapshot.live.department_index, Some(11));
        assert_eq!(snapshot.live.channel_index, Some(12));
        assert_eq!(snapshot.live.freq_hz, Some(155_250_000));
        assert_eq!(snapshot.live.channel_target, Some("CFREQ"));
        assert!(snapshot.live.department_held);
        assert!(snapshot.live.channel_held);
        assert!(snapshot.live.recording);
        assert_eq!(snapshot.live.signal_bars, Some(4));
    }

    #[test]
    fn parses_sds_gst_waterfall_metadata() {
        let reply = "GST,00000,L1,,L2,,L3,,L4,,L5,,0,,,1,1552500,,,1553000,1552000,1554000,,";
        let status = parse_gst(reply).unwrap();
        assert!(status.waterfall_mode);
        assert_eq!(status.marker_hz, Some(155_250_000));
        assert_eq!(status.center_hz, Some(155_300_000));
        assert_eq!(status.lower_hz, Some(155_200_000));
        assert_eq!(status.upper_hz, Some(155_400_000));
    }

    #[test]
    fn waterfall_requires_exactly_240_points() {
        let values = (0..240).map(|value| value.to_string()).collect::<Vec<_>>();
        let points = parse_waterfall(&format!("GWF,{}", values.join(","))).unwrap();
        assert_eq!(points.len(), 240);
        assert_eq!(points[239], 239);
        assert!(parse_waterfall("GWF,1,2").is_err());
    }

    #[test]
    fn parses_sds_charge_details() {
        let status =
            parse_charge_status("GCS,CST=4,VOLT=4184mV:100%,CURR=0000mA,TEMP= 27.65C\n").unwrap();
        assert_eq!(status.state, 4);
        assert_eq!(status.millivolts, 4184);
        assert_eq!(status.percent, 100);
        assert_eq!(status.current_ma, 0);
        assert_eq!(status.temperature_c, 27.65);
    }
}
