/// Core encoding detection logic
pub mod detector;
/// Pattern matching for glyph ID analysis
pub mod patterns;
/// Statistical analysis of glyph distributions
pub mod statistics;
/// Heuristics for font type detection
pub mod heuristics;

pub use detector::{EncodingDetector, EncodingDetection, DetectionConfidence, PdfFontInfo};
pub use patterns::{PatternMatcher, PatternMatch};
pub use statistics::GlyphStatistics;

#[cfg(test)]
mod tests;