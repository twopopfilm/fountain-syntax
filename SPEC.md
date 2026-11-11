# SPEC: how fountain-syntax reads and writes Fountain

**Status: accepted for 0.1 (issue #4).** The maintainer approved it on 2026-11-05. There is no code yet. Every test ID
in this file names a test to be written in milestones F2 to F4. Section 13 lists what is still open.

This is the contract for the 0.1 release. It states what the library treats as standard, which extensions it supports,
how every ambiguous case is decided, and the shape of the API and of the hashes. It is written in our own words:
the text of fountain.io has no licence, so none of it is copied.

## Contents

1. [Scope and sources](#1-scope-and-sources)
2. [Ruling principles](#2-ruling-principles)
3. [Input model](#3-input-model)
4. [Standard elements](#4-standard-elements)
5. [Extensions and feature flags](#5-extensions-and-feature-flags)
6. [Decision table](#6-decision-table)
7. [Diagnostics](#7-diagnostics)
8. [Syntax tree API sketch](#8-syntax-tree-api-sketch)
9. [Typed view API sketch](#9-typed-view-api-sketch)
10. [Writer](#10-writer)
11. [Hashes: elements, scenes and documents](#11-hashes-elements-scenes-and-documents)
12. [Test IDs](#12-test-ids)
13. [Open items](#13-open-items)

## 1. Scope and sources

**Standard** means Fountain 1.1 (14 March 2014), the current official revision. **Extension** means syntax or behaviour
that 1.1 does not define. A **decision** is a ruling in the [decision table](#6-decision-table) on something 1.1 leaves
open or on a place where tools disagree.

Inputs, all merged and all evidence rather than rulings:

- [docs/research/official.md](docs/research/official.md): the official syntax element by element, and the 24 open
  cases `OFF-1` to `OFF-24`, each with competing readings (Reading A, B, C and so on). This file uses its reading
  letters.
- [docs/research/reference-parser.md](docs/research/reference-parser.md): a static reading of the Objective-C reference
  parser for each case: its verdict against the literal text (agrees, contradicts, does not cover), the reading it
  matches, and where its older "legacy" regex parser differs. The legacy notes are lower confidence. Where that file
  says "not determined by reading", this file carries the uncertainty.
- [docs/research/extensions.md](docs/research/extensions.md): what six apps and libraries do beyond 1.1, a ranking of
  candidate extensions, and the points that `SPEC.md` has to settle.
- [docs/brief.md](docs/brief.md): the goals of the crate.

Words of obligation: **must** is required for conformance with this file, **should** is a strong recommendation, **may**
is optional.

**Two layers.** The library has two views of a document, and the rest of this file keeps them apart:

- the **syntax tree** (sections 8 and 10) keeps every byte of the source and is the thing the writer reproduces;
- the **typed view** (section 9) is the screenplay as an author thinks of it: title page, scenes, elements. Every
  recognition rule below decides what the typed view contains. The tree always keeps the sigils, whitespace and hidden
  text that the view leaves out.

## 2. Ruling principles

The maintainer's rulings follow three principles, applied in this order. Every row of the decision table cites the
principle that decided it.

1. **P1, literal text first.** Where the literal text of Fountain 1.1 decides a case, follow it, even when the result is
   unusual. Where two statements in the spec collide, the more specific one wins, and the row says so.
2. **P2, silent text: lose nothing, fall back to Action.** Where the spec is silent, take the reading that loses no
   visible text and, when in doubt, returns the text as Action (the spec's own Error Handling principle). Text must
   never disappear from the typed view because of a guess.
3. **P3, disagreement is reported, not copied.** Where the reference parser or an app reads the same input differently,
   this library does not copy it. It emits a [diagnostic](#7-diagnostics) when the difference could hide or demote text,
   or when the author may have meant something else. A row cites P3 when it has such a diagnostic.

A row that applies P1 or P2 may also list a diagnostic. Diagnostics never change the tree or the typed view; they only
tell the caller that an input was a judgement call.

## 3. Input model

### 3.1 Text and encoding

- `parse` takes `&str`. Fountain is a text format and the library never fails. Invalid UTF-8 cannot be held in a `&str`,
  and silently replacing bytes would break byte-for-byte round trips, so the caller decides at the boundary: decode, or
  refuse. The library offers no lossy decoding in 0.1. Bytes would only move the decision into the library without a
  lossless answer to give.
- A byte-order mark (U+FEFF) at the very start is kept in the tree as leading trivia. It is not content. A U+FEFF
  anywhere else is an ordinary character.
- Spans are byte offsets into the `&str` and always fall on character boundaries.
- Text is never converted: no curly quotes, dashes or ellipses (the spec's Punctuation section). Parsing and the typed
  view apply no Unicode normalisation either. Only the scheme 2 hashes do, on their own copy of the text (11.3).

### 3.2 Lines

- A **line break** is `\n`, `\r\n` or `\r`. A file may mix them. The exact bytes are kept in the tree.
- No other character is a line break: U+0085, U+2028, U+2029, vertical tab and form feed are ordinary characters.
- A final line break is optional. A file that ends with one has no extra empty line after it, and a file that does not
  is the same document.
- The **text of a line** is its content without the line break. Rules below that talk about "the last character" mean
  the last character of the text of the line, so a `\r` never counts as trailing whitespace.

### 3.3 Whitespace, blank lines, the edges of the document

- **Whitespace** in a line means spaces (U+0020) and tabs. Other Unicode spaces are ordinary characters.
- A **blank line** is a line whose text is empty or holds only whitespace, with two exceptions that are not blank:
  - a **continuation line** inside a Dialogue block: a line of exactly two spaces (decision rows `OFF-1b`, `OFF-1c`);
  - the same line inside an open Note.
- The **start of the document counts as preceded by a blank line, and the end counts as followed by one.** So a rule
  "needs a blank line before" is met at the first line, and a rule "must not be followed by a blank line" is not met by
  the last line. A title page ends at a blank line, which is also what lets the first body element follow it.
- **Leading whitespace is read as absent for recognition** (the spec's Indenting section: tabs and spaces are ignored
  for every element except Action). The tree and Action text keep it. Details in `OFF-18`.
- **Trailing whitespace is not significant** for recognition, with one exception that the spec states: spaces after a
  `TO:` line (`OFF-23a`). Whitespace after a dual-dialogue caret is ignored too (`OFF-23c`).

### 3.4 Hidden spans and transparent lines

Notes (`[[ ]]`) and Boneyard (`/* */`) hide text; Sections, Synopses and Page Breaks are single lines that do not print.
They affect recognition as follows. The tree keeps all of them.

1. **Finding spans.** Scan the document from the start. At each `/*` or `[[`, the opener that comes first wins and
   everything up to its closer is opaque (a `[[` inside a Boneyard is Boneyard text, and the other way round).
   - A Boneyard closes at the first `*/` after its opener (the search starts after the two opener characters, so `/**/`
     is an empty Boneyard). It may span blank lines.
   - A Note closes at the first `]]` after its opener. The search never crosses a blank line (the spec's "double line
     break" limit), except through continuation lines.
   - An opener with no closer is **plain text** and the scan resumes right after it. A closer with no opener is plain
     text.
2. **Removal before detection.** For recognition, hidden spans are deleted as characters, including any line breaks
   inside them. If a deleted span began on one line and ended on another, the two lines fuse into one.
3. **Invisible lines.** A line that held nothing but hidden spans and whitespace is **invisible**: it is neither blank
   nor text. A Section, Synopsis or Page Break line is **transparent** in the same way. Invisible and transparent lines
   are skipped when a rule looks at the neighbouring line, so they neither end a Dialogue block nor satisfy a "blank
   line before" rule.
4. A hidden span in the middle of a line leaves the rest of the line as it is: `Mara /* aside */ waits.` is read as
   `Mara  waits.`

### 3.5 Shared definitions

These are used by many rows, so they are defined once.

- **Uppercase text** has at least one uppercase letter and no lowercase letter, by the Unicode Uppercase and Lowercase
  properties. Digits, punctuation and caseless letters (such as Han characters) are neutral. The word "uppercase" in any
  row means this.
- **Alphanumeric** means a Unicode letter or a Unicode decimal digit. `OFF-7` and `OFF-8` both use it.
- **Sigil** means a forcing character at the start of a line: `.`, `!`, `@`, `~`, `>` (and `!!` with the `shot` flag).
  Structural markers (`#`, `=`, `===`, `^`, `<`, `[[ ]]`, `/* */`) are not called sigils.
- **Visible line** means a line after hidden-span removal that is not invisible or transparent.

### 3.6 The reference parser's two root causes

[reference-parser.md](docs/research/reference-parser.md) traces most of its surprising results to two habits. This
library has neither, and the rows `OFF-1`, `OFF-2` and `OFF-18` say so by name.

- **Root cause 1:** it removes all leading whitespace from the whole document before anything else, so the first line of
  a file is never tested with its indent and leading blank lines vanish. Here nothing is removed. The first line is read
  by the same rules as every other line, and leading blank lines are trivia in the tree.
- **Root cause 2:** it treats only a zero-length line as empty. Here a whitespace-only line is blank (with the narrow
  exceptions above), because an invisible stray space must not glue two paragraphs together or keep a Dialogue block
  open. Beat, Highland and Fade In treat non-printing lines as blank in some cases too.

## 4. Standard elements

Each subsection gives the recognition rule, the sigil, what the typed view exposes, and the element tests (`EL-`). The
element tests cover the ordinary case; the decision table covers every ambiguous case.

### 4.0 Recognition order

For each visible line, the first rule that fits wins. The title page is handled first, at the start of the document
(4.1).

1. **A sigil or structural marker as the first non-whitespace character**, in every context (also inside a Dialogue
   block, see `OFF-24`):
   - `~` makes Lyrics;
   - `!` makes Action;
   - `@` followed by at least one non-whitespace character makes a Character cue;
   - `.` followed by an alphanumeric character makes a Scene Heading;
   - `>` makes Centered text or a Transition (`OFF-10`);
   - `#` makes a Section;
   - `=` makes a Page Break or a Synopsis (`OFF-19`).
2. **Inside a Dialogue block:** a Parenthetical or Dialogue line (4.5, 4.6).
3. **By framing:** a Scene Heading, a Transition or a Character cue, each of which needs blank lines around it.
4. **Otherwise Action.**

**Carve-outs from the framing rules** (each is a decision row):

- an uppercase line that ends in `TO:` plus spaces or tabs is never a Transition and never a Character cue; it is Action
  (`OFF-23a`, `OFF-23b`);
- a cue name needs an uppercase letter (`OFF-4c`), well-formed extension groups (`OFF-3c`) and a caret that is last
  (`OFF-11c`); a line that fails any of these goes on to the later rules;
- a bare prefix is a heading (`OFF-6d`), and a line that is a heading by prefix is never a title-page key (`OFF-17a`).

Forced elements need no blank lines around them. A forcing sigil is only looked at as the first character after any
leading whitespace, and only that one character is a sigil (`OFF-22`).

### 4.1 Title page

- **Recognition.** The first block of the document, if it starts with a key line. A **key line** is a key, a colon and
  an optional value on the same line. A key is made of alphanumerics, spaces, hyphens, underscores, apostrophes and
  ampersands. It ends at the first colon. A line that is a Scene Heading by prefix is never a key line. A key with no
  inline value starts a title page only when an indented line follows it. Details and edge cases: `OFF-17a` to
  `OFF-17f`.
- **Values.** On the key line, or on the lines below it indented by three or more spaces or a tab, or both. Keys are
  kept as written, may repeat, and may be any text (open keys, see 5.1).
- **End.** At the first blank line. A line inside the block that is neither a key line nor a valid continuation is kept
  as a **stray line**, not attached to any key (`OFF-17d`).
- **Sigil.** None.
- **View.** `Screenplay.title_page`, a list of entries with typed access to the common keys.
- **Tests.** `EL-title-page-1` (keys with inline values), `EL-title-page-2` (indented multi-line values),
  `EL-title-page-3` (typed access ignores key case; unknown keys are kept), `EL-title-page-4` (the body starts after the
  first blank line).

### 4.2 Scene Heading

- **Recognition.** A visible line whose text starts with `INT`, `EXT`, `EST`, `INT./EXT`, `INT/EXT` or `I/E`, in any
  case, followed by a period or a space, with a blank line before and after (`OFF-6`).
- **Sigil.** A single `.` followed by an alphanumeric character forces a heading, with no prefix and no blank lines
  needed (`OFF-7`). The view's text leaves the `.` out.
- **Scene number.** `#n#` at the end of the heading, after a space or tab, made of alphanumerics, `-` and `.` (`OFF-8`).
  The view exposes it apart from the heading text.
- **Tests.** `EL-heading-1` (each prefix, upper and lower case), `EL-heading-2` (forced heading), `EL-heading-3` (scene
  number), `EL-heading-4` (not a heading without a blank line before or after).

### 4.3 Action

- **Recognition.** The fallback. A run of consecutive visible lines that no other rule claims. Every line break is kept,
  and blank lines between paragraphs are not content.
- **Sigil.** `!` forces Action, for uppercase lines that would read as a cue. The view's text leaves the `!` out. The
  lines that follow a `!` line join its paragraph.
- **Whitespace.** The view keeps leading spaces and tabs in Action text. A helper expands each tab to four spaces for
  display only; the tree and the hashes are unaffected (`OFF-18c`).
- **Tests.** `EL-action-1` (a paragraph keeps its lines), `EL-action-2` (indentation is kept in the view),
  `EL-action-3` (forced Action and the lines joined to it), `EL-action-4` (tab expansion is display only).

### 4.4 Character cue

- **Recognition.** A visible line with a blank line before it and a visible, non-blank line after it, whose **name** is
  uppercase text, optionally followed by parenthetical **extensions** in any case, optionally followed by a caret
  (`OFF-3`, `OFF-4`, `OFF-5`, `OFF-11`). An uppercase line ending in `TO:` plus trailing whitespace is never a cue
  (`OFF-23b`).
- **Sigil.** `@` forces a cue and keeps the name's case. The view's name leaves the `@` out.
- **View.** `Cue { name, extensions, forced }`. The caret is a token in the tree and decides pairing (4.7). `(CONT'D)`
  is an ordinary extension, kept as written.
- **Tests.** `EL-character-1` (uppercase cue with an extension), `EL-character-2` (forced `@` cue keeps case),
  `EL-character-3` (an uppercase line with nothing after it is Action).

### 4.5 Dialogue

- **Recognition.** The visible lines after a cue (or a Parenthetical) up to the next blank line. This is a **Dialogue
  block**. Only a blank line, or a forced `.`, `!`, `>` or `@` line, ends it (`OFF-24`).
- **Line breaks.** Lines are kept as written. An intentionally empty line inside the block is a line of exactly two
  spaces (`OFF-1b`).
- **Tests.** `EL-dialogue-1` (lines kept), `EL-dialogue-2` (two-space continuation line), `EL-dialogue-3` (a blank line
  ends the block and the next line is Action).

### 4.6 Parenthetical

- **Recognition.** Inside a Dialogue block, a line that starts with `(` and ends with `)`, after trimming (`OFF-12`).
- **Tests.** `EL-parenthetical-1` (between cue and dialogue, and between two dialogue stretches).

### 4.7 Dual Dialogue

- **Recognition.** A cue whose last non-whitespace character is `^`, directly after a Dialogue block, pairs the two
  blocks (`OFF-11`).
- **View.** `DualDialogue { left, right }`. The tree keeps the caret as a token of the second cue.
- **Tests.** `EL-dual-1` (a pair), `EL-dual-2` (the caret is kept in the tree and left out of the view's name).

### 4.8 Lyrics

- **Recognition.** A line starting with `~`. Always forced, never detected. A run of consecutive `~` lines is one Lyrics
  element. The view's text leaves each `~` out (`OFF-13`).
- **Tests.** `EL-lyrics-1` (a run of lines), `EL-lyrics-2` (a lyric among dialogue lines).

### 4.9 Transition

- **Recognition.** A visible line that is uppercase, ends exactly in `TO:` and has a blank line before and after
  (`OFF-9`, `OFF-23`).
- **Sigil.** `>` forces any line to be a Transition, in any case, without blank lines. The view's text leaves the `>`
  out (`OFF-10`, `OFF-22`).
- **Tests.** `EL-transition-1` (automatic), `EL-transition-2` (forced).

### 4.10 Centered text

- **Recognition.** A line whose first non-whitespace character is `>` and whose last is `<`. It is an Action element
  marked as centered. The view's text leaves both marks out and trims (`OFF-10`).
- **Tests.** `EL-centered-1` (marks flush against the text), `EL-centered-2` (spaces inside the marks, and trailing
  whitespace after `<`).

### 4.11 Emphasis

- **Recognition.** An inline layer over the visible text of any element, in the view only. `*` is italics, `**` bold,
  `***` bold italics, `_` underline. Spans nest and never cross a line break (`OFF-14`). Emphasis never affects which
  element a line is: a line that starts with `**` is not a heading, whatever follows.
- **Escape.** A backslash before `*` or `_` makes it literal; the view drops the backslash.
- **Tests.** `EL-emphasis-1` (each style and nesting), `EL-emphasis-2` (escapes and unmatched marks stay literal).

### 4.12 Page Break

- **Recognition.** A line of three or more `=` and nothing else (`OFF-19`).
- **Tests.** `EL-page-break-1` (three and more equals signs).

### 4.13 Sections and Synopses

- **Section.** A line starting with `#`. Depth is the number of leading `#`, with no maximum (`OFF-20`).
- **Synopsis.** A line starting with a single `=` (`OFF-19`). A line starting with `==` that is not a Page Break is
  Action.
- Neither prints, neither needs blank lines around it, and both are transparent (3.4).
- **Tests.** `EL-section-1` (depths 1 to 4), `EL-synopsis-1` (a synopsis after a heading).

### 4.14 Notes and Boneyard

- **Note.** `[[ ... ]]`, inline or on its own line, possibly over several lines (`OFF-15`).
- **Boneyard.** `/* ... */`, inline or over many lines, blank lines included (`OFF-16`).
- Both are hidden for recognition and hashes. The view keeps them: a whole-line one is an element in scene order, an
  inline one is a run inside its text.
- **Tests.** `EL-note-1` (inline and own-line notes), `EL-boneyard-1` (a multi-line Boneyard hides a whole scene from
  the view but not from the tree).

### 4.15 Punctuation, line breaks and indenting

- `EL-punctuation-1`: typed punctuation passes through the view unchanged.
- `EL-linebreak-1`: every line break in Action and Dialogue is kept.
- `EL-indent-1`: a Character cue, Parenthetical and Dialogue with any indentation (tabs, 12, 8, 4 or 0 spaces) are read
  as if unindented (`OFF-18`).

## 5. Extensions and feature flags

The survey ([extensions.md](docs/research/extensions.md)) ranks 15 candidates. The ruling follows the maintainer's
choice: a safe set that changes no parse of valid 1.1 input, plus two behind Cargo feature flags. Nothing else is
supported in 0.1.

### 5.1 Always on (no parse change)

These add no flag because valid 1.1 input parses the same with or without them.

| Item | What is preserved | What the view exposes | Tests |
|---|---|---|---|
| **Open title-page keys** (survey 2) | Any key, in any case, repeated or not, is kept. | Case-insensitive lookup. Typed accessors for title, credit, author or authors, source, draft date, contact, notes, copyright and date. Highland's image keys (`TC`, `CC`, `BC`) are ordinary keys with no accessor. | `EXT-open-keys-1`, `EXT-open-keys-2` |
| **Sections of any depth** (survey 3) | Any number of `#`. | `Section.depth` as `usize`. | `EXT-sections-1` |
| **`(CONT'D)` kept as written** (survey 5) | The extension stays exactly as typed. The library never adds, strips or normalises it. Scheme 2 hashes leave it out and scheme 1 hashes ignore every extension (section 11). | `Extension::is_contd()`, true for `CONT'D` in any case with a straight or curly apostrophe. | `EXT-contd-1`, `EXT-contd-2` |
| **Beat's revision boneyard** (survey 4) | The block Beat writes at the end of a file is an ordinary Boneyard: hidden from recognition and hashes, written back byte for byte. | Nothing special. `Boneyard::text()` returns its raw text. The library does not parse the JSON inside it. | `EXT-revision-1` |
| **Note conventions as an opt-in typed view** (survey 1) | The note is an ordinary Note. | `Note::conventions()` returns `Option<NoteConvention>`, computed on demand: a colour (a word, `color` and a word, or `#` and six hex digits), a marker (`marker`, an optional colour, an optional colon and text, as Beat uses) or a tag (one word and a colon at the start, as Slugline uses). Nothing is interpreted unless the caller asks. | `EXT-notes-view-1`, `EXT-notes-view-2` |

The research does not give Beat's list of colour names or the exact limit of Slugline's "short prefix" (section 13).
`NoteConvention` therefore returns the words as typed and leaves their meaning to the caller.

### 5.2 Behind Cargo feature flags

Both flags are off by default. Each changes how some valid 1.1 input parses, which is why it is a flag. Compile-time
features only: a parse depends on the input and the enabled features, never on runtime options.

| Flag | Syntax | Changes for valid 1.1 input | Tests |
|---|---|---|---|
| `locked-scene-numbers` | A heading ends with whitespace, `##`, one or more alphanumerics, `-` or `.`, and `#` (Slugline's locked number, `##12A#`). | Without the flag the trailing `##12A#` is plain heading text (it does not match the 1.1 number rule, `OFF-8g`). With it, the heading has a `SceneNumber` with `locked: true` and the heading text shortens. | 5.4 |
| `shot` | A line starting with `!!` (Beat's Shot, which Beat's manual says is its own feature). | Without the flag a `!!` line is forced Action whose text begins with `!` (`OFF-22g`). With it, the line is a `Shot` element, forced like `!`: no blank lines needed, the two marks left out of the view's text. | 5.4 |

### 5.3 Not supported in 0.1

None of these changes the tree: the text is kept and written back, and the view treats it as ordinary Action, Dialogue
or heading text. Braces, endnotes and similar markup are kept as opaque text.

| Survey item | Why it is not supported |
|---|---|
| 8, wider heading prefixes (`e/i`) and Beat's lowercase handling | Low value and a single app. Lowercase prefixes are already headings (1.1 is case-insensitive). New prefixes would change valid input. |
| 9, brace directives `{{ ... }}` | Beat and Highland use the same braces for different things. Kept as opaque text. |
| 10, line tags `#tag` | A single library, off by default there, and the text after the first tag is cut off. It shares `#` with Sections and scene numbers. |
| 11, Highland text styles (highlight, strikethrough, term lists, checklists, right alignment) | A single app, and the strikethrough token is not determined. |
| 12, endnotes `((...))` and `[^label]` references | A single app. `((` is ordinary text in 1.1. Kept as opaque text. |
| 13, full-width sigils | Niche, and such a line is Action in 1.1. |
| 14, extra automatic transitions (`FADE OUT.` and any `... TO:` line) | This is a 1.1 edge case, not an extension. It is ruled in `OFF-9`, and `FADE OUT.` stays Action with a diagnostic. |
| 15, includes, snippets, watermarks | Tool behaviour that rewrites text. Out of scope for a parser. |
| Revision marks beyond the boneyard block | The survey found no shared syntax. |
| Stage-play variants | None of the six defines stage-play syntax. |
| Final Draft (FDX) interchange | A separate crate after 0.1. |

### 5.4 Extension tests

| ID | Test |
|---|---|
| `EXT-open-keys-1` | Unknown title-page keys, repeated keys and keys in any case are all kept, in order. |
| `EXT-open-keys-2` | The typed accessors find `Author` and `Authors`, `Draft date`, `Notes`, `Copyright` and `Date` ignoring case. |
| `EXT-sections-1` | Sections of depth 1 to 8 are Sections with the right depth. |
| `EXT-contd-1` | `(CONT'D)`, `(cont'd)` and `(CONT’D)` are kept as written and `is_contd` is true for each. |
| `EXT-contd-2` | The library never adds or strips `(CONT'D)`: a cue without it and a cue with it both write back unchanged. |
| `EXT-revision-1` | A Beat-style revision block at the end of a file is an ordinary Boneyard: hidden from recognition and hashes, written back unchanged. |
| `EXT-notes-view-1` | `Note::conventions` finds a colour, a marker and a tag, and returns `None` for an ordinary note. |
| `EXT-notes-view-2` | Calling `conventions` changes nothing: the parse, the tree and the hashes are the same with or without it. |
| `EXT-locked-scene-numbers-1` | With the flag, `##12A#` at the end of a heading is a locked scene number. |
| `EXT-locked-scene-numbers-2` | Without the flag, the same input is plain heading text and `scene-number-rejected` is not required. |
| `EXT-locked-scene-numbers-3` | With the flag, an empty `##`, `###x#` and a missing space before the marks are not locked numbers. |
| `EXT-locked-scene-numbers-4` | With the flag, locked and ordinary numbers round-trip byte for byte and the hash ignores them. |
| `EXT-shot-1` | With the flag, a `!!` line is a Shot with the two marks left out of the text. |
| `EXT-shot-2` | Without the flag, `!!text` is forced Action whose text starts with `!`. |
| `EXT-shot-3` | With the flag, `!!!text` is a Shot with the text `!text`, and a `!!` line inside a Dialogue block ends the block. |

## 6. Decision table

### 6.1 How to read the table

- **One table per case.** Every case `OFF-1` to `OFF-24` is split into lettered rows, one per question. The row ID (for
  example `OFF-23b`) is the **case** (`OFF-23`) plus a letter, and it is also the **test ID** of that row: a test in F2
  to F4 carries the ID in its name.
- **Reading** is the letter of the competing reading in [official.md](docs/research/official.md) that the decision
  matches. "n/a" means the row asks a question official.md does not list.
- **Rule** is the principle from section 2 that decided the row (P1, P2, and P3 when the row has a diagnostic caused by
  disagreement).
- **Reference parser** restates what [reference-parser.md](docs/research/reference-parser.md) records for the case: its
  verdict against the literal text, the reading it matches, and where the legacy parser differs (lower confidence, as
  that file says). "Default" means the default line-by-line parser.
- **Diagnostic** is a code from [section 7](#7-diagnostics), or "none".
- The tree always keeps every byte. A decision is about what the typed view and the hashes see.

### 6.2 The cases

#### OFF-1: what is an empty line?

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-1a | Which whitespace-only lines are blank? | A line with only spaces and tabs is blank, apart from the continuation lines in 1b and 1c. This answers root cause 2 (3.6): a whitespace-only line is not content. The start and end of the document count as blank. | B | P2, P3 | Does not cover. Matches Reading A, plus an exact-two exception in Dialogue: only a zero-length line is empty (root cause 2). Legacy: no special rule, any whitespace-only line stays in the block. | `blank-lookalike` |
| OFF-1b | A line of exactly two spaces in a Dialogue block. | It is a continuation line: part of the block, an intentionally empty dialogue line. One space, three or more, or tabs are blank and end the block. | B | P1 | Agrees: exactly two whitespace characters keep the Dialogue going (it also accepts two tabs; this library does not). | none |
| OFF-1c | A line of exactly two spaces inside an open Note. | It does not end the Note. Any other blank line does (OFF-15g). | B | P1 | Does not cover: the Notes form of the two-space rule is not implemented (see OFF-15). | none |
| OFF-1d | Is leading whitespace removed from the whole document first? | No (root cause 1, 3.6). Leading blank lines and the first line's indent stay in the tree, and the first line is read like every other line. | n/a | P1, P2 | Contradicts on the first line: it removes leading blank lines, whitespace-only lines and the first content line's indent before parsing. Legacy: removes only leading line feeds. | none |

#### OFF-2: line endings, trailing newline, encoding

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-2a | Which bytes end a line? | `\n`, `\r\n` and `\r` each end one line. The bytes are kept per line, so mixed endings round-trip. Neither root cause applies: nothing is stripped from the document (root cause 1), and a `\r` is never part of a line's text. | A | P2 | Does not cover. Matches Reading A: all three become one line feed before any rule runs. Legacy: the title page is found before line endings are normalised, so a CRLF-only file appears to lose it. | `mixed-line-endings` |
| OFF-2b | Does a final newline matter? | It is optional and creates no extra line. The end of the document already counts as a blank line (3.3). | A | P2 | Does not cover. Matches A; two line feeds are appended, so a missing or present final newline changes nothing. | none |
| OFF-2c | Encoding and byte-order mark. | `parse` takes `&str`; invalid UTF-8 is the caller's decision. A leading U+FEFF is leading trivia, never content. | n/a | P2 | Does not cover. It reads UTF-8 and returns an empty result with only a log message if that fails. A byte-order mark is left to the platform (not determined by reading). | none |
| OFF-2d | Other Unicode line separators (U+0085, U+2028, U+2029, vertical tab, form feed). | Not line breaks. They are ordinary characters. | n/a | P2 | Does not cover. The platform's newline set may also split on some of these (not determined by reading). | none |

#### OFF-3: Character cue with extensions

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-3a | Case of the extension, and the name test. | The uppercase test applies to the name, the text before the first `(`. Extensions may be in any case: `NELL (V.O.)`, `MARA (cont'd)` and `MARA (on the radio, softly)` are all cues. `Mara (V.O.)` has a lowercase name and is Action. | A | P1 | Contradicts. Matches Reading B (whole line, the 1.0 rule) plus one hard-coded lowercase form, `(cont'd)`, so `(on the radio, softly)` makes Action. Legacy: no lowercase anywhere, `(cont'd)` fails too. | none |
| OFF-3b | More than one extension group. | Any number of groups, each a balanced pair of parentheses with no nested pair: `OTTO (V.O.) (CONT'D)` is a cue. | A | P2 | Agrees for `(V.O.) (CONT'D)`, which is a cue there. | none |
| OFF-3c | Text after a group, or an unclosed or nested group. | Not a cue. The line falls to the next rule, usually Action. | C | P2 | Does not cover. Any line with no ASCII lowercase letter is a cue there, so `MARA (HI) THERE` is a cue. | none |
| OFF-3d | An extension after a forced `@` name. | Allowed: `@deLaCruz (V.O.)`. The name keeps its case and is not tested for uppercase. | n/a | P2 | Does not cover; the `@` rule runs first and keeps the whole text. | none |
| OFF-3e | What happens to `(CONT'D)`. | It is an extension like any other, kept exactly as written. The library never adds, strips or rewrites it. Scheme 2 hashes leave `(CONT'D)` out of a cue and see every other extension, so the hash of a cue is the same with or without it. Scheme 1 hashes ignore every extension (11.3, 11.7). | n/a | P2 | Related: accepts only the exact lowercase `(cont'd)`. Per extensions.md, screenplay-tools strips it on read and adds it on write; Slugline adds it only when drawing. | none |

#### OFF-4: non-ASCII uppercase and caseless names

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-4a | Uppercase in other scripts. | Unicode-aware (3.5). `JOSÉ`, `ЕЛЕНА` and `ΣΩΚΡΑΤΗΣ` are cues. An all-lowercase `елена` is not. | A | P2 | Does not cover. Matches Reading C and looser: the test is only "no ASCII lowercase letter", so an all-lowercase Greek or Cyrillic line is a cue too. | none |
| OFF-4b | Caseless names such as `李明`. | Not an automatic cue (no uppercase letter). The author forces it with `@`. | A | P2, P3 | Cue (Reading C). | `cue-needs-force` |
| OFF-4c | A line with no letter, such as `23`. | Not a cue: the literal text asks for at least one alphabetical character. A name needs an uppercase letter. | n/a | P1, P3 | Contradicts. There is no letter test, so a line of digits is a cue. | `cue-needs-force` |
| OFF-4d | Digits and punctuation inside a name. | Allowed when an uppercase letter is present: `R2D2`, `MR. O'BRIEN`. | n/a | P1 | Agrees: both are cues. | none |

#### OFF-5: a cue at the end of the file, or followed by a blank line

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-5a | An uppercase line as the last line. | Action. **Tie-breaker adopted**: the end of the document counts as a blank line after the cue, so there is no dialogue. | B | P2 | Does not cover. Matches B: padded line feeds mean the last cue is never followed by text. Legacy: the same. | none |
| OFF-5b | An uppercase line followed by a blank line, including a whitespace-only one. | Action (the spec says a cue has no blank line after it). | B | P1 | Matches for a zero-length line. A whitespace-only line after it is not blank there, so it is still a cue. | none |

#### OFF-6: Scene Heading versus Action

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-6a | Are blank lines needed before and after? | Yes, both, as the spec says. Without them the line is part of the surrounding Action (or Dialogue). A forced heading (OFF-7) needs neither. Invisible and transparent lines are skipped when looking at the neighbours (3.4). | A | P1, P3 | Agrees on the blank line before. The blank line after is only partly enforced: a heading followed directly by a sigil, `TO:`, `=` or `#` line stays a heading. Legacy: lenient, one line break each side (Reading B). | `unframed-heading` |
| OFF-6b | What may follow the prefix? | A period or a space only. `INT HOUSE` is a heading; `INT\tHOUSE`, `INT-HOUSE` and `INTERIOR` are not. Only the six listed prefixes count. | A | P1 | Contradicts in part: also accepts a tab, a hyphen, and the prefixes `I/EXT`, `INT/E` and `I./E.`. Legacy: `INT` or `EXT` needs no separator at all. | none |
| OFF-6c | Prose that starts with a prefix, such as `Ext. 12 is busy.` | It is a heading when framed, because matching is case-insensitive. The author uses `!` to keep it Action. | C | P1 | Agrees: it is a heading there too. | none |
| OFF-6d | A bare prefix, such as `INT.` or `INT ` (with a trailing space). | A heading with empty location text: the prefix and a period or space is all the rule asks. | n/a | P1, P3 | Contradicts: `INT.` alone is not a heading there (the prefix needs a separator and one more character); `INT. ` with a trailing space is. | `empty-heading` |
| OFF-6e | Does the start of the file, or the end of a title page, count as "preceded by a blank line"? | Yes (3.3). | B2 | P2 | Agrees: both count. | none |

#### OFF-7: forced heading and "alphanumeric"

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-7a | What does "alphanumeric" mean? | A Unicode letter or decimal digit (3.5): `.LAMP ROOM`, `.5 HOURS LATER` and `.ÉPILOGUE` are forced headings. | B | P2 | Does not cover on A and B: the character after the period is never tested for being a letter or digit. | none |
| OFF-7b | A period followed by a space or a symbol (`. SPACE AFTER`, `.>HEADING`). | Not forced. The line goes on to the ordinary rules, usually Action. The tree keeps the text and the period. | C is out | P1, P3 | Contradicts. Matches Reading C: anything except a second period forces a heading, so both are headings (the period is dropped; a demoted forced heading would lose its text). Legacy: `..DOUBLE` seems to be a heading too. | `period-not-forcing` |
| OFF-7c | Two or more periods (`..DOUBLE`, `...trailing off`). | Not forced. | A | P1 | Agrees: `..DOUBLE` and `...trailing off` are not forced. | none |
| OFF-7d | Does a forced heading need blank lines? Is the period in its text? | No blank lines. The view's text leaves the `.` out; the tree keeps it. A forced heading is never demoted, so no text can be lost. | n/a | P1 | Does not cover. The step needs no blank line before it, and a demoted forced heading loses its dot with its text. Its writer does not restore the dot for `.INT. X`, `.INTERCUT` or `.EXTRA`. | none |

#### OFF-8: scene numbers

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-8a | Which characters may a scene number hold? | One or more Unicode alphanumerics (3.5), `-` and `.`. `#12#`, `#4B#`, `#É1#` and `#2-A.1#` are numbers. | B | P2 | Contradicts in part: anything but `#` is allowed between the marks, so `#É1#` is a number but so is almost anything. | none |
| OFF-8b | A space or underscore inside (`#1 A#`, `#1_A#`). | Not a number. The text stays in the heading. | E is out | P1, P3 | Contradicts: both are numbers (`1 A`, `1_A`), Reading E. Legacy: most likely reads the `#` pair as a Section and drops the heading (medium confidence). | `scene-number-rejected` |
| OFF-8c | Where may the number sit? | At the end of the line, with trailing whitespace allowed, and after at least one space or tab. Text after it (`#7# trailing words`) voids it. | C | P1, P3 | Agrees on the end of the line (`#7# trailing words` is not a number). | `scene-number-rejected` |
| OFF-8d | May a forced heading carry a number? | Yes: `.TITLE CARD #3#`. | D | P2 | Agrees: a number is taken for forced headings too. | none |
| OFF-8e | An empty pair (`##`), several pairs, a `# 12 #` line. | `##` is not a number. With several pairs only the last pair at the end counts and the earlier ones stay as text. A line starting with `#` is a Section (OFF-20e), never a heading. | n/a | P2, P3 | An empty pair gives an empty-string number (differs). With several pairs only the last counts, and `# 12 #` is a Section (both match). | `scene-number-rejected` |
| OFF-8f | Is the number part of the heading text? | No. The view has `heading.text` without the number and a separate `SceneNumber`. The tree keeps both. | n/a | P2 | The number is removed from the heading text; an automatic heading keeps the space that stood before it. | none |
| OFF-8g | Locked numbers (`##12A#`). | Plain heading text unless the `locked-scene-numbers` flag is on (5.2), because a number may not start with `#`. | n/a | P1 | Not covered by the default parser. Slugline-only (extensions.md). | none |

#### OFF-9: Transitions

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-9a | Framed lines that do not end in `TO:` (`FADE OUT.`, `FADE IN:`). | Action. Only `TO:` (or a forced `>`) makes a Transition. | A | P1, P3 | Contradicts: `FADE OUT.`, `FADE TO BLACK.` and `CUT TO BLACK.` are Transitions through a built-in list; `FADE IN:` is Action. | `transition-lookalike` |
| OFF-9b | A `TO:` line between text lines (`BACK TO:`). | Not a Transition. It is part of the Action paragraph. | A | P1, P3 | Contradicts: a Transition without any blank line, and the next line is joined to its text. | `unframed-transition` |
| OFF-9c | `CUT TO:` with a blank line before and a text line after (the heading line directly under it). | Not a Transition (no blank line after). The Character rule then fits (uppercase, blank before, text after): a Character cue whose dialogue is the heading line. This is the literal result and it is odd, hence the warning. | C | P1, P3 | Contradicts, and not Reading C: it is a Transition and the heading line is joined into its text. | `unframed-transition` |
| OFF-9d | Must the line be uppercase, and must `TO:` be exact? | Yes: uppercase text ending in exactly `TO:`. `Cut to:` is not automatic. | n/a | P1 | Contradicts: the test finds `TO:` in capitals at the end and does not check what comes before, so a mixed-case line qualifies. | none |
| OFF-9e | Forced `>`. | Any line, any case, no blank lines needed (see OFF-10 for the centered form). | n/a | P1 | Agrees: a `>` line without a closing `<` is a Transition. | none |

#### OFF-10: centered text versus forced Transition

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-10a | How are they told apart? | A line whose first non-whitespace character is `>` and last non-whitespace character is `<` is centered. Any other line starting with `>` is a forced Transition. | A | P1 | Does not cover. Matches A: last character `<` is centered, otherwise a Transition. | none |
| OFF-10b | Trailing whitespace after the `<`. | Ignored: `>FIN< ` is centered. The spec's own centered example shows a trailing space. | B | P1, P3 | Contradicts that example: trailing whitespace is not ignored, so `>FIN< ` becomes a Transition with the text `FIN<`. Legacy: tolerates whitespace after `<`. | none |
| OFF-10c | Nested marks (`>> FIN <<`). | Centered. Only the outermost pair is syntax, so the text is `> FIN <`. | C | P2 | Matches C: centered, text `> FIN <`. | none |
| OFF-10d | `> FIN`, `>FIN< and more`. | Forced Transitions. Text `FIN` and `FIN< and more`. | A | P1 | Agrees: both are Transitions with the same text. Legacy: `>FIN< and more` seems to become a centered Transition (low confidence). | none |
| OFF-10e | Are the marks part of the text? | The view's text leaves out `>` and `<` and trims. The tree keeps them. | E | P2 | Marks are dropped (Reading E); the writer later normalises `>FIN<` to `> FIN <` and turns a Transition `FIN<` into centered Action. | none |

#### OFF-11: dual dialogue

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-11a | A caret cue with no preceding dialogue block. | An ordinary Dialogue block. The caret stays in the tree and the view marks it unpaired. **Tie-breaker rejected**: turning the whole block into Action would reclassify the speaker's lines as Action and drop a character from every scene summary, and the text is kept either way. | B | P2, P3 | Does not cover. Matches B: the first `OTTO ^` is flagged dual with no partner. Legacy: a caret cue before the third element is an ordinary cue with the caret in its text (Reading A). | `lone-caret` |
| OFF-11b | Zero spaces before the caret (`OTTO^`). | Allowed. | C | P1 | Agrees: a dual cue (confirmed). | none |
| OFF-11c | The caret and an extension. | The caret must be the last non-whitespace character, so `MARA (V.O.) ^` is dual. `NAME ^ (V.O.)` is not: the caret is then part of the name text. The same applies to a forced `@NAME ^`. | n/a | P1, P3 | `MARA (V.O.) ^` is a dual cue. `@NAME ^` keeps its caret and is not dual. screenplay-tools writes the caret before the extension (extensions.md). | `caret-not-last` |
| OFF-11d | A third caret block in a row. | Pairs are exactly two blocks. A caret block that follows one already used as a right half is unpaired. | D | P2, P3 | Matches D as "chains onto the block before it": the nearest earlier cue is flagged again, with no count of pairs. | `lone-caret` |
| OFF-11e | What is the partner? | The Dialogue block directly before it, skipping invisible and transparent lines, not itself a right half. Anything else between (Action, a heading) breaks the pairing. | n/a | P2 | The walk back has no distance limit and can reach a cue several scenes earlier. | none |

#### OFF-12: Parenthetical shape

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-12a | What is a Parenthetical? | Inside a Dialogue block, one line that starts with `(` and ends with `)`, after trimming. | A | P1 | Does not cover. Matches B inside a block (a line that only starts with `(`). | none |
| OFF-12b | `(whispering) and then speaks`, `(not closed`, a two-line parenthesis. | Dialogue lines, each judged alone. **Tie-breaker adopted**, with one change: inside a block the fallback is Dialogue, not Action, because the spec says Dialogue is whatever follows a cue. | A | P1, P3 | `(whispering) and then speaks` and `(not closed` each become a Parenthetical, and `paragraph)` is Dialogue. Legacy: not determined by reading. | `partial-parenthetical` |
| OFF-12c | A parenthesis line with no cue above. | Action (the tie-breaker). | A | P2 | As given, with no cue, all four lines form one Action paragraph. | none |

#### OFF-13: Lyrics

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-13a | A `~` line after a cue. | Lyrics. `~` is always forced and wins inside a block. It interrupts the block without ending it. | B | P1 | Does not cover. Matches B: checked before every other rule. Legacy: no lyric rule, so Dialogue there. | none |
| OFF-13b | A `~` line alone. | Lyrics. No blank lines needed. | n/a | P1 | Lyrics with no neighbours needed. | none |
| OFF-13c | Leading whitespace before `~`. | Ignored (OFF-18, OFF-22a): ` ~ indented lyric` is Lyrics. | C (second alternative) | P1 | Differs: only the first character is tested, so it is Action there (Reading C). | none |
| OFF-13d | Several `~` lines. | One Lyrics element per run of consecutive `~` lines. | n/a | P2 | Lyrics groups separated by a blank line get a one-space Lyrics spacer between them. | none |

#### OFF-14: emphasis

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-14a | Which Markdown? | None is named. This library states its own rules in plain terms (below). An opening mark must be followed by a non-whitespace character and a closing mark must follow one. `*` is italics, `**` bold, `***` bold italics, `_` underline. | A is out | P2 | Does not cover: no Markdown dialect; seven fixed patterns in the sample renderer. | none |
| OFF-14b | May a span cross a single line break? | No (the Emphasis section). A span never crosses a line break, so the second `*across` / `lines*` pair stays literal. | B | P1, P3 | Matches C: crosses a single line break, stops at a blank line. | `emphasis-line-break` |
| OFF-14c | Intraword underscores (`snake_case_name`). | An `_` between two alphanumerics neither opens nor closes underline. `*` may be intraword. | n/a | P2 | `snake_case_name` underlines `case`. | none |
| OFF-14d | Crossing spans, mismatched runs (`*start _nest* end_`, `***triple*`). | Spans must nest and a closing run must have the same length as its opening run. Otherwise the first-opened span wins and the other marks are literal. `*bold **inside** italic*` is italic with bold inside. | n/a | P2 | `*bold **inside** italic*`: only `inside` is bold; `*start _nest* end_`: `start _nest` is italic; `***triple*`: `**triple` is italic. | none |
| OFF-14e | The escape line (`\_literal\_`, `\\`, `\#`, a trailing `\`). | A backslash before `*` or `_` makes it literal and is left out of the view's text. A backslash before anything else, and a trailing one, is literal text. `\\` is two literal backslashes. | D | P1 | Contradicts on the escape line: `\_literal\_` is underlined with both backslashes kept; only the single-asterisk pattern checks for a backslash. | none |
| OFF-14f | Emphasis inside Notes and Boneyard, and emphasis versus element detection. | Hidden spans are not scanned. Recognition runs before emphasis, so `**INT. X**` is Action. | n/a | P2 | Emphasis is applied before Notes are removed, and inline Boneyard is never removed, so `Mara /* aside */ waits.` italicises ` aside `. | none |

#### OFF-15: Notes

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-15a | Nested Notes (`[[outer [[inner]] outer]]`). | No nesting. The first `]]` closes the Note; the rest (` outer]] world.`) is visible text with a stray `]]`. | A | P2, P3 | Does not cover. The renderer matches A: ends at the first `]]`. The parser sees only a standalone-line Note. | `nested-opener` |
| OFF-15b | An unclosed `[[`. | Plain text. The search for `]]` never crosses a blank line (OFF-15g). **Tie-breaker adopted.** | C | P2, P3 | An unclosed `[[` stays as text (Reading C, first alternative). Legacy: the same. screenplay-tools closes it at the blank line (extensions.md). | `unclosed-note` |
| OFF-15c | A stray `]]`. | Plain text. | n/a | P2 | Plain text for the parser. | `stray-closer` |
| OFF-15d | A Note over two lines. | One Note. A Note may contain single line breaks, and its lines are not tested for being blank. | n/a | P1 | Contradicts: not recognised as one (only single-line standalone Notes), and the renderer's inline removal does not cross a line break either, so the text prints. Legacy: a multi-line standalone Note works. | none |
| OFF-15e | A standalone Note inside a Dialogue block or Action paragraph. | The line is invisible (3.4): the block continues across it. Blank lines around it still end the block. | D | P2 | Matches D: blank lines around a standalone Note split a block; a Note with no blank line before it is Dialogue text. | none |
| OFF-15f | Notes and Boneyard together, and a Note in the title page. | The opener that comes first wins and its contents are opaque. A Note in the title page is hidden like any other. | n/a | P2 | Not covered (the spec is silent). | none |
| OFF-15g | How far is a closer searched for? | Not past a blank line, with the two-space continuation line (OFF-1c) as the only exception. | n/a | P1 | Confirmed for Boneyard; Notes and emphasis are not searched for by line shape (OFF-14, OFF-15). | none |

#### OFF-16: Boneyard

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-16a | An inline Boneyard (`Mara /* aside */ waits.`). | Hidden. The visible text is `Mara  waits.` | D | P1 | Contradicts: an inline Boneyard is not recognised and ` aside ` prints, italicised between two slashes. | none |
| OFF-16b | Nested openers (`/* outer /* inner */ still outer? */`). | No nesting. The first `*/` closes. ` still outer? ` is visible text, and the final `*/` is a stray closer. | A | P2, P3 | Contradicts Reading A on one line: the whole line is one Boneyard with nothing left over. Across lines, nesting ends at the inner close (A). | `nested-opener` |
| OFF-16c | An unclosed `/*`. | **Own ruling.** It is plain text, and the rest of the file stays visible and hashed. Not the tie-breaker: the spec exempts Boneyard from the search limit. P2 decides, because hiding the rest of the file on a guess would lose text. | C (second alternative) | P2, P3 | Matches C, first alternative: runs to the end of the file and the collected text is silently lost. Legacy: unclosed stays text. | `unclosed-boneyard` |
| OFF-16d | A closer with no opener, or text before `*/`. | Plain text, including any text before it on the line. | E | P2, P3 | A line that ends with `*/` outside a block makes an empty Boneyard and the line's other text is lost; on a closing line the text before `*/` is discarded. | `stray-closer` |
| OFF-16e | Removal before or after element detection, and the spec's own layout (a Transition line directly above `/*` and a heading directly below `*/`). | Before (3.4): `/* x */INT. A` is a heading. A whole-line Boneyard makes an invisible line, so the lines around it are adjacent. The spec's own layout therefore does not frame its Transition or heading; it breaks the framing rules by its own text. Blank lines inside a Boneyard are inside the span and are not blank lines. | D | P1 | Reading D does not hold: `/* x */INT. A` opens a block and the whole line is lost; text directly under a one-line Boneyard is joined into it and hidden. | none |

#### OFF-17: title page

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-17a | What starts a title page? | The first visible non-blank line of the document, if it is a key line (4.1). Any key will do. A heading-shaped line is never a key. A bare `Key:` needs an indented line under it. A first block of ordinary prose with a colon is therefore a title page (corpus scripts keep the first block ordinary, section 12). | C | P1 | Matches C and is wider: any line of the first block with a key shape makes the whole block the title page, so `Note: this is a first line of action.` is the entire title page. A line ending in a colon (`FADE IN:`) also opens one. Legacy: the first line must hold the colon. | none |
| OFF-17b | Where does it end? | At the first blank line. | A | P2 | Matches A: first zero-length line. | none |
| OFF-17c | Keys: case, duplicates, unknown keys. | Matched ignoring case, stored as written. Duplicates are all kept, in order. Unknown keys are kept. | D | P2 | Matches D: lowercased keys, duplicates kept, unknown keys kept; `Author` becomes `authors`. | none |
| OFF-17d | The indent of a continued value. | A continuation line is indented by three or more spaces or a tab. A line indented by one or two spaces that is not itself a key line is a **stray line**: kept, attached to no key. | E | P1, P3 | Contradicts: under a bare key any indent or none continues a value (legacy: two to eight spaces or a tab). Orphaned lines can attach to the next bare key. | `title-page-stray-line` |
| OFF-17e | Inline value plus indented lines, an empty value, a colon inside a value. | Allowed: values are the inline text, if any, then the continuation lines. An empty value is allowed. The key ends at the first colon, so `Draft date: 5:30 pm` works. | E | P2 | Indented lines after an inline value are lost when an inline key follows. A value may hold colons. | none |
| OFF-17f | Blank lines before the first key. | Allowed: they are leading trivia. | F | P2 | Matches F: leading blank lines are stripped first. | none |

#### OFF-18: tabs, indentation, recognition first

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-18a | Which comes first, recognition or indentation? | Recognition reads each line as if its leading whitespace were absent (the spec's key statement). This breaks the circle the same way for every line, and neither root cause applies: the document is not stripped first (root cause 1), and blank means whitespace-only (root cause 2). | A | P1 | Does not cover. Matches B (the raw line) with whitespace-tolerant tests; cues coincide with A. A tab-indented `INT. HOUSE - DAY` is not a heading there, except on the first line, whose indent root cause 1 removes. | none |
| OFF-18b | Is the indent kept? | In Action, yes (the view). Everywhere else the view trims it. The tree always keeps it. | A | P1 | The indent stays in the stored text of cues, Dialogue, Parentheticals, Transitions and Action. | none |
| OFF-18c | Tabs to spaces. | Only in the display helper for Action text: four spaces per tab. Never in the tree and never in hashes. | C | P2 | Not implemented: no tab is converted. | none |
| OFF-18d | The spec's indent levels (prose and example differ). | Not relevant: every indent is ignored outside Action. | A | P1 | Not covered by the default parser. | none |

#### OFF-19: `=` runs

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-19a | Three or more `=` alone. | Page Break. No upper limit. | A | P1 | Confirmed. Legacy: three to eight of `=`, `-` or `_`, so `---` breaks pages there. | none |
| OFF-19b | `==`, and `=== text`. | Action. Neither a Page Break nor a clear Synopsis, so the fallback applies. | A | P2, P3 | `==` is a Synopsis with the text `=`; `=== text` a Synopsis with `== text`. Legacy: `=` and `==` are not Synopses. | `ambiguous-equals` |
| OFF-19c | A lone `=`, `= text`, `= = =`. | Synopses. `=` alone has empty text; `= = =` has the text `= =`. | A | P2 | `=` alone is a Synopsis with empty text, `= Summary text` a Synopsis, and `= = =` a Synopsis with the text ` = =`. Matches. Legacy: needs two characters after the `=`. | none |
| OFF-19d | Whitespace around a run (`  ===`, `===  `). | Ignored (leading whitespace is read as absent, trailing is not significant). Both are Page Breaks. | B | P1 | Trailing whitespace is accepted, but a leading one voids the break: `  ===` is a Synopsis with the text `==`. | none |
| OFF-19e | A text line directly under a Page Break. | Its own element. Nothing is joined or hidden. | n/a | P2, P3 | Outside a Dialogue block it is joined to the Page Break and the renderer hides it. | `adjacent-outline-line` |

#### OFF-20: Sections

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-20a | A space after the `#`s. | Not required: `#Section` and `#1 priority item` are Sections. | A | P1 | Agrees: `#Section without space` is a Section with the text `Section without space` (Reading A). | none |
| OFF-20b | Empty text (`#`, `# `). | A Section with empty text. | C | P1 | `# ` is a Section with a one-space text; a bare `#` is dropped with a log message. | none |
| OFF-20c | Depth. | The count of leading `#`. No maximum. | A | P1 | No maximum (the Python library stops at seven, extensions.md). | none |
| OFF-20d | `INT. HOUSE - DAY #5#` against a Section. | A line is a Section only if it starts with `#`. The heading ends with a scene number. `# 12 #` is a Section with the text `12 #`. | D | P1 | Matches D: the Section reading wins at the line start. Legacy: unanchored; a heading with a `#` pair most likely becomes a Section. | none |
| OFF-20e | Blank lines around a Section. | None needed. It is transparent (3.4). | n/a | P1 | No blank line needed. A text line under it is joined to it and hidden by the renderer. | `adjacent-outline-line` |

#### OFF-21: Sections and Synopses inside blocks

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-21a | Inside a Dialogue block. | The line is its own element. The block continues across it. | A | P2 | Does not cover. Matches A, as separate elements. Legacy: appears to break the Dialogue (Reading B), not determined by reading. | none |
| OFF-21b | Inside an Action paragraph. | The same: the paragraph continues across it. | A | P2 | A text line under a Section, or under a Synopsis with no blank before it, is joined to it. | none |
| OFF-21c | Does a Section or Synopsis count as a blank line for framing? | No. It is transparent: the lines around it are adjacent. | n/a | P2 | The Synopsis step leaves the blank-line count as it was. | `adjacent-outline-line` |

#### OFF-22: sigils

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-22a | Leading whitespace before a sigil. | Skipped: `  !indented action` is forced Action. | B | P1 | Does not cover. Matches A: only the first character counts, so it is plain text (forced only on the file's first line, after root cause 1). | none |
| OFF-22b | Stacked sigils (`!.dot`, `@!after at`, `!>x`). | Only the first sigil counts; the rest is text. | C | P2 | Matches C: the first that fits wins and the rest is plain text. | none |
| OFF-22c | A line that merely starts with `>` (`>>> a line of arrows`). | A forced Transition, text `>> a line of arrows`. | D | P1 | Agrees (Reading D). | none |
| OFF-22d | Which sigils leave the view's text? | All of them: `.`, `!`, `@`, `~`, the Transition `>`, and the `>` `<` pair. The tree keeps them. | E | P2 | A mix: `.` and `>` are dropped in the parser's output; `!`, `@` and `~` are kept for the renderer to remove. | none |
| OFF-22e | `.>HEADING`. | Not a forced heading (OFF-7b) and not a Transition. Ordinary rules apply, usually Action; it is not `.` followed by an alphanumeric. | n/a | P1, P3 | Contradicts: a Scene Heading with the text `>HEADING`. | `period-not-forcing` |
| OFF-22f | A sigil followed by nothing useful (`!`, `@`, `~` alone or then spaces). | `!` and `~` force with empty text. `@` with no name is not a cue and the line is Action. | n/a | P2 | Not covered: the entry has no bare-sigil example. Its sigil steps test only the first character. | none |
| OFF-22g | `!!`. | Forced Action whose text starts with `!`, unless the `shot` flag is on (5.2). | n/a | P2 | Not covered (Beat's Shot, extensions.md). | none |

#### OFF-23: trailing whitespace

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-23a | `CUT TO: ` (spaces after the colon), on its own. | Not a Transition. It is Action, as the spec says. | A | P1 | Confirmed: not a Transition, and Action only if no text line follows. | `to-trailing-space` |
| OFF-23b | `CUT TO: ` with a text line directly under it. | Action, joined with the text line. **Two literal rules collide**: the Transition section says spaces after the colon make Action, and the Character rule fits (uppercase, blank before, text after). The more specific statement wins, so the line is never a cue. | A | P1, P3 | Challenged: with a text line under it, it is a Character cue. | `to-trailing-space` |
| OFF-23c | A caret followed by trailing whitespace (`MARA ^ `). | Still dual: trailing whitespace after the caret is ignored. An invisible space must not flip the meaning, and the tree keeps it. | A | P2 | Agrees: whitespace after the caret is accepted. Not a contradiction (the literal text is silent). | none |
| OFF-23d | Two trailing spaces. | Mean nothing. The 1.0 rule is gone. Trailing whitespace is otherwise not significant. | C is out | P1 | Confirmed: plain Action with the spaces kept in the text. | none |
| OFF-23e | Does a carriage return count as trailing whitespace? | No (3.2). | D is out | P2 | Carriage returns become line feeds first, so they never count. | none |

#### OFF-24: what a Dialogue block may contain

| Case / test ID | Question | Decision | Reading | Rule | Reference parser records | Diagnostic |
|---|---|---|---|---|---|---|
| OFF-24a | The default content. | Every visible line in the block is Dialogue, or a Parenthetical (OFF-12), until a blank line. | A | P1 | Does not cover. | none |
| OFF-24b | Forced `.`, `!`, `>`, `@` lines in the block. | They take effect, as forced elements override detection, and they end the block. Lines after them are not Dialogue. | B | P1, P3 | Matches B, but dialogue resumes after them (a new Dialogue element). | `sigil-in-dialogue` |
| OFF-24c | Lyrics, Sections, Synopses, Page Breaks. | They interrupt the block without ending it. Only a blank line (or 24b) ends it. | A | P2 | Matches A and not C: only a blank line ends the block. | none |
| OFF-24d | Notes and Boneyard. | Hidden (3.4). | A | P2 | A Note with no blank line before it stays Dialogue text; a Boneyard is its own element. Legacy: not determined by reading. | none |

### 6.3 The literal-text calls, restated

[reference-parser.md](docs/research/reference-parser.md) ends with a table of 12 calls where the literal text and the
default parser meet. Each is restated here as a decision.

| Call (reference-parser.md) | Reference parser result | Decision in this library | Rows |
|---|---|---|---|
| OFF-7: only a period followed by an alphanumeric forces a heading | Challenged: `. SPACE AFTER` and `.>HEADING` are headings there | Follow the literal text. Neither is forced. | OFF-7b, OFF-22e |
| OFF-8: alphanumerics, dashes and periods only, and the number is at the end | Challenged (characters), confirmed (end of line) | Follow both. `#1 A#` and `#1_A#` are not numbers; `#7# trailing words` is not a number. | OFF-8b, OFF-8c |
| OFF-17: three or more spaces or a tab | Challenged | Follow it. A one- or two-space line is a stray line. | OFF-17d |
| OFF-23: two trailing spaces force nothing; `CUT TO: ` is Action | Confirmed, but challenged where a text line follows | Follow it in both cases. The specific statement beats the Character rule. | OFF-23a, OFF-23b, OFF-23d |
| OFF-9: a Transition needs blank lines and an uppercase `TO:` ending | Challenged | Follow it. `BACK TO:` and `CUT TO:` between text are not Transitions; `FADE OUT.` is Action. | OFF-9a, OFF-9b, OFF-9c, OFF-9d |
| OFF-6: a heading needs blank lines before and after; case-insensitive | Confirmed for the line before; challenged in part for the line after | Follow it on both sides. `Ext. 12 is busy.` is a heading. | OFF-6a, OFF-6c |
| OFF-11: "any number of spaces" includes zero | Confirmed | Follow it. `OTTO^` is dual. | OFF-11b |
| OFF-3: extensions in either case | Challenged | Follow it. `MARA (on the radio, softly)` is a cue. | OFF-3a |
| OFF-4: a cue needs an alphabetical character | Challenged | Follow it. A line of digits is not a cue. | OFF-4c |
| OFF-19: three or more `=` and nothing else | Confirmed (default); the legacy parser challenges it | Follow it. `====` is a Page Break and `==` is not. | OFF-19a, OFF-19b |
| The caret must be the last character (OFF-11, OFF-23) | Not settled; the parser ignores trailing whitespace after the caret | Ignore trailing whitespace after the caret; the caret must otherwise be last. | OFF-23c, OFF-11c |
| Error Handling: a closing mark is not searched for past a double line break; Boneyard is exempt | Confirmed for Boneyard; not applicable to Notes and emphasis | Notes stop at a blank line; emphasis stops at a line break; an unclosed Boneyard is plain text, not a span to the end of the file. | OFF-15b, OFF-15g, OFF-14b, OFF-16c |

### 6.4 The four tie-breaker cases

official.md ends with the Error Handling tie-breaker, "when in doubt, return Action", and says that `SPEC.md` decides
whether to adopt it for four cases.

| Case | Ruling | Why |
|---|---|---|
| OFF-5, a cue at the end of the file | **Adopted** (`OFF-5a`) | No dialogue follows, so the line is Action. |
| OFF-11, a caret cue with no partner | **Rejected** (`OFF-11a`) | The block stays a Dialogue block with an unpaired caret and a warning. Action would drop the speaker from the scene. |
| OFF-12, a parenthetical that is not a whole line | **Adopted** (`OFF-12b`, `OFF-12c`) | Action with no cue; inside a block it is Dialogue, because the spec says Dialogue is any text after a cue. |
| OFF-15, an unclosed `[[` | **Adopted** (`OFF-15b`) | Plain text with a warning. |

OFF-16's unclosed `/*` is outside the tie-breaker (the spec exempts Boneyard) and has its own ruling in `OFF-16c`.

## 7. Diagnostics

A diagnostic is a note about a place in the input that the parser read one way but that an author, or another tool, may
read differently. **Diagnostics never change the tree or the typed view.** The same input always gives the same list,
ordered by start of span.

- **Severity.** *Info* means the input is read as specified, and another tool may read it differently. *Warning* means
  the input is probably not what the author meant, or text was demoted or left unclosed.
- **Narrow triggers.** A code fires only on the shape named in its trigger, so ordinary valid input stays quiet.
- **Tests.** Each code has a `DIAG-<code>` test with at least one input that fires it and one near miss that does not.

The six candidates listed in issue #9 are covered as follows.

| Candidate from issue #9 | Code |
|---|---|
| An unclosed `/*` that swallows the rest of the file | `unclosed-boneyard` |
| Text before `*/` on a closing line | `stray-closer` |
| A heading demoted after its forced marker was removed | `period-not-forcing` and `unframed-heading` (this library never removes the marker from the tree, and a forced heading is never demoted) |
| A title-page line attached to the wrong key | `title-page-stray-line` |
| A line joined into a Section, Page Break or Boneyard and hidden | `adjacent-outline-line` (nothing is joined or hidden here; the code marks the layout that other parsers hide) |
| A lone caret cue with no partner | `lone-caret` |

| Code | Severity | Trigger | Rows | Test ID |
|---|---|---|---|---|
| `blank-lookalike` | Info | A whitespace-only line that is not a continuation line, with a non-blank line directly before and after it. | OFF-1a | `DIAG-blank-lookalike` |
| `mixed-line-endings` | Info | The document uses more than one of `\n`, `\r\n` and `\r`. | OFF-2a | `DIAG-mixed-line-endings` |
| `cue-needs-force` | Info | A line with a blank line before and a text line after, with no lowercase letter and no uppercase letter, but with a digit or a caseless letter. | OFF-4b, OFF-4c | `DIAG-cue-needs-force` |
| `unframed-heading` | Info | A line that starts with a heading prefix and a period or space, and is not a heading only because a blank line is missing before or after it. | OFF-6a | `DIAG-unframed-heading` |
| `empty-heading` | Info | A heading that is only a prefix and its period or space. | OFF-6d | `DIAG-empty-heading` |
| `period-not-forcing` | Info | A line that starts with a single `.` followed by a character that is neither alphanumeric nor `.`. | OFF-7b, OFF-22e | `DIAG-period-not-forcing` |
| `scene-number-rejected` | Info | A heading line ends with something shaped like `#...#` that is not a scene number: a bad character, or a good number that is followed by text. | OFF-8b, OFF-8c, OFF-8e | `DIAG-scene-number-rejected` |
| `transition-lookalike` | Info | A framed, uppercase, one-line paragraph that is `FADE IN:`, `FADE OUT.`, `FADE TO BLACK.` or `CUT TO BLACK.`. | OFF-9a | `DIAG-transition-lookalike` |
| `unframed-transition` | Warning | An uppercase line ending in exactly `TO:` that is not a Transition because a blank line is missing before or after it. | OFF-9b, OFF-9c | `DIAG-unframed-transition` |
| `to-trailing-space` | Warning | An uppercase line ending in `TO:` followed by spaces or tabs, with a blank line before it. | OFF-23a, OFF-23b | `DIAG-to-trailing-space` |
| `lone-caret` | Warning | A Character cue with a caret that has no partner: no directly preceding Dialogue block, or one that is already a right half. | OFF-11a, OFF-11d | `DIAG-lone-caret` |
| `caret-not-last` | Info | A cue line that has a `^` followed by more text, such as `NAME ^ (V.O.)`. | OFF-11c | `DIAG-caret-not-last` |
| `partial-parenthetical` | Info | A line inside a Dialogue block that starts with `(` and does not end with `)`, or that ends with `)` and contains no `(`. | OFF-12b | `DIAG-partial-parenthetical` |
| `emphasis-line-break` | Info | An emphasis opener with no closer on its line, and a matching closer on a later line of the same paragraph. | OFF-14b | `DIAG-emphasis-line-break` |
| `nested-opener` | Info | A `[[` inside a Note, or a `/*` inside a Boneyard. | OFF-15a, OFF-16b | `DIAG-nested-opener` |
| `unclosed-note` | Warning | A `[[` with no `]]` before the next blank line. | OFF-15b | `DIAG-unclosed-note` |
| `unclosed-boneyard` | Warning | A `/*` with no `*/` after it. | OFF-16c | `DIAG-unclosed-boneyard` |
| `stray-closer` | Info | A `]]` or `*/` outside any span, including after text on the same line. | OFF-15c, OFF-16d | `DIAG-stray-closer` |
| `title-page-stray-line` | Warning | A line in the title page block that is neither a key line nor a valid continuation. | OFF-17d | `DIAG-title-page-stray-line` |
| `adjacent-outline-line` | Info | A text line directly after a Section, Synopsis or Page Break line, with no blank line between, where the text line is not itself a Section, Synopsis or Page Break. | OFF-19e, OFF-20e, OFF-21c | `DIAG-adjacent-outline-line` |
| `ambiguous-equals` | Info | A line that starts with `==` and is not a Page Break. | OFF-19b | `DIAG-ambiguous-equals` |
| `sigil-in-dialogue` | Info | A line with a forcing sigil (`.`, `!`, `>`, `@`) directly inside a Dialogue block. | OFF-24b | `DIAG-sigil-in-dialogue` |

## 8. Syntax tree API sketch

A sketch of signatures only, so that F2 to F4 can build against one shape. Names may change in review; the properties
below may not.

### 8.1 Entry point and crate features

```rust
/// Parses any text. It never fails and never panics.
/// The returned tree reproduces `source` exactly.
pub fn parse(source: &str) -> Document;
```

- **`&str`, not bytes** (3.1). One owned copy of the source lives inside the `Document`, so spans stay valid.
- **No runtime options.** A parse depends on the input and the enabled Cargo features only.
- **Features.** `default = []`.

| Feature | Adds | Dependency |
|---|---|---|
| `serde` | `Serialize` and `Deserialize` on the tree, the view and `Diagnostic` | `serde` |
| `locked-scene-numbers` | Slugline's `##n#` (5.2) | none |
| `shot` | Beat's `!!` Shot (5.2) | none |
| `hash` | The hash methods and digests of section 11 (scheme 2 and scheme 1) | `sha2` 0.10 and `unicode-normalization` |

The core crate has no dependency other than `serde` (behind `serde`), and `sha2` and `unicode-normalization` (behind
`hash`), each optional. The maintainer accepted `unicode-normalization` on 2026-11-05 (11.2, and section 13, item 7).

### 8.2 The tree

```rust
pub struct Document { /* the source, a node arena, the diagnostics */ }

impl Document {
    pub fn source(&self) -> &str;
    pub fn root(&self) -> Node<'_>;
    pub fn diagnostics(&self) -> &[Diagnostic];
    pub fn screenplay(&self) -> Screenplay; // the typed view, section 9
}

/// Half-open byte range into `Document::source`, on character boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Document, Gap,
    TitlePage, TitleEntry, TitleKey, TitleValue,
    SceneHeading, SceneNumber,
    Action, DialogueBlock, CharacterCue, CharacterName, Extension, Caret, Parenthetical, Dialogue,
    Transition, Centered, Lyrics,
    Section, Synopsis, PageBreak,
    Note, Boneyard,
    #[cfg(feature = "shot")] Shot,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind { Bom, Text, Whitespace, LineBreak, Sigil, Marker, Colon }

pub struct Node<'a> { /* a handle into the arena */ }

impl<'a> Node<'a> {
    pub fn kind(&self) -> NodeKind;
    pub fn span(&self) -> Span;
    pub fn text(&self) -> &'a str;
    pub fn parent(&self) -> Option<Node<'a>>;
    pub fn children(&self) -> impl Iterator<Item = Child<'a>>;
}

pub enum Child<'a> { Node(Node<'a>), Token(Token<'a>) }

pub struct Token<'a> { /* kind, span, text */ }
```

### 8.3 Invariants

1. **Exact cover.** The tokens, in order, have no gaps and no overlaps, and their text concatenated equals the source.
   This is what lets the writer reproduce any input.
2. **Node spans.** A node's span runs from the start of its first child to the end of its last child.
3. **Character boundaries.** Every span starts and ends on a character boundary.
4. **Never lost.** Every sigil, marker, hidden span and whitespace byte is a token. Nothing is discarded because the
   typed view leaves it out.
5. **Diagnostics are separate.** They carry spans but do not appear in the tree.

### 8.4 Trivia and whitespace ownership

The tree is flat: a `Document` holds top-level element nodes and `Gap` nodes, and scenes exist only in the typed view.

- **BOM.** A leading byte-order mark is the first token of the `Document`.
- **Line breaks inside an element** (between two lines of one Action paragraph, between a cue and its dialogue line) are
  `LineBreak` tokens inside that element's node. A `DialogueBlock` node holds its cue, Parentheticals, Dialogue lines
  and any interrupting Lyrics, Section, Synopsis, Page Break, Note or Boneyard line. An `Action` node holds its own
  interrupting Section, Synopsis, Page Break, Note or Boneyard lines the same way.
- **Leading and trailing whitespace of a line** is a `Whitespace` token inside the node of the element the line belongs
  to. So an element's span starts at its first byte and ends at its last byte before a line break.
- **Everything between two top-level nodes is a `Gap`:** the line break that ended the previous element, the blank
  lines, and at the start of the file any leading blank lines. The final line break of the file is in the last `Gap`. A
  document with no elements is one `Gap`.
- **Hidden spans.** A `Note` or `Boneyard` node is a child of the element node it sits in. One that fills whole lines is
  a top-level node (or a child of a `DialogueBlock` or `Action` it interrupts).
- **Inline emphasis** is a view-level layer. It is not in the tree, because it never changes which element a line is.

### 8.5 Diagnostics

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub span: Span,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticCode {
    BlankLookalike, MixedLineEndings, CueNeedsForce, UnframedHeading, EmptyHeading, PeriodNotForcing,
    SceneNumberRejected, TransitionLookalike, UnframedTransition, ToTrailingSpace, LoneCaret, CaretNotLast,
    PartialParenthetical, EmphasisLineBreak, NestedOpener, UnclosedNote, UnclosedBoneyard, StrayCloser,
    TitlePageStrayLine, AdjacentOutlineLine, AmbiguousEquals, SigilInDialogue,
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str; // the kebab-case code used in section 7
    pub fn severity(self) -> Severity;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity { Info, Warning }
```

### 8.6 Serde

With the `serde` feature, `Document`, `Span`, `NodeKind`, `TokenKind`, `Diagnostic` and every type in section 9 derive
`Serialize` and `Deserialize`. Deserialising a `Document` checks invariants 1 to 3 and returns a serde error rather than
a broken tree. The exact encoding of the arena is left to F2.

## 9. Typed view API sketch

Built on demand from the tree by `Document::screenplay()`. All types are plain owned data: they hold their own text, so
a view can be built in memory (issue #12), serialised and read back (issue #8) without the document. A `span` is an
optional back-reference to the source, `None` for a value that was built and not parsed.

```rust
pub struct Screenplay {
    pub title_page: Option<TitlePage>,
    pub preamble: Preamble, // elements before the first heading, except those attached to scene 1; hashed by scheme 2
    pub scenes: Vec<Scene>,
}

pub struct TitlePage {
    pub entries: Vec<TitleEntry>,
    pub stray_lines: Vec<Text>, // OFF-17d
    pub span: Option<Span>,
}

pub struct TitleEntry {
    pub key: Text,       // as written
    pub values: Vec<Text>,
    pub span: Option<Span>,
}

impl TitlePage {
    pub fn get(&self, key: &str) -> impl Iterator<Item = &TitleEntry>; // ignores case
    pub fn title(&self) -> Option<&TitleEntry>;
    pub fn author(&self) -> Option<&TitleEntry>; // "Author" or "Authors"
    // credit, source, draft_date, contact, notes, copyright, date: the same shape
}

pub struct Scene {
    pub heading: Heading,
    pub number: Option<SceneNumber>,
    pub sections: Vec<Section>,   // the run of Sections right before the heading
    pub synopses: Vec<Synopsis>,  // Synopses right before or right after the heading
    pub elements: Vec<Element>,   // everything else, in source order
    pub span: Option<Span>,       // from the first byte of the first attached Section or Synopsis, or of the heading,
                                  // to the last byte of the last element; no line break or blank line after it
    // and a private scheme 1 hash scope, filled in by `screenplay()` (11.7)
}

impl Scene {
    /// A scene with this heading and no number, Sections, Synopses, elements or span. Its scheme 1 hash scope is
    /// empty, so the scope is its own `synopses` and `elements`.
    pub fn new(heading: Heading) -> Scene;
}

pub struct Heading { pub text: Text, pub forced: bool, pub span: Option<Span> }

pub struct SceneNumber {
    pub text: String,
    #[cfg(feature = "locked-scene-numbers")]
    pub locked: bool,
    pub span: Option<Span>,
}

/// The elements before the first scene heading, apart from the title page and apart from the Sections and Synopses
/// directly before that heading, which belong to the first scene.
pub struct Preamble { pub elements: Vec<Element> }

#[non_exhaustive]
pub enum Element {
    Action(Action),
    Dialogue(DialogueBlock),
    DualDialogue(DualDialogue),
    Transition(Transition),
    Lyrics(Lyrics),
    Section(Section),
    Synopsis(Synopsis),
    PageBreak(PageBreak),
    Note(Note),         // a whole-line Note
    Boneyard(Boneyard), // a whole-line Boneyard
    #[cfg(feature = "shot")]
    Shot(Shot),
}

#[cfg(feature = "shot")]
pub struct Shot { pub text: Text }

pub struct Action {
    pub text: Text,
    pub centered: bool,
    pub forced: bool,
    pub interjections: Vec<Element>, // a Section, Synopsis or Page Break line inside the paragraph (OFF-21b)
}
pub struct Transition { pub text: Text, pub forced: bool }
pub struct Lyrics { pub lines: Vec<Text> }

pub struct DialogueBlock {
    pub cue: Cue,
    pub items: Vec<DialogueItem>,
    pub unpaired_caret: bool, // OFF-11a
}

pub enum DialogueItem {
    Line(Text),
    Parenthetical(Text),
    Interjection(Element), // a Lyrics, Section, Synopsis, Page Break, Note or Boneyard line inside the block
}

pub struct Cue {
    pub name: Text,
    pub extensions: Vec<Extension>,
    pub forced: bool,
    pub span: Option<Span>,
}

pub struct Extension { pub text: Text }
impl Extension { pub fn is_contd(&self) -> bool; }

pub struct DualDialogue { pub left: DialogueBlock, pub right: DialogueBlock }

pub struct Section { pub depth: usize, pub text: Text }
pub struct Synopsis { pub text: Text }
pub struct PageBreak { pub span: Option<Span> }

pub struct Note { pub text: String, pub span: Option<Span> }
impl Note { pub fn conventions(&self) -> Option<NoteConvention>; }

#[non_exhaustive]
pub enum NoteConvention {
    Colour(String),                                // the word as typed
    Marker { colour: Option<String>, text: String },
    Tag { tag: String, text: String },
}

pub struct Boneyard { pub text: String, pub span: Option<Span> }

/// Inline content: plain runs, emphasis, escapes and hidden spans.
pub struct Text { pub span: Option<Span>, pub runs: Vec<Run> }

pub enum Run {
    Plain(String),
    Emphasis { bold: bool, italic: bool, underline: bool, runs: Vec<Run> },
    Escaped(char), // the character after a backslash
    Note(Note),
    Boneyard(Boneyard),
}

impl Text {
    /// The visible text: no sigils, no hidden spans, no emphasis marks.
    pub fn plain(&self) -> String;
}
```

How the awkward pieces appear:

- **Dual dialogue.** A pair is one `Element::DualDialogue`. A caret with no partner stays a `DialogueBlock` with
  `unpaired_caret` set (`OFF-11a`). The caret itself is a token in the tree, never in `Cue::name`.
- **Notes.** A whole-line Note is an `Element::Note` in scene order. An inline Note is a `Run::Note` inside the `Text`
  it sits in. Neither enters `plain()`.
- **Boneyard.** The same, as `Element::Boneyard` and `Run::Boneyard`. A Boneyard that hides a whole scene still leaves
  the scene's tree nodes; the view shows an `Element::Boneyard` where it sits and no scene for text inside it.
- **Forced elements.** `forced` is true when a sigil was used. The sigil is left out of every `Text`.
- **Interjections do not split an element.** A Section, Synopsis or Page Break line inside an Action paragraph is kept
  in `Action.interjections`, and one inside a Dialogue block is a `DialogueItem::Interjection`. The paragraph or the run
  of Dialogue lines continues across it, as one element. A whole-line Note or Boneyard inside an Action paragraph is a
  `Run` of its text.
- **Section and Synopsis placement.** A run of Sections and Synopses directly before a heading belongs to that scene;
  Synopses directly after it belong to it too (`synopses`). Any other Section or Synopsis stays in `elements`. Scheme 2
  hashes follow this placement. **Only scheme 1 follows a different rule:** a Synopsis directly before a heading belongs
  to the earlier scene for `Scene::hash_v1` (11.7).
- **The hash scope field is serialised.** With the `serde` feature the private scheme 1 hash scope of `Scene` is
  serialised with the scene, so that a view read back hashes as it did. When the field is absent from the input it reads
  as empty, which gives the scope of a scene built in memory (11.7). The derived `PartialEq` includes it.
- **Hashes.** The hash methods of the types above are listed once, in 11.2, with their signatures:
  `Element::hashes`, `Scene::{hash, element_hashes, source_hash, hash_v1}`, `Preamble::hash` and
  `Screenplay::{hash, title_page_hash, source_hash}` need the `hash` feature, and `Scene::{normalized_text_v1,
  heading_line_v1, element_lines_v1}` exist in every build. They are methods of the view, so a view built in memory can
  be hashed.
- **The note-conventions view is opt-in.** `Note::conventions` is computed on demand and has no Cargo feature.

## 10. Writer

```rust
impl Document {
    /// Exactly `source()` for a tree that has not been edited.
    pub fn write(&self) -> String;

    /// Replaces one span, re-parses, and leaves every other byte untouched. The named edits
    /// of issue #13 are built on this primitive.
    pub fn with_replaced(&self, span: Span, text: &str) -> Document;
}

impl core::fmt::Display for Document { /* the same as write() */ }

/// A canonical Fountain text for a document built from scratch (issue #12).
pub fn canonical(screenplay: &Screenplay) -> String;
```

### 10.1 Byte-for-byte round trip

- `write` concatenates the tokens in order (invariant 1). It makes no decisions, so it cannot lose a byte: forced
  headings, Transitions, dual-dialogue carets, scene numbers, odd whitespace, mixed line endings, a byte-order mark and
  a missing final newline come back as they went in.
- **Why the source is kept and not regenerated.** The reference writer rebuilds text from its model, and
  [reference-parser.md](docs/research/reference-parser.md) records that it does not round-trip several constructs:
  forced headings (`.INT. X` comes back as an automatic heading, `.INTERCUT` stops being a heading, `. SPACE AFTER`
  loses its space), Transitions (always written with `>` and a space, so each round trip of a `TO:` line adds another
  `>`), centered text (`>FIN<` comes back as `> FIN <`), dual-dialogue carets (written on every second flagged cue, so
  an unpaired one moves), and scene numbers on automatic headings (a doubled space). Regenerating from the view would
  break byte-for-byte output for all of these, so the tree keeps the source.

### 10.2 Edits

- A **targeted edit** changes only the bytes of the span it replaces (issue #13). `with_replaced` is the one primitive;
  set a scene number (writes `#12A#`; the locked form `##12A#` is possible only with `locked-scene-numbers`),
  insert or replace a synopsis, add a note and rename a cue are named wrappers around it.
- New lines use the first line break found in the document (`\n` if there is none).
- After an edit the document is re-parsed. Spans outside the edit move by the length difference; the wrappers check that
  the edited text still parses as the element it was meant to be.

### 10.3 Canonical form for new documents

`canonical` writes a view that was built in memory, not parsed (its spans are `None`, which is why the view owns its
text). It follows these rules:

- `\n` line breaks, a final line break, no trailing whitespace, one blank line between elements.
- Title page as `Key: value`, with continuation lines indented by four spaces, then one blank line.
- A sigil only where it is needed for the text to re-parse as the same element: `.` for a heading with no prefix, `!`
  for Action that would read as something else, `@` for a cue that is not uppercase, `>` for a Transition that does not
  end in `TO:`, `~` for Lyrics, `>` and `<` for centered text.
- A scene number as `#n#` after the heading (`##n#` only with `locked-scene-numbers`).
- Whole-line Notes and Boneyards as their own blocks; inline ones in place.

The canonical text re-parses to a view equal to the one given, and writing it again changes nothing.

### 10.4 Tests

| ID | Test |
|---|---|
| `WR-1` | Every corpus script round-trips byte for byte. |
| `WR-2` | Line-ending cases round-trip: CRLF, lone CR, mixed endings, a byte-order mark, no final newline, only blank lines. |
| `WR-3` | A property test: any string parses, never panics, and writes back unchanged. |
| `WR-4` | The constructs the reference writer breaks round-trip: `.INT. X`, `. SPACE AFTER`, `>FIN<`, a `>`-forced `TO:` line, an unpaired caret, `INT. X #1#` with its original spacing. |
| `WR-5` | `canonical` output re-parses to a view equal to its input. |
| `WR-6` | `canonical` is stable: writing its own output again gives the same text. |
| `WR-7` | `with_replaced` changes only the replaced span: every other byte is identical, and spans after the edit are shifted correctly. |
| `WR-8` | `canonical` uses a sigil only where the text would otherwise re-parse as a different element. |

## 11. Hashes: elements, scenes and documents

Tools compare drafts, decide which work is stale and look for the element that changed. For that the library hashes a
screenplay at three levels (an **element**, a **scene**, a **document**) and hashes the exact bytes of any span. There
are two **schemes**, which are two complete sets of rules:

- **Scheme 2 is the default and the recommended scheme for every application.** It follows best practice for hashing
  structured content, and it is described in 11.3 to 11.6.
- **Scheme 1 is kept, frozen, as a compatibility scheme** (11.7). It is the rule of the Twopop importer (`parse.py` in
  `twopopfilm/twopop`), applied to this library's parse. It exists so that Twopop Studio can prove parity with its
  Python oracle until the oracle retires, and so that stored hashes can be migrated without false staleness (11.8).

The maintainer decided on 2026-11-05 to apply best practice in the core library even where the applications that use it
have to adapt. Scheme 2 is therefore not shaped by any existing consumer, and the migration is the application's work.

Scheme 2 follows six design rules, which every later rule serves:

1. A hash changes if and only if what it names changes in a way that matters to the work.
2. The encoding is **injective** (two different inputs never share their bytes) and **domain-separated** (a hash of one
   kind can never equal a hash of another kind).
3. Hashes are **hierarchical**: element, then scene, then document. A tool can see which element changed.
4. **Every digest names its scheme**, in its type and in its text form.
5. The lossless tree always allows a byte-exact hash too (the source facet, 11.3).
6. Any change to a rule is a **new scheme number**, never a silent change to an existing scheme.

### 11.1 Digests and their text form

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HashScheme { V1, V2 }

impl HashScheme {
    pub const fn number(self) -> u32; // 1 or 2
}

/// The scheme that new code should use.
pub const HASH_SCHEME_CURRENT: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest {
    pub scheme: HashScheme,
    pub bytes: [u8; 32],
}

impl core::fmt::Display for Digest { /* the text form below */ }
impl core::str::FromStr for Digest { type Err = ParseDigestError; }
pub struct ParseDigestError { /* what was wrong */ }
```

**Text form.** A scheme 2 digest prints as `fs2:` and 64 lowercase hexadecimal digits. A scheme 1 digest prints as the
bare 64 lowercase hexadecimal digits, which is exactly what the importer and Twopop Studio store today. `FromStr`
accepts those two forms and nothing else: a bare 64-digit string is scheme 1, and `fs2:` plus 64 digits is scheme 2.
Uppercase digits, spaces, a wrong length and any other prefix (`fs1:` and `fs3:` included) are errors, so a digest from
a scheme this crate does not know is never mistaken for one it knows. Text, parsed and printed again, is unchanged.

`Digest` equality compares the scheme as well as the bytes, so a scheme 1 digest never equals a scheme 2 digest. A
digest records how its bytes were made, not which level it hashes (element, scene, document, span): the name of the
method or field says that.

`Digest`, `HashScheme`, `HASH_SCHEME_CURRENT` and the text form need no dependency and exist in every build.

### 11.2 Features and dependencies

**Decided (maintainer, 2026-11-05).** One Cargo feature, **`hash`**, off by default. It adds every method that computes
a digest and depends on two crates:

- `sha2`, version 0.10, so that it unifies with Twopop Studio's lockfile;
- `unicode-normalization`, for NFC. Every scheme 2 hash needs NFC (11.3), so scheme 2 cannot be offered without it
  (section 13, item 7).

**Without `hash`**, these are available in every build and need no dependency:

- the scheme 1 line methods `normalized_text_v1`, `heading_line_v1` and `element_lines_v1` (11.7). They apply neither
  NFC nor SHA, so a consumer with its own hashing (Twopop Studio has) can keep it and feed it these lines;
- the types of 11.1, and the spans and source text of the lossless tree, which a caller can hash with its own tools.

The hash API, in one place. They are methods of the types in section 9.

```rust
// Every build.
impl Scene {
    pub fn normalized_text_v1(&self) -> String;
    pub fn heading_line_v1(&self) -> String;
    pub fn element_lines_v1(&self) -> Vec<String>;
}

// With the `hash` feature.
impl Element {
    pub fn hashes(&self) -> Vec<Digest>;
}
impl Scene {
    pub fn hash(&self) -> Digest;
    pub fn element_hashes(&self) -> Vec<Digest>;
    pub fn source_hash(&self, document: &Document) -> Option<Digest>;
    pub fn hash_v1(&self) -> Digest;
}
impl Preamble {
    pub fn hash(&self) -> Digest;
}
impl Screenplay {
    pub fn hash(&self) -> Digest;
    pub fn title_page_hash(&self) -> Digest;
    pub fn source_hash(&self, document: &Document) -> Digest;
}
impl Document {
    pub fn source_hash(&self, span: Span) -> Digest;
}
impl<'a> Node<'a> {
    pub fn source_hash(&self) -> Digest;
}
```

- `Scene::hash`, `Preamble::hash` and `Screenplay::hash` are the scene, preamble and document hashes of scheme 2.
  `Scene::element_hashes` lists the element hashes that `Scene::hash` is made of, in order.
- `Element::hashes` gives the hashes of the records one view element contributes, in order: none for a Section, Page
  Break, Note or Boneyard, and several for a dialogue block (a Character, then its Dialogue and Parentheticals). The
  view's `Element` is not one hashed element, which is why this is a list.
- The source hashes take the document because the view holds spans and no source text (section 9).
  `Scene::source_hash` is `None` for a scene built in memory, which has no span. `Screenplay::source_hash` hashes the
  whole input. `Document::source_hash(span)` and `Node::source_hash` hash any span or node.
- `title_page_hash` of a screenplay with no title page is the hash of a title page with no entries.
- Every method with a scheme in its name is scheme 1; every method without one is scheme 2.

### 11.3 Scheme 2: what is hashed

**Two facets.** The **story** facet hashes the content that matters to the work; it is what `hash`, `element_hashes`,
`Preamble::hash` and `title_page_hash` give. The **source** facet hashes the exact bytes of a span; it changes on any
edit at all, whitespace included, and is a cache key for anything derived from the bytes (`source_hash`).

**The typed text of a piece of content** is the text as written, with the element's own sigils and marks removed and
nothing else: no forcing `.`, `!`, `@`, `>` or `~`, no `>` and `<` of centered text, no `=` of a Synopsis, no `!!`, no
parentheses of a Parenthetical, no scene number, no caret. It is not `Text::plain()`: emphasis marks (`*`, `_`) and
backslash escapes stay as typed, so a change of italics or underline is a change of content.

**Normalisation of story text**, in this order:

1. **Delete hidden spans** (Notes and Boneyards), as characters, including any line breaks inside them.
   `Mara /* aside */ waits.` becomes `Mara  waits.`
2. **NFC**: Unicode canonical composition. `é` as one character and as `e` plus U+0301 become the same text.
3. **Collapse white space.** Each run of Unicode `White_Space` characters becomes one space (U+0020), and the ends are
   trimmed. `White_Space` is U+0009 to U+000D, U+0020, U+0085, U+00A0, U+1680, U+2000 to U+200A, U+2028,
   U+2029, U+202F, U+205F and U+3000 (not U+001C to U+001F). Line breaks inside an element become spaces; Lyrics are the
   exception, because each `~` line is kept apart (see the record below).
4. **Upper-case, for the heading, the cue name, the cue extensions and the title-page keys only.** Use the full Unicode
   upper-case mapping, independent of locale (Rust's `str::to_uppercase`), then apply NFC again, because upper-casing
   can leave text that is not in NFC. The result is what is hashed. Case is kept everywhere else.

Nothing else is changed: not punctuation, quotes, emphasis marks, escapes or text that only looks like a tag
(`#prop-coin`). Normalisation is applied to the hashed copy only. The view, the tree and the writer never normalise
Unicode.

**What a scene holds, in what order.** A scheme 2 scene is the view's `Scene` (section 9) and nothing else: its heading,
and its elements in the order `synopses`, then `elements`, which is source order. So a Synopsis or Section directly
before a heading belongs to the scene that follows, and a Synopsis directly after a heading belongs to that scene.
Scheme 2 has no separate hash scope. The **preamble** is `Preamble::elements`: the elements before the first heading
that the view does not attach to the first scene.

**The element records.** Every scene and the preamble is a list of records. A view element can give none, one or
several, as the last column says. A record is **dropped** if its text is empty after normalisation: a Character with an
empty name, a Lyrics element none of whose `~` lines has text, and every other kind with empty text. A dropped record is
not counted.

| Kind (tag in 11.4) | Fields, in order | Typed text and notes |
|---|---|---|
| Action | `text`, `centered` | The paragraph's text, with its line breaks as spaces. `centered` is true for `>` `<` text. Forced (`!`) or not makes no difference. |
| Character | `name`, `extensions`, `dual` | The cue name, normalised and upper-cased. `extensions` is the list of extension groups in order: the text inside each pair of parentheses, normalised and upper-cased, **leaving out every group for which `Extension::is_contd` is true**. A group with empty text stays in the list as an empty string. `dual` is true for both cues of a `DualDialogue` and false for every other cue, an unpaired caret included. A forced (`@`) cue is an ordinary cue. |
| Dialogue | `text` | The lines of one run, joined with spaces and collapsed. A run is the Dialogue lines between two Parentheticals, or between a Parenthetical and an end of the block. |
| Parenthetical | `text` | The text inside the parentheses, normalised. `( quietly )` and `(quietly)` give `quietly`. |
| Transition | `text` | The text, without a forcing `>`. Forced or not makes no difference. |
| Lyrics | `lines` | One entry per `~` line of the run, each normalised, without the `~`. Empty lines are dropped from the list. The verse's line breaks are content. |
| Synopsis | `text` | The text, without the `=`. |
| Shot | `text` | The text, without the `!!`. Exists only with the `shot` feature. |

The heading is not a record. It is the first field of the scene hash: the heading's visible text with the forcing `.`
and the scene number (locked or not) removed, normalised and upper-cased. It has no tag.

**Not hashed in the story facet:** Sections, Page Breaks, Notes and Boneyards (whole-line or inline), scene numbers,
forcing sigils and the `forced` flags, the caret, the title page (it has its own hash), the stray lines of a title page,
and the way a line was wrapped. A Note or Boneyard that is a whole line inside an Action paragraph or a Dialogue block
neither adds a record nor ends the run.

**Records of a dialogue block, in order.** A `DialogueBlock` gives its Character record, then its items:

- each run of consecutive `Line` items is one Dialogue record, and each Parenthetical item is one Parenthetical record
  that ends the run before it;
- an interjection gives the records of an element of its own kind (a Lyrics or Synopsis one; a Section, Page Break, Note
  or Boneyard gives none);
- an interjection with `Line` items on both sides of it, in the same run, does not split the run: its records come
  straight after that run's Dialogue record. Any other interjection keeps its place among the items.

An `Action` gives its record, then the records of its interjections (a Synopsis gives one; the rest give none). A
`DualDialogue` gives the records of its left block, then those of its right block, with no marker between them, and the
`dual` flag of both cues is true.

### 11.4 Scheme 2: the byte layout

**Primitives.**

- `u32`: four bytes, big endian.
- `bool`: one byte, `0x00` or `0x01`.
- `string`: a `u32` count of bytes, then the UTF-8 bytes of the normalised text.
- `list<T>`: a `u32` count of items, then the items.
- A hash inside another hash is its 32 raw bytes, not its text form.
- A count or length that does not fit a `u32` is outside the scheme. An implementation must not wrap or truncate it (it
  may panic, as no real screenplay reaches 4 GiB in one string).

**Tags**, one byte each, at the start of an element record:

| Tag | Kind | Fields after the tag |
|---|---|---|
| `0x01` | Action | `string` text, `bool` centered |
| `0x02` | Character | `string` name, `list<string>` extensions, `bool` dual |
| `0x03` | Dialogue | `string` text |
| `0x04` | Parenthetical | `string` text |
| `0x05` | Transition | `string` text |
| `0x06` | Lyrics | `list<string>` lines |
| `0x07` | Synopsis | `string` text |
| `0x08` | Shot | `string` text |

The tags `0x00` and `0x09` to `0xFF` are reserved. Using one is a new scheme number.

**Domains.** `domain(name)` is the ASCII bytes of `fountain-syntax/` + `name` + `/2`, then one NUL byte (`0x00`). The
six names are `element`, `scene`, `preamble`, `document`, `title-page` and `source`. A name contains no NUL, so the NUL
ends the domain and no domain is the start of another.

**The hashes.** `||` is concatenation. `SHA-256` is applied to the whole byte string.

```text
element_hash    = SHA-256( domain("element")    || record )
scene_hash      = SHA-256( domain("scene")      || string(heading) || u32(n) || e_1 || ... || e_n )
preamble_hash   = SHA-256( domain("preamble")   || u32(n) || e_1 || ... || e_n )
document_hash   = SHA-256( domain("document")   || preamble_hash || u32(m) || s_1 || ... || s_m )
title_page_hash = SHA-256( domain("title-page") || u32(k) || entry_1 || ... || entry_k )
source_hash     = SHA-256( domain("source")     || the bytes of the span )
```

- `record` is the tag and then the fields of the tag table above, in order.
- `e_i` are the element hashes of the kept records, in order, and `n` is their number. `s_j` are the scene hashes in
  document order and `m` is their number.
- `entry` is `string(key)` followed by `list<string>(lines)`. The key is normalised and upper-cased. The lines are the
  entry's value lines in order (the inline value, then the continuation lines), each normalised (case is kept), with
  empty lines dropped. An entry whose key is empty after normalisation is kept.
- The bytes of `source_hash` are the last field, so they need no length. A scene's span is `Scene::span` (section 9).

**Why the layout is injective.** The first byte of a record names the kind, so the fields that follow have one fixed
shape. A string and a list carry their length, so the next field starts at a known place. A scene hash holds fixed-width
children and a count. The domains are prefix-free, so no two kinds of hash share a pre-image. A scene's hash does not
depend on its position, its number or the scenes around it, so two identical scenes have equal hashes.

### 11.5 Scheme 2: known answers

These are the vectors for the tests `HASH2-40` to `HASH2-45`. An implementation, or a reader with `sha256sum` and a hex
editor, can check them. The pre-image of the first one, in full (54 bytes):

```text
domain    666f756e7461696e2d73796e7461782f656c656d656e742f32 00   ("fountain-syntax/element/2", NUL)
tag       01                                                      (Action)
text      00000016 4d6172612066696c6c7320746865206b6574746c652e   (22 bytes: "Mara fills the kettle.")
centered  00
```

Digests are shown in their text form. `H` is the heading `INT. KITCHEN - DAY`, whose string is `00000012` and its 18
bytes.

| Vector | Input | What is hashed | Digest |
|---|---|---|---|
| `HASH2-40` | A scene with only the heading `INT. KITCHEN - DAY`. | `scene_hash` with `string(H)` and `u32(0)`. The pre-image is 50 bytes. | `fs2:59bb346594a6fee2987e19850929c14dfc8ceb86ee6293e8eb24a72ebd617248` |
| `HASH2-41` | The Action `Mara fills the kettle.` | `element_hash` of the 54-byte pre-image above. | `fs2:1cd74cd5159cd81b65c689eff8d55d8dc7bfa6de082dbb341873240f6595850b` |
| `HASH2-41` | The scene `H` with that one Action. | `scene_hash` with `string(H)`, `u32(1)` and the element hash above. The pre-image is 82 bytes. | `fs2:39ad14125a7c97078c25a78852cc36793e0fd890ce68c33491a1a08fa3bf9c78` |
| `HASH2-42` | The cue `MARA (V.O.) (CONT'D)` | A Character: name `MARA`, extensions `["V.O."]`, `dual` false. 48-byte pre-image. | `fs2:1a556f7deab4f5fa5628f2e5d969f50285b3c00f75fe18d6a8dd00f26646af95` |
| `HASH2-42` | The Parenthetical `(quietly)` | A Parenthetical with the text `quietly`. | `fs2:07e731a7acbe2ca8fa63f9a0a9e38dfb1abb4af25c2287e3e715e0bfe8ab45a3` |
| `HASH2-42` | The Dialogue `Tea?` | A Dialogue with the text `Tea?`. | `fs2:b37c03f30bd33f9849e35026d903755b6bd92eacdd9bc69a90b303295bc651c1` |
| `HASH2-42` | The scene `H` with those three, in that order. | `scene_hash` with `string(H)`, `u32(3)` and the three hashes above. 146-byte pre-image. | `fs2:1605f74bdad918ae3f826b7c5ea6f8b36b59466c10f2dc426514262b359d62aa` |
| `HASH2-43` | The Action `Caf` + U+00E9 (precomposed). | An Action with the text of 5 bytes `43 61 66 c3 a9`, `centered` false. | `fs2:dd8fe5281b946df777823637c8a3b957734f80784d8574d0c49b9f0fa8e7f050` |
| `HASH2-43` | The Action `Cafe` + U+0301 (decomposed). | After NFC the same text and the same record. | `fs2:dd8fe5281b946df777823637c8a3b957734f80784d8574d0c49b9f0fa8e7f050` |
| `HASH2-44` | `> The end <` as Action. | An Action with the text `The end` and `centered` true. | `fs2:d110baae28cb16a64562de38a45bed9c1ec7afda132311c1b01a9793c98cd192` |
| `HASH2-44` | The right-hand cue `OTTO (v.o.) (cont'd) (filtered) ^` of a dual pair. | A Character: name `OTTO`, extensions `["V.O.", "FILTERED"]`, `dual` true. | `fs2:e9959f1d4a68e8749ae4c32bdb89ab58869100e8fcc05016cdc7b26070247dea` |
| `HASH2-44` | The two lines `~La la` and `~Lo lo`. | A Lyrics with the lines `["La la", "Lo lo"]`. | `fs2:29e754ee12c86a5ed2ccefa29c894e9a2336092760afb4abf2b78e9dc3b8be44` |
| `HASH2-45` | The document in the next paragraph: its preamble. | `preamble_hash` with `u32(0)`. | `fs2:354e078bdd8af223f493f82eec629de6f22e72660aef1936ebf92739fd443cac` |
| `HASH2-45` | The same document. | `document_hash`: the preamble hash, `u32(1)` and the scene hash of the `HASH2-41` scene. | `fs2:2b182de81221644c685c35b4917275421b0c85a053d77b012275aceaf641987f` |
| `HASH2-45` | The same document. | `title_page_hash`: `u32(1)`, `string("TITLE")`, a list of the one string `Tea Time`. | `fs2:9ea6372e0b7d474e70cd1a48cda79753a36058c111d4792a4c4d21700b0d962c` |
| `HASH2-45` | The same document, whole. | `source_hash` of its 60 bytes. | `fs2:84f98daa89e4fd67b4dc773e449db8f969237b077469878eb521229d96634482` |
| `HASH2-45` | The scene's span of the same document (without the first blank line and the final line break). | `source_hash` of its 42 bytes. | `fs2:9e15d23ca2039b27e7ed261a564741edddedecd07a07db5139b77b8100f93a00` |

The document of `HASH2-45`, with `\n` line breaks and a final one:

```text
Title: Tea Time

INT. KITCHEN - DAY

Mara fills the kettle.
```

The `HASH2-42` scene is this text. Its cue shows the exclusion of `(CONT'D)`, and its Parenthetical the removal of the
parentheses:

```text
INT. KITCHEN - DAY

MARA (V.O.) (CONT'D)
(quietly)
Tea?
```

### 11.6 Scheme 2: what the hashes guarantee

**Stable (G1).** None of these changes any story hash:

- the amount or kind of spaces and tabs inside a line, or at its start or end (inside the parentheses of a Parenthetical
  too);
- the line endings (`\n`, `\r\n`, `\r`);
- re-wrapping the lines of one Action paragraph or one run of Dialogue lines (a line break to a space or the reverse).
  Lyrics are the exception: each `~` line is its own entry;
- adding or removing extra blank lines between elements, as long as one remains;
- adding, changing or removing a scene number, a Section, a Note, a Boneyard or a Page Break;
- the spelling of a forcing sigil: `.INT. X` against `INT. X`, `>CUT TO:` against `CUT TO:`;
- adding or removing `(CONT'D)`, in any case and with either apostrophe;
- the case of a heading, a cue name or a cue extension;
- NFC against NFD spelling of the same text;
- the spacing around the dual-dialogue caret (`OTTO^`, `OTTO ^`, `OTTO ^ `);
- the title page: it is in no scene, preamble or document hash of the story facet.

This holds provided the edit does not move non-whitespace text to or from the start of a line in a way that changes how
the line is read, and does not create or remove a blank line inside an element. That proviso is the boundary, below.

**Changes (G2).** A story hash changes for any change of kept content:

- a different word; the case of Action or Dialogue text; an emphasis mark or an escape;
- the **kind** of an element with the same words (Action made into a Transition by `>`, Dialogue made into Action);
- any other **extension** of a cue, added, removed, changed or reordered (`(V.O.)`, `(O.S.)`);
- the **dual** flag (a block paired or unpaired by a caret) and the **centered** flag;
- the line structure of Lyrics (`~a b` against `~a` and `~b`);
- reordering elements or lines, and adding or removing an element of a kept kind (a Synopsis included);
- the preamble (it changes the preamble hash and the document hash, and no scene hash).

**The hierarchy.** An edit to one element changes that element's hash, the hash of its scene and the document hash, and
no other element hash and no other scene hash. Adding, removing or reordering scenes changes the document hash only.

**The boundary.** A blank line added inside an Action paragraph splits it into two Action records, and a blank line
added inside a run of Dialogue lines splits it into Dialogue and Action. Those edits change how the text is read, so
they change the hash. A blank line under a bare cue (`OFF-5b`) turns the cue into Action: the kind changes, so the hash
changes. Scheme 2 has no hole of the kind scheme 1 has (11.7).

### 11.7 Scheme 1: the compatibility scheme

Scheme 1 is **frozen**. Its rules and its output never change; a different rule would be a different scheme. It is the
rule of the Twopop importer (`parse.py` in `twopopfilm/twopop`), applied to this library's parse. That is deliberate.
Twopop Studio stores each scene's hash (`text_sha`, a bare 64-digit string), feeds it into the input hash of every
breakdown and shot-list record, and uses it for exact scene matching when a script is imported again. Staleness is a
comparison of stored hashes, so a different algorithm would make all stored breakdown and shot-list work stale. Studio's
own hash code reproduces the importer's rule byte for byte and has to keep matching it until the Python oracle is
retired. This subsection is complete in itself: it needs nothing from 11.3 to 11.6, only the methods of 11.2.

**Amended before release (maintainer, 2026-11-10).** The first draft of rule 4 gave one line per Action paragraph.
That matched only 2 of the 6 captured scenes. The importer's parser, screenplay-tools, merges consecutive Action
elements by default (`mergeActions`, applied in `_add_element` of its `parser.py`), across blank lines and whole-line
Notes. Rule 4 now does the same, and with it all six captured `text_sha` values match. Scheme 1 had not shipped, so
this is an amendment of the draft and not a change to a frozen scheme.

Scheme 1 hashes **scenes only** (no element, preamble or document hash), and it yields a `Digest` of scheme `V1`, which
prints as the bare hexadecimal text that Studio stores.

**What a scene is, for scheme 1 (the hash scope).**

- A scene starts at a scene heading and runs to the next heading or the end of the document. **Every kept element
  belongs to the last heading seen before it.** This is the importer's rule.
- **This differs from the typed view** for Synopses directly before a heading. The view attaches the Sections and
  Synopses directly before a heading to the scene that follows (section 9), but for scheme 1 such a Synopsis belongs to
  the earlier scene, because it comes after that scene's last element and before the next heading. A Section is never
  hashed, so only Synopses show the difference. The scheme 1 methods follow this **hash scope**, not the view's span.
  The view records each scene's hash scope in a private field that `screenplay()` fills in, so the methods work on a
  `Scene` alone. Only scheme 1 uses it. The scope is, in order:
  1. the scene's `synopses`, minus those that came before the heading;
  2. then its `elements`;
  3. then the **trailing Synopses**, which come after this scene's last element and before the next heading (the view
     attaches them to the next scene).

  For a scene built in memory with `Scene::new(heading)` (section 9) the scope is its own `synopses` and `elements`,
  because nothing came before the heading and nothing trails the elements.
- **The preamble** is everything before the first heading, apart from the title page. Scheme 1 does **not** hash it; the
  importer only counts those elements. A kept element before the first heading (a Synopsis directly before it, say) is
  in no scene's hash scope.

**The normalised text.**

1. **Shape.** The text is the heading line, then `\n`, then the element lines joined with `\n`, with no newline after
   the last element line. **A scene with no kept elements still ends with the `\n`**: its text is the heading line and a
   newline, so a heading-only scene hashes `HEADING` plus a newline and not `HEADING` alone.
2. **Hidden spans, then whitespace.** Hidden spans (Notes, Boneyards) are deleted from every piece of text. Then each
   run of Unicode white-space characters becomes a single space and leading and trailing white space is removed. Line
   breaks inside an element become spaces. `Mara /* aside */ waits.` gives `Mara waits.` No other normalisation: not
   Unicode normal forms, and not case, apart from rules 3 and 5.
3. **Heading line.** The heading's visible text, collapsed, then upper-cased. It leaves out the forcing `.`, the scene
   number (locked or not) and hidden spans.
4. **Element lines.** Each kept element gives one line, which is that element's text only, collapsed, **with no kind
   prefix**. Case is kept. The importer merges consecutive Action elements, so the lines are made by walking the hash
   scope in source order with a current **Action group**:
   - a **non-centered Action** joins the group. The consecutive Actions of a group give **one line**: their typed texts
     joined with a space and collapsed. A blank line between two Action lines makes no difference, except next to a Section, Page Break or Synopsis line (G1).
     An Action whose collapsed text is empty joins the group and adds nothing;
   - a whole-line **Note** or **Boneyard** (`Element::Note`, `Element::Boneyard`) is skipped and **does not end the
     group**;
   - an Action with a **Synopsis** among its interjections **ends the group**: the group's line comes first, then the
     Synopsis lines (see below). A Section or Page Break among an Action's interjections gives no line and does not end
     the group;
   - **anything else ends the group**: a centered Action, a Dialogue block, a dual pair, a Transition, Lyrics, a
     Section, a Synopsis, a Page Break or a Shot. A Section and a Page Break end the group and give no line. A group
     that is ended gives its line at that point, so the lines keep source order.

   The other kept kinds are:
   - a centered Action: its own line (the `>` and `<` marks are dropped), never merged;
   - the Character cue (rule 5);
   - Dialogue: one line per Dialogue element as the view defines it, meaning the lines of a run between Parentheticals,
     joined and collapsed, continuing across interjections;
   - Parenthetical: the text inside the parentheses, trimmed and collapsed, with the parentheses removed. The line
     `(quietly)` gives `quietly`;
   - Transition, without a forcing `>`;
   - Lyrics: **one line per `~` line**, not per run, without the `~`;
   - Synopsis, without the `=`;
   - Shot, only with the `shot` flag, without the `!!`.

   In a dual dialogue the left block comes before the right block, with no marker. An element whose collapsed text is
   empty is dropped. A Synopsis inside an Action paragraph or a Dialogue block, and a Lyrics line inside a Dialogue
   block, give their own lines straight after the line of the element they interrupt, in source order. The interrupted element is still one line (section 13, item 10).
5. **Cue line.** The name only, collapsed and upper-cased. There are **no extension groups** at all, neither `(V.O.)`
   nor `(CONT'D)`, and no caret and no dual flag. (The importer keeps the extension and the dual flag as separate
   fields that it does not hash.)
6. **Not kept:** Sections, Page Breaks, Notes, Boneyards (whole-line or inline), the title page, and the sigils `.`,
   `!`, `@`, `~`, `>`, `<`.
7. **Kept as typed:** emphasis marks and escapes, so a change in italics or underline is a change of content. Text that
   only looks like a tag, such as `#prop-coin`, stays in the text; tags are not supported (5.3).
8. **Hash.** The SHA-256 of the UTF-8 bytes of the normalised text, as a `Digest` of scheme `V1`. Its text form is the
   lowercase hexadecimal of the 32 bytes.

Known answer: the text `HEADING ONLY` plus a newline (13 bytes) hashes to
`f7eb426cff87a0a1af2b2d5390cc9a2477df78d309f88f77f176751c57863e6f`.

The methods are those of 11.2: `Scene::normalized_text_v1` gives the text of rules 1 to 7 over the scene's hash scope,
`heading_line_v1` and `element_lines_v1` give its pieces (the heading line, and the element lines in order), and
`Scene::hash_v1` (with `hash`) gives the digest.

**What scheme 1 guarantees.**

*Stable (G1).* The hash does not change for a whitespace-only edit, meaning one of these:

- the amount or kind of spaces and tabs inside a line, or at its start or end (inside the parentheses of a Parenthetical
  too);
- the line endings (`\n`, `\r\n`, `\r`);
- re-wrapping the lines of one Action paragraph or one run of Dialogue lines (a line break to a space or the reverse).
  Lyrics are the exception: each `~` line is its own hash line;
- adding or removing extra blank lines between elements, as long as one remains;
- **adding or removing a blank line between two Action lines**, whether they are in one paragraph or two. Consecutive
  Actions merge into one line (rule 4), so the paragraphs may be joined or split freely. A whole-line Note or Boneyard
  between them does not separate them either. The exception is a blank line next to a Section, Page Break or Synopsis
  line, because it turns an interjection (inside the paragraph) into an element between paragraphs (see below);
- adding, changing or removing a scene number, a Note, a Boneyard, or a Section or Page Break that is not between two
  Action paragraphs;
- adding, changing or removing any cue extension (`(CONT'D)`, `(V.O.)`) or the dual-dialogue caret, or changing the case
  of a heading or a cue name.

This holds provided the edit does not move non-whitespace text to or from the start of a line in a way that changes how
the line is read, and does not create or remove a blank line inside a Dialogue block or a Lyrics run (the blank lines
of an Action paragraph are the exception above). A Section, a Page Break, a Synopsis, a centered Action or any other
element that is not Action (a whole-line Note or Boneyard excepted) and sits between two Action paragraphs **splits
them into two lines**: adding or removing it changes the hash. A Section or Page Break inside an Action paragraph (an
interjection) does not. So `A`, `# Sec`, `B` on consecutive lines is one Action with an interjection, and the line is
`A B`; blank lines around the Section make it Action, Section, Action, and the lines are `A` and `B`. The same holds for
a Page Break, and for a Synopsis (which also moves the Synopsis line after the Action line).

*Changes (G2).* The hash changes for any change of kept text: a different word, case in Action or Dialogue, an
emphasis mark, a different cue name, a reordered line, an added or removed element of a kept kind, a different
Synopsis. Moving a Synopsis across a heading moves it between the hash scopes of two scenes.
Adding or removing a non-Action element other than a whole-line Note or Boneyard between two Action paragraphs changes
the hash, because it splits or joins the two lines.

*Limits (G3).* Scheme 1 **does not see a change of element kind or of a cue's extension when the text is the same**,
because the importer does not. Action made into a Transition with the same words, Dialogue made into Action, a cue
that gains or loses `(V.O.)`: the hash stays. The exception is a change that makes an Action join or leave a group
of Actions, because the merge then changes the lines (a Transition between two Actions that becomes Action is one
line instead of three). It also does not hash the preamble. This is a known limit, accepted for compatibility with the
importer, and scheme 2 removes it.

*The boundary.* A blank line added inside an Action paragraph does **not** change the hash (unless it is next to a
Section, Page Break or Synopsis line, G1): the two paragraphs merge
again (rule 4). A blank line added inside a run of Dialogue lines splits it into Dialogue and Action, which changes how
the text is read, so it changes the hash. A blank line under a cue (`OFF-5b`) turns the cue into Action, and now the
hash **always changes**: the cue line and the line after it were two lines, and as two Action paragraphs they merge into
one. Under a bare cue, `BEN` and `Hello` (the lines `BEN` and `Hello`) become the single line `BEN Hello`. Under a cue
with an extension, `BEN (V.O.)` and `Hello` become `BEN (V.O.) Hello`, which holds the extension as well. (Before the
merge, the bare cue kept the hash by G3. It no longer does.)

**How scheme 1 agrees with the importer.** The hashing rule is the importer's, as verified against its code and its
captured fixtures: the same shape, the same heading line, the same line for each kind, the same scene membership, and
the same merging of consecutive Action elements (rule 4), which is part of the hash rule and not of the parser. So the
differences that remain are all caused by **parsing, not hashing**. Where a ruling in section 6 splits or joins elements
differently from the importer's parser, or decides a case differently, the elements and so the hash differ for that
input. The F4 diff harness classifies each such difference. This library claims parity with the importer for scheme 1
on the importer's own fixtures and nowhere else.

| Known case | Why the elements can differ |
|---|---|
| Unframed headings (`OFF-6`) | A heading line without blank lines around it is decided by the rulings in section 6; the importer's parser may decide it differently. |
| `CUT TO:` forms (`OFF-9`, `OFF-23`) | Whether a `TO:` line is a Transition, a cue or Action depends on the blank lines and on trailing whitespace, and the parsers may disagree. |
| A Note spanning two lines (`OFF-15`) | Hidden spans are deleted before the text is read here, so lines can join or vanish differently from the importer's result. |
| Sections, Synopses or Lyrics inside a Dialogue block (`OFF-21`, `OFF-24`) | The block continues across them here, as one Dialogue element; the importer's parser may split it. |
| Tag text (`OFF-20`) | `#prop-coin` inside a line stays in the text; a line that starts with `#` is a Section here, which is not hashed. |
| Title page edge cases (`OFF-17`) | Which opening lines form the title page decides which text is an element, so the parsers can differ on the first scene. |

Whitespace and case: the collapse here uses Unicode White_Space and the upper-casing uses Rust's full mapping. If the
importer splits with Python's default `split()`, that also treats U+001C to U+001F as separators; the two agree on
ordinary text and are not compared on those characters (section 13, item 6).

**What this means for a consumer.** A tool that stores `text_sha` from the importer can adopt this crate without
re-hashing, for every script both parsers read the same way: `hash_v1` gives the same value. A script that is parsed
differently shows as one `changed` scene on its first re-import. `HASH1-21` and `HASH1-22` check the agreement on the
importer's own fixtures. Moving to scheme 2 is a separate step (11.8).

### 11.8 Migrating an application from scheme 1 to scheme 2

The library makes the migration possible; the application runs it. An application that stored scheme 1 digests (Twopop
Studio's `text_sha`) migrates **lazily**, a scene at a time, as it meets each stored value. Twopop Studio keeps scheme 1
while its Python oracle exists, so that parity can be proven, and migrates after. Twopop Studio tracks its own
migration as an issue in its repository (`twopopfilm/twopop-studio#252`, "M4.11 Migrate scene hashes to fountain-syntax
scheme 2"), which waits for the parity gate and for this crate's issues #10 and #35.

**Rules for any application:**

1. Store the digest in its text form (`Digest` printed), so that the scheme is stored with every hash. A scheme 2 value
   starts with `fs2:`.
2. Read a stored value with `FromStr`. A bare 64-digit value is scheme 1. Do not guess the scheme any other way.
3. Compare only digests of the same scheme. Never compare a scheme 1 digest with a scheme 2 digest, even when the bytes
   are equal: they are never equal as `Digest` values.
4. Build any input hash that embeds a scene hash from the digest's text form, so that a change of scheme is visible in
   the input hash.

**Rebaseline a scene whose stored digest is scheme 1:**

1. Compute `hash_v1()` of the scene as it is now, from the same parse.
2. If it equals the stored digest, the scene has not changed by the rule that made the stored value. Store `hash()` (the
   scheme 2 digest) **without marking any dependent work stale**.
3. If it differs, the scene changed. Handle it as any change (dependent work becomes stale), and store `hash()`.
4. Scene matching on re-import follows the same rule: while a stored value is scheme 1, match it against the new
   scene's `hash_v1()`; once it is scheme 2, match it against `hash()`.

**Input hashes that embed a scene hash** (a breakdown's input hash, say) change when the embedded digest changes from
scheme 1 to scheme 2. The application must **re-base them in the same step** as step 2, by computing them again with the
scheme 2 digest, and only for records whose stored input hash matched the old computation. A record that was already
stale stays stale. The library cannot do this: it does not know what an application derives from a hash.

**One blind spot, once.** Between the last scheme 1 comparison and the rebaseline, a change of element kind or of a cue
extension is invisible, as it is today (11.7, G3). From the rebaseline on, scheme 2 sees it. This is the price of
migrating without false staleness.

**The library's job:** keep scheme 1 frozen and available (`hash_v1`, `normalized_text_v1`), give both digests from one
parse, and make the scheme part of every digest. **The application's job:** store the scheme, run the rebaseline in one
transaction with the input hashes, and never compare across schemes.

### 11.9 Tests

Scheme 1 tests, `HASH1-n`. Methods are those of 11.7. Tests marked `hash` need that feature.

| ID | Test |
|---|---|
| `HASH1-1` | Spaces and tabs inside lines, at their ends and at their starts do not change the hash. |
| `HASH1-2` | `\n`, `\r\n` and `\r` give the same hash. |
| `HASH1-3` | Re-wrapping the lines of one Action paragraph and one Dialogue block does not change the hash. |
| `HASH1-4` | Extra blank lines between elements do not change the hash. |
| `HASH1-5` | Adding or removing a Note, a Boneyard (inline or whole-line), a Section or a Page Break does not change the hash, also when it sits inside an Action paragraph or a Dialogue block. The exception is a Section or a Page Break between two Action paragraphs: it splits them into two lines, so adding or removing it changes the hash (`HASH1-27`). |
| `HASH1-6` | Adding, changing or removing a scene number, locked or not, does not change the hash. |
| `HASH1-7` | Case of the heading and of a cue name does not change the hash. |
| `HASH1-8` | Adding or removing `(CONT'D)` on a cue, adding, removing or changing `(V.O.)`, and adding or removing the dual-dialogue caret do not change the hash. |
| `HASH1-9` | A forced heading (`.INT. X`) and the same heading without the sigil have the same hash. |
| `HASH1-10` | Changing a word in Action, Dialogue, a Transition or a Synopsis changes the hash. |
| `HASH1-11` | Changing the case of Action or Dialogue text changes the hash. |
| `HASH1-12` | Adding or removing an emphasis mark changes the hash. |
| `HASH1-13` | Changing the element kind while the text stays the same (Action to Transition by `>`, Dialogue to Action) does **not** change the hash: the known limit G3. The Action in the test has no Action neighbour, since a neighbour would merge with it (`HASH1-27`). |
| `HASH1-14` | Swapping two lines changes the hash. |
| `HASH1-15` | The boundary in 11.7. A blank line inside an Action paragraph does **not** change the hash (the paragraphs merge, `HASH1-27`). A blank line inside a run of Dialogue lines changes it. A blank line under a bare cue changes it (`BEN` and `Hello` become the one line `BEN Hello`), and so does one under a cue with an extension (`BEN (V.O.)` and `Hello` become `BEN (V.O.) Hello`). |
| `HASH1-16` | A Synopsis is hashed and a Section is not. A Synopsis after a scene's last element and before the next heading is in the earlier scene's hash and not in the next one's; a Synopsis directly after a heading is in its own scene's. |
| `HASH1-17` | Elements before the first heading (the preamble, and a Synopsis directly before the first heading) and the title page are in no scene's hash. |
| `HASH1-18` | A scene with only a heading has the text `HEADING` plus a newline. Two scenes with equal text have equal hashes. |
| `HASH1-19` | Dual dialogue hashes the left block, then the right, with no marker. |
| `HASH1-20` | A known-answer test: the text `HEADING ONLY` plus a newline (13 bytes) gives `f7eb426cff87a0a1af2b2d5390cc9a2477df78d309f88f77f176751c57863e6f`, as the text form of `hash_v1`. Needs `hash`. |
| `HASH1-21` | Parity with the importer on `edge_cases.fountain`, a fixture original to the Twopop project (MIT OR Apache-2.0; the test records its source and licence in the corpus README). The `THE ROOF AT DAWN` scene's `hash_v1` is `6601466ae39394c1f0dfed5d957f82226c7609b2ad63a9aaa17728e28c10e625` and the `INT. LAUNDERETTE - DAY` scene's is `7f243b3424fe00de4b1dc7bc2ae3e735aa7eee24297afa2f8b0df943b1443508`. Needs `hash`. |
| `HASH1-22` | Parity with the importer on its other captured scripts, by the same method: each scene's `hash_v1` equals the stored value, and each difference is classified in the F4 diff harness as a parsing difference (11.7). Needs `hash`. |
| `HASH1-23` | A Parenthetical line is the text inside the parentheses, trimmed and collapsed: `(quietly)` gives `quietly`, and `( quietly )` gives the same. |
| `HASH1-24` | Lyrics give one line per `~` line, not one per run: two `~` lines are two lines in the text. |
| `HASH1-25` | The line shapes: a cue line is the name only, upper-cased (no extension, no caret); a centered Action line has no `>` or `<`; a Shot line (with the `shot` flag) has no `!!`; no line has a kind prefix; `#prop-coin` stays in the text. |
| `HASH1-26` | `hash_v1` returns a `Digest` of scheme `V1` whose text form is the bare hexadecimal digits (no prefix), and it is the same on every call. The three `_v1` line methods exist and work in a build without `hash`. |
| `HASH1-27` | The Action merge (11.7, rule 4). Consecutive non-centered Actions give one line, joined with a space and collapsed, across blank lines and across a whole-line Note or Boneyard, so adding or removing a blank line between two Action lines does not change the hash, except next to a Section, Page Break or Synopsis line, where it turns an interjection into an element between paragraphs. A centered Action, a Section, a Synopsis, a Page Break, a Transition, Lyrics, a Dialogue block or a Shot between two Actions ends the group, so the two Actions stay two lines and the Synopsis gives its own line. An Action with a Synopsis interjection gives its group line first, then the Synopsis line, and ends the group. |

Scheme 2 tests, `HASH2-n`. Methods are those of 11.2, and every test needs `hash`.

| ID | Test |
|---|---|
| `HASH2-1` | G1: spaces and tabs inside lines, at their ends and at their starts (inside the parentheses of a Parenthetical too) do not change any hash of the scene. |
| `HASH2-2` | G1: `\n`, `\r\n` and `\r` give the same hashes. |
| `HASH2-3` | G1: re-wrapping the lines of one Action paragraph and of one Dialogue run does not change any hash. Two `~` lines are two entries however they are spaced. |
| `HASH2-4` | G1: extra blank lines between elements do not change any hash. |
| `HASH2-5` | G1: adding or removing a Note, a Boneyard (inline or whole-line), a Section or a Page Break does not change any hash of the scene, also inside an Action paragraph or a Dialogue block, and does not split a Dialogue run. |
| `HASH2-6` | G1: adding, changing or removing a scene number, locked or not, does not change any hash. |
| `HASH2-7` | G1: `.INT. X` and `INT. X`, a `!` paragraph and the same Action unforced, an `@` cue and the same cue unforced, and a `>` Transition and the same `TO:` line unforced have equal hashes. |
| `HASH2-8` | G1: `(CONT'D)`, `(cont'd)` and `(CONT’D)` added or removed give equal hashes, and a cue with only `(CONT'D)` hashes as the bare cue. |
| `HASH2-9` | G1: the case of a heading, a cue name and a cue extension does not change any hash. |
| `HASH2-10` | G1: NFC and NFD spellings of the same text in a heading, an Action, a cue name, an extension, a Lyrics line and a title-page key give equal hashes. |
| `HASH2-11` | G1: the spacing around the caret (`OTTO^`, `OTTO ^`, `OTTO ^ `) gives equal hashes. |
| `HASH2-12` | G1: editing, adding or removing the title page changes no scene hash, no preamble hash and not `Screenplay::hash`. |
| `HASH2-13` | G2: changing a word in the heading, Action, Dialogue, a Parenthetical, a Transition, Lyrics, a Synopsis or (with `shot`) a Shot changes that hash, and only the hashes above it. |
| `HASH2-14` | G2: changing the case of Action or Dialogue text changes the hash. |
| `HASH2-15` | G2: adding or removing an emphasis mark, and adding or removing an escape, changes the hash. |
| `HASH2-16` | G2: the kind changes with the same words: Action to Transition by `>`, Dialogue to Action, Action to Synopsis, Action to Parenthetical. Each changes the hash. |
| `HASH2-17` | G2: adding, removing, changing or reordering a cue extension other than `(CONT'D)` changes the hash. |
| `HASH2-18` | G2: the dual flag. A block paired by a caret changes the hash of both cues, a caret on a block with no partner changes nothing, and unpairing changes it back. |
| `HASH2-19` | G2: the centered flag. `> FIN <` and `FIN` have different hashes. |
| `HASH2-20` | G2: Lyrics. One run of two lines and one line of the two joined differ; two runs and one run of the same lines differ; an empty `~` line is dropped. |
| `HASH2-21` | G2: swapping two elements, two lines of one Dialogue run and two scenes changes the hash. |
| `HASH2-22` | G2: adding or removing an element of a kept kind changes the hash; adding or removing a Synopsis changes it and adding a Section does not. |
| `HASH2-23` | G2: the preamble. Adding, changing or removing a preamble element changes `Preamble::hash` and `Screenplay::hash`, and no scene hash. An empty preamble has one fixed hash. |
| `HASH2-24` | The boundary: a blank line inside an Action paragraph, inside a run of Dialogue lines, and under a bare cue, and under a cue with an extension, each change the hash. |
| `HASH2-25` | Scene membership follows the view: a Synopsis directly before a heading is in the scene that follows and not in the one before; a Synopsis directly after a heading is in its own scene; a Synopsis or Section before the first heading belongs to scene 1 and not to the preamble. |
| `HASH2-26` | Dropped records: a Parenthetical `()`, a Synopsis `=` with no text, a forced Action line whose only text is a Note, a Lyrics element of empty `~` lines and a cue with an empty name give no record. The scene's hash equals that of the scene without them. An empty extension group `()` is kept. |
| `HASH2-27` | A Parenthetical is the text inside the parentheses, trimmed and collapsed: `(quietly)` and `( quietly )` give equal hashes, and `(Quietly)` differs. |
| `HASH2-28` | Dialogue records. A Lyrics or Synopsis line inside a Dialogue run does not split the run and its record comes after the run's; one between a cue and its first line comes before the run's. An Action's interjection records come after its record. A dual pair gives the left block's records, then the right's. |
| `HASH2-29` | With `shot`, a `!!` line gives a Shot record without the marks; without it the line is an Action whose text begins with `!`. |
| `HASH2-30` | Injectivity. Each pair has different hashes: Action `ab` then Action `c`, against Action `a` then Action `bc`; Dialogue `ab`, a Parenthetical, Dialogue `c`, against Dialogue `a`, a Parenthetical, Dialogue `bc`; Lyrics `["a b"]` against `["a","b"]`; the cue `X (A) (B)` against `X (A B)`; the cue `X (Y)` against the cue `X Y`; an Action `x` against a Transition `x` against a Synopsis `x`; a heading `A` with one Action `B` against a heading `A B` with no element; two scenes against one scene with the same elements. |
| `HASH2-31` | Domain separation. The same payload bytes hashed under each of the six domains give six different digests, so an element hash never equals a scene hash, a preamble hash, a document hash, a title-page hash or a source hash. A scene with no elements and a preamble with no elements differ. |
| `HASH2-32` | A scene's hash does not depend on its position or number: the same scene text in two places, or with another scene number, gives equal hashes, and two identical scenes in one document have equal hashes. |
| `HASH2-33` | `Scene::element_hashes` has one digest per kept record, in order, equal to the concatenation of `Element::hashes` over the scene, and `Scene::hash` is recomputable from the heading and those digests. |
| `HASH2-34` | The hierarchy. Changing one element changes its own hash, its scene's hash and the document hash and no other element or scene hash. |
| `HASH2-35` | The document hash changes when a scene is added, removed or reordered, and when the preamble changes; it does not change with the title page. |
| `HASH2-36` | `title_page_hash` changes with a key, a value or its lines, an added entry or reordered entries, not with key case, NFC against NFD or spacing, and not with a stray line. It does not depend on the story hashes. A screenplay with no title page gives the hash of zero entries. |
| `HASH2-37` | Source hashes change on a whitespace-only edit (an indent, a trailing space, a line ending, a blank line) for a scene's span, the whole input and any node, and are equal for equal bytes. `Scene::source_hash` is `None` for a scene built in memory. |
| `HASH2-38` | `Digest` text form. A scheme 2 digest prints as `fs2:` and 64 lowercase digits and a scheme 1 digest prints bare; both parse back to an equal value; a bare 64-digit string parses as scheme 1; uppercase digits, a wrong length, an `fs1:` or `fs3:` prefix and spaces are errors. |
| `HASH2-39` | Scheme tags. `HASH_SCHEME_CURRENT` is 2 and equals `HashScheme::V2.number()`; `Scene::hash` and the other scheme 2 methods give `V2`; `hash_v1` gives `V1`; a `V1` and a `V2` digest with equal bytes are not equal. |
| `HASH2-40` | Known answer (11.5): the scene with only a heading. |
| `HASH2-41` | Known answers (11.5): the Action and the scene with one Action. |
| `HASH2-42` | Known answers (11.5): the cue with `(V.O.)` and `(CONT'D)`, the Parenthetical, the Dialogue and the scene of the three. |
| `HASH2-43` | Known answers (11.5): the Action with a precomposed `é` and the same Action with a decomposed one have the same recorded digest. |
| `HASH2-44` | Known answers (11.5): the centered Action, the dual right-hand cue and the Lyrics. |
| `HASH2-45` | Known answers (11.5): the empty preamble, the document, the title page and the two source hashes. |
| `HASH2-46` | Migration (11.8). With a stored scheme 1 digest of a scene, a test application stores the scheme 2 digest and marks nothing stale when `hash_v1` equals the stored value, and marks the scene changed when it does not; it re-bases an input hash that embeds the digest in the same step; and it never compares a `V1` with a `V2`. A change of kind that scheme 1 cannot see is seen by `hash` afterwards. |
| `HASH2-47` | Features. In a build without `hash` the types of 11.1, the `_v1` line methods and the spans exist and no digest method does (a compile-fail test); with `hash` every method of 11.2 exists. |

## 12. Test IDs

Every decision, element, extension, diagnostic, hash rule and writer property in this file has an ID. **Each ID names a
test to be written** in F2 to F4: the parser, view, diagnostics and scheme 2 hash tests in F2 (issues #7 to #10), the
scheme 1 hash tests in F2 (#35), the writer tests in F3 (#12, #13) and the conformance corpus in F4 (#15, #16). The test
carries the ID in its name, so `grep` finds the test for any decision.

| Scheme | Meaning | Where defined |
|---|---|---|
| `EL-<element>-n` | An ordinary case of one standard element. | Section 4 |
| `OFF-<n><letter>` | One row of the decision table for case `OFF-<n>`. | Section 6 |
| `EXT-<flag>-n` | An extension or feature flag. The always-on items use their own short names. | Section 5 |
| `DIAG-<code>` | One diagnostic code: an input that fires it and a near miss that does not. | Section 7 |
| `HASH1-n` | A scheme 1 (compatibility) hash and normalisation rule. | Section 11 |
| `HASH2-n` | A scheme 2 (default) hash, encoding, digest or migration rule. | Section 11 |
| `WR-n` | A writer property. | Section 10 |

**The conformance corpus (issue #15).** Each `OFF` row gets a corpus script that exercises it. **Every corpus script
keeps its first block ordinary** (a plain Action paragraph, with no colon), and puts the case after it. Otherwise
another parser may read the opening lines of an edge case as a title page (`OFF-17a`), and a difference found by the
diff harness would come from the start of the file and not from the case.

### 12.1 Index of every ID

<!-- ID-INDEX:start -->
**Elements (section 4)**, 37 IDs:

`EL-action-1`, `EL-action-2`, `EL-action-3`, `EL-action-4`, `EL-boneyard-1`, `EL-centered-1`, `EL-centered-2`,
`EL-character-1`, `EL-character-2`, `EL-character-3`, `EL-dialogue-1`, `EL-dialogue-2`, `EL-dialogue-3`, `EL-dual-1`,
`EL-dual-2`, `EL-emphasis-1`, `EL-emphasis-2`, `EL-heading-1`, `EL-heading-2`, `EL-heading-3`, `EL-heading-4`,
`EL-indent-1`, `EL-linebreak-1`, `EL-lyrics-1`, `EL-lyrics-2`, `EL-note-1`, `EL-page-break-1`, `EL-parenthetical-1`,
`EL-punctuation-1`, `EL-section-1`, `EL-synopsis-1`, `EL-title-page-1`, `EL-title-page-2`, `EL-title-page-3`,
`EL-title-page-4`, `EL-transition-1`, `EL-transition-2`

**Decision rows (section 6)**, 114 IDs:

`OFF-1a`, `OFF-1b`, `OFF-1c`, `OFF-1d`, `OFF-2a`, `OFF-2b`, `OFF-2c`, `OFF-2d`, `OFF-3a`, `OFF-3b`, `OFF-3c`,
`OFF-3d`, `OFF-3e`, `OFF-4a`, `OFF-4b`, `OFF-4c`, `OFF-4d`, `OFF-5a`, `OFF-5b`, `OFF-6a`, `OFF-6b`, `OFF-6c`,
`OFF-6d`, `OFF-6e`, `OFF-7a`, `OFF-7b`, `OFF-7c`, `OFF-7d`, `OFF-8a`, `OFF-8b`, `OFF-8c`, `OFF-8d`, `OFF-8e`,
`OFF-8f`, `OFF-8g`, `OFF-9a`, `OFF-9b`, `OFF-9c`, `OFF-9d`, `OFF-9e`, `OFF-10a`, `OFF-10b`, `OFF-10c`, `OFF-10d`,
`OFF-10e`, `OFF-11a`, `OFF-11b`, `OFF-11c`, `OFF-11d`, `OFF-11e`, `OFF-12a`, `OFF-12b`, `OFF-12c`, `OFF-13a`,
`OFF-13b`, `OFF-13c`, `OFF-13d`, `OFF-14a`, `OFF-14b`, `OFF-14c`, `OFF-14d`, `OFF-14e`, `OFF-14f`, `OFF-15a`,
`OFF-15b`, `OFF-15c`, `OFF-15d`, `OFF-15e`, `OFF-15f`, `OFF-15g`, `OFF-16a`, `OFF-16b`, `OFF-16c`, `OFF-16d`,
`OFF-16e`, `OFF-17a`, `OFF-17b`, `OFF-17c`, `OFF-17d`, `OFF-17e`, `OFF-17f`, `OFF-18a`, `OFF-18b`, `OFF-18c`,
`OFF-18d`, `OFF-19a`, `OFF-19b`, `OFF-19c`, `OFF-19d`, `OFF-19e`, `OFF-20a`, `OFF-20b`, `OFF-20c`, `OFF-20d`,
`OFF-20e`, `OFF-21a`, `OFF-21b`, `OFF-21c`, `OFF-22a`, `OFF-22b`, `OFF-22c`, `OFF-22d`, `OFF-22e`, `OFF-22f`,
`OFF-22g`, `OFF-23a`, `OFF-23b`, `OFF-23c`, `OFF-23d`, `OFF-23e`, `OFF-24a`, `OFF-24b`, `OFF-24c`, `OFF-24d`

**Extensions (section 5)**, 15 IDs:

`EXT-contd-1`, `EXT-contd-2`, `EXT-locked-scene-numbers-1`, `EXT-locked-scene-numbers-2`,
`EXT-locked-scene-numbers-3`, `EXT-locked-scene-numbers-4`, `EXT-notes-view-1`, `EXT-notes-view-2`, `EXT-open-keys-1`,
`EXT-open-keys-2`, `EXT-revision-1`, `EXT-sections-1`, `EXT-shot-1`, `EXT-shot-2`, `EXT-shot-3`

**Diagnostics (section 7)**, 22 IDs:

`DIAG-adjacent-outline-line`, `DIAG-ambiguous-equals`, `DIAG-blank-lookalike`, `DIAG-caret-not-last`,
`DIAG-cue-needs-force`, `DIAG-emphasis-line-break`, `DIAG-empty-heading`, `DIAG-lone-caret`,
`DIAG-mixed-line-endings`, `DIAG-nested-opener`, `DIAG-partial-parenthetical`, `DIAG-period-not-forcing`,
`DIAG-scene-number-rejected`, `DIAG-sigil-in-dialogue`, `DIAG-stray-closer`, `DIAG-title-page-stray-line`,
`DIAG-to-trailing-space`, `DIAG-transition-lookalike`, `DIAG-unclosed-boneyard`, `DIAG-unclosed-note`,
`DIAG-unframed-heading`, `DIAG-unframed-transition`

**Hashes, scheme 1 (section 11)**, 27 IDs:

`HASH1-1`, `HASH1-2`, `HASH1-3`, `HASH1-4`, `HASH1-5`, `HASH1-6`, `HASH1-7`, `HASH1-8`, `HASH1-9`, `HASH1-10`,
`HASH1-11`, `HASH1-12`, `HASH1-13`, `HASH1-14`, `HASH1-15`, `HASH1-16`, `HASH1-17`, `HASH1-18`, `HASH1-19`,
`HASH1-20`, `HASH1-21`, `HASH1-22`, `HASH1-23`, `HASH1-24`, `HASH1-25`, `HASH1-26`, `HASH1-27`

**Hashes, scheme 2 (section 11)**, 47 IDs:

`HASH2-1`, `HASH2-2`, `HASH2-3`, `HASH2-4`, `HASH2-5`, `HASH2-6`, `HASH2-7`, `HASH2-8`, `HASH2-9`, `HASH2-10`,
`HASH2-11`, `HASH2-12`, `HASH2-13`, `HASH2-14`, `HASH2-15`, `HASH2-16`, `HASH2-17`, `HASH2-18`, `HASH2-19`,
`HASH2-20`, `HASH2-21`, `HASH2-22`, `HASH2-23`, `HASH2-24`, `HASH2-25`, `HASH2-26`, `HASH2-27`, `HASH2-28`,
`HASH2-29`, `HASH2-30`, `HASH2-31`, `HASH2-32`, `HASH2-33`, `HASH2-34`, `HASH2-35`, `HASH2-36`, `HASH2-37`,
`HASH2-38`, `HASH2-39`, `HASH2-40`, `HASH2-41`, `HASH2-42`, `HASH2-43`, `HASH2-44`, `HASH2-45`, `HASH2-46`, `HASH2-47`

**Writer (section 10)**, 8 IDs:

`WR-1`, `WR-2`, `WR-3`, `WR-4`, `WR-5`, `WR-6`, `WR-7`, `WR-8`

**Total: 270 IDs.**
<!-- ID-INDEX:end -->

## 13. Open items

Things the research could not settle, or that needed the maintainer's decision. Nothing here blocks F2. Items 1 to 4
and 7 were resolved by the maintainer on 2026-11-05.

1. **Hash schemes.** *Resolved by the maintainer on 2026-11-05:* hash scheme 2 is the default and the recommended
   scheme, and scheme 1 (the importer's rule) is retained, frozen, for compatibility. The instruction was to apply best
   practice in the core library even if the applications that use it must adapt (11, and the migration in 11.8).
2. **Where SHA-256 comes from.** *Resolved by the maintainer on 2026-11-05:* option B, an optional feature with `sha2`
   0.10 (11.2). The feature is now called `hash`, because it also carries scheme 2.
3. **Results of "literal first" that may surprise authors.** *Resolved by the maintainer on 2026-11-05:* all rulings
   approved as written. They follow from P1 and each has a diagnostic where one helps: `Ext. 12 is busy.` and a bare
   `INT.` are headings (`OFF-6c`, `OFF-6d`); `CUT TO:` directly above a heading is a Character cue (`OFF-9c`) while
   `CUT TO: ` with a trailing space is Action (`OFF-23b`); `#1 priority item` is a Section (`OFF-20a`); a first block
   such as `Note: ...` is a title page (`OFF-17a`).
4. **Where the tree stops and the view starts for scenes.** *Resolved by the maintainer on 2026-11-05:* the tree is
   flat and scenes exist only in the typed view (8.4).
5. **Colour names and tag length for notes conventions.** The research gives neither Beat's list of colour names nor the
   exact limit of Slugline's "short prefix" (extensions.md: not determined). `NoteConvention` returns the words as typed
   and the tag as one word; the meaning is the caller's.
6. **Whitespace and case mapping against Python (scheme 1 only).** The importer's hashing rule is verified (11.7), but
   the collapse here uses Unicode White_Space and the upper-casing uses Rust's full mapping, and neither was compared
   with Python's on unusual characters. `HASH1-22` covers only the captured scripts. Scheme 2 has no Python to match.
7. **The `unicode-normalization` dependency.** *Resolved by the maintainer on 2026-11-05: accepted.* Scheme 2 applies
   NFC to everything it hashes (11.3), which needs the `unicode-normalization` crate behind the `hash` feature. NFC is a
   text-identity best practice, so that `é` typed as one character and as `e` plus an accent are the same word. *Cost
   of dropping it:* NFC would become a documented non-goal, canonically equivalent spellings would
   hash differently, and the tests `HASH2-10` and `HASH2-43` and the NFC step of 11.3 would go.
8. **Reference parser, legacy results.** Several legacy-parser results are "not determined by reading" (`OFF-12`,
   `OFF-21`, `OFF-24`) or low confidence. No decision depends on them.
9. **Serde encoding of the tree** (8.6) and the exact tokenisation (`TokenKind`) are left to F2. The invariants in 8.3
   are the contract.
10. **Oracle behaviour on multi-line Lyrics and interrupted blocks (scheme 1 parity).** *Partly settled on 2026-11-10:*
    the importer's merging of consecutive Action elements is settled by the captured scripts, where all six `text_sha`
    values match with the merge (11.7, rule 4, `HASH1-27`). Still open: how the importer treats several `~` lines and a
    Dialogue block or Action paragraph interrupted by a Section, Synopsis or Lyrics line. Only the F4 parity tests
    (`HASH1-22`) settle those, and the merging of consecutive Dialogue lines was read from the parser's default
    settings, not from a run of every shape. Scheme 2 does not depend on it.
11. **Details of scheme 2 chosen by the writer, for review.** They are rules now, and each is easy to change before F2:
    the `dual` flag is true for both cues of a pair and false for an unpaired caret; empty `~` lines are dropped from a
    Lyrics list but an empty extension group `()` is kept; the stray lines of a title page are not hashed; the heading
    has no tag and is the first field of the scene hash; `Element::hashes` returns a list; and the source hashes of a
    scene and a screenplay take the `Document`, because the view holds no source text.
