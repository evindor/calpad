/// Smoke test: evaluate the guide document and report any errors or panics.
/// Run with: cargo run --example test_guide
use calpad_core::evaluate_document;

fn main() {
    let guide = include_str!("../../../web/static/guide.txt");

    let results = evaluate_document(guide);
    let mut errors = 0;

    for r in &results {
        if r.display.starts_with("Error") {
            eprintln!("Line {}: {:?} -> {}", r.line_index + 1, r.input, r.display);
            errors += 1;
        }
    }

    if errors > 0 {
        eprintln!("\n{errors} error(s) found in guide.txt");
        std::process::exit(1);
    } else {
        eprintln!("guide.txt: all {} lines OK", results.len());
    }
}
