pub mod parser;
pub mod eval;
pub mod types;
pub mod units;
pub mod document;

pub use document::evaluate_document;
pub use document::evaluate_document_with_rates;
