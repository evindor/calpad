use std::collections::HashMap;

use crate::parser::{BinOp, Const, Expr, Func, Func2};
use crate::types::{Unit, Value};
use crate::units;

pub struct EvalContext {
    pub variables: HashMap<String, Value>,
    pub currency_rates: HashMap<String, f64>,
    pub now_timestamp: Option<f64>,
}

impl EvalContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            currency_rates: HashMap::new(),
            now_timestamp: None,
        }
    }

    pub fn set(&mut self, name: &str, val: Value) {
        self.variables.insert(name.to_string(), val);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set_currency_rates(&mut self, rates: HashMap<String, f64>) {
        self.currency_rates = rates;
    }
}

pub fn eval(expr: &Expr, ctx: &EvalContext) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),

        Expr::Constant(c) => match c {
            Const::Pi => Value::Number(std::f64::consts::PI),
            Const::E => Value::Number(std::f64::consts::E),
        },

        Expr::Now => {
            let ts = if let Some(t) = ctx.now_timestamp {
                t
            } else {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64()
            };
            Value::DateTime(ts)
        }

        Expr::Variable(name) => {
            if let Some(val) = ctx.get(name) {
                val.clone()
            } else {
                Value::Error(format!("undefined variable: {name}"))
            }
        }

        Expr::UnaryMinus(inner) => {
            let v = eval(inner, ctx);
            match v {
                Value::Number(n) => Value::Number(-n),
                Value::WithUnit(n, u) => Value::WithUnit(-n, u),
                other => other,
            }
        }

        Expr::ValueWithUnit(inner, unit_id) => {
            let v = eval(inner, ctx);
            match v.as_f64() {
                Some(n) => {
                    if let Some(unit_def) = units::lookup_unit_by_id(unit_id) {
                        Value::WithUnit(n, Unit {
                            id: unit_def.id.to_string(),
                            display: unit_def.display.to_string(),
                        })
                    } else {
                        Value::Error(format!("unknown unit: {unit_id}"))
                    }
                }
                None => v,
            }
        }

        Expr::Convert(inner, target_unit_id) => {
            let v = eval(inner, ctx);
            let target_def = match units::lookup_unit_by_id(target_unit_id) {
                Some(d) => d,
                None => return Value::Error(format!("unknown target unit: {target_unit_id}")),
            };

            match &v {
                Value::WithUnit(n, source_unit) => {
                    let source_def = match units::lookup_unit_by_id(&source_unit.id) {
                        Some(d) => d,
                        None => return Value::Error(format!("unknown source unit: {}", source_unit.id)),
                    };

                    // Currency conversion uses runtime rates
                    if units::is_currency(source_def) && units::is_currency(target_def) {
                        match units::convert_currency(*n, source_def.id, target_def.id, &ctx.currency_rates) {
                            Ok(result) => Value::WithUnit(result, Unit {
                                id: target_def.id.to_string(),
                                display: target_def.display.to_string(),
                            }),
                            Err(e) => Value::Error(e),
                        }
                    } else {
                        match units::convert(*n, source_def, target_def) {
                            Ok(result) => Value::WithUnit(result, Unit {
                                id: target_def.id.to_string(),
                                display: target_def.display.to_string(),
                            }),
                            Err(e) => Value::Error(e),
                        }
                    }
                }
                Value::Number(n) => {
                    // Dimensionless number — treat as target unit (identity)
                    Value::WithUnit(*n, Unit {
                        id: target_def.id.to_string(),
                        display: target_def.display.to_string(),
                    })
                }
                other => other.clone(),
            }
        }

        Expr::FormatAs(inner, format) => {
            let v = eval(inner, ctx);
            match v.as_f64() {
                Some(n) => {
                    let formatted = units::format_display(n, *format);
                    // Return as a special formatted value — we'll use Error-free display
                    // Actually, we need a way to carry the formatted string.
                    // We'll use Value::Formatted or just return the number and handle in display.
                    // For now, let's add a Formatted variant... but that changes types.rs.
                    // Simpler: return a Value with the formatted string as display.
                    // We can use WithUnit with a special "display-only" unit.
                    Value::WithUnit(n, Unit {
                        id: format!("__fmt_{:?}", format),
                        display: formatted,
                    })
                }
                None => v,
            }
        }

        Expr::Percent(inner) => {
            let v = eval(inner, ctx);
            match v.as_f64() {
                Some(n) => Value::Percentage(n / 100.0),
                None => Value::Error("cannot compute percentage".to_string()),
            }
        }

        Expr::PercentOf(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), &val) {
                (Some(p), Value::Number(n)) => Value::Number((p / 100.0) * n),
                (Some(p), Value::WithUnit(n, u)) => Value::WithUnit((p / 100.0) * n, u.clone()),
                _ => Value::Error("cannot compute percent of".to_string()),
            }
        }

        Expr::PercentOn(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), &val) {
                (Some(p), Value::Number(n)) => Value::Number(n * (1.0 + p / 100.0)),
                (Some(p), Value::WithUnit(n, u)) => Value::WithUnit(n * (1.0 + p / 100.0), u.clone()),
                _ => Value::Error("cannot compute percent on".to_string()),
            }
        }

        Expr::PercentOff(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), &val) {
                (Some(p), Value::Number(n)) => Value::Number(n * (1.0 - p / 100.0)),
                (Some(p), Value::WithUnit(n, u)) => Value::WithUnit(n * (1.0 - p / 100.0), u.clone()),
                _ => Value::Error("cannot compute percent off".to_string()),
            }
        }

        Expr::AsPercentOf(a_expr, b_expr) => {
            let a = eval(a_expr, ctx);
            let b = eval(b_expr, ctx);
            match (a.as_f64(), b.as_f64()) {
                (Some(a_val), Some(b_val)) => {
                    if b_val == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Percentage(a_val / b_val)
                    }
                }
                _ => Value::Error("cannot compute relative percentage".to_string()),
            }
        }

        Expr::AsPercentOn(a_expr, b_expr) => {
            let a = eval(a_expr, ctx);
            let b = eval(b_expr, ctx);
            match (a.as_f64(), b.as_f64()) {
                (Some(a_val), Some(b_val)) => {
                    if b_val == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Percentage((a_val - b_val) / b_val)
                    }
                }
                _ => Value::Error("cannot compute relative percentage".to_string()),
            }
        }

        Expr::AsPercentOff(a_expr, b_expr) => {
            let a = eval(a_expr, ctx);
            let b = eval(b_expr, ctx);
            match (a.as_f64(), b.as_f64()) {
                (Some(a_val), Some(b_val)) => {
                    if b_val == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Percentage((b_val - a_val) / b_val)
                    }
                }
                _ => Value::Error("cannot compute relative percentage".to_string()),
            }
        }

        Expr::PercentOfWhat(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), val.as_f64()) {
                (Some(p), Some(v)) => {
                    let fraction = p / 100.0;
                    if fraction == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Number(v / fraction)
                    }
                }
                _ => Value::Error("cannot compute inverse percentage".to_string()),
            }
        }

        Expr::PercentOnWhat(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), val.as_f64()) {
                (Some(p), Some(v)) => {
                    let divisor = 1.0 + p / 100.0;
                    if divisor == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Number(v / divisor)
                    }
                }
                _ => Value::Error("cannot compute inverse percentage".to_string()),
            }
        }

        Expr::PercentOffWhat(pct_expr, val_expr) => {
            let pct = eval(pct_expr, ctx);
            let val = eval(val_expr, ctx);
            match (pct.as_f64(), val.as_f64()) {
                (Some(p), Some(v)) => {
                    let divisor = 1.0 - p / 100.0;
                    if divisor == 0.0 {
                        Value::Error("division by zero".to_string())
                    } else {
                        Value::Number(v / divisor)
                    }
                }
                _ => Value::Error("cannot compute inverse percentage".to_string()),
            }
        }

        Expr::BinOp(lhs, op, rhs) => {
            let l = eval(lhs, ctx);
            let r = eval(rhs, ctx);
            eval_binop(&l, *op, &r, &ctx.currency_rates)
        }

        Expr::FnCall(func, arg) => {
            // FromUnix returns a DateTime, not a Number
            if matches!(func, Func::FromUnix) {
                let v = eval(arg, ctx);
                return match v.as_f64() {
                    Some(n) => Value::DateTime(n),
                    None => Value::Error("fromunix: expected a number".to_string()),
                };
            }

            let v = eval(arg, ctx);
            match v.as_f64() {
                Some(n) => {
                    let result = match func {
                        Func::Sqrt => n.sqrt(),
                        Func::Cbrt => n.cbrt(),
                        Func::Abs => n.abs(),
                        Func::Round => n.round(),
                        Func::Ceil => n.ceil(),
                        Func::Floor => n.floor(),
                        Func::Ln => n.ln(),
                        Func::Log => n.log10(),
                        Func::Fact => factorial(n),
                        Func::Sin => n.sin(),
                        Func::Cos => n.cos(),
                        Func::Tan => n.tan(),
                        Func::Arcsin => n.asin(),
                        Func::Arccos => n.acos(),
                        Func::Arctan => n.atan(),
                        Func::Sinh => n.sinh(),
                        Func::Cosh => n.cosh(),
                        Func::Tanh => n.tanh(),
                        Func::FromUnix => unreachable!("handled above"),
                    };
                    Value::Number(result)
                }
                None => Value::Error("cannot evaluate function argument".to_string()),
            }
        }

        Expr::FnCallTwo(func, arg1, arg2) => {
            let v1 = eval(arg1, ctx);
            let v2 = eval(arg2, ctx);
            match (v1.as_f64(), v2.as_f64()) {
                (Some(a), Some(b)) => {
                    let result = match func {
                        Func2::LogBase => b.log(a),
                        Func2::Root => b.powf(1.0 / a),
                    };
                    Value::Number(result)
                }
                _ => Value::Error("cannot evaluate function arguments".to_string()),
            }
        }
    }
}

fn eval_binop(l: &Value, op: BinOp, r: &Value, currency_rates: &HashMap<String, f64>) -> Value {
    // Handle percentage on the right side: Number +/- Percentage
    if let Value::Percentage(frac) = r {
        match op {
            BinOp::Add => {
                return match l {
                    Value::Number(n) => Value::Number(n * (1.0 + frac)),
                    Value::WithUnit(n, u) => Value::WithUnit(n * (1.0 + frac), u.clone()),
                    _ => Value::Error("cannot add percentage".to_string()),
                };
            }
            BinOp::Sub => {
                return match l {
                    Value::Number(n) => Value::Number(n * (1.0 - frac)),
                    Value::WithUnit(n, u) => Value::WithUnit(n * (1.0 - frac), u.clone()),
                    _ => Value::Error("cannot subtract percentage".to_string()),
                };
            }
            _ => {}
        }
    }

    match (l, r) {
        // DateTime + WithUnit(duration) → DateTime
        (Value::DateTime(ts), Value::WithUnit(n, u)) => {
            let secs = duration_to_seconds(*n, &u.id);
            match (op, secs) {
                (BinOp::Add, Some(s)) => Value::DateTime(ts + s),
                (BinOp::Sub, Some(s)) => Value::DateTime(ts - s),
                _ => Value::Error("invalid operation on date/time".to_string()),
            }
        }

        // DateTime + Number → treat number as seconds
        (Value::DateTime(ts), Value::Number(n)) => {
            match op {
                BinOp::Add => Value::DateTime(ts + n),
                BinOp::Sub => Value::DateTime(ts - n),
                _ => Value::Error("invalid operation on date/time".to_string()),
            }
        }

        // DateTime - DateTime → duration in seconds
        (Value::DateTime(a), Value::DateTime(b)) => {
            match op {
                BinOp::Sub => {
                    let diff = a - b;
                    Value::WithUnit(diff, Unit {
                        id: "second".to_string(),
                        display: "s".to_string(),
                    })
                }
                _ => Value::Error("invalid operation on date/time".to_string()),
            }
        }

        // Both have units
        (Value::WithUnit(a, u_a), Value::WithUnit(b, u_b)) => {
            match op {
                BinOp::Add | BinOp::Sub => {
                    // Convert right to left's unit, then operate
                    let left_def = match units::lookup_unit_by_id(&u_a.id) {
                        Some(d) => d,
                        None => return numeric_binop(*a, op, *b, Some(u_a.clone())),
                    };
                    let right_def = match units::lookup_unit_by_id(&u_b.id) {
                        Some(d) => d,
                        None => return numeric_binop(*a, op, *b, Some(u_a.clone())),
                    };

                    if !units::categories_compatible(left_def.category, right_def.category) {
                        // Incompatible — just do numeric
                        return numeric_binop(*a, op, *b, None);
                    }

                    // Currency conversion uses runtime rates
                    let b_converted = if units::is_currency(left_def) && units::is_currency(right_def) {
                        match units::convert_currency(*b, right_def.id, left_def.id, currency_rates) {
                            Ok(v) => v,
                            Err(_) => return numeric_binop(*a, op, *b, Some(u_a.clone())),
                        }
                    } else {
                        match units::convert(*b, right_def, left_def) {
                            Ok(v) => v,
                            Err(_) => return numeric_binop(*a, op, *b, None),
                        }
                    };

                    let result = match op {
                        BinOp::Add => *a + b_converted,
                        BinOp::Sub => *a - b_converted,
                        _ => unreachable!(),
                    };
                    Value::WithUnit(result, u_a.clone())
                }
                BinOp::Mul | BinOp::Div => {
                    // Multiplying/dividing two unit values: strip units
                    numeric_binop(*a, op, *b, None)
                }
                _ => numeric_binop(*a, op, *b, None),
            }
        }

        // Left has unit, right is dimensionless
        (Value::WithUnit(a, u), Value::Number(b)) => {
            match op {
                BinOp::Mul | BinOp::Div | BinOp::Pow => {
                    let result = apply_numeric_op(*a, op, *b);
                    match result {
                        Some(n) => Value::WithUnit(n, u.clone()),
                        None => Value::Error("division by zero".to_string()),
                    }
                }
                BinOp::Add | BinOp::Sub => {
                    // Adding dimensionless to unit — preserve unit
                    let result = apply_numeric_op(*a, op, *b);
                    match result {
                        Some(n) => Value::WithUnit(n, u.clone()),
                        None => Value::Error("arithmetic error".to_string()),
                    }
                }
                _ => {
                    let result = apply_numeric_op(*a, op, *b);
                    match result {
                        Some(n) => Value::Number(n),
                        None => Value::Error("arithmetic error".to_string()),
                    }
                }
            }
        }

        // Left is dimensionless, right has unit
        (Value::Number(a), Value::WithUnit(b, u)) => {
            match op {
                BinOp::Mul => {
                    Value::WithUnit(*a * *b, u.clone())
                }
                _ => {
                    let result = apply_numeric_op(*a, op, *b);
                    match result {
                        Some(n) => Value::Number(n),
                        None => Value::Error("arithmetic error".to_string()),
                    }
                }
            }
        }

        // Both dimensionless
        (Value::Number(a), Value::Number(b)) => {
            let result = apply_numeric_op(*a, op, *b);
            match result {
                Some(n) => Value::Number(n),
                None => Value::Error("division by zero".to_string()),
            }
        }

        _ => Value::Error("cannot evaluate expression".to_string()),
    }
}

fn numeric_binop(a: f64, op: BinOp, b: f64, unit: Option<Unit>) -> Value {
    match apply_numeric_op(a, op, b) {
        Some(n) => match unit {
            Some(u) => Value::WithUnit(n, u),
            None => Value::Number(n),
        },
        None => Value::Error("division by zero".to_string()),
    }
}

fn apply_numeric_op(a: f64, op: BinOp, b: f64) -> Option<f64> {
    Some(match op {
        BinOp::Add => a + b,
        BinOp::Sub => a - b,
        BinOp::Mul => a * b,
        BinOp::Div => {
            if b == 0.0 {
                return None;
            }
            a / b
        }
        BinOp::Pow => a.powf(b),
        BinOp::Mod => a % b,
        BinOp::BitAnd => ((a as i64) & (b as i64)) as f64,
        BinOp::BitOr => ((a as i64) | (b as i64)) as f64,
        BinOp::BitXor => ((a as i64) ^ (b as i64)) as f64,
        BinOp::Shl => ((a as i64) << (b as i64)) as f64,
        BinOp::Shr => ((a as i64) >> (b as i64)) as f64,
    })
}

/// Convert a value with a time unit to seconds.
/// Returns None if the unit is not a time unit.
fn duration_to_seconds(value: f64, unit_id: &str) -> Option<f64> {
    let unit_def = units::lookup_unit_by_id(unit_id)?;
    if unit_def.category != units::UnitCategory::Time {
        return None;
    }
    // to_base converts to the base unit of Time, which is seconds
    Some(unit_def.to_base(value))
}

fn factorial(n: f64) -> f64 {
    if n < 0.0 || n != n.floor() || n > 170.0 {
        return f64::NAN;
    }
    let n = n as u64;
    let mut result: f64 = 1.0;
    for i in 2..=n {
        result *= i as f64;
    }
    result
}
