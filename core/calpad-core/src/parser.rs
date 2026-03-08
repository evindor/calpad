
use crate::units;

/// AST node for expressions.
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    Constant(Const),
    Variable(String),
    UnaryMinus(Box<Expr>),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    FnCall(Func, Box<Expr>),
    /// log base n (value): FnCallTwo(LogBase, base, value)
    FnCallTwo(Func2, Box<Expr>, Box<Expr>),
    /// Number with unit: "5 kg" → ValueWithUnit(Number(5), "kg")
    ValueWithUnit(Box<Expr>, String),
    /// Conversion: "expr in meters" → Convert(expr, "meters")
    Convert(Box<Expr>, String),
    /// Display format conversion: "255 in hex" → FormatAs(expr, format)
    FormatAs(Box<Expr>, units::DisplayFormat),
    /// Current date/time: `now` or `time`
    Now,
    /// Percentage literal: 5% → Percent(Number(5))
    Percent(Box<Expr>),
    /// "20% of 10" → PercentOf(20, 10)
    PercentOf(Box<Expr>, Box<Expr>),
    /// "5% on 30" → PercentOn(5, 30)  means 30 + 5% of 30
    PercentOn(Box<Expr>, Box<Expr>),
    /// "6% off 40" → PercentOff(6, 40) means 40 - 6% of 40
    PercentOff(Box<Expr>, Box<Expr>),
    /// "50 as a % of 100" → AsPercentOf(50, 100)
    AsPercentOf(Box<Expr>, Box<Expr>),
    /// "70 as a % on 20" → AsPercentOn(70, 20)
    AsPercentOn(Box<Expr>, Box<Expr>),
    /// "20 as a % off 70" → AsPercentOff(20, 70)
    AsPercentOff(Box<Expr>, Box<Expr>),
    /// "5% of what is 6" → PercentOfWhat(5, 6)
    PercentOfWhat(Box<Expr>, Box<Expr>),
    /// "5% on what is 6" → PercentOnWhat(5, 6)
    PercentOnWhat(Box<Expr>, Box<Expr>),
    /// "5% off what is 6" → PercentOffWhat(5, 6)
    PercentOffWhat(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Pi,
    E,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone, Copy)]
pub enum Func {
    Sqrt,
    Cbrt,
    Abs,
    Round,
    Ceil,
    Floor,
    Ln,
    Log,    // log base 10
    Fact,
    Sin,
    Cos,
    Tan,
    Arcsin,
    Arccos,
    Arctan,
    Sinh,
    Cosh,
    Tanh,
    FromUnix,
}

#[derive(Debug, Clone, Copy)]
pub enum Func2 {
    LogBase,
    Root,
}

/// Parse a full expression string into an AST.
pub fn parse_expr(input: &str) -> Result<Expr, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("empty expression".to_string());
    }
    let mut parser = Parser::new(input);
    let expr = parser.parse_full_expr()?;
    parser.skip_ws();
    if parser.pos < parser.input.len() {
        return Err(format!(
            "unexpected trailing input: '{}'",
            &parser.input[parser.pos..]
        ));
    }
    Ok(expr)
}

// ---------------------------------------------------------------------------
// Hand-written recursive descent / Pratt parser
// winnow is available but for a Pratt parser with precedence climbing,
// a hand-rolled approach is cleaner and more controllable.
// ---------------------------------------------------------------------------

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn peek_word(&self) -> &'a str {
        let rem = self.remaining();
        let end = rem
            .find(|c: char| !c.is_ascii_alphabetic() && c != '_')
            .unwrap_or(rem.len());
        &rem[..end]
    }

    fn eat_char(&mut self, c: char) -> bool {
        if self.peek_char() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }

    /// Parse a full expression, including trailing conversion ("in meters", "to kg", etc.)
    /// and "as a % of/on/off" patterns.
    fn parse_full_expr(&mut self) -> Result<Expr, String> {
        let expr = self.parse_expr(0)?;
        // Check for "as a % of/on/off" before conversion
        let expr = self.try_parse_as_percent(expr)?;
        self.try_parse_conversion(expr)
    }

    /// Try to parse "as a % of EXPR", "as a % on EXPR", "as a % off EXPR".
    fn try_parse_as_percent(&mut self, expr: Expr) -> Result<Expr, String> {
        let saved_pos = self.pos;
        self.skip_ws();

        let rem = self.remaining();

        // Match "as a % of", "as a % on", "as a % off" (case-insensitive for "as a")
        if !rem.starts_with("as ") && !rem.starts_with("As ") && !rem.starts_with("AS ") {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // Save and try to match the full pattern
        let try_pos = self.pos;
        self.pos += 3; // skip "as "
        self.skip_ws();

        let rem2 = self.remaining();
        if !(rem2.starts_with("a ") || rem2.starts_with("A ")) {
            self.pos = saved_pos;
            return Ok(expr);
        }
        self.pos += 2; // skip "a "
        self.skip_ws();

        let rem3 = self.remaining();
        if !rem3.starts_with('%') {
            self.pos = saved_pos;
            return Ok(expr);
        }
        self.pos += 1; // skip "%"
        self.skip_ws();

        let rem4 = self.remaining();
        let (kind, skip_len) = if rem4.starts_with("off ") || rem4 == "off" {
            ("off", 3)
        } else if rem4.starts_with("on ") || rem4 == "on" {
            ("on", 2)
        } else if rem4.starts_with("of ") || rem4 == "of" {
            ("of", 2)
        } else {
            self.pos = saved_pos;
            return Ok(expr);
        };

        self.pos += skip_len;
        self.skip_ws();

        if self.remaining().is_empty() {
            self.pos = try_pos;
            self.pos = saved_pos;
            return Ok(expr);
        }

        let rhs = self.parse_expr(0)?;

        Ok(match kind {
            "of" => Expr::AsPercentOf(Box::new(expr), Box::new(rhs)),
            "on" => Expr::AsPercentOn(Box::new(expr), Box::new(rhs)),
            "off" => Expr::AsPercentOff(Box::new(expr), Box::new(rhs)),
            _ => unreachable!(),
        })
    }

    /// Try to parse a trailing conversion keyword (in/to/as/into) + unit or format.
    fn try_parse_conversion(&mut self, expr: Expr) -> Result<Expr, String> {
        let saved_pos = self.pos;
        self.skip_ws();

        let rem = self.remaining();
        // Check for conversion keywords: "in", "to", "as", "into"
        let keyword_len = if rem.starts_with("into ")
            || (rem.starts_with("into") && rem.len() == 4)
        {
            4
        } else if rem.starts_with("in ")
            || (rem.starts_with("in") && rem.len() == 2)
        {
            2
        } else if rem.starts_with("to ")
            || (rem.starts_with("to") && rem.len() == 2)
        {
            2
        } else if (rem.starts_with("as ")
            || (rem.starts_with("as") && rem.len() == 2))
            && !is_as_percent_pattern(rem)
        {
            2
        } else {
            self.pos = saved_pos;
            return Ok(expr);
        };

        self.pos += keyword_len;
        self.skip_ws();

        let target_rem = self.remaining();
        if target_rem.is_empty() {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // Check for display format first (hex, binary, octal, scientific)
        let target_word = {
            let end = target_rem
                .find(|c: char| !c.is_ascii_alphabetic())
                .unwrap_or(target_rem.len());
            &target_rem[..end]
        };

        if let Some(fmt) = units::match_display_format(target_word) {
            self.pos += target_word.len();
            return Ok(Expr::FormatAs(Box::new(expr), fmt));
        }

        // Try to match a unit phrase
        if let Some((unit_def, matched_len)) = units::match_unit_phrase(target_rem) {
            self.pos += matched_len;
            return Ok(Expr::Convert(
                Box::new(expr),
                unit_def.id.to_string(),
            ));
        }

        // No match — backtrack
        self.pos = saved_pos;
        Ok(expr)
    }

    // Precedence levels (higher = tighter binding):
    // 0: bitwise or, xor
    // 1: bitwise and
    // 2: shifts << >>
    // 3: +, -
    // 4: *, /, mod
    // 5: ^ (right-assoc)
    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;

        loop {
            self.skip_ws();
            if let Some((op, prec, right_assoc)) = self.peek_binop() {
                if prec < min_prec {
                    break;
                }
                self.consume_binop(&op);
                let next_prec = if right_assoc { prec } else { prec + 1 };
                let rhs = self.parse_expr(next_prec)?;
                lhs = Expr::BinOp(Box::new(lhs), op, Box::new(rhs));
            } else {
                // Check for compound units: "1 meter 20 cm" (implicit addition)
                // This is: number+unit followed by number+unit with compatible categories
                if self.is_compound_unit_candidate(&lhs) {
                    let prec = 3u8; // same as addition
                    if prec >= min_prec {
                        let rhs = self.parse_unary()?;
                        lhs = Expr::BinOp(Box::new(lhs), BinOp::Add, Box::new(rhs));
                        continue;
                    }
                }

                // Check for implicit multiplication: number followed by '(' or a function/variable
                self.skip_ws();
                if self.is_implicit_mul_candidate(&lhs) {
                    let prec = 4u8;
                    if prec >= min_prec {
                        let rhs = self.parse_expr(prec + 1)?;
                        lhs = Expr::BinOp(Box::new(lhs), BinOp::Mul, Box::new(rhs));
                        continue;
                    }
                }
                break;
            }
        }

        Ok(lhs)
    }

    fn is_compound_unit_candidate(&self, lhs: &Expr) -> bool {
        // Only if lhs is a ValueWithUnit and next thing starts with a digit
        if !matches!(lhs, Expr::ValueWithUnit(..)) {
            return false;
        }
        let saved = self.pos;
        let rem = &self.input[saved..];
        let rem = rem.trim_start();
        if rem.is_empty() {
            return false;
        }
        // Must start with a digit
        rem.as_bytes()[0].is_ascii_digit()
    }

    fn is_implicit_mul_candidate(&self, _lhs: &Expr) -> bool {
        let rem = self.remaining();
        if rem.starts_with('(') {
            return match _lhs {
                Expr::Number(_) | Expr::Constant(_) | Expr::Variable(_) => true,
                _ => false,
            };
        }
        false
    }

    fn peek_binop(&self) -> Option<(BinOp, u8, bool)> {
        let rem = self.remaining();

        // Two-char operators first
        if rem.starts_with("<<") {
            return Some((BinOp::Shl, 2, false));
        }
        if rem.starts_with(">>") {
            return Some((BinOp::Shr, 2, false));
        }

        // Word operators — but NOT conversion keywords
        let word = self.peek_word();
        match word {
            "plus" => return Some((BinOp::Add, 3, false)),
            "minus" => return Some((BinOp::Sub, 3, false)),
            "times" => return Some((BinOp::Mul, 4, false)),
            "mod" => return Some((BinOp::Mod, 4, false)),
            "xor" => return Some((BinOp::BitXor, 0, false)),
            _ => {}
        }

        // Multi-word operators
        if rem.starts_with("multiplied by") || rem.starts_with("multiplied By") {
            return Some((BinOp::Mul, 4, false));
        }
        if rem.starts_with("divided by") || rem.starts_with("divide by") {
            return Some((BinOp::Div, 4, false));
        }

        // Single-char operators
        match rem.chars().next()? {
            '+' => Some((BinOp::Add, 3, false)),
            '-' => Some((BinOp::Sub, 3, false)),
            '*' => Some((BinOp::Mul, 4, false)),
            '/' => Some((BinOp::Div, 4, false)),
            '^' => Some((BinOp::Pow, 5, true)),
            '&' => Some((BinOp::BitAnd, 1, false)),
            '|' => Some((BinOp::BitOr, 0, false)),
            _ => None,
        }
    }

    fn consume_binop(&mut self, op: &BinOp) {
        let rem = self.remaining();
        match op {
            BinOp::Shl => { self.pos += 2; }
            BinOp::Shr => { self.pos += 2; }
            BinOp::Add => {
                if rem.starts_with("plus") {
                    self.pos += 4;
                } else {
                    self.pos += 1;
                }
            }
            BinOp::Sub => {
                if rem.starts_with("minus") {
                    self.pos += 5;
                } else {
                    self.pos += 1;
                }
            }
            BinOp::Mul => {
                if rem.starts_with("multiplied by") {
                    self.pos += 13;
                } else if rem.starts_with("times") {
                    self.pos += 5;
                } else {
                    self.pos += 1; // *
                }
            }
            BinOp::Div => {
                if rem.starts_with("divided by") {
                    self.pos += 10;
                } else if rem.starts_with("divide by") {
                    self.pos += 9;
                } else {
                    self.pos += 1; // /
                }
            }
            BinOp::Pow => { self.pos += 1; }
            BinOp::Mod => { self.pos += 3; }
            BinOp::BitAnd => { self.pos += 1; }
            BinOp::BitOr => { self.pos += 1; }
            BinOp::BitXor => { self.pos += 3; }
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.skip_ws();

        // Unary minus
        if self.eat_char('-') {
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryMinus(Box::new(operand)));
        }

        // Unary plus (just skip it)
        if self.peek_char() == Some('+') && !self.remaining().starts_with("++") {
            self.pos += 1;
            return self.parse_unary();
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let atom = self.parse_atom()?;
        // Try scale suffix first (k, M, thousand, million, billion)
        let atom = self.try_apply_scale(atom)?;
        // Try percent suffix
        let atom = self.try_parse_percent(atom)?;
        // Try unit attachment
        self.try_attach_unit(atom)
    }

    /// Check for scale suffixes: k, M, thousand, million, billion.
    /// Only applies to number-like expressions.
    fn try_apply_scale(&mut self, expr: Expr) -> Result<Expr, String> {
        let is_number_like = matches!(&expr, Expr::Number(_));
        if !is_number_like {
            return Ok(expr);
        }

        let saved_pos = self.pos;
        // Don't skip whitespace for single-char suffixes (k, M) — they must be adjacent or after space
        // But do allow space for word suffixes (thousand, million, billion)
        let rem_no_ws = self.remaining();

        // Check for single-char scale suffixes immediately adjacent (no space): 2k, 5M, 3B, 1T
        if let Some(first) = rem_no_ws.chars().next() {
            let scale = match first {
                'k' | 'K' => Some(1_000.0),
                'M' => Some(1_000_000.0),
                'B' => Some(1_000_000_000.0),
                'T' => Some(1_000_000_000_000.0),
                _ => None,
            };
            if let Some(mult) = scale {
                let after = &rem_no_ws[1..];
                let at_boundary = after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric();
                // k/K: reject if followed by alpha (kg, km, KB, KiB)
                // M: reject if followed by alphanumeric (MB, MiB, Mb)
                // B: reject if followed by alpha (bytes, bits, BRL) but not digits
                // T: reject if followed by alpha (TB, TiB, THB, TRY, TWD)
                if at_boundary {
                    self.pos += 1;
                    if let Expr::Number(n) = expr {
                        return Ok(Expr::Number(n * mult));
                    }
                }
            }
        }

        // Check for word scale suffixes (allow space before): thousand, million, billion, trillion
        self.skip_ws();
        let rem = self.remaining();
        let word = {
            let end = rem.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(rem.len());
            &rem[..end]
        };

        let multiplier = match word.to_lowercase().as_str() {
            "thousand" => Some(1_000.0),
            "mil" | "million" => Some(1_000_000.0),
            "bil" | "billion" => Some(1_000_000_000.0),
            "trillion" => Some(1_000_000_000_000.0),
            _ => None,
        };

        if let Some(mult) = multiplier {
            // Make sure the word ends at a boundary
            let after = &rem[word.len()..];
            if after.is_empty() || !after.chars().next().unwrap().is_ascii_alphanumeric() {
                self.pos += word.len();
                if let Expr::Number(n) = expr {
                    return Ok(Expr::Number(n * mult));
                }
            }
        }

        self.pos = saved_pos;
        Ok(expr)
    }

    /// Check for `%` suffix and percentage operations (of, on, off, of what is, etc.)
    fn try_parse_percent(&mut self, expr: Expr) -> Result<Expr, String> {
        let is_number_like = matches!(&expr, Expr::Number(_) | Expr::Constant(_) | Expr::UnaryMinus(_));
        if !is_number_like {
            return Ok(expr);
        }

        let saved_pos = self.pos;
        self.skip_ws();

        if !self.eat_char('%') {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // We have a percentage. Now check what follows.
        let pct_expr = expr;

        self.skip_ws();
        let rem = self.remaining();

        // Check for "of what is", "on what is", "off what is"
        if rem.starts_with("of ") || rem.starts_with("on ") || rem.starts_with("off ") {
            let (keyword, kw_len) = if rem.starts_with("off ") {
                ("off", 3)
            } else if rem.starts_with("on ") {
                ("on", 2)
            } else {
                ("of", 2)
            };

            let after_kw_pos = self.pos + kw_len;
            // Check for "what is" after keyword
            let after_kw = &self.input[after_kw_pos..].trim_start();
            if after_kw.starts_with("what ") || after_kw.starts_with("What ") {
                // Find "what" position
                let ws_skip = self.input[after_kw_pos..].len() - after_kw.len();
                let what_pos = after_kw_pos + ws_skip;
                let after_what = &self.input[what_pos + 4..].trim_start();
                if after_what.starts_with("is ") || after_what.starts_with("Is ") || after_what.starts_with("IS ") {
                    let ws2 = self.input[what_pos + 4..].len() - after_what.len();
                    let is_pos = what_pos + 4 + ws2;
                    self.pos = is_pos + 2; // skip "is"
                    self.skip_ws();
                    let val = self.parse_expr(0)?;
                    return Ok(match keyword {
                        "of" => Expr::PercentOfWhat(Box::new(pct_expr), Box::new(val)),
                        "on" => Expr::PercentOnWhat(Box::new(pct_expr), Box::new(val)),
                        "off" => Expr::PercentOffWhat(Box::new(pct_expr), Box::new(val)),
                        _ => unreachable!(),
                    });
                }
            }

            // Just "of EXPR", "on EXPR", "off EXPR"
            self.pos += kw_len;
            self.skip_ws();
            let val = self.parse_expr(0)?;
            return Ok(match keyword {
                "of" => Expr::PercentOf(Box::new(pct_expr), Box::new(val)),
                "on" => Expr::PercentOn(Box::new(pct_expr), Box::new(val)),
                "off" => Expr::PercentOff(Box::new(pct_expr), Box::new(val)),
                _ => unreachable!(),
            });
        }

        // Just a bare percentage: 5%
        Ok(Expr::Percent(Box::new(pct_expr)))
    }

    /// After parsing a number, check if followed by a unit phrase.
    /// If so, wrap in ValueWithUnit.
    fn try_attach_unit(&mut self, expr: Expr) -> Result<Expr, String> {
        // Only attach units to numbers (or expressions that evaluate to numbers)
        let is_number_like = matches!(&expr, Expr::Number(_) | Expr::Constant(_));
        if !is_number_like {
            return Ok(expr);
        }

        let saved_pos = self.pos;
        self.skip_ws();

        let rem = self.remaining();
        if rem.is_empty() {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // Don't try to match a unit if the next thing is clearly an operator
        let first = rem.chars().next().unwrap();
        if matches!(first, '+' | '-' | '*' | '/' | '^' | '(' | ')' | '&' | '|' | '<' | '>' | '%') {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // Don't match "in"/"to"/"as"/"into" as units — they are conversion keywords
        if is_conversion_keyword(rem) {
            self.pos = saved_pos;
            return Ok(expr);
        }

        // Try to match a unit phrase
        if let Some((unit_def, matched_len)) = units::match_unit_phrase(rem) {
            self.pos += matched_len;
            return Ok(Expr::ValueWithUnit(
                Box::new(expr),
                unit_def.id.to_string(),
            ));
        }

        self.pos = saved_pos;
        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        self.skip_ws();

        let rem = self.remaining();
        if rem.is_empty() {
            return Err("unexpected end of input".to_string());
        }

        // Check for currency prefix symbols ($, €, £, ¥, ₹, ₽, ₩, ₪, ฿, 元)
        // Also multi-char prefixes like C$, A$, R$, S$, HK$, NZ$, NT$
        if let Some((default_currency_id, prefix_len)) = self.try_match_currency_prefix() {
            let saved = self.pos;
            self.pos += prefix_len;
            // Parse the number that follows
            if self.pos < self.input.len() && (self.input.as_bytes()[self.pos].is_ascii_digit() || self.input.as_bytes()[self.pos] == b'.') {
                let num_expr = self.parse_number()?;
                // Apply scale suffix (e.g., $2k)
                let num_expr = self.try_apply_scale(num_expr)?;
                // Check if an explicit currency code follows (which overrides the prefix symbol)
                let final_unit_id = self.try_match_currency_code_suffix()
                    .unwrap_or(default_currency_id);
                return Ok(Expr::ValueWithUnit(Box::new(num_expr), final_unit_id));
            }
            // Not followed by a number — backtrack
            self.pos = saved;
        }

        // Parenthesized expression
        if rem.starts_with('(') {
            self.pos += 1;
            let expr = self.parse_full_expr()?;
            self.skip_ws();
            if !self.eat_char(')') {
                return Err("missing closing parenthesis".to_string());
            }
            return Ok(expr);
        }

        // Number literals (including 0x, 0b, 0o)
        if rem.starts_with("0x") || rem.starts_with("0X") {
            return self.parse_hex();
        }
        if rem.starts_with("0b") || rem.starts_with("0B") {
            return self.parse_bin();
        }
        if rem.starts_with("0o") || rem.starts_with("0O") {
            return self.parse_oct();
        }

        if rem.as_bytes()[0].is_ascii_digit() || rem.starts_with('.') {
            return self.parse_number();
        }

        // Keywords / functions / constants / variables
        let word = self.peek_word().to_string();
        if word.is_empty() {
            // Could be ° symbol or similar
            if rem.starts_with('\u{00b0}') {
                // degree symbol on its own — not valid as atom
                return Err(format!("unexpected character: '{}'", rem.chars().next().unwrap()));
            }
            return Err(format!("unexpected character: '{}'", rem.chars().next().unwrap()));
        }

        match word.as_str() {
            "pi" | "PI" => {
                self.pos += word.len();
                Ok(Expr::Constant(Const::Pi))
            }
            "e" if !self.remaining()[word.len()..].starts_with(|c: char| c.is_ascii_alphabetic()) => {
                self.pos += 1;
                Ok(Expr::Constant(Const::E))
            }
            "now" | "time" => {
                self.pos += word.len();
                Ok(Expr::Now)
            }
            "fromunix" => self.parse_func_call(Func::FromUnix),
            "sqrt" => self.parse_func_call(Func::Sqrt),
            "cbrt" => self.parse_func_call(Func::Cbrt),
            "abs" => self.parse_func_call(Func::Abs),
            "round" => self.parse_func_call(Func::Round),
            "ceil" => self.parse_func_call(Func::Ceil),
            "floor" => self.parse_func_call(Func::Floor),
            "ln" => self.parse_func_call(Func::Ln),
            "fact" => self.parse_func_call(Func::Fact),
            "sin" => {
                if self.remaining().starts_with("sinh") {
                    self.parse_func_call(Func::Sinh)
                } else {
                    self.parse_func_call(Func::Sin)
                }
            }
            "sinh" => self.parse_func_call(Func::Sinh),
            "cos" => {
                if self.remaining().starts_with("cosh") {
                    self.parse_func_call(Func::Cosh)
                } else {
                    self.parse_func_call(Func::Cos)
                }
            }
            "cosh" => self.parse_func_call(Func::Cosh),
            "tan" => {
                if self.remaining().starts_with("tanh") {
                    self.parse_func_call(Func::Tanh)
                } else {
                    self.parse_func_call(Func::Tan)
                }
            }
            "tanh" => self.parse_func_call(Func::Tanh),
            "arcsin" => self.parse_func_call(Func::Arcsin),
            "arccos" => self.parse_func_call(Func::Arccos),
            "arctan" => self.parse_func_call(Func::Arctan),
            "log" => self.parse_log_call(),
            "root" => self.parse_root_call(),
            _ => {
                // Variable name
                self.pos += word.len();
                Ok(Expr::Variable(word))
            }
        }
    }

    fn parse_func_call(&mut self, func: Func) -> Result<Expr, String> {
        let name_len = match func {
            Func::Sqrt => 4,
            Func::Cbrt => 4,
            Func::Abs => 3,
            Func::Round => 5,
            Func::Ceil => 4,
            Func::Floor => 5,
            Func::Ln => 2,
            Func::Log => 3,
            Func::Fact => 4,
            Func::Sin => 3,
            Func::Cos => 3,
            Func::Tan => 3,
            Func::Arcsin => 6,
            Func::Arccos => 6,
            Func::Arctan => 6,
            Func::Sinh => 4,
            Func::Cosh => 4,
            Func::Tanh => 4,
            Func::FromUnix => 8,
        };
        self.pos += name_len;
        self.skip_ws();
        let arg = self.parse_unary()?;
        Ok(Expr::FnCall(func, Box::new(arg)))
    }

    fn parse_log_call(&mut self) -> Result<Expr, String> {
        self.pos += 3; // consume "log"
        self.skip_ws();

        let saved_pos = self.pos;

        if self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
            let base = self.parse_number()?;
            self.skip_ws();
            if self.peek_char() == Some('(') {
                self.pos += 1; // (
                let val = self.parse_full_expr()?;
                self.skip_ws();
                if !self.eat_char(')') {
                    return Err("missing closing parenthesis in log".to_string());
                }
                return Ok(Expr::FnCallTwo(Func2::LogBase, Box::new(base), Box::new(val)));
            }
            self.pos = saved_pos;
        }

        let arg = self.parse_unary()?;
        Ok(Expr::FnCall(Func::Log, Box::new(arg)))
    }

    fn parse_root_call(&mut self) -> Result<Expr, String> {
        self.pos += 4; // consume "root"
        self.skip_ws();

        let saved_pos = self.pos;

        if self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
            let n = self.parse_number()?;
            self.skip_ws();
            if self.peek_char() == Some('(') {
                self.pos += 1;
                let val = self.parse_full_expr()?;
                self.skip_ws();
                if !self.eat_char(')') {
                    return Err("missing closing parenthesis in root".to_string());
                }
                return Ok(Expr::FnCallTwo(Func2::Root, Box::new(n), Box::new(val)));
            }
            self.pos = saved_pos;
        }

        let arg = self.parse_unary()?;
        Ok(Expr::FnCall(Func::Sqrt, Box::new(arg)))
    }

    /// Try to match a currency prefix symbol at current position.
    /// Returns (default_currency_id, prefix_byte_length) if found.
    fn try_match_currency_prefix(&self) -> Option<(String, usize)> {
        let rem = self.remaining();

        // Check multi-char prefixes first (HK$, NZ$, NT$, C$, A$, R$, S$)
        for &(prefix, currency_id) in units::CURRENCY_PREFIXES_MULTI {
            if rem.starts_with(prefix) {
                return Some((currency_id.to_string(), prefix.len()));
            }
        }

        // Check single-char prefixes
        let first = rem.chars().next()?;
        for &(symbol, currency_id) in units::CURRENCY_PREFIXES {
            if first == symbol {
                return Some((currency_id.to_string(), first.len_utf8()));
            }
        }

        None
    }

    /// After parsing a currency-prefixed number, check if an explicit currency code follows.
    /// E.g., "$30 CAD" — the "CAD" overrides the "$" default of USD.
    fn try_match_currency_code_suffix(&mut self) -> Option<String> {
        let saved = self.pos;
        self.skip_ws();
        let rem = self.remaining();
        if rem.is_empty() {
            self.pos = saved;
            return None;
        }

        // Only match 3-letter uppercase ASCII currency codes
        let bytes = rem.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_uppercase()
            && bytes[1].is_ascii_uppercase()
            && bytes[2].is_ascii_uppercase()
        {
            let candidate = &rem[..3];
            if let Some(unit_def) = units::lookup_unit_by_id(candidate) {
                if units::is_currency(unit_def) {
                    let at_boundary = bytes.len() == 3
                        || !bytes[3].is_ascii_alphanumeric();
                    if at_boundary {
                        self.pos += 3;
                        return Some(candidate.to_string());
                    }
                }
            }
        }

        self.pos = saved;
        None
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        let bytes = self.input.as_bytes();

        // Integer part
        while self.pos < bytes.len() && bytes[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        // Handle comma grouping: 1,000,000
        while self.pos < bytes.len() && bytes[self.pos] == b',' {
            if self.pos + 3 < bytes.len()
                && bytes[self.pos + 1].is_ascii_digit()
                && bytes[self.pos + 2].is_ascii_digit()
                && bytes[self.pos + 3].is_ascii_digit()
            {
                let after = self.pos + 4;
                if after >= bytes.len() || !bytes[after].is_ascii_digit() {
                    self.pos += 4;
                    continue;
                }
            }
            break;
        }

        // Handle space grouping: 1 000 000 (3 digits after space)
        loop {
            if self.pos < bytes.len() && bytes[self.pos] == b' ' {
                if self.pos + 3 < bytes.len()
                    && bytes[self.pos + 1].is_ascii_digit()
                    && bytes[self.pos + 2].is_ascii_digit()
                    && bytes[self.pos + 3].is_ascii_digit()
                {
                    let after = self.pos + 4;
                    if after >= bytes.len() || !bytes[after].is_ascii_digit() {
                        self.pos += 4;
                        continue;
                    }
                }
            }
            break;
        }

        // Decimal part
        if self.pos < bytes.len() && bytes[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < bytes.len() && bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let raw = &self.input[start..self.pos];
        let clean: String = raw.chars().filter(|c| *c != ',' && *c != ' ').collect();
        let n: f64 = clean
            .parse()
            .map_err(|e| format!("invalid number '{raw}': {e}"))?;
        Ok(Expr::Number(n))
    }

    fn parse_hex(&mut self) -> Result<Expr, String> {
        self.pos += 2; // skip 0x
        let start = self.pos;
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len()
            && (bytes[self.pos].is_ascii_hexdigit() || bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let hex_str: String = self.input[start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        let n = u64::from_str_radix(&hex_str, 16)
            .map_err(|e| format!("invalid hex: {e}"))?;
        Ok(Expr::Number(n as f64))
    }

    fn parse_bin(&mut self) -> Result<Expr, String> {
        self.pos += 2; // skip 0b
        let start = self.pos;
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && (bytes[self.pos] == b'0' || bytes[self.pos] == b'1' || bytes[self.pos] == b'_') {
            self.pos += 1;
        }
        let bin_str: String = self.input[start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        let n = u64::from_str_radix(&bin_str, 2)
            .map_err(|e| format!("invalid binary: {e}"))?;
        Ok(Expr::Number(n as f64))
    }

    fn parse_oct(&mut self) -> Result<Expr, String> {
        self.pos += 2; // skip 0o
        let start = self.pos;
        let bytes = self.input.as_bytes();
        while self.pos < bytes.len() && (bytes[self.pos] >= b'0' && bytes[self.pos] <= b'7' || bytes[self.pos] == b'_') {
            self.pos += 1;
        }
        let oct_str: String = self.input[start..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();
        let n = u64::from_str_radix(&oct_str, 8)
            .map_err(|e| format!("invalid octal: {e}"))?;
        Ok(Expr::Number(n as f64))
    }
}

/// Check if `rem` starts with "as a % of/on/off" pattern.
fn is_as_percent_pattern(rem: &str) -> bool {
    // rem starts with "as " — check if it continues with "a % of/on/off"
    let after_as = rem[3..].trim_start();
    if !(after_as.starts_with("a ") || after_as.starts_with("A ")) {
        return false;
    }
    let after_a = after_as[2..].trim_start();
    if !after_a.starts_with('%') {
        return false;
    }
    let after_pct = after_a[1..].trim_start();
    after_pct.starts_with("of") || after_pct.starts_with("on") || after_pct.starts_with("off")
}

/// Check if the remaining text starts with a conversion keyword
/// followed by a space (or at end of input).
fn is_conversion_keyword(rem: &str) -> bool {
    for kw in &["into ", "in ", "to ", "as "] {
        if rem.starts_with(kw) {
            return true;
        }
    }
    // Also at end of string
    matches!(rem, "in" | "to" | "as" | "into")
}
