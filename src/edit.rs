//! Targeted edits (`SPEC.md` 10.2): replace one span of the source and re-parse.

use crate::{Document, Span, parse};

impl Document {
    /// Replaces the bytes of `span` with `text`, re-parses, and leaves every other byte
    /// untouched. The named edits are built on this primitive.
    ///
    /// ```
    /// use fountain_syntax::{Span, parse};
    ///
    /// let doc = parse("INT. A #1#\n\nHello.\n");
    /// let edited = doc.with_replaced(Span::new(7, 10), "#12A#");
    /// assert_eq!(edited.source(), "INT. A #12A#\n\nHello.\n");
    /// ```
    ///
    /// # Panics
    ///
    /// When the span is out of range, inverted, or not on character boundaries, as
    /// [`String::replace_range`] does.
    pub fn with_replaced(&self, span: Span, text: &str) -> Document {
        let mut source = self.source().to_owned();
        source.replace_range(span.start..span.end, text);
        parse(&source)
    }
}
