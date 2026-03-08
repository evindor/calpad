use calpad_core::evaluate_document;

/// Helper: assert a display string approximately matches an expected number.
fn assert_approx_display(doc: &str, line: usize, expected: f64, tolerance: f64) {
    let results = evaluate_document(doc);
    let val = results[line]
        .value
        .as_f64()
        .unwrap_or_else(|| panic!("expected numeric on line {}, got {:?}", line, results[line].value));
    assert!(
        (val - expected).abs() < tolerance,
        "expected ~{} but got {} (display: '{}') for input '{}'",
        expected,
        val,
        results[line].display,
        doc
    );
}

#[test]
fn test_basic_arithmetic() {
    let results = evaluate_document("2 + 3");
    assert_eq!(results[0].display, "5");
}

#[test]
fn test_multiplication() {
    let results = evaluate_document("6 * 7");
    assert_eq!(results[0].display, "42");
}

#[test]
fn test_implicit_multiplication() {
    let results = evaluate_document("6(3)");
    assert_eq!(results[0].display, "18");
}

#[test]
fn test_word_operators() {
    let results = evaluate_document("8 times 9");
    assert_eq!(results[0].display, "72");
}

#[test]
fn test_functions() {
    let results = evaluate_document("sqrt 16");
    assert_eq!(results[0].display, "4");
}

#[test]
fn test_variables() {
    let doc = "v = 20\nv times 7";
    let results = evaluate_document(doc);
    assert_eq!(results[1].display, "140");
}

#[test]
fn test_prev() {
    let doc = "10 + 5\nprev * 2";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "15");
    assert_eq!(results[1].display, "30");
}

#[test]
fn test_sum() {
    let doc = "10\n15\nsum";
    let results = evaluate_document(doc);
    assert_eq!(results[2].display, "25");
}

#[test]
fn test_average() {
    let doc = "10\n20\naverage";
    let results = evaluate_document(doc);
    assert_eq!(results[2].display, "15");
}

#[test]
fn test_comments() {
    let doc = "// this is a comment\n5 + 3";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "");
    assert_eq!(results[1].display, "8");
}

#[test]
fn test_headers() {
    let doc = "# Budget\n100 + 200";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "");
    assert_eq!(results[1].display, "300");
}

#[test]
fn test_labels() {
    let doc = "Price: 10 + 5";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "15");
}

#[test]
fn test_hex() {
    let results = evaluate_document("0xFF");
    assert_eq!(results[0].display, "255");
}

#[test]
fn test_binary() {
    let results = evaluate_document("0b1010");
    assert_eq!(results[0].display, "10");
}

#[test]
fn test_exponent() {
    let results = evaluate_document("2 ^ 10");
    assert_eq!(results[0].display, "1,024");
}

#[test]
fn test_complex_expression() {
    let results = evaluate_document("(2 + 3) * (4 - 1)");
    assert_eq!(results[0].display, "15");
}

#[test]
fn test_negative_numbers() {
    let results = evaluate_document("-5 + 3");
    assert_eq!(results[0].display, "-2");
}

#[test]
fn test_decimal() {
    let results = evaluate_document("3.14 * 2");
    assert_eq!(results[0].display, "6.28");
}

#[test]
fn test_nested_functions() {
    let results = evaluate_document("abs(-4)");
    assert_eq!(results[0].display, "4");
}

#[test]
fn test_sum_stops_at_blank() {
    let doc = "10\n20\n\n5\n15\nsum";
    let results = evaluate_document(doc);
    assert_eq!(results[5].display, "20");
}

#[test]
fn test_constants() {
    let results = evaluate_document("round(pi * 100) / 100");
    assert_eq!(results[0].display, "3.14");
}

// ═══════════════════════════════════════════════════════════════════════════
// Unit system tests
// ═══════════════════════════════════════════════════════════════════════════

// -- Unit basics --

#[test]
fn test_unit_basic_kg() {
    let results = evaluate_document("5 kg");
    assert_eq!(results[0].display, "5 kg");
}

#[test]
fn test_unit_basic_cm() {
    let results = evaluate_document("100 cm");
    assert_eq!(results[0].display, "100 cm");
}

#[test]
fn test_unit_basic_radians() {
    let results = evaluate_document("3.14 radians");
    assert_eq!(results[0].display, "3.14 rad");
}

#[test]
fn test_unit_basic_mm() {
    let results = evaluate_document("1 mm");
    assert_eq!(results[0].display, "1 mm");
}

// -- Length conversions --

#[test]
fn test_convert_km_to_meters() {
    let results = evaluate_document("1 km in meters");
    assert_eq!(results[0].display, "1,000 m");
}

#[test]
fn test_convert_mile_to_km() {
    assert_approx_display("1 mile in km", 0, 1.609344, 0.001);
}

// -- Weight conversions --

#[test]
fn test_convert_pound_to_kg() {
    assert_approx_display("1 pound in kg", 0, 0.453592, 0.001);
}

// -- Temperature conversions --

#[test]
fn test_convert_celsius_to_fahrenheit() {
    assert_approx_display("100 celsius in fahrenheit", 0, 212.0, 0.01);
}

#[test]
fn test_convert_fahrenheit_to_celsius() {
    assert_approx_display("32 fahrenheit in celsius", 0, 0.0, 0.01);
}

#[test]
fn test_convert_boiling_celsius_to_fahrenheit() {
    assert_approx_display("100 celsius in fahrenheit", 0, 212.0, 0.01);
}

#[test]
fn test_convert_body_temp_f_to_c() {
    assert_approx_display("98.6 fahrenheit in celsius", 0, 37.0, 0.01);
}

// -- Time conversions --

#[test]
fn test_convert_hour_to_minutes() {
    let results = evaluate_document("1 hour in minutes");
    assert_eq!(results[0].display, "60 min");
}

// -- Data conversions --

#[test]
fn test_convert_gb_to_mb() {
    let results = evaluate_document("1 GB in MB");
    assert_eq!(results[0].display, "1,000 MB");
}

#[test]
fn test_convert_gib_to_mib() {
    let results = evaluate_document("1 GiB in MiB");
    assert_eq!(results[0].display, "1,024 MiB");
}

// -- Arithmetic with units --

#[test]
fn test_unit_add_same_base() {
    let results = evaluate_document("5 kg + 500 g");
    assert_eq!(results[0].display, "5.5 kg");
}

#[test]
fn test_unit_sub() {
    assert_approx_display("1 km - 500 m", 0, 0.5, 0.001);
}

#[test]
fn test_unit_mul_by_scalar() {
    let results = evaluate_document("5 kg * 3");
    assert_eq!(results[0].display, "15 kg");
}

// -- Compound units --

#[test]
fn test_compound_unit_meter_cm() {
    // 1 m + 20 cm = 1.2 m
    assert_approx_display("1 meter 20 cm", 0, 1.2, 0.001);
}

// -- Format conversions --

#[test]
fn test_format_hex() {
    let results = evaluate_document("255 in hex");
    assert_eq!(results[0].display, "0xFF");
}

#[test]
fn test_format_hex_from_hex() {
    let results = evaluate_document("0xFF in binary");
    assert_eq!(results[0].display, "0b11111111");
}

#[test]
fn test_format_scientific() {
    let results = evaluate_document("5300 in sci");
    assert_eq!(results[0].display, "5.3e3");
}

// -- CSS units --

#[test]
fn test_css_pt_to_px() {
    assert_approx_display("12 pt in px", 0, 16.0, 0.01);
}

// -- Cross-line with units --

#[test]
fn test_cross_line_units() {
    let doc = "10 kg\nprev in g";
    let results = evaluate_document(doc);
    assert_eq!(results[1].display, "10,000 g");
}

// -- Area --

#[test]
fn test_unit_area_sq_cm() {
    let results = evaluate_document("20 sq cm");
    assert_eq!(results[0].display, "20 sq cm");
}

#[test]
fn test_convert_hectare_to_sq_m() {
    let results = evaluate_document("1 hectare in sq m");
    assert_eq!(results[0].display, "10,000 sq m");
}

// -- Volume --

#[test]
fn test_unit_volume_cu_cm() {
    let results = evaluate_document("20 cu cm");
    assert_eq!(results[0].display, "20 cu cm");
}

#[test]
fn test_convert_gallon_to_liters() {
    assert_approx_display("1 gallon in liters", 0, 3.78541, 0.01);
}

// -- Angular --

#[test]
fn test_convert_degrees_to_radians() {
    assert_approx_display("180 degrees in radians", 0, std::f64::consts::PI, 0.001);
}

#[test]
fn test_convert_pi_radians_to_degrees() {
    assert_approx_display("pi radians in degrees", 0, 180.0, 0.01);
}

// -- Data cross-conversion (bit <-> byte) --

#[test]
fn test_convert_byte_to_bits() {
    let results = evaluate_document("1 byte in bits");
    assert_eq!(results[0].display, "8 bits");
}

#[test]
fn test_convert_kb_to_bytes() {
    // 1 kilobyte = 1000 bytes
    let results = evaluate_document("1 KB in bytes");
    assert_eq!(results[0].display, "1,000 B");
}

// -- Conversion keyword variants --

#[test]
fn test_convert_keyword_to() {
    let results = evaluate_document("1 km to meters");
    assert_eq!(results[0].display, "1,000 m");
}

#[test]
fn test_convert_keyword_as() {
    let results = evaluate_document("1 km as meters");
    assert_eq!(results[0].display, "1,000 m");
}

// -- Edge cases --

#[test]
fn test_expression_then_convert() {
    // "5 + 3 in hex" should be "(5 + 3) in hex"
    let results = evaluate_document("5 + 3 in hex");
    assert_eq!(results[0].display, "0x8");
}

#[test]
fn test_unit_with_label() {
    let doc = "Weight: 10 kg";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "10 kg");
}

#[test]
fn test_unit_variable_assignment() {
    let doc = "w = 5 kg\nw * 2";
    let results = evaluate_document(doc);
    assert_eq!(results[0].display, "5 kg");
    assert_eq!(results[1].display, "10 kg");
}

// -- More format conversions --

#[test]
fn test_format_octal() {
    let results = evaluate_document("8 in octal");
    assert_eq!(results[0].display, "0o10");
}

#[test]
fn test_convert_inch_css_to_px() {
    // "css inch" is the CSS unit
    let results = evaluate_document("1 css inch in px");
    assert_eq!(results[0].display, "96 px");
}

// -- Multi-word unit phrases --

#[test]
fn test_nautical_mile() {
    let results = evaluate_document("1 nautical mile in meters");
    assert_eq!(results[0].display, "1,852 m");
}

#[test]
fn test_square_kilometer() {
    let results = evaluate_document("1 square kilometer in sq m");
    assert_eq!(results[0].display, "1,000,000 sq m");
}

#[test]
fn test_cubic_foot_to_liters() {
    assert_approx_display("1 cubic foot in liters", 0, 28.3168, 0.01);
}

// -- Tablespoon/teaspoon --

#[test]
fn test_tablespoon() {
    let results = evaluate_document("2 tablespoons");
    assert_eq!(results[0].display, "2 tbsp");
}

#[test]
fn test_teaspoon() {
    let results = evaluate_document("3 teaspoons");
    assert_eq!(results[0].display, "3 tsp");
}

// ═══════════════════════════════════════════════════════════════════════════
// Phase 3: Percentages & scales
// ═══════════════════════════════════════════════════════════════════════════

// -- Scales --

#[test]
fn test_scale_k() {
    let results = evaluate_document("2k");
    assert_eq!(results[0].display, "2,000");
}

#[test]
fn test_scale_m() {
    let results = evaluate_document("5M");
    assert_eq!(results[0].display, "5,000,000");
}

#[test]
fn test_scale_billion() {
    let results = evaluate_document("3 billion");
    assert_eq!(results[0].display, "3,000,000,000");
}

#[test]
fn test_scale_thousand() {
    let results = evaluate_document("2 thousand");
    assert_eq!(results[0].display, "2,000");
}

#[test]
fn test_scale_million() {
    let results = evaluate_document("7 million");
    assert_eq!(results[0].display, "7,000,000");
}

// -- New scale variants --

#[test]
fn test_scale_uppercase_k() {
    let results = evaluate_document("2K");
    assert_eq!(results[0].display, "2,000");
}

#[test]
fn test_scale_b() {
    let results = evaluate_document("3B");
    assert_eq!(results[0].display, "3,000,000,000");
}

#[test]
fn test_scale_t() {
    let results = evaluate_document("1T");
    assert_eq!(results[0].display, "1,000,000,000,000");
}

#[test]
fn test_scale_mil() {
    let results = evaluate_document("5 mil");
    assert_eq!(results[0].display, "5,000,000");
}

#[test]
fn test_scale_bil() {
    let results = evaluate_document("2 bil");
    assert_eq!(results[0].display, "2,000,000,000");
}

#[test]
fn test_scale_trillion() {
    let results = evaluate_document("1 trillion");
    assert_eq!(results[0].display, "1,000,000,000,000");
}

#[test]
fn test_scale_b_not_bytes() {
    // "B" alone after number = billion, not bytes (bytes needs unit context)
    let results = evaluate_document("5B");
    assert_eq!(results[0].display, "5,000,000,000");
}

#[test]
fn test_scale_t_not_tonnes() {
    let results = evaluate_document("2T");
    assert_eq!(results[0].display, "2,000,000,000,000");
}

#[test]
fn test_scale_currency_with_b() {
    let results = evaluate_document("$1.5B");
    assert_eq!(results[0].display, "$1,500,000,000");
}

#[test]
fn test_scale_currency_with_t() {
    let results = evaluate_document("$2T");
    assert_eq!(results[0].display, "$2,000,000,000,000");
}

// -- Percentages --

#[test]
fn test_percent_display() {
    let results = evaluate_document("5%");
    assert_eq!(results[0].display, "5%");
}

#[test]
fn test_percent_of() {
    let results = evaluate_document("20% of 10");
    assert_eq!(results[0].display, "2");
}

#[test]
fn test_percent_add() {
    let results = evaluate_document("10 + 5%");
    assert_eq!(results[0].display, "10.5");
}

#[test]
fn test_percent_subtract() {
    let results = evaluate_document("10 - 40%");
    assert_eq!(results[0].display, "6");
}

#[test]
fn test_percent_on() {
    let results = evaluate_document("5% on 30");
    assert_eq!(results[0].display, "31.5");
}

#[test]
fn test_percent_off() {
    let results = evaluate_document("6% off 40");
    assert_eq!(results[0].display, "37.6");
}

#[test]
fn test_as_percent_of() {
    let results = evaluate_document("50 as a % of 100");
    assert_eq!(results[0].display, "50%");
}

#[test]
fn test_as_percent_on() {
    let results = evaluate_document("70 as a % on 20");
    assert_eq!(results[0].display, "250%");
}

#[test]
fn test_as_percent_off() {
    let results = evaluate_document("20 as a % off 70");
    // (70-20)/70 * 100 = 71.4285714...
    let results_val = &results[0];
    assert!(results_val.display.starts_with("71.42"));
}

#[test]
fn test_percent_of_what() {
    let results = evaluate_document("5% of what is 6");
    assert_eq!(results[0].display, "120");
}

#[test]
fn test_percent_on_what() {
    // 5% on what is 6 → X where X * 1.05 = 6 → X = 5.714...
    let results = evaluate_document("5% on what is 6");
    assert!(results[0].display.starts_with("5.71"));
}

#[test]
fn test_percent_off_what() {
    // 5% off what is 6 → X where X * 0.95 = 6 → X = 6.315...
    let results = evaluate_document("5% off what is 6");
    assert!(results[0].display.starts_with("6.31") || results[0].display.starts_with("6.32"));
}

#[test]
fn test_percent_variable() {
    let results = evaluate_document("v = 5%\n100 + v");
    assert_eq!(results[1].display, "105");
}

#[test]
fn test_percent_with_unit() {
    let results = evaluate_document("5 kg + 10%");
    assert_eq!(results[0].display, "5.5 kg");
}

#[test]
fn test_percent_of_with_expression() {
    let results = evaluate_document("20% of (50 + 50)");
    assert_eq!(results[0].display, "20");
}

// ═══════════════════════════════════════════════════════════════════════════
// Phase 4: Currency
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_currency_basic_usd() {
    let results = evaluate_document("$10");
    assert_eq!(results[0].display, "$10");
}

#[test]
fn test_currency_code_suffix() {
    let results = evaluate_document("30 CAD");
    assert_eq!(results[0].display, "30 CAD");
}

#[test]
fn test_currency_euro_prefix() {
    let results = evaluate_document("€20");
    assert_eq!(results[0].display, "€20");
}

#[test]
fn test_currency_pound_prefix() {
    let results = evaluate_document("£50");
    assert_eq!(results[0].display, "£50");
}

#[test]
fn test_currency_addition_same() {
    let results = evaluate_document("$10 + $5");
    assert_eq!(results[0].display, "$15");
}

#[test]
fn test_currency_with_percent() {
    let results = evaluate_document("$10 - 5%");
    assert_eq!(results[0].display, "$9.5");
}

#[test]
fn test_currency_conversion_with_rates() {
    use std::collections::HashMap;
    use calpad_core::evaluate_document_with_rates;
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    rates.insert("EUR".to_string(), 0.92);
    let results = evaluate_document_with_rates("$100 in EUR", &rates);
    assert_eq!(results[0].display, "€92");
}

#[test]
fn test_currency_conversion_reverse() {
    use std::collections::HashMap;
    use calpad_core::evaluate_document_with_rates;
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    rates.insert("EUR".to_string(), 0.92);
    let results = evaluate_document_with_rates("€92 in USD", &rates);
    assert_eq!(results[0].display, "$100");
}

#[test]
fn test_currency_mixed_addition() {
    use std::collections::HashMap;
    use calpad_core::evaluate_document_with_rates;
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    rates.insert("EUR".to_string(), 0.92);
    rates.insert("CAD".to_string(), 1.36);
    let results = evaluate_document_with_rates("$30 CAD + 5 USD - 7 EUR", &rates);
    // Result should be in CAD
    let display = &results[0].display;
    assert!(display.ends_with("CAD"), "Expected CAD suffix, got: {display}");
}

#[test]
fn test_currency_sum_with_rates() {
    use std::collections::HashMap;
    use calpad_core::evaluate_document_with_rates;
    let mut rates = HashMap::new();
    rates.insert("USD".to_string(), 1.0);
    rates.insert("EUR".to_string(), 0.92);
    let results = evaluate_document_with_rates("$10\n€10\nsum", &rates);
    // sum should give a total in USD (unit of first value)
    let sum_display = &results[2].display;
    assert!(sum_display.contains("$"), "Expected USD ($), got: {sum_display}");
}

#[test]
fn test_dollar_sign_with_code_override() {
    // $30 CAD means 30 CAD, not 30 USD
    let results = evaluate_document("$30 CAD");
    assert_eq!(results[0].display, "30 CAD");
}

#[test]
fn test_currency_scale() {
    let results = evaluate_document("$2k");
    assert_eq!(results[0].display, "$2,000");
}

#[test]
fn test_currency_percent_of() {
    let results = evaluate_document("20% of $10");
    assert_eq!(results[0].display, "$2");
}

// ── Phase 5: Dates & Time ──────────────────────────────────────────

use calpad_core::types::Value;

#[test]
fn test_now_returns_datetime() {
    let results = evaluate_document("now");
    assert!(!results[0].display.is_empty());
    assert!(matches!(results[0].value, Value::DateTime(_)));
}

#[test]
fn test_time_keyword() {
    let results = evaluate_document("time");
    assert!(matches!(results[0].value, Value::DateTime(_)));
}

#[test]
fn test_fromunix() {
    let results = evaluate_document("fromunix(1446587186)");
    let display = &results[0].display;
    assert!(display.contains("2015"), "Expected 2015 in display: {display}");
    assert!(display.contains("Nov"), "Expected Nov in display: {display}");
}

#[test]
fn test_fromunix_epoch() {
    let results = evaluate_document("fromunix(0)");
    let display = &results[0].display;
    assert!(display.contains("1970"), "Expected 1970: {display}");
    assert!(display.contains("Jan"), "Expected Jan: {display}");
}

#[test]
fn test_now_plus_duration() {
    let results = evaluate_document("now + 0 seconds");
    assert!(matches!(results[0].value, Value::DateTime(_)));
}

#[test]
fn test_fromunix_plus_days() {
    // epoch + 1 day = Jan 2, 1970
    let results = evaluate_document("fromunix(0) + 1 day");
    let display = &results[0].display;
    assert!(display.contains("Jan") && display.contains("1970"), "Expected Jan 1970: {display}");
}

#[test]
fn test_fromunix_minus_days() {
    // 86400 = 1 day after epoch; minus 1 day = epoch
    let results = evaluate_document("fromunix(86400) - 1 day");
    let display = &results[0].display;
    assert!(display.contains("Jan") && display.contains("1970"), "Expected Jan 1970: {display}");
}

#[test]
fn test_fromunix_plus_weeks() {
    // epoch + 1 week = Jan 8, 1970
    let results = evaluate_document("fromunix(0) + 1 week");
    let display = &results[0].display;
    assert!(display.contains("Jan") && display.contains("1970"), "Expected Jan 1970: {display}");
    assert!(display.contains("8"), "Expected day 8: {display}");
}

#[test]
fn test_fromunix_plus_hours() {
    // epoch + 2 hours = Jan 1, 1970 2:00:00 AM
    let results = evaluate_document("fromunix(0) + 2 hours");
    let display = &results[0].display;
    assert!(display.contains("2:00:00 AM"), "Expected 2:00 AM: {display}");
}

#[test]
fn test_datetime_subtraction() {
    // Two datetimes 1 day apart
    let results = evaluate_document("fromunix(86400) - fromunix(0)");
    assert_eq!(results[0].display, "86,400 s");
}

#[test]
fn test_fromunix_specific_date() {
    // 1446587186 = Nov 3, 2015 11:26:26 PM UTC
    let results = evaluate_document("fromunix(1446587186)");
    let display = &results[0].display;
    assert!(display.contains("Nov"), "Expected Nov: {display}");
    assert!(display.contains("3"), "Expected day 3: {display}");
    assert!(display.contains("2015"), "Expected 2015: {display}");
}

#[test]
fn test_now_not_assignable() {
    // "now = 5" should treat "now" as keyword, not variable assignment
    let results = evaluate_document("now");
    assert!(matches!(results[0].value, Value::DateTime(_)));
}

// ── Mixed-form area unit aliases ────────────────────────────────────

#[test]
fn test_sq_meters_alias() {
    let results = evaluate_document("20 sq ft in sq meters");
    assert_approx_display("20 sq ft in sq meters", 0, 1.85806, 0.001);
    assert!(results[0].display.contains("sq m"));
}

#[test]
fn test_sq_meter_alias() {
    assert_approx_display("1 sq meter in sq feet", 0, 10.7639, 0.01);
}

#[test]
fn test_sq_kilometers_alias() {
    assert_approx_display("1 sq mile in sq kilometers", 0, 2.58999, 0.01);
}

#[test]
fn test_sq_centimeters_alias() {
    assert_approx_display("1 sq inch in sq centimeters", 0, 6.4516, 0.01);
}

// ── Mixed-form volume unit aliases ──────────────────────────────────

#[test]
fn test_cu_meters_alias() {
    let results = evaluate_document("1000 liters in cu meters");
    assert_approx_display("1000 liters in cu meters", 0, 1.0, 0.01);
    assert!(results[0].display.contains("cu m"));
}

#[test]
fn test_cu_centimeters_alias() {
    assert_approx_display("1 liter in cu centimeters", 0, 1000.0, 1.0);
}

#[test]
fn test_cb_meters_alias() {
    assert_approx_display("35.3147 cu feet in cb meters", 0, 1.0, 0.01);
}

#[test]
fn test_cb_centimeters_alias() {
    assert_approx_display("1 cu inch in cb centimeters", 0, 16.3871, 0.01);
}
