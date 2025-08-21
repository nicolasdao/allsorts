//! Phase 2 API enhancements for PDF font subsetting

pub mod pdf;
pub mod builder;

pub use self::pdf::{PdfFontContext, PdfFontType, PdfSubsetResult, SubsetStatistics, subset_and_map_for_pdf};
pub use self::builder::{PdfSubsetBuilder, subset_for_pdf};