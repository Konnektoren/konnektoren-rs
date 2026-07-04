// One struct per file; the `Faq` struct lives in `faq/faq.rs` by convention.
#[allow(clippy::module_inception)]
mod faq;
mod faq_data;

pub use faq::Faq;
pub use faq_data::FaqData;
