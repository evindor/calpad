use std::sync::LazyLock;

use ratatui::style::Style;
use ratatui::text::Span;
use regex::Regex;

use super::theme::Theme;

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        // Currency amounts: $10, €50.5K
        r"[\$\u{20ac}\u{00a3}\u{00a5}\u{20b9}\u{20bd}\u{20a9}\u{20aa}\u{0e3f}][\d,.]+[kKMBT]?",
        // Hex/binary/octal
        r"|0[xX][\da-fA-F]+|0[bB][01]+|0[oO][0-7]+",
        // Decimal numbers with optional scale/percent
        r"|\d[\d,.]*(?:[kKMBT]|%)?",
        // Keywords
        r"|\b(?:in|to|as|into|of|on|off|plus|minus|times|multiplied|divided|mod|xor|and|or|what)\b",
        // Special variables
        r"|\b(?:sum|total|average|avg|prev|now|time)\b",
        // Functions
        r"|\b(?:sqrt|cbrt|abs|round|ceil|floor|ln|log|fact|sin|cos|tan|arcsin|arccos|arctan|sinh|cosh|tanh|root|fromunix)\b",
        // Units & currency codes
        r"|\b(?:kg|g|mg|km|cm|mm|mil|meter|meters|mile|miles|foot|feet|ft|inch|inches|yard|yards|yd",
        r"|rod|chain|furlong|league|nautical|cable|hand",
        r"|pound|pounds|lb|lbs|ounce|ounces|oz|gram|grams|tonne|carat|centner|stone",
        r"|liter|liters|gallon|gallons|gal|pint|quart|cup|cups",
        r"|teaspoon|teaspoons|tsp|tablespoon|tablespoons|tbsp",
        r"|celsius|fahrenheit|kelvin|degree|degrees|radian|radians",
        r"|px|pt|em|ppi",
        r"|byte|bytes|bit|bits|[KMGT]i?[Bb]",
        r"|second|seconds|sec|minute|minutes|min|hour|hours|hr|day|days|week|weeks|month|months|year|years",
        r"|hectare|acre|are",
        r"|USD|EUR|GBP|CAD|JPY|AUD|CHF|CNY|INR|RUB|BRL|KRW|MXN|SGD|HKD|SEK|NOK|DKK|PLN|THB|NZD|ZAR|TWD|CZK|ILS)\b",
        // Degree sign
        r"|\u{00b0}",
        // Operators
        r"|[+\-*/^&|]+|<<|>>|=",
    ))
    .unwrap()
});

static KEYWORDS: &[&str] = &[
    "in", "to", "as", "into", "of", "on", "off", "plus", "minus", "times",
    "multiplied", "divided", "mod", "xor", "and", "or", "what",
    "sum", "total", "average", "avg", "prev", "now", "time",
];

static FUNCTIONS: &[&str] = &[
    "sqrt", "cbrt", "abs", "round", "ceil", "floor", "ln", "log", "fact",
    "sin", "cos", "tan", "arcsin", "arccos", "arctan", "sinh", "cosh", "tanh",
    "root", "fromunix",
];

fn classify_color(token: &str, theme: &Theme) -> ratatui::style::Color {
    let first = token.chars().next().unwrap_or(' ');

    // Currency symbols
    if "$\u{20ac}\u{00a3}\u{00a5}\u{20b9}\u{20bd}\u{20a9}\u{20aa}\u{0e3f}".contains(first) {
        return theme.number;
    }

    // Numbers
    if first.is_ascii_digit() {
        return theme.number;
    }

    // Operators
    if "+-*/^&|<>=".contains(first) || token == "<<" || token == ">>" {
        return theme.operator;
    }

    // Degree sign
    if first == '\u{00b0}' {
        return theme.unit;
    }

    let lower = token.to_lowercase();

    if KEYWORDS.contains(&lower.as_str()) {
        return theme.keyword;
    }

    if FUNCTIONS.contains(&lower.as_str()) {
        return theme.unit;
    }

    // Must be a unit/currency code
    theme.unit
}

/// Detect label pattern: "Word: rest" or "Multiple Words: rest"
fn detect_label(line: &str) -> Option<usize> {
    let colon_pos = line.find(':')?;
    let label_part = &line[..colon_pos];
    if label_part.trim().is_empty() {
        return None;
    }
    if !label_part.chars().all(|c| c.is_ascii_alphabetic() || c == ' ') {
        return None;
    }
    let after = line[colon_pos + 1..].trim();
    if after.is_empty() {
        return None;
    }
    Some(colon_pos)
}

fn tokenize_spans<'a>(line: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut last_end = 0;

    for m in TOKEN_RE.find_iter(line) {
        if m.start() > last_end {
            spans.push(Span::styled(
                &line[last_end..m.start()],
                Style::default().fg(theme.fg),
            ));
        }
        let color = classify_color(m.as_str(), theme);
        spans.push(Span::styled(
            &line[m.start()..m.end()],
            Style::default().fg(color),
        ));
        last_end = m.end();
    }

    if last_end < line.len() {
        spans.push(Span::styled(
            &line[last_end..],
            Style::default().fg(theme.fg),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(line, Style::default().fg(theme.fg)));
    }

    spans
}

pub fn highlight_line_spans<'a>(line: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    if line.is_empty() {
        return vec![Span::raw("")];
    }

    let trimmed = line.trim();

    if trimmed.starts_with('#') {
        return vec![Span::styled(
            line,
            Style::default().fg(theme.header).bold(),
        )];
    }

    if trimmed.starts_with("//") {
        return vec![Span::styled(
            line,
            Style::default().fg(theme.comment).italic(),
        )];
    }

    // Label: "Word: rest"
    if let Some(colon_pos) = detect_label(line) {
        let label_end = if line.as_bytes().get(colon_pos + 1) == Some(&b' ') {
            colon_pos + 2
        } else {
            colon_pos + 1
        };
        let label = &line[..label_end];
        let rest = &line[label_end..];
        let mut spans = vec![Span::styled(label, Style::default().fg(theme.label))];
        spans.extend(tokenize_spans(rest, theme));
        return spans;
    }

    tokenize_spans(line, theme)
}
