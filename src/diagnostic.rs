//! The diagnostic types (SPEC 7 and 8.5). Diagnostics are notes about places in the input that
//! the parser read one way and an author or another tool may read differently. They never change
//! the tree or the typed view.

use crate::tree::Span;

/// A note about a place in the input. It never changes the tree or the typed view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// Which note this is.
    pub code: DiagnosticCode,
    /// Always [`DiagnosticCode::severity`] of `code`.
    pub severity: Severity,
    /// The part of the source the note is about, on character boundaries.
    pub span: Span,
}

impl Diagnostic {
    /// A diagnostic with the severity its code has.
    pub(crate) fn new(code: DiagnosticCode, span: Span) -> Diagnostic {
        Diagnostic {
            code,
            severity: code.severity(),
            span,
        }
    }
}

/// What a [`Diagnostic`] is about. The variants follow the table in SPEC 7, and each code fires
/// only on the shape named in its trigger.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticCode {
    /// A whitespace-only line between two non-blank lines that is not a continuation line.
    BlankLookalike,
    /// The document uses more than one of `\n`, `\r\n` and `\r`.
    MixedLineEndings,
    /// A line with a blank line before and a text line after, with no lowercase and no uppercase
    /// letter but a digit or a caseless letter: it needs `@` to be a cue.
    CueNeedsForce,
    /// A line with a heading prefix that is not a heading because a blank line is missing.
    UnframedHeading,
    /// A heading that is only a prefix and its period or space.
    EmptyHeading,
    /// A line that starts with a single `.` followed by a character that is neither alphanumeric nor `.`,
    /// outside the title page.
    PeriodNotForcing,
    /// A heading ends with something shaped like `#...#` that is not a scene number.
    SceneNumberRejected,
    /// A framed, uppercase, one-line paragraph that is exactly `FADE OUT.`, `FADE TO BLACK.` or
    /// `CUT TO BLACK.`. `FADE IN:` is Action in every reading, so it stays quiet.
    TransitionLookalike,
    /// An uppercase line ending in `TO:` that is not a Transition because a blank line is missing.
    UnframedTransition,
    /// An uppercase line ending in `TO:` and then spaces or tabs.
    ToTrailingSpace,
    /// A Character cue with a caret that has no partner.
    LoneCaret,
    /// A cue line with a `^` followed by more text.
    CaretNotLast,
    /// A Dialogue line that starts with `(` and does not end with `)`, or that ends with `)`, contains no `(`,
    /// and closes such a line earlier in the same block (no line ending in `)` between them).
    PartialParenthetical,
    /// An emphasis opener with no closer on its line and a closer on a later line.
    EmphasisLineBreak,
    /// A `[[` inside a Note, or a `/*` inside a Boneyard.
    NestedOpener,
    /// A `[[` with no `]]` before the next blank line.
    UnclosedNote,
    /// A `/*` with no `*/` after it.
    UnclosedBoneyard,
    /// A `]]` or `*/` outside any span.
    StrayCloser,
    /// A line in the title page block that is neither a key line nor a valid continuation.
    TitlePageStrayLine,
    /// A text line directly after a Section, Synopsis or Page Break line, with no blank between, where
    /// the text line is not itself a Section, Synopsis or Page Break.
    AdjacentOutlineLine,
    /// A line that starts with `==` and is not a Page Break.
    AmbiguousEquals,
    /// A line with a forcing sigil directly inside a Dialogue block.
    SigilInDialogue,
}

impl DiagnosticCode {
    /// The kebab-case name used in SPEC 7, such as `"mixed-line-endings"`.
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticCode::BlankLookalike => "blank-lookalike",
            DiagnosticCode::MixedLineEndings => "mixed-line-endings",
            DiagnosticCode::CueNeedsForce => "cue-needs-force",
            DiagnosticCode::UnframedHeading => "unframed-heading",
            DiagnosticCode::EmptyHeading => "empty-heading",
            DiagnosticCode::PeriodNotForcing => "period-not-forcing",
            DiagnosticCode::SceneNumberRejected => "scene-number-rejected",
            DiagnosticCode::TransitionLookalike => "transition-lookalike",
            DiagnosticCode::UnframedTransition => "unframed-transition",
            DiagnosticCode::ToTrailingSpace => "to-trailing-space",
            DiagnosticCode::LoneCaret => "lone-caret",
            DiagnosticCode::CaretNotLast => "caret-not-last",
            DiagnosticCode::PartialParenthetical => "partial-parenthetical",
            DiagnosticCode::EmphasisLineBreak => "emphasis-line-break",
            DiagnosticCode::NestedOpener => "nested-opener",
            DiagnosticCode::UnclosedNote => "unclosed-note",
            DiagnosticCode::UnclosedBoneyard => "unclosed-boneyard",
            DiagnosticCode::StrayCloser => "stray-closer",
            DiagnosticCode::TitlePageStrayLine => "title-page-stray-line",
            DiagnosticCode::AdjacentOutlineLine => "adjacent-outline-line",
            DiagnosticCode::AmbiguousEquals => "ambiguous-equals",
            DiagnosticCode::SigilInDialogue => "sigil-in-dialogue",
        }
    }

    /// How serious this code is: `Warning` for the codes SPEC 7 marks so, `Info` for the rest.
    pub fn severity(self) -> Severity {
        match self {
            DiagnosticCode::UnframedTransition
            | DiagnosticCode::ToTrailingSpace
            | DiagnosticCode::LoneCaret
            | DiagnosticCode::UnclosedNote
            | DiagnosticCode::UnclosedBoneyard
            | DiagnosticCode::TitlePageStrayLine => Severity::Warning,
            _ => Severity::Info,
        }
    }
}

/// How serious a [`Diagnostic`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// The input is read as specified, and another tool may read it differently.
    Info,
    /// The input is probably not what the author meant, or text was demoted or left unclosed.
    Warning,
}
