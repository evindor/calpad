#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    WithUnit(f64, Unit),
    Percentage(f64),
    DateTime(f64), // Unix timestamp in seconds
    Error(String),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub id: String,
    pub display: String,
}

#[derive(Debug, Clone)]
pub struct LineResult {
    pub line_index: usize,
    pub input: String,
    pub value: Value,
    pub display: String,
}

impl Value {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::WithUnit(n, _) => Some(*n),
            Value::Percentage(f) => Some(*f),
            Value::DateTime(ts) => Some(*ts),
            _ => None,
        }
    }

    pub fn is_percentage(&self) -> bool {
        matches!(self, Value::Percentage(_))
    }
}

/// Format a value for display.
/// - Integers shown without decimals: `5` not `5.0`
/// - Decimals trimmed: `6.28` not `6.280000`
/// - Comma grouping for large numbers: `1,024`
pub fn format_value(val: &Value) -> String {
    match val {
        Value::Number(n) => format_f64(*n),
        Value::Percentage(f) => format_percentage(*f),
        Value::WithUnit(n, unit) => {
            // Special case: format display (hex, binary, etc.) — the display IS the formatted string
            if unit.id.starts_with("__fmt_") {
                return unit.display.clone();
            }
            // Currency display: single-char symbol currencies use prefix format
            if let Some(prefix) = crate::units::currency_display_prefix(&unit.id) {
                let formatted = format_f64(*n);
                if *n < 0.0 {
                    // -$10 style
                    return format!("-{}{}", prefix, format_f64(n.abs()));
                }
                return format!("{}{}", prefix, formatted);
            }
            format!("{} {}", format_f64(*n), unit.display)
        }
        Value::DateTime(ts) => format_datetime(*ts),
        Value::Error(e) => format!("Error: {e}"),
        Value::Empty => String::new(),
    }
}

/// Format a Unix timestamp as a human-readable date/time string.
/// Output format: "Mar 8, 2026 3:45 PM" (UTC)
fn format_datetime(ts: f64) -> String {
    let total_secs = ts.floor() as i64;
    let (year, month, day, hour, min, sec) = timestamp_to_civil(total_secs);

    let month_abbr = match month {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
        5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
        9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "???",
    };

    let (h12, ampm) = if hour == 0 {
        (12, "AM")
    } else if hour < 12 {
        (hour, "AM")
    } else if hour == 12 {
        (12, "PM")
    } else {
        (hour - 12, "PM")
    };

    format!("{} {}, {} {}:{:02}:{:02} {}", month_abbr, day, year, h12, min, sec, ampm)
}

/// Convert a Unix timestamp (seconds since 1970-01-01 00:00:00 UTC) to (year, month, day, hour, min, sec).
/// Uses the civil_from_days algorithm (Howard Hinnant).
fn timestamp_to_civil(ts: i64) -> (i64, u32, u32, u32, u32, u32) {
    let secs_per_day: i64 = 86400;
    let mut days = ts.div_euclid(secs_per_day);
    let day_secs = ts.rem_euclid(secs_per_day) as u32;

    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;

    // civil_from_days: days since 1970-01-01 → (year, month, day)
    days += 719468; // shift epoch from 1970-01-01 to 0000-03-01
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = (days - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y, m, d, hour, min, sec)
}

fn format_percentage(fraction: f64) -> String {
    let pct = fraction * 100.0;
    format!("{}%", format_f64(pct))
}

fn format_f64(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }

    let is_negative = n < 0.0;
    let abs = n.abs();

    // Check if it's effectively an integer
    if abs == abs.floor() && abs < 1e15 {
        let int_val = abs as i64;
        let formatted = comma_group_int(int_val);
        if is_negative {
            format!("-{formatted}")
        } else {
            formatted
        }
    } else {
        // Format with enough precision, then trim trailing zeros
        let s = format!("{:.10}", abs);
        let s = trim_trailing_zeros(&s);
        let formatted = comma_group_decimal(&s);
        if is_negative {
            format!("-{formatted}")
        } else {
            formatted
        }
    }
}

fn comma_group_int(n: i64) -> String {
    let s = n.to_string();
    insert_commas(&s)
}

fn comma_group_decimal(s: &str) -> String {
    if let Some(dot) = s.find('.') {
        let int_part = &s[..dot];
        let frac_part = &s[dot..];
        format!("{}{frac_part}", insert_commas(int_part))
    } else {
        insert_commas(s)
    }
}

fn insert_commas(s: &str) -> String {
    if s.len() <= 3 {
        return s.to_string();
    }
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

fn trim_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        return s.to_string();
    }
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}
