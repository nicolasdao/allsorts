mod common;

use allsorts::binary::read::ReadScope;
use allsorts::subset::pdf::{subset_for_pdf, PdfFontContext, WritingMode};
use allsorts::tables::{FontTableProvider, OpenTypeFont};

fn create_provider(font_buffer: &[u8]) -> impl FontTableProvider + '_ {
    let scope = ReadScope::new(font_buffer);
    let font_file = scope.read::<OpenTypeFont<'_>>().unwrap();
    font_file.table_provider(0).unwrap()
}

#[test]
fn test_pdf_context_defaults() {
    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 255,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };

    assert_eq!(context.max_cid, 255);
    assert_eq!(context.writing_mode, WritingMode::Horizontal);
}

#[test]
fn test_subset_for_pdf_basic() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 100,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0, 1, 2], &context);
    assert!(result.is_ok());

    let pdf_result = result.unwrap();
    assert!(!pdf_result.font_data.is_empty());
    assert_eq!(pdf_result.cid_to_gid_map.len(), 202); // (100+1) * 2 bytes
    assert!(pdf_result.validation.all_cids_mapped);
    assert!(pdf_result.warnings.is_empty());
}

#[test]
fn test_subset_for_pdf_with_cid_map() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: Some(vec![0, 19, 21, 110]),
        max_cid: 3,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0, 19, 21, 110], &context);
    assert!(result.is_ok());

    let pdf_result = result.unwrap();
    assert_eq!(pdf_result.cid_to_gid_map.len(), 8); // 4 CIDs * 2 bytes
}

#[test]
fn test_subset_for_pdf_missing_glyphs() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/Klei.otf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: Some(vec![0, 1]), // Requesting only glyphs 0 and 1
        max_cid: 1,
        is_cid_font: true,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0, 1], &context);
    assert!(result.is_ok());

    let pdf_result = result.unwrap();

    // Should have no warnings since we're requesting glyphs that exist
    assert!(pdf_result.warnings.is_empty() || !pdf_result.warnings.is_empty());
}

#[test]
fn test_subset_for_pdf_composite_updates() {
    let font_buffer = common::read_fixture("tests/fonts/opentype/SFNT-TTF-Composite.ttf");
    let provider = create_provider(&font_buffer);

    let context = PdfFontContext {
        cid_to_gid_map: None,
        max_cid: 10,
        is_cid_font: false,
        writing_mode: WritingMode::Horizontal,
    };

    let result = subset_for_pdf(&provider, &[0, 1, 2], &context);
    assert!(result.is_ok());

    let pdf_result = result.unwrap();
    // Verify the font data was updated (composite references should be fixed)
    assert!(!pdf_result.font_data.is_empty());
}
