//! Parser helpers for the BCT8 fixed-width legacy remote protocol.

use super::Error;
use super::live::{DisplayLine, DisplayStatus, LiveState, ScanMode};

fn wire_frequency(value: &str) -> Option<u64> {
    (value.len() == 8)
        .then(|| value.parse::<u64>().ok().map(|wire| wire * 100))
        .flatten()
}

/// Apply an `MA` channel record to normalized live state.
pub fn apply_channel(reply: &str, state: &mut LiveState) -> Result<(), Error> {
    let mut fields = reply.split_whitespace();
    let channel = fields
        .next()
        .ok_or(Error::MissingField { cmd: "MA", pos: 0 })?;
    let frequency = fields
        .next()
        .ok_or(Error::MissingField { cmd: "MA", pos: 1 })?;
    let channel = channel
        .strip_prefix('C')
        .ok_or_else(|| Error::UnexpectedReply {
            expected: "C###",
            reply: reply.to_owned(),
        })?;
    let frequency = frequency
        .strip_prefix('F')
        .and_then(wire_frequency)
        .ok_or_else(|| Error::BadValue {
            cmd: "MA",
            pos: 1,
            value: frequency.to_owned(),
        })?;
    state.channel = Some(format!("Channel {}", channel.trim_start_matches('0')));
    state.channel_index = channel.parse().ok();
    state.freq_hz = Some(frequency);
    state.tgid = None;
    Ok(())
}

/// Parse the BCT8 `MD` mode response.
pub fn parse_mode(reply: &str) -> Result<(ScanMode, bool), Error> {
    let code = reply
        .strip_prefix("MD")
        .ok_or_else(|| Error::UnexpectedReply {
            expected: "MD",
            reply: reply.to_owned(),
        })?;
    let held = matches!(
        code,
        "01" | "03" | "05" | "07" | "09" | "12" | "14" | "16" | "18" | "20" | "22"
    );
    let mode = match code {
        "00" | "01" | "13" | "14" | "17" | "18" | "21" | "22" => ScanMode::Scan,
        "02" | "03" | "11" | "12" | "15" | "16" | "19" | "20" => ScanMode::Custom,
        "04" | "05" => ScanMode::Service(""),
        "06" | "07" | "08" | "09" => ScanMode::Highway,
        "23" => ScanMode::QuickSearch,
        _ => {
            return Err(Error::BadValue {
                cmd: "MD",
                pos: 0,
                value: code.to_owned(),
            });
        }
    };
    Ok((mode, held))
}

fn bracket_fields(reply: &str) -> Vec<&str> {
    reply
        .split(['[', ']'])
        .enumerate()
        .filter_map(|(index, value)| (index % 2 == 1).then_some(value.trim()))
        .collect()
}

/// Frequency shown by an `LCD FRQ` reply, when that field contains a frequency.
pub fn parse_lcd_frequency(reply: &str) -> Option<u64> {
    let value = bracket_fields(reply).first()?.parse::<f64>().ok()?;
    Some((value * 1_000_000.0).round() as u64)
}

/// Combine targeted `LCD BNK`, `LCD CHN`, and `LCD FRQ` replies into the virtual LCD.
pub fn parse_lcd(bank: &str, channel: &str, frequency: &str) -> Result<DisplayStatus, Error> {
    let banks = bank
        .strip_prefix("BNK ")
        .ok_or_else(|| Error::UnexpectedReply {
            expected: "BNK",
            reply: bank.to_owned(),
        })?;
    let channel_fields = bracket_fields(channel);
    let frequency_fields = bracket_fields(frequency);
    if channel_fields.len() < 2 || frequency_fields.len() < 2 {
        return Err(Error::MissingField { cmd: "LCD", pos: 1 });
    }
    Ok(DisplayStatus {
        lines: vec![
            DisplayLine {
                text: format!("BANK {banks}"),
                modes: String::new(),
            },
            DisplayLine {
                text: format!("{} {}", channel_fields[0], channel_fields[1])
                    .trim()
                    .to_owned(),
                modes: String::new(),
            },
            DisplayLine {
                text: format!("{} {}", frequency_fields[0], frequency_fields[1])
                    .trim()
                    .to_owned(),
                modes: String::new(),
            },
        ],
        large: vec![false, true, true],
        ..DisplayStatus::default()
    })
}

/// Parse selected scan banks from `SB`.
pub fn parse_banks(reply: &str) -> Result<[bool; 5], Error> {
    let selected = reply
        .strip_prefix("SB ")
        .ok_or_else(|| Error::UnexpectedReply {
            expected: "SB",
            reply: reply.to_owned(),
        })?;
    Ok(std::array::from_fn(|index| {
        selected.contains(char::from(b'A' + index as u8))
    }))
}

/// True when a BCT8 line is asynchronous rather than the reply to `command`.
pub fn is_unsolicited(command: &str, line: &str) -> bool {
    line.starts_with("ID S ")
        || line.starts_with("ID E ")
        || matches!(line, "HA+" | "HP ALERT" | "PST" | "PRT")
        || (matches!(line, "+" | "-") && command != "SQ")
}

/// Fold one asynchronous BCT8 line into normalized state and display indicators.
pub fn apply_unsolicited(line: &str, state: &mut LiveState, display: &mut DisplayStatus) {
    if let Some(rest) = line.strip_prefix("ID S ") {
        if let Some((id, frequency)) = rest.rsplit_once(' ') {
            state.tgid = Some(id.to_owned());
            state.freq_hz = wire_frequency(frequency);
            state.squelch_open = true;
        }
    } else if line.starts_with("ID E ") {
        state.squelch_open = false;
    } else {
        match line {
            "+" => state.squelch_open = true,
            "-" => {
                state.squelch_open = false;
                display.weather_alert = false;
            }
            "HA+" | "HP ALERT" => display.weather_alert = true,
            "PST" => display.lines.push(DisplayLine {
                text: "PRIORITY".to_owned(),
                modes: String::new(),
            }),
            "PRT" => display.lines.retain(|line| line.text != "PRIORITY"),
            _ => {}
        }
    }
    display.squelch_open = state.squelch_open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixed_width_channel_and_modes() {
        let mut state = LiveState::default();
        apply_channel("C015 F04060125 TF DN LF AF RF N000", &mut state).unwrap();
        assert_eq!(state.channel.as_deref(), Some("Channel 15"));
        assert_eq!(state.channel_index, Some(15));
        assert_eq!(state.freq_hz, Some(406_012_500));
        assert_eq!(parse_mode("MD07").unwrap(), (ScanMode::Highway, true));
        assert_eq!(parse_mode("MD00").unwrap(), (ScanMode::Scan, false));
    }

    #[test]
    fn parses_lcd_and_scan_banks() {
        let display = parse_lcd(
            "BNK ++*--",
            "CHN [250][      ]",
            "FRQ [956.0000][           ]",
        )
        .unwrap();
        assert_eq!(display.lines[0].text, "BANK ++*--");
        assert_eq!(display.lines[1].text, "250");
        assert_eq!(display.lines[2].text, "956.0000");
        assert_eq!(
            parse_lcd_frequency("FRQ [956.0000][           ]"),
            Some(956_000_000)
        );
        assert_eq!(
            parse_banks("SB ACE").unwrap(),
            [true, false, true, false, true]
        );
    }

    #[test]
    fn routes_and_applies_unsolicited_events() {
        assert!(is_unsolicited("MD", "ID S 001234 08510125"));
        assert!(is_unsolicited("MD", "-"));
        assert!(!is_unsolicited("SQ", "-"));
        let mut state = LiveState::default();
        let mut display = DisplayStatus::default();
        apply_unsolicited("ID S 001234 08510125", &mut state, &mut display);
        assert_eq!(state.tgid.as_deref(), Some("001234"));
        assert_eq!(state.freq_hz, Some(851_012_500));
        assert!(state.squelch_open);
        apply_unsolicited("HP ALERT", &mut state, &mut display);
        assert!(display.weather_alert);
        apply_unsolicited("-", &mut state, &mut display);
        assert!(!state.squelch_open);
        assert!(!display.weather_alert);
    }
}
