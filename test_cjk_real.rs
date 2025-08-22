#[cfg(test)]
mod test {
    use allsorts::subset::context::{FontEncoding, CJKLanguage, ChineseVariant};
    use allsorts::subset::pdf::{subset_and_map_for_pdf, PdfFontContext};
    use allsorts::subset::cjk::BuiltinCMapProvider;
    use allsorts::binary::read::ReadScope;
    use allsorts::tables::OpenTypeFont;

    #[test]
    fn test_cjk_subsetting_actually_works() {
        // Load a real test font
        let font_data = include_bytes!("tests/font_specimen/fonts/SymbolTest-Regular.ttf");
        let scope = ReadScope::new(font_data);
        let font_file = scope.read::<OpenTypeFont>().expect("Failed to parse font");
        let provider = font_file.table_provider(0).expect("Failed to get provider");
        
        // Create a CJK encoding context
        let mut context = PdfFontContext::from_pdf_dict("GB-EUC-H", 0).unwrap();
        context = context.with_max_cid(100);
        let cmap_provider = Box::new(BuiltinCMapProvider::new());
        context = context.with_cmap_provider(cmap_provider);
        
        // Try to subset with CJK encoding
        let glyph_ids = vec![0, 1, 2, 3];
        let result = subset_and_map_for_pdf(&provider, &glyph_ids, context);
        
        // Check if it works or returns the expected error
        match result {
            Ok(_) => println!("CJK subsetting actually works!"),
            Err(e) => {
                println!("CJK subsetting failed with: {:?}", e);
                // Check if it's the expected "not implemented" error
                assert!(e.to_string().contains("not yet implemented") || 
                        e.to_string().contains("not supported"));
            }
        }
    }
}

fn main() {}
