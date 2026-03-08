use std::collections::HashMap;

use crate::eval::{eval, EvalContext};
use crate::parser::parse_expr;
use crate::types::{format_value, LineResult, Value};

/// Evaluate a multi-line document. Each line is parsed and evaluated in order.
/// Variables persist across lines. Special keywords like `prev`, `sum`, `total`,
/// `average`, `avg` are handled. Comments (`//`) and headers (`#`) are skipped.
/// Labels (`Label: expr`) are supported. Variable assignment (`name = expr`) is supported.
pub fn evaluate_document(input: &str) -> Vec<LineResult> {
    evaluate_document_with_rates(input, &HashMap::new())
}

/// Evaluate a multi-line document with currency exchange rates.
/// Rates map currency_id (e.g. "EUR") to rate relative to USD
/// (e.g. EUR=0.92 means 1 USD = 0.92 EUR).
pub fn evaluate_document_with_rates(input: &str, rates: &HashMap<String, f64>) -> Vec<LineResult> {
    let lines: Vec<&str> = input.lines().collect();
    let mut results: Vec<LineResult> = Vec::with_capacity(lines.len());
    let mut ctx = EvalContext::new();
    ctx.set_currency_rates(rates.clone());
    let mut prev_value: Option<Value> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Empty line
        if trimmed.is_empty() {
            results.push(LineResult {
                line_index: i,
                input: line.to_string(),
                value: Value::Empty,
                display: String::new(),
            });
            continue;
        }

        // Comment line
        if trimmed.starts_with("//") {
            results.push(LineResult {
                line_index: i,
                input: line.to_string(),
                value: Value::Empty,
                display: String::new(),
            });
            continue;
        }

        // Header line
        if trimmed.starts_with('#') {
            results.push(LineResult {
                line_index: i,
                input: line.to_string(),
                value: Value::Empty,
                display: String::new(),
            });
            continue;
        }

        // Check for special aggregate keywords
        let lower = trimmed.to_lowercase();
        if lower == "sum" || lower == "total" {
            let (sum, unit) = compute_aggregate_with_unit(&results, i, Aggregate::Sum, &ctx.currency_rates);
            let val = match unit {
                Some(u) => Value::WithUnit(sum, u),
                None => Value::Number(sum),
            };
            let display = format_value(&val);
            prev_value = Some(val.clone());
            results.push(LineResult {
                line_index: i,
                input: line.to_string(),
                value: val,
                display,
            });
            continue;
        }

        if lower == "average" || lower == "avg" {
            let (avg, unit) = compute_aggregate_with_unit(&results, i, Aggregate::Average, &ctx.currency_rates);
            let val = match unit {
                Some(u) => Value::WithUnit(avg, u),
                None => Value::Number(avg),
            };
            let display = format_value(&val);
            prev_value = Some(val.clone());
            results.push(LineResult {
                line_index: i,
                input: line.to_string(),
                value: val,
                display,
            });
            continue;
        }

        // Set `prev` in context
        if let Some(ref pv) = prev_value {
            ctx.set("prev", pv.clone());
        }

        // Check for variable assignment: `name = expr`
        if let Some(expr_str) = try_variable_assignment(trimmed) {
            let var_name = expr_str.0;
            let expr_text = expr_str.1;

            match parse_expr(expr_text) {
                Ok(ast) => {
                    let val = eval(&ast, &ctx);
                    let display = format_value(&val);
                    ctx.set(&var_name, val.clone());
                    prev_value = Some(val.clone());
                    results.push(LineResult {
                        line_index: i,
                        input: line.to_string(),
                        value: val,
                        display,
                    });
                }
                Err(e) => {
                    let val = Value::Error(e);
                    let display = format_value(&val);
                    results.push(LineResult {
                        line_index: i,
                        input: line.to_string(),
                        value: val,
                        display,
                    });
                }
            }
            continue;
        }

        // Check for label: `Label: expr`
        let expr_text = if let Some(colon_pos) = try_label(trimmed) {
            &trimmed[colon_pos + 1..]
        } else {
            trimmed
        };

        match parse_expr(expr_text) {
            Ok(ast) => {
                let val = eval(&ast, &ctx);
                let display = format_value(&val);
                prev_value = Some(val.clone());
                results.push(LineResult {
                    line_index: i,
                    input: line.to_string(),
                    value: val,
                    display,
                });
            }
            Err(e) => {
                let val = Value::Error(e);
                let display = format_value(&val);
                results.push(LineResult {
                    line_index: i,
                    input: line.to_string(),
                    value: val,
                    display,
                });
            }
        }
    }

    results
}

/// Try to parse `name = expr`. Returns Some((name, expr_str)) if it matches.
/// The name must be a simple identifier (alphabetic + underscores).
/// We must be careful not to match things like `0xFF = ...` or expressions.
fn try_variable_assignment(line: &str) -> Option<(String, &str)> {
    let eq_pos = line.find('=')?;

    // Name is everything before '='
    let name_part = line[..eq_pos].trim();

    // Must be a valid identifier: starts with alpha/underscore, rest is alphanumeric/underscore
    if name_part.is_empty() {
        return None;
    }

    let mut chars = name_part.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }

    // Don't match keywords
    let lower = name_part.to_lowercase();
    if matches!(
        lower.as_str(),
        "sum" | "total" | "average" | "avg" | "prev" | "pi" | "e"
            | "sqrt" | "cbrt" | "abs" | "round" | "ceil" | "floor"
            | "ln" | "log" | "fact" | "sin" | "cos" | "tan"
            | "arcsin" | "arccos" | "arctan" | "sinh" | "cosh" | "tanh"
            | "root" | "mod" | "plus" | "minus" | "times"
            | "now" | "time" | "fromunix"
    ) {
        return None;
    }

    let expr_part = &line[eq_pos + 1..];
    Some((name_part.to_string(), expr_part))
}

/// Try to detect a label pattern: `Word: expr` or `Multiple Words: expr`.
/// Returns Some(colon_position) if it looks like a label.
/// A label is: one or more words (alpha + spaces), followed by colon, followed by something.
fn try_label(line: &str) -> Option<usize> {
    let colon_pos = line.find(':')?;

    // The part before the colon should be a label (words)
    let label_part = &line[..colon_pos];

    // Must not be empty
    if label_part.trim().is_empty() {
        return None;
    }

    // Must be only alphabetic characters and spaces
    if !label_part
        .chars()
        .all(|c| c.is_ascii_alphabetic() || c == ' ')
    {
        return None;
    }

    // There must be something after the colon
    let after = line[colon_pos + 1..].trim();
    if after.is_empty() {
        return None;
    }

    Some(colon_pos)
}

enum Aggregate {
    Sum,
    Average,
}

/// Compute sum or average of numeric lines above, stopping at the first blank line.
/// Returns (result, optional_unit) — unit is taken from the first value that has one.
/// If currencies are mixed, values are converted to the first currency's unit.
fn compute_aggregate_with_unit(
    results: &[LineResult],
    _current_index: usize,
    kind: Aggregate,
    currency_rates: &HashMap<String, f64>,
) -> (f64, Option<crate::types::Unit>) {
    let mut values: Vec<f64> = Vec::new();
    let mut target_unit: Option<crate::types::Unit> = None;

    // Walk backwards from the line before current, collect in reverse
    let mut collected: Vec<&LineResult> = Vec::new();
    for j in (0..results.len()).rev() {
        let r = &results[j];
        match &r.value {
            Value::Empty => break,
            Value::Number(_) | Value::WithUnit(_, _) => collected.push(r),
            _ => {}
        }
    }
    collected.reverse(); // restore original order

    for r in &collected {
        match &r.value {
            Value::WithUnit(n, u) => {
                if target_unit.is_none() {
                    target_unit = Some(u.clone());
                    values.push(*n);
                } else if let Some(ref tu) = target_unit {
                    // Try converting to target unit if both are currencies
                    let src_def = crate::units::lookup_unit_by_id(&u.id);
                    let tgt_def = crate::units::lookup_unit_by_id(&tu.id);
                    if let (Some(sd), Some(td)) = (src_def, tgt_def) {
                        if crate::units::is_currency(sd) && crate::units::is_currency(td) {
                            let converted = crate::units::convert_currency(*n, sd.id, td.id, currency_rates)
                                .unwrap_or(*n);
                            values.push(converted);
                        } else if crate::units::categories_compatible(sd.category, td.category) {
                            let converted = crate::units::convert(*n, sd, td).unwrap_or(*n);
                            values.push(converted);
                        } else {
                            values.push(*n);
                        }
                    } else {
                        values.push(*n);
                    }
                }
            }
            Value::Number(n) => values.push(*n),
            _ => {}
        }
    }

    if values.is_empty() {
        return (0.0, None);
    }

    let sum: f64 = values.iter().sum();
    match kind {
        Aggregate::Sum => (sum, target_unit),
        Aggregate::Average => (sum / values.len() as f64, target_unit),
    }
}
