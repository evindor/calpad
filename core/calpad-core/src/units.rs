use std::f64::consts::PI;

/// Category of a unit, for grouping and compatibility checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitCategory {
    Length,
    Weight,
    Time,
    Area,
    Volume,
    DataBit,
    DataByte,
    Angular,
    Temperature,
    Css,
    Currency,
}

/// How a unit converts to its base unit.
#[derive(Debug, Clone, Copy)]
pub enum Conversion {
    /// value_in_base = value * ratio
    Ratio(f64),
    /// Temperature: (to_kelvin_fn, from_kelvin_fn) stored as enum variant
    Celsius,
    Fahrenheit,
    Kelvin,
}

/// A single unit definition.
#[derive(Debug, Clone)]
pub struct UnitDef {
    pub id: &'static str,
    pub phrases: &'static [&'static str],
    pub category: UnitCategory,
    pub conversion: Conversion,
    pub display: &'static str,
}

impl UnitDef {
    /// Convert a value in this unit to the base unit of its category.
    pub fn to_base(&self, value: f64) -> f64 {
        match self.conversion {
            Conversion::Ratio(r) => value * r,
            Conversion::Celsius => value + 273.15,
            Conversion::Fahrenheit => (value - 32.0) * 5.0 / 9.0 + 273.15,
            Conversion::Kelvin => value,
        }
    }

    /// Convert a value from the base unit to this unit.
    pub fn from_base(&self, value: f64) -> f64 {
        match self.conversion {
            Conversion::Ratio(r) => value / r,
            Conversion::Celsius => value - 273.15,
            Conversion::Fahrenheit => (value - 273.15) * 9.0 / 5.0 + 32.0,
            Conversion::Kelvin => value,
        }
    }
}

/// Whether two categories can convert between each other.
pub fn categories_compatible(a: UnitCategory, b: UnitCategory) -> bool {
    if a == b {
        return true;
    }
    // bit <-> byte cross-conversion
    matches!(
        (a, b),
        (UnitCategory::DataBit, UnitCategory::DataByte)
            | (UnitCategory::DataByte, UnitCategory::DataBit)
    )
}

/// Cross-convert between data categories: bits to bytes or vice versa.
/// Returns the value in the target category's base unit.
pub fn cross_convert_data(value_in_source_base: f64, from: UnitCategory, to: UnitCategory) -> f64 {
    match (from, to) {
        (UnitCategory::DataBit, UnitCategory::DataByte) => value_in_source_base / 8.0,
        (UnitCategory::DataByte, UnitCategory::DataBit) => value_in_source_base * 8.0,
        _ => value_in_source_base,
    }
}

// ---------------------------------------------------------------------------
// Static unit definitions
// ---------------------------------------------------------------------------

macro_rules! unit {
    ($id:expr, $phrases:expr, $cat:expr, $ratio:expr, $display:expr) => {
        UnitDef {
            id: $id,
            phrases: $phrases,
            category: $cat,
            conversion: Conversion::Ratio($ratio),
            display: $display,
        }
    };
}

pub static UNITS: &[UnitDef] = &[
    // ── Length (base: meter) ──────────────────────────────────────────
    unit!("meter", &["meter", "meters", "metre", "metres", "m"], UnitCategory::Length, 1.0, "m"),
    unit!("millimeter", &["millimeter", "millimeters", "mm"], UnitCategory::Length, 0.001, "mm"),
    unit!("centimeter", &["centimeter", "centimeters", "cm"], UnitCategory::Length, 0.01, "cm"),
    unit!("kilometer", &["kilometer", "kilometers", "km"], UnitCategory::Length, 1000.0, "km"),
    unit!("inch", &["inch", "inches"], UnitCategory::Length, 0.0254, "inches"),
    unit!("foot", &["foot", "feet", "ft"], UnitCategory::Length, 0.3048, "ft"),
    unit!("yard", &["yard", "yards", "yd"], UnitCategory::Length, 0.9144, "yd"),
    unit!("mile", &["mile", "miles", "mi"], UnitCategory::Length, 1609.344, "mi"),
    unit!("mil", &["mil", "mils"], UnitCategory::Length, 0.0000254, "mil"),
    unit!("hand", &["hand", "hands"], UnitCategory::Length, 0.1016, "hands"),
    unit!("rod", &["rod", "rods"], UnitCategory::Length, 5.0292, "rods"),
    unit!("chain", &["chain", "chains"], UnitCategory::Length, 20.1168, "chains"),
    unit!("furlong", &["furlong", "furlongs"], UnitCategory::Length, 201.168, "furlongs"),
    unit!("cable", &["cable", "cables"], UnitCategory::Length, 185.2, "cables"),
    unit!("nautical_mile", &["nautical mile", "nautical miles", "nmi"], UnitCategory::Length, 1852.0, "nmi"),
    unit!("league", &["league", "leagues"], UnitCategory::Length, 4828.032, "leagues"),
    unit!("typo_point", &["point", "points"], UnitCategory::Length, 0.000352778, "points"),
    unit!("line", &["line", "lines"], UnitCategory::Length, 0.002116667, "lines"),

    // ── Weight (base: gram) ──────────────────────────────────────────
    unit!("gram", &["gram", "grams", "g"], UnitCategory::Weight, 1.0, "g"),
    unit!("kilogram", &["kilogram", "kilograms", "kg"], UnitCategory::Weight, 1000.0, "kg"),
    unit!("milligram", &["milligram", "milligrams", "mg"], UnitCategory::Weight, 0.001, "mg"),
    unit!("tonne", &["tonne", "tonnes", "t"], UnitCategory::Weight, 1_000_000.0, "t"),
    unit!("carat", &["carat", "carats", "ct"], UnitCategory::Weight, 0.2, "ct"),
    unit!("centner", &["centner", "centners"], UnitCategory::Weight, 100_000.0, "centners"),
    unit!("pound", &["pound", "pounds", "lb", "lbs"], UnitCategory::Weight, 453.592, "lb"),
    unit!("stone", &["stone", "stones", "st"], UnitCategory::Weight, 6350.29, "st"),
    unit!("ounce", &["ounce", "ounces", "oz"], UnitCategory::Weight, 28.3495, "oz"),

    // ── Area (base: square meter) ────────────────────────────────────
    unit!("hectare", &["hectare", "hectares", "ha"], UnitCategory::Area, 10_000.0, "ha"),
    unit!("are", &["are", "ares"], UnitCategory::Area, 100.0, "ares"),
    unit!("acre", &["acre", "acres"], UnitCategory::Area, 4046.86, "acres"),
    unit!("sq_cm", &["sq cm", "sq centimeter", "sq centimeters", "sqcm", "square centimeter", "square centimeters"], UnitCategory::Area, 0.0001, "sq cm"),
    unit!("sq_m", &["sq m", "sq meter", "sq meters", "sqm", "square meter", "square meters", "m2"], UnitCategory::Area, 1.0, "sq m"),
    unit!("sq_km", &["sq km", "sq kilometer", "sq kilometers", "sqkm", "square kilometer", "square kilometers", "km2"], UnitCategory::Area, 1_000_000.0, "sq km"),
    unit!("sq_inch", &["sq inch", "sq inches", "square inch", "square inches"], UnitCategory::Area, 0.00064516, "sq inches"),
    unit!("sq_foot", &["sq foot", "sq feet", "sq ft", "square foot", "square feet"], UnitCategory::Area, 0.092903, "sq ft"),
    unit!("sq_yard", &["sq yard", "sq yards", "square yard", "square yards"], UnitCategory::Area, 0.836127, "sq yards"),
    unit!("sq_mile", &["sq mile", "sq miles", "square mile", "square miles"], UnitCategory::Area, 2_589_988.11, "sq miles"),

    // ── Volume (base: cubic meter) ───────────────────────────────────
    unit!("liter", &["liter", "liters", "litre", "litres", "l", "L"], UnitCategory::Volume, 0.001, "L"),
    unit!("milliliter", &["milliliter", "milliliters", "ml", "mL"], UnitCategory::Volume, 0.000001, "mL"),
    unit!("pint", &["pint", "pints"], UnitCategory::Volume, 0.000473176, "pints"),
    unit!("quart", &["quart", "quarts", "qt"], UnitCategory::Volume, 0.000946353, "qt"),
    unit!("gallon", &["gallon", "gallons", "gal"], UnitCategory::Volume, 0.00378541, "gal"),
    unit!("teaspoon", &["tea spoon", "tea spoons", "teaspoon", "teaspoons", "tsp"], UnitCategory::Volume, 0.00000492892, "tsp"),
    unit!("tablespoon", &["table spoon", "table spoons", "tablespoon", "tablespoons", "tbsp"], UnitCategory::Volume, 0.0000147868, "tbsp"),
    unit!("cup", &["cup", "cups"], UnitCategory::Volume, 0.000236588, "cups"),
    unit!("cu_cm", &["cu cm", "cu centimeter", "cu centimeters", "cucm", "cb cm", "cb centimeter", "cb centimeters", "cbcm", "cubic centimeter", "cubic centimeters", "cm3"], UnitCategory::Volume, 0.000001, "cu cm"),
    unit!("cu_m", &["cu m", "cu meter", "cu meters", "cum", "cb m", "cb meter", "cb meters", "cbm", "cubic meter", "cubic meters", "m3"], UnitCategory::Volume, 1.0, "cu m"),
    unit!("cu_inch", &["cu inch", "cu inches", "cubic inch", "cubic inches"], UnitCategory::Volume, 0.0000163871, "cu inches"),
    unit!("cu_foot", &["cu foot", "cu feet", "cubic foot", "cubic feet"], UnitCategory::Volume, 0.0283168, "cu ft"),

    // ── Time (base: second) ──────────────────────────────────────────
    unit!("second", &["second", "seconds", "sec", "s"], UnitCategory::Time, 1.0, "s"),
    unit!("millisecond", &["millisecond", "milliseconds", "ms"], UnitCategory::Time, 0.001, "ms"),
    unit!("minute", &["minute", "minutes", "min"], UnitCategory::Time, 60.0, "min"),
    unit!("hour", &["hour", "hours", "hr", "h"], UnitCategory::Time, 3600.0, "hr"),
    unit!("day", &["day", "days"], UnitCategory::Time, 86400.0, "days"),
    unit!("week", &["week", "weeks"], UnitCategory::Time, 604800.0, "weeks"),
    unit!("month", &["month", "months"], UnitCategory::Time, 2628000.0, "months"),
    unit!("year", &["year", "years", "yr"], UnitCategory::Time, 31536000.0, "yr"),

    // ── Data: bit-based (base: bit) ──────────────────────────────────
    unit!("bit", &["bit", "bits"], UnitCategory::DataBit, 1.0, "bits"),
    unit!("kilobit", &["kilobit", "kilobits", "kb", "Kb"], UnitCategory::DataBit, 1000.0, "Kb"),
    unit!("megabit", &["megabit", "megabits", "Mb"], UnitCategory::DataBit, 1_000_000.0, "Mb"),
    unit!("gigabit", &["gigabit", "gigabits", "Gb"], UnitCategory::DataBit, 1_000_000_000.0, "Gb"),

    // ── Data: byte-based (base: byte) ────────────────────────────────
    unit!("byte", &["byte", "bytes", "B"], UnitCategory::DataByte, 1.0, "B"),
    unit!("kilobyte", &["kilobyte", "kilobytes", "KB", "kB"], UnitCategory::DataByte, 1000.0, "KB"),
    unit!("megabyte", &["megabyte", "megabytes", "MB"], UnitCategory::DataByte, 1_000_000.0, "MB"),
    unit!("gigabyte", &["gigabyte", "gigabytes", "GB"], UnitCategory::DataByte, 1_000_000_000.0, "GB"),
    unit!("terabyte", &["terabyte", "terabytes", "TB"], UnitCategory::DataByte, 1_000_000_000_000.0, "TB"),
    unit!("kibibyte", &["kibibyte", "kibibytes", "KiB"], UnitCategory::DataByte, 1024.0, "KiB"),
    unit!("mebibyte", &["mebibyte", "mebibytes", "MiB"], UnitCategory::DataByte, 1_048_576.0, "MiB"),
    unit!("gibibyte", &["gibibyte", "gibibytes", "GiB"], UnitCategory::DataByte, 1_073_741_824.0, "GiB"),
    unit!("tebibyte", &["tebibyte", "tebibytes", "TiB"], UnitCategory::DataByte, 1_099_511_627_776.0, "TiB"),

    // ── Angular (base: radian) ───────────────────────────────────────
    unit!("radian", &["radian", "radians", "rad"], UnitCategory::Angular, 1.0, "rad"),
    UnitDef {
        id: "degree",
        phrases: &["degree", "degrees", "deg", "\u{00b0}"],
        category: UnitCategory::Angular,
        conversion: Conversion::Ratio(PI / 180.0),
        display: "\u{00b0}",
    },

    // ── Temperature (special offset conversion, base: kelvin) ────────
    UnitDef {
        id: "kelvin",
        phrases: &["kelvin", "kelvins", "K"],
        category: UnitCategory::Temperature,
        conversion: Conversion::Kelvin,
        display: "K",
    },
    UnitDef {
        id: "celsius",
        phrases: &["celsius", "\u{00b0}C", "C"],
        category: UnitCategory::Temperature,
        conversion: Conversion::Celsius,
        display: "\u{00b0}C",
    },
    UnitDef {
        id: "fahrenheit",
        phrases: &["fahrenheit", "\u{00b0}F", "F"],
        category: UnitCategory::Temperature,
        conversion: Conversion::Fahrenheit,
        display: "\u{00b0}F",
    },

    // ── CSS (base: pixel) ────────────────────────────────────────────
    unit!("pixel", &["pixel", "pixels", "px"], UnitCategory::Css, 1.0, "px"),
    unit!("css_pt", &["pt"], UnitCategory::Css, 1.3333333333333333, "pt"),
    unit!("em", &["em"], UnitCategory::Css, 16.0, "em"),
    unit!("css_inch", &["css inch", "css inches"], UnitCategory::Css, 96.0, "css inches"),

    // ── Currency (base: USD, ratio=1.0 placeholder — actual rates injected at runtime) ──
    unit!("USD", &["USD", "usd", "dollar", "dollars"], UnitCategory::Currency, 1.0, "USD"),
    unit!("EUR", &["EUR", "eur", "euro", "euros"], UnitCategory::Currency, 1.0, "EUR"),
    unit!("GBP", &["GBP", "gbp", "pound sterling", "pounds sterling", "british pound", "british pounds"], UnitCategory::Currency, 1.0, "GBP"),
    unit!("JPY", &["JPY", "jpy", "yen"], UnitCategory::Currency, 1.0, "JPY"),
    unit!("CAD", &["CAD", "cad", "canadian dollar", "canadian dollars"], UnitCategory::Currency, 1.0, "CAD"),
    unit!("AUD", &["AUD", "aud", "australian dollar", "australian dollars"], UnitCategory::Currency, 1.0, "AUD"),
    unit!("CHF", &["CHF", "chf", "swiss franc", "swiss francs"], UnitCategory::Currency, 1.0, "CHF"),
    unit!("CNY", &["CNY", "cny", "yuan", "chinese yuan"], UnitCategory::Currency, 1.0, "CNY"),
    unit!("INR", &["INR", "inr", "rupee", "rupees", "indian rupee", "indian rupees"], UnitCategory::Currency, 1.0, "INR"),
    unit!("RUB", &["RUB", "rub", "ruble", "rubles", "rouble", "roubles"], UnitCategory::Currency, 1.0, "RUB"),
    unit!("BRL", &["BRL", "brl", "real", "reais"], UnitCategory::Currency, 1.0, "BRL"),
    unit!("KRW", &["KRW", "krw", "won"], UnitCategory::Currency, 1.0, "KRW"),
    unit!("MXN", &["MXN", "mxn", "mexican peso", "mexican pesos"], UnitCategory::Currency, 1.0, "MXN"),
    unit!("SGD", &["SGD", "sgd", "singapore dollar", "singapore dollars"], UnitCategory::Currency, 1.0, "SGD"),
    unit!("HKD", &["HKD", "hkd", "hong kong dollar", "hong kong dollars"], UnitCategory::Currency, 1.0, "HKD"),
    unit!("SEK", &["SEK", "sek", "swedish krona", "swedish kronor"], UnitCategory::Currency, 1.0, "SEK"),
    unit!("NOK", &["NOK", "nok", "norwegian krone", "norwegian kroner"], UnitCategory::Currency, 1.0, "NOK"),
    unit!("DKK", &["DKK", "dkk", "danish krone", "danish kroner"], UnitCategory::Currency, 1.0, "DKK"),
    unit!("PLN", &["PLN", "pln", "zloty", "złoty"], UnitCategory::Currency, 1.0, "PLN"),
    unit!("THB", &["THB", "thb", "baht"], UnitCategory::Currency, 1.0, "THB"),
    unit!("TRY", &["TRY", "turkish lira"], UnitCategory::Currency, 1.0, "TRY"),
    unit!("NZD", &["NZD", "nzd", "new zealand dollar", "new zealand dollars"], UnitCategory::Currency, 1.0, "NZD"),
    unit!("ZAR", &["ZAR", "zar", "rand", "south african rand"], UnitCategory::Currency, 1.0, "ZAR"),
    unit!("TWD", &["TWD", "twd", "taiwan dollar"], UnitCategory::Currency, 1.0, "TWD"),
    unit!("CZK", &["CZK", "czk", "czech koruna"], UnitCategory::Currency, 1.0, "CZK"),
    unit!("ILS", &["ILS", "ils", "shekel", "shekels"], UnitCategory::Currency, 1.0, "ILS"),
];

/// Sorted phrase index for longest-match-first lookup.
/// Each entry is (phrase, index into UNITS).
/// Built lazily.
use std::sync::LazyLock;

struct PhraseEntry {
    phrase: &'static str,
    unit_index: usize,
    /// true if the phrase is a short abbreviation (case-sensitive match)
    case_sensitive: bool,
}

static PHRASE_INDEX: LazyLock<Vec<PhraseEntry>> = LazyLock::new(|| {
    let mut entries: Vec<PhraseEntry> = Vec::new();
    for (idx, unit) in UNITS.iter().enumerate() {
        for &phrase in unit.phrases {
            let case_sensitive = is_short_form(phrase);
            entries.push(PhraseEntry {
                phrase,
                unit_index: idx,
                case_sensitive,
            });
        }
    }
    // Sort by phrase length descending for longest-match-first
    entries.sort_by(|a, b| b.phrase.len().cmp(&a.phrase.len()));
    entries
});

/// Determine if a phrase is a "short form" that needs case-sensitive matching.
/// Short forms are abbreviations like "m", "M", "kg", "KB", "Mb", etc.
/// Long forms are full words like "meter", "meters", "kilogram".
fn is_short_form(phrase: &str) -> bool {
    // If the phrase is all ASCII alphabetic and <= 4 chars, or contains
    // special chars like °, treat as short form (case-sensitive).
    // Exception: single common words are long forms.
    if phrase.contains(' ') {
        return false; // multi-word phrases are always case-insensitive
    }
    if phrase.contains('\u{00b0}') {
        return true; // °C, °F
    }
    // If all alphabetic and short, it's an abbreviation
    if phrase.len() <= 4 && phrase.chars().all(|c| c.is_ascii_alphabetic()) {
        // But some short words are actual words: "are", "cup", "rod", "day", "bit"
        // We check: if it contains any uppercase letter, it's definitely a short form
        if phrase.chars().any(|c| c.is_ascii_uppercase()) {
            return true;
        }
        // Short all-lowercase: could be abbreviation or word
        // Abbreviations: m, g, s, km, cm, mm, ft, yd, mi, oz, lb, hr, ha, ml, pt, px, em, rad, deg, sec, min, gal, qt, tsp
        // Words: are, cup, rod, day, bit, mil, hand
        // We'll treat <= 3 char all-lowercase as case-sensitive abbreviations,
        // except known words
        let known_words = ["are", "cup", "rod", "day", "bit", "mil"];
        if phrase.len() <= 3 && !known_words.contains(&phrase) {
            return true;
        }
        return false;
    }
    false
}

/// Try to match a unit phrase at the beginning of `text`.
/// Returns `(unit_def, matched_length)` if found.
/// `text` should already be trimmed of leading whitespace.
/// Does NOT match "in" as a unit (reserved for conversion).
pub fn match_unit_phrase(text: &str) -> Option<(&'static UnitDef, usize)> {
    for entry in PHRASE_INDEX.iter() {
        let phrase = entry.phrase;
        let plen = phrase.len();
        if text.len() < plen {
            continue;
        }

        let candidate = &text[..plen];
        let matches = if entry.case_sensitive {
            candidate == phrase
        } else {
            candidate.eq_ignore_ascii_case(phrase)
        };

        if matches {
            // Make sure the match is at a word boundary
            // (not followed by an alphanumeric character that would make it part of a longer word)
            if text.len() > plen {
                let next_char = text[plen..].chars().next().unwrap();
                if next_char.is_ascii_alphanumeric() || next_char == '_' {
                    continue;
                }
            }

            return Some((&UNITS[entry.unit_index], plen));
        }
    }
    None
}

/// Look up a unit by a phrase string (exact phrase, used after parsing).
pub fn lookup_unit(phrase: &str) -> Option<&'static UnitDef> {
    // Try exact match first, then case-insensitive
    for entry in PHRASE_INDEX.iter() {
        let matches = if entry.case_sensitive {
            phrase == entry.phrase
        } else {
            phrase.eq_ignore_ascii_case(entry.phrase)
        };
        if matches {
            return Some(&UNITS[entry.unit_index]);
        }
    }
    None
}

/// Look up a unit by its id.
pub fn lookup_unit_by_id(id: &str) -> Option<&'static UnitDef> {
    UNITS.iter().find(|u| u.id == id)
}

/// Convert a value from one unit to another.
/// Returns Err if units are incompatible.
pub fn convert(value: f64, from: &UnitDef, to: &UnitDef) -> Result<f64, String> {
    if !categories_compatible(from.category, to.category) {
        return Err(format!(
            "cannot convert {} to {}: incompatible units",
            from.display, to.display
        ));
    }

    let base_value = from.to_base(value);

    // Handle cross-category data conversion
    let base_value = if from.category != to.category {
        cross_convert_data(base_value, from.category, to.category)
    } else {
        base_value
    };

    Ok(to.from_base(base_value))
}

/// Display format specifiers (not unit conversions).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayFormat {
    Hex,
    Binary,
    Octal,
    Scientific,
}

/// Check if a phrase is a display format specifier.
pub fn match_display_format(phrase: &str) -> Option<DisplayFormat> {
    match phrase.to_lowercase().as_str() {
        "hex" | "hexadecimal" => Some(DisplayFormat::Hex),
        "binary" | "bin" => Some(DisplayFormat::Binary),
        "octal" | "oct" => Some(DisplayFormat::Octal),
        "sci" | "scientific" => Some(DisplayFormat::Scientific),
        _ => None,
    }
}

/// Format a number in the given display format.
pub fn format_display(value: f64, format: DisplayFormat) -> String {
    match format {
        DisplayFormat::Hex => {
            let n = value as i64;
            if n >= 0 {
                format!("0x{:X}", n)
            } else {
                format!("-0x{:X}", -n)
            }
        }
        DisplayFormat::Binary => {
            let n = value as i64;
            if n >= 0 {
                format!("0b{:b}", n)
            } else {
                format!("-0b{:b}", -n)
            }
        }
        DisplayFormat::Octal => {
            let n = value as i64;
            if n >= 0 {
                format!("0o{:o}", n)
            } else {
                format!("-0o{:o}", -n)
            }
        }
        DisplayFormat::Scientific => {
            if value == 0.0 {
                return "0".to_string();
            }
            let exp = value.abs().log10().floor() as i32;
            let mantissa = value / 10f64.powi(exp);
            if exp == 0 {
                format_sci_mantissa(mantissa)
            } else {
                format!("{}e{}", format_sci_mantissa(mantissa), exp)
            }
        }
    }
}

fn format_sci_mantissa(m: f64) -> String {
    // Format with reasonable precision, trim trailing zeros
    let s = format!("{:.10}", m);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

// ---------------------------------------------------------------------------
// Currency helpers
// ---------------------------------------------------------------------------

/// Currency prefix symbols and the currency ID they map to.
pub static CURRENCY_PREFIXES: &[(char, &str)] = &[
    ('$', "USD"),
    ('€', "EUR"),
    ('£', "GBP"),
    ('¥', "JPY"),  // Also CNY, but JPY is more common with ¥
    ('₹', "INR"),
    ('₽', "RUB"),
    ('₩', "KRW"),
    ('₪', "ILS"),
    ('฿', "THB"),
    ('元', "CNY"),
];

/// Multi-char currency prefix symbols (must be checked before single-char).
pub static CURRENCY_PREFIXES_MULTI: &[(&str, &str)] = &[
    ("C$", "CAD"),
    ("A$", "AUD"),
    ("R$", "BRL"),
    ("S$", "SGD"),
    ("HK$", "HKD"),
    ("NZ$", "NZD"),
    ("NT$", "TWD"),
];

/// Check if a currency unit ID has a single-character display prefix symbol.
pub fn currency_display_prefix(unit_id: &str) -> Option<&'static str> {
    match unit_id {
        "USD" => Some("$"),
        "EUR" => Some("€"),
        "GBP" => Some("£"),
        "JPY" => Some("¥"),
        "INR" => Some("₹"),
        "RUB" => Some("₽"),
        "KRW" => Some("₩"),
        "ILS" => Some("₪"),
        "THB" => Some("฿"),
        _ => None,
    }
}

/// Check if a unit is a currency.
pub fn is_currency(unit_def: &UnitDef) -> bool {
    unit_def.category == UnitCategory::Currency
}

/// Convert between currencies using runtime rates.
/// Rates map currency_id -> rate relative to USD (i.e., 1 USD = rate units of that currency).
/// So USD=1.0, EUR=0.92 means 1 USD = 0.92 EUR.
pub fn convert_currency(
    value: f64,
    from_id: &str,
    to_id: &str,
    rates: &std::collections::HashMap<String, f64>,
) -> Result<f64, String> {
    if rates.is_empty() {
        // No rates available — just pass through the numeric value
        return Ok(value);
    }

    let from_rate = rates.get(from_id).copied().unwrap_or_else(|| {
        if from_id == "USD" { 1.0 } else { 0.0 }
    });
    let to_rate = rates.get(to_id).copied().unwrap_or_else(|| {
        if to_id == "USD" { 1.0 } else { 0.0 }
    });

    if from_rate == 0.0 {
        return Err(format!("no exchange rate for {from_id}"));
    }
    if to_rate == 0.0 {
        return Err(format!("no exchange rate for {to_id}"));
    }

    // source_in_usd = value / from_rate
    // target_value = source_in_usd * to_rate
    let in_usd = value / from_rate;
    Ok(in_usd * to_rate)
}
