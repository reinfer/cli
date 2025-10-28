//! Text span conversion utilities

use openapi::models::{CommentSpanNewUtf16, EntityNewSpansInner, Span, TextSpan};

/// Convert TextSpan to CommentSpanNewUtf16 for API requests
pub fn convert_text_span(span: TextSpan) -> CommentSpanNewUtf16 {
    CommentSpanNewUtf16 {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
    }
}

/// Convert TextSpan to EntityNewSpansInner for API requests
pub fn convert_text_span_to_entity_spans_inner(span: TextSpan) -> EntityNewSpansInner {
    EntityNewSpansInner {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
        char_start: span.char_start,
        char_end: span.char_end,
    }
}

/// Convert TextSpan to Span for API requests
pub fn convert_text_span_to_span(span: TextSpan) -> Span {
    Span {
        content_part: span.content_part,
        message_index: span.message_index,
        utf16_byte_start: span.utf16_byte_start,
        utf16_byte_end: span.utf16_byte_end,
        char_start: span.char_start,
        char_end: span.char_end,
    }
}
