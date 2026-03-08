use wasm_bindgen::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date)]
    fn now() -> f64;
}

static CURRENCY_RATES: Mutex<Option<HashMap<String, f64>>> = Mutex::new(None);

#[derive(Serialize)]
pub struct JsLineResult {
    pub line_index: usize,
    pub input: String,
    pub display: String,
    pub is_error: bool,
    pub raw_number: Option<f64>,
}

#[wasm_bindgen]
pub fn set_currency_rates(rates: JsValue) {
    let map: HashMap<String, f64> = serde_wasm_bindgen::from_value(rates).unwrap_or_default();
    *CURRENCY_RATES.lock().unwrap() = Some(map);
}

#[wasm_bindgen]
pub fn evaluate(document: &str) -> JsValue {
    let rates = CURRENCY_RATES.lock().unwrap();
    let empty = HashMap::new();
    let rates_ref = rates.as_ref().unwrap_or(&empty);

    let now_secs = now() / 1000.0; // Date.now() returns milliseconds
    let results = calpad_core::evaluate_document_full(document, rates_ref, Some(now_secs));
    let js_results: Vec<JsLineResult> = results
        .into_iter()
        .map(|r| {
            let is_error = matches!(r.value, calpad_core::types::Value::Error(_));
            let raw_number = if is_error { None } else { r.value.as_f64() };
            JsLineResult {
                line_index: r.line_index,
                input: r.input,
                display: r.display,
                is_error,
                raw_number,
            }
        })
        .collect();
    serde_wasm_bindgen::to_value(&js_results).unwrap()
}
