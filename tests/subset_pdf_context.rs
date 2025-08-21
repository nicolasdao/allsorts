// Phase 2 TDD Tests - PDF Context

use allsorts::subset::context::FontEncoding;
use allsorts::subset::SubsetError;
use allsorts::subset::pdf::{PdfFontContext, PdfFontType};

#[test]
fn test_pdf_context_identity_h_constructor() {
    let ctx = PdfFontContext::identity_h();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: false });
    assert_eq!(ctx.max_cid, None);
    assert!(!ctx.preserve_identity);
    assert!(!ctx.is_symbolic);
}

#[test]
fn test_pdf_context_identity_v_constructor() {
    let ctx = PdfFontContext::identity_v();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: true });
}

#[test]
fn test_pdf_context_from_pdf_dict_valid() {
    let result = PdfFontContext::from_pdf_dict("Identity-H", 0);
    assert!(result.is_ok());
    
    let ctx = result.unwrap();
    assert_eq!(ctx.encoding, FontEncoding::Identity { vertical: false });
    assert!(!ctx.is_symbolic);
}

#[test]
fn test_pdf_context_from_pdf_dict_symbolic_flag() {
    let result = PdfFontContext::from_pdf_dict("Identity-H", 0x04);
    assert!(result.is_ok());
    
    let ctx = result.unwrap();
    assert!(ctx.is_symbolic);  // 0x04 is symbolic flag
}

#[test]
fn test_pdf_context_from_pdf_dict_invalid_encoding() {
    let result = PdfFontContext::from_pdf_dict("Unknown-Encoding", 0);
    assert!(result.is_err());
    
    match result {
        Err(SubsetError::UnsupportedEncoding(enc)) => {
            assert_eq!(enc, "Unknown-Encoding");
        }
        _ => panic!("Expected UnsupportedEncoding error"),
    }
}

#[test]
fn test_pdf_context_builder_methods() {
    let ctx = PdfFontContext::identity_h()
        .with_max_cid(1000)
        .preserve_identity();
    
    assert_eq!(ctx.max_cid, Some(1000));
    assert!(ctx.preserve_identity);
}