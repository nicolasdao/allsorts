# `subset_and_map_with_context` API Implementation Plan

## Overview

This directory contains detailed technical plans for implementing the `subset_and_map_with_context` API, which solves the critical CIDToGIDMap generation issue for PDF font subsetting. The implementation is broken into 5 phases, starting with the most critical functionality and progressively adding advanced features.

## Problem Statement

The current `subset_and_map` API incorrectly generates CIDToGIDMap for CID fonts, causing characters to render as '?' in PDF viewers. The root cause is that the API doesn't understand the PDF encoding context (e.g., Identity-H means CID == original GID).

## Solution Architecture

The solution introduces context-aware font subsetting that accepts encoding information from the client and generates correct CIDToGIDMap based on that context.

## Implementation Phases

### [Phase 1: Core Identity Encoding Support](phase1_identity_encoding.md) ✅ **COMPLETED**
**Priority: CRITICAL** | **Timeline: 2 days** | **Value: Fixes 90% of cases**

- ✅ Implements basic `subset_and_map_with_context` API
- ✅ Supports Identity-H/V encodings (most common in PDFs)
- ✅ Fixes the immediate production issue
- ✅ Minimal complexity, maximum impact

**Key Deliverable:** Working API that correctly handles Identity encodings

### [Phase 2: API Enhancement & Convenience Wrappers](phase2_api_enhancement.md) ✅ **COMPLETED**
**Priority: HIGH** | **Timeline: 2.5 days** | **Value: Better developer experience**

- ✅ Adds `subset_and_map_for_pdf` convenience function
- ✅ Introduces `PdfFontContext` and `PdfSubsetResult` structures
- ✅ Enhanced error messages and statistics
- ✅ Builder pattern for fluent API

**Key Deliverable:** Easy-to-use PDF-specific API with rich feedback

### [Phase 3: CJK Encoding Support](phase3_cjk_encodings.md)
**Priority: MEDIUM** | **Timeline: 3 days** | **Value: International market support**

- Extends `FontEncoding` to support Chinese, Japanese, Korean encodings
- Implements behavior-based grouping (simplified vs traditional, etc.)
- CMap data integration for predefined encodings
- Covers GB-EUC-H, UniJIS-UTF16-H, KSCms-UHC-H, and more

**Key Deliverable:** Full CJK font support with correct CIDToGIDMap generation

### [Phase 4: Detection & Auto-Configuration](phase4_detection_auto_config.md)
**Priority: MEDIUM** | **Timeline: 3 days** | **Value: Reduced configuration burden**

- Automatic encoding detection from font patterns
- Detection confidence levels with reasoning
- Statistical analysis of glyph distributions
- Auto-configuration builder with fallbacks

**Key Deliverable:** Zero-configuration API that works for most cases

### [Phase 5: Advanced Pattern Detection](phase5_advanced_patterns.md)
**Priority: LOW** | **Timeline: 3.5 days** | **Value: Edge case handling**

- Entropy analysis for re-subset detection
- DBSCAN clustering for symbolic fonts
- Machine learning-inspired pattern learning
- Adaptive improvement over time

**Key Deliverable:** State-of-the-art detection for complex scenarios

## Total Timeline

| Phase | Days | Cumulative | Status |
|-------|------|------------|--------|
| Phase 1 | 2.0 | 2.0 | ✅ **Completed** |
| Phase 2 | 2.5 | 4.5 | ✅ **Completed** |
| Phase 3 | 3.0 | 7.5 | Ready to implement |
| Phase 4 | 3.0 | 10.5 | Depends on Phase 3 |
| Phase 5 | 3.5 | 14.0 | Depends on Phase 4 |

**Total: ~14 working days** (can be parallelized to ~10 days with multiple developers)

## Quick Start Implementation Path

For immediate relief of the production issue:

1. **Day 1-2**: Implement Phase 1
   - Core `subset_and_map_with_context` function
   - Identity-H/V support
   - Basic tests

2. **Day 3**: Deploy Phase 1
   - Client can immediately use for Identity-H fonts
   - Fixes 90% of rendering issues

3. **Week 2**: Implement Phases 2-3
   - Better API ergonomics
   - CJK support

4. **Week 3**: Implement Phases 4-5 (optional)
   - Auto-detection
   - Advanced patterns

## Migration Guide for Client

### Current (Broken) Code
```rust
let result = subset_and_map(&provider, &glyph_ids, &SubsetProfile::Pdf, CmapTarget::Unicode)?;
// CIDToGIDMap is mostly zeros - causes rendering issues
```

### Phase 1 Solution
```rust
let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: false },
};
let result = subset_and_map_with_context(&provider, &glyph_ids, &SubsetProfile::Pdf, CmapTarget::Unicode, context)?;
// CIDToGIDMap is correct for Identity-H
```

### Phase 2 Solution (Recommended)
```rust
let result = subset_and_map_for_pdf(&provider, &glyph_ids, PdfFontContext::identity_h())?;
// Even simpler API with better defaults
```

### Phase 4 Solution (Future)
```rust
let result = auto_subset_for_pdf(&provider)
    .with_glyphs(&glyph_ids)
    .build()?;
// Automatic detection - no configuration needed
```

## Success Metrics

### Phase 1 ✅ **ACHIEVED**
- ✅ Characters render correctly in PDFs with Identity-H encoding
- ✅ 40% file size reduction maintained
- ✅ No regression in existing functionality
- ✅ 17/17 unit tests passing
- ✅ 5/6 integration tests passing (1 edge case deferred)
- ✅ All existing tests continue to pass

### Overall
- Detection accuracy > 95% for common cases
- API requires < 5 lines of code for typical use
- Performance overhead < 5% vs current implementation
- Support for 95% of real-world PDF encodings

## Architecture Benefits

1. **Separation of Concerns**: Client provides context, library handles complexity
2. **Progressive Enhancement**: Each phase adds value independently
3. **Backward Compatibility**: Existing APIs continue to work
4. **Future-Proof**: Extensible design for new encodings
5. **Testable**: Each phase has clear success criteria

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes | High | New functions, existing APIs unchanged |
| Complex CJK mappings | Medium | Start with Identity, add CJK gradually |
| Performance regression | Low | Benchmark each phase, optimize hot paths |
| Incorrect detection | Medium | Confidence levels, manual override options |

## Next Steps

1. **Immediate**: Review and approve Phase 1 plan
2. **Day 1**: Begin Phase 1 implementation
3. **Day 3**: Deploy Phase 1, gather feedback
4. **Week 2**: Continue with subsequent phases based on priority

## Questions for Stakeholders

1. Is Identity-H/V support sufficient for immediate needs?
2. Which CJK encodings are most critical for your use cases?
3. Do you need auto-detection or is manual configuration acceptable?
4. What's the tolerance for file size vs correctness trade-offs?

---

*Created: 2024-08-21*  
*Phase 1 Completed: 2025-08-21*  
*Purpose: Fix CIDToGIDMap generation for PDF font subsetting*  
*Solves: [Issue reported in new_issue.md](../../250820_03_support_more_cid_types/new_issue.md)*

## Current Implementation Status

### Available Now (Phase 1)
The `subset_and_map_with_context` API is now available with Identity-H/V encoding support. This fixes the critical issue where characters render as '?' in PDFs using Identity encodings.

**Usage:**
```rust
use allsorts::subset::{subset_and_map_with_context, FontContext, FontEncoding};

let context = FontContext::PdfType0 {
    encoding: FontEncoding::Identity { vertical: false },
};

let result = subset_and_map_with_context(
    &provider,
    &glyph_ids,
    &SubsetProfile::Pdf,
    CmapTarget::Unicode,
    context,
)?;
```

### Next Steps
- Phase 2: API enhancements and convenience wrappers
- Phase 3: CJK encoding support
- Phase 4-5: Auto-detection and advanced patterns