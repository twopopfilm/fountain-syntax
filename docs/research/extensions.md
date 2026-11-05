# Fountain extensions used in practice: a survey

Input to `SPEC.md` (issue #4), from issue #3 (F1.2). [official.md](official.md) records the official 1.1 syntax and its
24 open cases (`OFF-1` to `OFF-24`). [reference-parser.md](reference-parser.md) records how the Objective-C reference
parser handles them. This file adds the third input: what six apps and libraries do **beyond** the 1.1 text, and where
they **diverge** from it on ordinary input. It also ranks candidate extensions and keeps notes for a later Final Draft
(FDX) crate. It makes **no rulings**: everything here is **evidence, not a ruling**, and `SPEC.md` (#4) rules.

In this file an **extension** is syntax or behaviour that 1.1 does not define, and a **divergence** is an app treating
input that 1.1 does define in a different way from the literal text.

## Source

Access date for every row: **2026-11-05**.

| App | Kind | Public sources | Pin | Licence |
|---|---|---|---|---|
| Beat | Open source app (macOS, iOS) | [Repository][beat-repo], [manual][beat-manual], [file format note][beat-format], [changelog][beat-changelog] | Commit `ab2382184ca41e50440ff695373b17b4e57c915f` on `main` (committed 2026-11-02). The newest release, `v2.1.18` (2026-05-26), is older than the pin, so the pinned code can differ from the shipped app. | GPL-3.0-or-later, read from the [licence file][beat-license] (GitHub reports "no assertion") |
| Highland 2 | Closed app (Quote-Unquote Apps) | [Fountain syntax][hl-syntax], [Notes, Synopses and Such][hl-nsa], [text formatting][hl-fmt], [headers and footers][hl-hf], [what's new in 2.6][hl-26], [Revision Mode article][hl-rev] | Help pages for Highland 2 only (no version shown). Highland Pro: not determined from public sources. | Proprietary; terms not examined |
| Slugline | Closed app (macOS, iOS) | [Dual dialogue][sl-dual], [FDX][sl-fdx], [CONT'D][sl-contd], [Slugline 2 launch][sl-2], [scene numbers][sl-sn], [outlining][sl-outline], [notes][sl-notes] | Slugline 2 for macOS, pages dated 2013 to 2020 | Proprietary; terms not examined |
| Afterwriting | Open source web app, CLI and parser | [`afterwriting-labs`][aw-labs], [`aw-parser`][awp-repo], [CLI document][aw-clients] | `afterwriting-labs` commit `f1912c08d0905bac42e1805830a1e72f98b88c2f` (2024-09-24, `package.json` version 1.17.3). The parser it depends on is `aw-parser` 0.1.1, read at tag `v0.1.1`, commit `cf3c2c246e1d90992c69e3c11d237205e611543a` (2018-07-28). | MIT, per each repository's licence file and `package.json` |
| Fade In | Closed app (GCC Productions Inc.), Fountain import and export | [Changelog ("older changes")][fi-cl], [Final Draft FAQ][fi-fd], [import FAQ][fi-punct], [Fountain app list][fountain-apps] | Newest dated changelog entry read: 4.2.0 (2025-04-08). The current changelog page was not reachable. | Proprietary; terms not examined |
| screenplay-tools | Open source library (Python, JavaScript, C#, C++) | [Repository][st-repo], [README][st-readme] | Commit `e3659406de7ab293b17125e065ed8b4f9af74203` on `main` (committed 2026-10-20). The newest release, `v0.0.10` (2026-01-05), is older than the pin. | MIT, per the repository's licence file |

### Method

- **Public pages and static reading only.** Open source code was read in the repository at the pinned commit. Nothing
  was cloned, built, run or executed, and no test was run. The head of each default branch was checked again after
  reading and still equalled its pin. Every code link below is a permalink to the pin.
- **Closed apps** are recorded from their published help pages, release notes and the vendor's own blog. Where those do
  not settle a point, the entry says **not determined from public sources**. No closed app was installed or run.
- **Help pages lose markup characters.** Several help pages are rendered from Markdown, so the page text can show
  `*` or `/*` as emphasis instead of as the typed token. Tokens below were checked against the page source, not only
  the rendered text. Where the source still does not show the exact token, the entry says so.
- **Hosts.** The Slugline pages were read on the vendor's alternative host. The same paths on `slugline.co`, which
  are linked here, resolve to the same posts.
- **Code reading is by hand and is lower confidence than a run.** Where a result could not be settled by reading, the
  entry says **not determined by reading**.
- **Code in GitHub's `main` is not the shipped app.** For Beat and screenplay-tools the pin is the repository, which can
  be ahead of any release.
- **Fade In dates.** The older entries in its changelog carry no date. Only entries that show a date are cited with
  one; the others are marked undated.
- **Leads.** Search-result summaries of third-party pages were used only to find first-party pages, and are not cited
  for any claim.

### Own words, original examples

Beat is GPL-licensed, the others are MIT or closed. This file copies from none of them.

- Behaviour is **described in our own words**. No code, regular expression, help text, comment text or sample script
  from any source appears here. Patterns are described by what they accept, in plain English.
- Examples are **original single lines**, written for this file, and bare syntax tokens (`#`, `[[`, `{{`, `^`, `!!`).
  No source example is reproduced.
- Nothing from books, courses or paid or private sources was used.
- Names of app authors are not recorded. Apps, projects and companies are named.

### How to read the entries

- **Fountain 1.1** means the literal text as paraphrased in [official.md](official.md). `OFF-n` links go to its open
  cases.
- **Label** on each bullet: *extension* (not in 1.1), *divergence* (1.1 covers it; the app differs), *agrees* (same as
  1.1), or *not determined*.
- A bullet states one claim and links its source. Where one source supports several bullets, it is linked each time.

## Entries

### Revision marks

Fountain 1.1 has no syntax for revisions (official.md lists none; [brief.md](../brief.md) records that the spec names
marking changes only as a future goal). The survey finds **no shared in-text syntax** among the six.

- **Beat** (*extension, out of band*): revisions are tracked as character ranges and saved as a JSON block in a
  comment at the very end of the `.fountain` file, together with other document settings. The comment is a normal
  boneyard (`/* ... */`), opened by an explanatory sentence and the token `BEAT:` and closed with `END_BEAT`. Other
  Fountain apps see it as a comment; the revised text itself stays plain in the file, and text that was marked for
  removal shows as ordinary text in other apps ([file format note][beat-format]). The ranges are saved in two lists, one
  for additions and one for suggested removals ([`BeatRevisions.m`][beat-rev]). The code lists eight revision
  generations, each with a colour and a margin mark made of one or two of `*`, `+`, `@`, `#`; a header comment in the
  same file still says four ([`BeatRevisions.m`][beat-rev]). Related to [OFF-16](official.md#off-16-nested-unclosed-and-inline-boneyard):
  the block is an ordinary boneyard, so a lossless parser keeps it and a writer must write it back unchanged.
- **Highland 2** (*not determined*): Revision Mode tracks changes, lets the writer pick a highlight colour, puts star
  marks in the margin and can print the marks in PDF ([article, 13 June 2019][hl-rev]). Where revision data is stored,
  and whether it reaches Fountain or FDX export, is **not determined from public sources**; the article does not say.
- **Slugline** (*extension, out of band*): "Live Compare" compares the open screenplay against a second file and
  marks the differences in the right margin, in the editor, in preview and in print. The launch post says this was done
  without changing the Fountain file format ([Slugline 2 launch][sl-2]). It is a comparison, not stored revision sets.
  Revision colours and A-pages: not determined from public sources.
- **Afterwriting**: no revision feature was found in the repository (no file or setting named for revisions in the
  trees read; the CLI options list has none, [CLI document][aw-clients]). Not determined beyond that.
- **Fade In** (*not determined for Fountain*): revisions are native. The changelog records revision sets, per-revision
  marks, customizable revision marks (2.0.546, 2016-07-31), and FDX import and export of all revision marks including
  deletions and colours (2.0.519, 2015-04-15) ([changelog][fi-cl]). How revisions are written to Fountain
  export is **not determined from public sources**.
- **screenplay-tools**: none. The model has no revision field ([README][st-readme]), and the FDX writer does not write
  revisions ([Python FDX writer][st-fdxw]).

### Notes conventions

1.1 defines `[[ ... ]]` as a Note (inline or on its own line, with two spaces for a blank line inside). See
[OFF-15](official.md#off-15-nested-and-unclosed-notes) for nesting and unclosed notes and
[OFF-1](official.md#off-1-what-is-an-empty-line) for what counts as an empty line.

- **Beat**:
  - *extension, inside notes:* a note whose text names a colour (`[[teal]]`, `[[color teal]]` or a seven-character hex
    value) at the end of a scene heading, section or synopsis colours that item in the outline, timeline and cards; only
    the last such note on the line counts ([`ContinuousFountainParser.m`][beat-parser], `colorForHeading:`;
    [manual][beat-manual], "Colorizing your scenes").
  - *extension, inside notes:* a note that begins with the word `marker`, optionally followed by a colour name and a
    colon, drops a coloured marker that shows in the outline (`[[marker amber: fix pacing]]`)
    ([`ContinuousFountainParser.m`][beat-parser], `markerForLine:`; [manual][beat-manual], "Markers").
  - *agrees, with a stale line in the docs:* the file format note says notes spanning several lines were not yet
    supported; the note parser at the pin handles notes that span lines and describes its own rules for whitespace in them
    ([`+Notes.m`][beat-notes]). Which blank-line forms it accepts inside a note: not determined by reading.
  - Beat has options to print notes, sections and synopses ([changelog][beat-changelog]).
- **Highland 2**:
  - *agrees:* notes are wrapped in double brackets ([Notes, Synopses and Such][hl-nsa]).
  - *extension, a different element:* "endnotes" are numbered notes written with double parentheses
    (`Salt cures the hide.((Brine was also used.))`) or inserted from the Format menu. "Reference notes" pair a
    label in `[^label]` with a definition elsewhere in the form `[^label]: text`. A `{{ENDNOTES}}` directive moves
    where they print, and they link in the PDF ([what's new in 2.6][hl-26]).
  - *extension:* a named bookmark is written as `{{%m:label}}`, inserted from the Tools menu ([Markers][hl-markers]).
  - Colours, tags or other conventions inside `[[ ]]`: **not determined from public sources**; no page describes any.
- **Slugline**:
  - *agrees:* notes can sit inline in any text outside the title page, or on their own line; they do not print
    ([Notes post][sl-notes]).
  - *extension, inside notes:* a short prefix ending in a colon, such as `[[Check: confirm tides]]`, is a "Note Tag".
    Tags can be given colours, and a range of notes can be converted from one tag to another in a batch (Slugline 2,
    2020-05-05, [launch post][sl-2]).
  - *convention, not syntax:* a post from 2013 describes a `TODO:` prefix inside a note to mark gaps
    ([outlining post][sl-outline]).
  - Notes appear in the Outline Navigator and on the timeline ([notes post][sl-notes], [launch post][sl-2]).
  - Slugline also exports notes to FDX and reads them back ([launch post][sl-2]).
- **Afterwriting**:
  - *agrees:* a `print_notes` setting (default off) chooses whether notes print ([CLI document][aw-clients]).
  - *divergence in method:* when notes are not printed, the text is rewritten **before parsing** so that each `[[` becomes
    a boneyard opener and each `]]` a boneyard closer, with a neighbouring space adjusted. Boneyards nest by counting
    ([`parser.js`][awp-parser]). So a note and a boneyard are treated as one kind of thing there, and a note that holds a
    boneyard marker changes the count. Cross-link: [OFF-15](official.md#off-15-nested-and-unclosed-notes),
    [OFF-16](official.md#off-16-nested-unclosed-and-inline-boneyard).
  - FDX import turns each Final Draft script note into a `[[ ]]` note, placed after the paragraph text, or as its own
    block after a scene heading ([`finaldraft.js`][aw-fdx]).
- **Fade In** (*not determined for the syntax*): notes are native objects (sticky notes, mouseover notes). The changelog
  records an improvement to Fountain note handling (3.0.583, 2018-05-14) without saying what changed, and says that
  hiding the note and synopsis icons leaves them out of export to formats that carry them (2.0.550, 2016-11-08).
  FDX notes are handled natively; fixes are recorded for export (4.0.10, 2021-08-04) and import (4.0.12, 2022-02-23)
  ([changelog][fi-cl]).
- **screenplay-tools**:
  - *agrees, with a divergence:* notes can sit inside a line or span lines. They are lifted out of the text, kept in a
    side list, and restored by the writer. A note that is **unclosed at an empty line is closed there**
    ([`parser.py`][st-parser]; compare [OFF-15](official.md#off-15-nested-and-unclosed-notes)).
  - Boneyards are handled the same way, but an unclosed boneyard is not closed early ([`parser.py`][st-parser];
    [OFF-16](official.md#off-16-nested-unclosed-and-inline-boneyard)).
  - The Python FDX writer drops notes ([`fdx/writer.py`][st-fdxw]).

### Sections and synopses

1.1: `#` lines are Sections (depth by number of `#`), `=` lines are Synopses, both ignored in formatted output; the
spec sets no depth limit. See [OFF-20](official.md#off-20-section-edges),
[OFF-21](official.md#off-21-synopsis-and-section-placement-inside-blocks) and
[OFF-19](official.md#off-19--runs-page-break-synopsis-and-whitespace).

- **Beat** (*agrees; extra uses*): sections and synopses feed the outline, index cards and timeline views, and sections
  nest. They do not print unless the writer asks for options to print them ([manual][beat-manual];
  [changelog][beat-changelog]). A "top-level section" resets the counter of Beat's `{{panel}}` macro
  ([manual][beat-manual]). Any depth limit: not determined by reading.
- **Highland 2** (*agrees*): sections start with one or more `#`, synopses with `=` ([Notes, Synopses and Such][hl-nsa]).
  The Preview window can include synopses in the PDF ([export options][hl-export]). A `%#` token inside a header
  directive inserts the current section title ([headers and footers][hl-hf]). `{{SERIES}}` inside a section
  line numbers chapters automatically ([Series][hl-series]).
- **Slugline** (*agrees*): a section nests under one with fewer `#`. A synopsis is a line starting with `=`, usually
  after a section but allowed anywhere in a scene. Neither prints by default; a document setting, "Include Outline
  Elements", prints them and is saved with the screenplay ([notes post][sl-notes], [outlining post][sl-outline]).
  Where that setting is stored: **not determined from public sources**.
- **Afterwriting**:
  - *agrees:* depth is the count of `#`, with no cap in the parser ([`parser.js`][awp-parser]). Both print only if the
    `print_sections` or `print_synopsis` setting is on, and `number_sections` numbers sections automatically
    ([CLI document][aw-clients]).
  - *divergence:* an empty synopsis (`=` and nothing else) becomes a blank-line separator. A line of `=` followed by
    another `=` is not a synopsis ([`parser.js`][awp-parser]; [OFF-19](official.md#off-19--runs-page-break-synopsis-and-whitespace)).
  - FDX import writes a scene's description from Final Draft as `=` lines after the heading ([`finaldraft.js`][aw-fdx]).
- **Fade In** (*not determined for the syntax*): the changelog records Fountain section import work (1.2.387,
  undated), outline-level handling for Fountain import (1.2.477, 2014-06-11), a Navigator rename from "outline level" to
  "folder level" (1.2.470, 2014-03-03), and copying as Fountain with sections and synopses (2.0.510, 2015-02-09)
  ([changelog][fi-cl]). How `#` sections map to its folders, and the export markup: **not determined from public
  sources**.
- **screenplay-tools**:
  - *extension:* the README says sections go up to six levels, where it calls three the norm
    ([README][st-readme]); the Python code stops counting at seven `#` ([`parser.py`][st-parser]). The 1.1 text sets no
    limit, so the README's "three" is not the spec's rule ([official.md](official.md#sections-and-synopses)). The
    other three language versions: not read.
  - Both are dropped when writing FDX ([`fdx/writer.py`][st-fdxw]).

### Stage-play variants

1.1 has no stage-play rules. The finding is that **none of the six defines stage-play Fountain syntax**.

- **Beat** (*not syntax*): a "Stageplay" stylesheet exists beside the Screenplay and Novel stylesheets. It sets layout
  rules for the same element types (character, dialogue, parenthetical, heading, and so on) and has no rules of its
  own for new elements ([`Stageplay.beatCSS`][beat-stage]). How a document selects it: not determined by reading.
- **Highland 2**, **Slugline**, **Afterwriting**: none found; **not determined from public sources**.
- **Fade In** (*not syntax*): the stage-play templates are native layouts, and its 1.2 release added true multi-column
  formatting for stage, AV and radio scripts (1.2.380, undated). The changelog records updated stage-play templates
  (1.2.424, 2013-03-12; 2.0.543, 2016-05-17), page-break behaviour for column-based stage-play formats and dual-dialogue
  handling for stage-play formats (2.0.541, 2016-04-27) ([changelog][fi-cl]). How a stage-play script is written to
  Fountain: **not determined from public sources**.
- **screenplay-tools**: none ([README][st-readme]).

### Dual dialogue

1.1: a caret `^` after the second speaker's cue; the spec lets any number of spaces precede it and says the caret is the
last character ([OFF-11](official.md#off-11-dual-dialogue-without-a-preceding-dialogue-and-caret-placement),
[OFF-23](official.md#off-23-significant-trailing-whitespace)). Every app below uses the same caret, so this is not an
extension. Differences are in detail.

- **Beat** (*agrees*): a cue ending in `^` is a dual cue, also after `@` forcing; the rule checks the last non-space
  character and wants the name in capitals, ignoring the part in parentheses ([`+ParsingRules.m`][beat-rules],
  [`ContinuousFountainParser.m`][beat-parser]). A Format menu item adds the caret ([manual][beat-manual]).
- **Highland 2** (*agrees*): caret after the second name; the page's own example writes the caret with no space before
  it. The PDF and preview put the two side by side ([Fountain syntax][hl-syntax]).
- **Slugline** (*agrees*): caret after the second cue, with a space; it is indented correctly in the editor, and the
  side-by-side layout shows in preview ([dual dialogue FAQ][sl-dual]). Older releases drew the pair wrongly when the
  first block was longer ([Slugline 1.2][sl-sn12]).
- **Afterwriting** (*agrees; reads backwards*): a trailing `^` marks the cue as the right-hand speaker and then marks
  the **earlier** block as the left-hand one. A `use_dual_dialogue` setting controls whether the pair is drawn in
  columns; with it off, the caret is still dropped from the cue but the blocks are not marked as a pair
  ([`parser.js`][awp-parser], [CLI document][aw-clients]). An early caret with no earlier block is the case in
  [OFF-11](official.md#off-11-dual-dialogue-without-a-preceding-dialogue-and-caret-placement); how this code behaves
  there: not determined by reading.
- **Fade In** (*not determined for the syntax*): Format > Dual Dialogue is a native feature, with changelog fixes to
  its layout (for example 4.1.1, 2024-08-31) ([changelog][fi-cl]). Fountain export markup: **not determined from public
  sources**.
- **screenplay-tools** (*agrees*): a trailing caret on a cue sets `isDualDialogue`; spaces before it are ignored
  ([README][st-readme], [`parser.py`][st-parser]). The Fountain writer emits the caret after the name and before an
  extension, as `NAME ^ (V.O.)` ([`writer.py`][st-writer]); the 1.1 text says the caret is the last character on the
  line, which would put it after the extension (OFF-3, OFF-11).
- **FDX:** dual dialogue is a wrapper element, not a flag ([FDX mapping notes](#fdx-mapping-notes)).

### Scene numbers

1.1: a number wrapped in `#`, appended to a scene heading, made of letters, digits, dashes and periods
([OFF-8](official.md#off-8-scene-number-character-set-and-placement)).

- **Beat** (*agrees; extension in tooling*): `#NUMBER#` after a heading is a manual number; a manual number does not
  change the automatic count of the scenes after it. "Lock Scene Numbers" writes the numbers into the file, and
  "Clear Scene Numbers" removes them ([manual][beat-manual]). The manual's text on the syntax follows the spec's own,
  including that the opening `#` needs a space before it. Whether the code at the pin enforces the same character set:
  not determined by reading.
- **Highland 2** (*agrees*): numbers follow the heading wrapped in `#` ([Notes, Synopses and Such][hl-nsa]).
  Locking: **not determined from public sources**.
- **Slugline** (*extension: locked numbers*): numbers are `#...#` at the end of a heading. A **locked** number is written
  with a double hash before the number and a single one after (`EXT. QUARRY - DUSK ##12A#`), and the bulk "Remove
  Scene Numbers" command leaves locked numbers alone. Adding numbers continues an existing sequence and adds letter
  suffixes where needed ([scene numbers FAQ][sl-sn]). By the 1.1 literal text, `##12A#` does not match the number rule
  (a number may not start with `#`); see OFF-8.
- **Afterwriting** (*divergence*): the number is anything between a pair of `#` in the heading, with no check on the
  characters, and it is taken from anywhere in the text, not only the end. When absent, the parser numbers headings
  itself ([`parser.js`][awp-parser]). Output can print numbers on the left, right or both (`scenes_numbers`)
  ([CLI document][aw-clients]).
- **Fade In** (*not determined for the syntax*): scene numbering is native, with scene-number formatting options (2.0.530,
  2015-09-02). FDX export writes even unlocked numbers (2.0.550, 2016-11-08) ([changelog][fi-cl]).
- **screenplay-tools** (*agrees*): letters, digits, dashes and periods, at the end of the heading, after optional
  space ([`parser.py`][st-parser]). The writer puts the number back as `#n#` ([`writer.py`][st-writer]).

### Title-page keys

1.1: `Key: value` lines at the top of the file, any key allowed, with a few suggested ones; unsupported keys are ignored
in formatted output ([OFF-17](official.md#off-17-title-page-edges)).

- **Beat** (*agrees; open set*): recognises keys for title, credit, author or authors, source, contact (also the
  plural and "contact info"), and draft date, ignoring case; any other key becomes an "unknown" title-page key and is
  kept. Values may take several lines. Macros (see "Other extensions") are resolved in the title page
  ([`+TitlePage.m`][beat-title], [`+ParsingRules.m`][beat-rules]). The Title Page Editor is a form for these keys.
- **Highland 2** (*extension*): the 2.6 notes add keys that place an image on the title page: `TC:`, `CC:` and `BC:`
  followed by Markdown image syntax, for top, middle and bottom centre. Its own example uses `Date:` where the spec
  uses `Draft date:` ([what's new in 2.6][hl-26]). `{{TIMESTAMP}}` can appear in title pages ([Timestamp][hl-ts]).
  A form, Format > Title Page, inserts the block ([Title Page Builder][hl-tpb]).
- **Slugline**: a Title Page helper is mentioned in the launch post; the key list is **not determined from public
  sources** ([launch post][sl-2]).
- **Afterwriting** (*divergence: fixed key list*): the parser starts a title page only for a line containing one of:
  title, credit, author, authors, source, notes, draft date, date, contact, copyright (any case). The test is not
  anchored to the start of the line or the file, so by reading, a body line before the first scene heading that
  contains one of those words before a colon can be taken as a title-page key ([`parser.js`][awp-parser]; by reading,
  not run). Any other key (such as `Revised:`) is not a title-page key there. Title page ends at the first empty line.
  A related case: [OFF-17](official.md#off-17-title-page-edges).
- **Fade In** (*possible divergence*): the changelog says that import of unrecognised title-page key and value pairs now
  "digests" them without complaint (3.0.623, 2019-08-31); the wording suggests such keys are not kept, but this is **not
  determined from public sources**. An earlier fix covered title-page metadata that was not imported at all (2.0.520,
  2015-04-26) ([changelog][fi-cl]).
- **screenplay-tools** (*agrees; open set*): any `Key: value` made of letters, digits and spaces is a title entry;
  a value is empty on the key line and continues on lines indented three or more spaces or with a tab
  ([`parser.py`][st-parser]). Matches 1.1's literal indent rule (OFF-17).

### Other extensions

Each line is an extension unless marked otherwise.

- **Shots, `!!`** (Beat): a line starting with two exclamation marks is a "Shot" element. The manual says this is
  Beat's own feature, not part of Fountain, and that other apps may print the marks ([manual][beat-manual],
  [`+ParsingRules.m`][beat-rules]). Under 1.1, `!!text` is forced Action; whether one or both marks are dropped
  from the text is open ([OFF-22](official.md#off-22-forcing-sigils-leading-whitespace-stacking-and-lines-starting-with-)). FDX has a "Shot"
  paragraph type that other converters read as a scene heading ([FDX mapping notes](#fdx-mapping-notes)).
  Afterwriting uses the word "shot" for a different thing: an uppercase line on its own, followed by an empty line,
  which 1.1 would leave to [OFF-5](official.md#off-5-a-cue-at-the-end-of-the-file-or-followed-by-an-empty-line)
  ([`parser.js`][awp-parser]).
- **Macros, `{{ ... }}`** (Beat): text macros (`{{ship = Marlin}}` then `{{ship}}`), a `{{date}}` macro with a format
  string, and numbered "serial" macros that count up (`{{serial scene = 1}}`), plus a built-in panel counter for comic
  scripts ([manual][beat-manual]). The macros can sit in the title page.
- **Directives, `{{ ... }}`** (Highland 2): header and footer text (`{{HEADER: Draft 3}}`), page number start
  (`{{PAGENUMBER: 3}}`), file include (`{{INCLUDE: ...}}`), watermark, endnote position, series counter, timestamp and
  marker ([headers and footers][hl-hf], [Notes, Synopses and Such][hl-nsa], [what's new in 2.6][hl-26],
  [Series][hl-series], [Timestamp][hl-ts], [Markers][hl-markers]). **Beat and Highland use the same braces for
  different things.** A review posted to the Fountain developers' group in August 2016 questioned how consistent
  Highland's proposed braces syntax was ([thread][hl-dev]; third-party opinion, kept for context).
- **Text styling** (Highland 2): `+highlight+`, backticks for code, a strikethrough form (shown struck through in the
  page source, with the typed token not visible, so **not determined**), and right-aligned text with a trailing `>`
  ([text formatting][hl-fmt]). The 2016 review discusses highlight, term lists and right alignment too
  ([thread][hl-dev]).
- **Term lists, `Key :: Value`** (Highland 2) ([Notes, Synopses and Such][hl-nsa]).
- **Checklists** (Highland 2.6): `[ ]` and `[x]` at a line start ([what's new in 2.6][hl-26]).
- **Tags, `#tag`** (screenplay-tools): an optional switch, off by default, attaches data items to a line. A tag is a `#`
  followed by non-space characters, at the **end** of a line (`Hello there. #sfx:rain`). A `#` as the first character
  of a line is still a Section ([README][st-readme], [`parser.py`][st-parser]). It shares the `#` character with
  `#n#` scene numbers ([OFF-8](official.md#off-8-scene-number-character-set-and-placement)); by reading the pattern, a
  well-formed `#n#` is not taken as a tag. When on, the text after the first tag on a line is cut off. Beat's FDX
  export has a separate, native tagging system for breakdown categories ([`BeatFDXExport.m`][beat-fdx]).
- **Full-width sigils** (Beat): full-width forms of `!`, `@`, `~`, `.` and `#` are accepted as forcing marks
  ([`+ParsingRules.m`][beat-rules]).
- **Extra heading prefix** (Beat): `e/i` and `e./i` as well as `i/e` and `i./e`. The manual lists `EST` as a prefix, as
  the spec does; the rule table at the pin has no `est` entry (not determined by reading what the app does).
- **Lowercase extension text** (Slugline): text inside the parentheses after a cue does not have to be uppercase
  (`HARRY (V.O.)` and others typed by hand) ([CONT'D FAQ][sl-contd]; [OFF-3](official.md#off-3-character-cue-with-extensions-such-as-contd-or-vo)).
- **`(CONT'D)`**: Slugline adds it automatically when drawing and it **does not appear in the file**; a setting turns it
  off, and a hand-typed one stays text ([CONT'D FAQ][sl-contd]). Afterwriting's master branch (not the 0.1.1 tag)
  adds ` (CONT'D)` to a repeated speaker when a `dialogue_contd` setting is on ([`parser.js` at master][awp-master]).
  Fade In keeps explicitly typed ones on import (1.2.471, 2014-03-20) ([changelog][fi-cl]). screenplay-tools **removes**
  ones it reads and the Fountain writer **adds** them for a repeated speaker ([`parser.py`][st-parser],
  [`writer.py`][st-writer]). That matters for byte-exact round trips.
- **Endnotes and references** (Highland 2.6): see "Notes conventions".
- **Snippets, `$name`** (Afterwriting): a CLI config can define replacements that are substituted in the text before
  parsing. This lives in the config, not the script ([CLI document][aw-clients], [`preprocessor.js`][aw-pre]).
- **Boneyard with nesting** (Afterwriting): see "Notes conventions".
- **Editor and tooling items, not syntax:** Beat's colour-coded scenes and gender statistics, Slugline's Outline
  Navigator and timeline, Highland's Navigator, Fade In's Fountain editor view (4.0.0, 2021-03-11) ([changelog][fi-cl]).

### Divergences on standard input, per app

These are places where an app treats input that 1.1 covers in a way the literal text rules out or does not state.

- **Beat** ([`+ParsingRules.m`][beat-rules]):
  - The empty-line rule lists an empty string and a single space as matches; what its allowed-whitespace count means is
    not determined by reading ([OFF-1](official.md#off-1-what-is-an-empty-line)).
  - Scene-heading prefixes are accepted in lowercase and are made uppercase on save, except forced headings
    ([file format note][beat-format]).
  - A forced heading excludes only a second period after the first
    ([OFF-7](official.md#off-7-forced-heading--and-the-meaning-of-alphanumeric)).
  - A page break and a lyric accept leading whitespace
    ([OFF-13](official.md#off-13-lyrics-next-to-cues-and-dialogue),
    [OFF-19](official.md#off-19--runs-page-break-synopsis-and-whitespace)).
  - A character cue may have leading whitespace and needs at least two characters.
- **Highland 2**: the help pages show no conflict with the standard elements; edge cases are **not determined from public
  sources**. Fade In's changelog says its Fountain import treats non-printing lines as blank in some cases, in the way
  Highland does (4.0.12, 2022-02-23); which lines that means is not stated ([changelog][fi-cl]).
- **Slugline**: text after a cue may be lowercase, the `CONT'D` form is not stored, and locked scene numbers use a
  double `#`. Its basics page says an all-uppercase line becomes a cue and that Dialogue cannot exist without a cue
  ([basics][sl-basics]); other edges: **not determined from public sources**.
- **Afterwriting** ([`parser.js`][awp-parser], by reading):
  - A heading needs no empty line before it, and may start with up to three `*` and an `_` (emphasis marks)
    ([OFF-6](official.md#off-6-scene-heading-versus-action-and-the-blank-line-and-prefix-edges)).
  - A transition is `FADE OUT.`, `FADE TO BLACK.`, `CUT TO BLACK.` or any line ending in an uppercase `TO:` after a
    space, with or without empty lines around it; a line starting with `>` is also one
    ([OFF-9](official.md#off-9-transitions-that-do-not-end-in-to-and-the-blank-line-edges)).
  - A line of exactly two spaces is a hard line break, not an empty line (OFF-1).
  - Lyrics (`~`) are turned into italics, not kept as an element (OFF-13).
  - A cue is any line that equals its own uppercase form, after an empty line and with a non-empty line below it. Digits
    and symbols therefore qualify, and so does a line with no letters
    ([OFF-4](official.md#off-4-non-ascii-uppercase-and-caseless-names)). A lowercase extension makes the line fail the
    test (OFF-3).
  - An uppercase line with no text under it is a "shot" (OFF-5).
  - Scene numbers and title-page keys: see above.
- **Fade In**: it supported the 1.0 rule that two trailing spaces force a line break (1.2.380, undated), and a later
  clean-up of leftover deprecated import behaviour is recorded (2.0.536, 2015-12-29). The rule is deprecated in 1.1
  ([official.md](official.md#the-11-changelog-in-brief); [changelog][fi-cl]). A fix in 1.2.473 (2014-04-12) toned down
  how readily import took a line as a transition, which suggests it once accepted more transitions than 1.1 does
  (OFF-9). Current behaviour: **not determined from public sources**.
- **screenplay-tools** ([`parser.py`][st-parser], by reading):
  - `FADE IN:` is read as a scene heading.
  - A line of uppercase text ending in `TO:` is a transition only if an empty line precedes it and an empty line, or the
    end of the file, follows (the parser waits one line to decide).
  - A cue starts with a capital letter A to Z, has no lowercase letter outside a trailing parenthesis, follows an empty
    line and has a non-empty line below it; an extension may hold lowercase text (OFF-3).
  - A `>` line that does not end in `<` is a transition
    ([OFF-10](official.md#off-10-centered-text-versus-forced-transition)); the forcing sigils are checked in a fixed order
    ([OFF-22](official.md#off-22-forcing-sigils-leading-whitespace-stacking-and-lines-starting-with-)).
  - Sections go to seven levels, and notes close at an empty line (both above).
  - The project is the planned conformance oracle ([brief.md](../brief.md)), so these show up as diffs in F4.

## Ranking

A ranking for `SPEC.md` (#4) to weigh. It is **not a decision**. Columns:

- **Apps**: how many of the six record the extension in the sources above (out of six).
- **Conflict with 1.1**: whether the literal text says something different, with the `OFF-n` case.
- **Changes standard input?**: whether turning the extension on changes how input that is valid Fountain 1.1 parses.
  "No" means the extension only reads text that 1.1 already treats as opaque (note text, action text or an unknown
  title-page key).

| # | Candidate | Apps | Conflict with 1.1 | Changes standard input? | Notes |
|---|---|---|---|---|---|
| 1 | Conventions inside notes: colour names, `marker ...`, `Tag:` prefixes | 2 (Beat, Slugline) | None. Inside `[[ ]]`, so only [OFF-15](official.md#off-15-nested-and-unclosed-notes) applies. | No | Parse as a typed view over note text; keep the raw text. |
| 2 | Open title-page keys with typed access for common ones (author, authors, notes, copyright, date, image keys) | 2 keep any key (Beat, screenplay-tools); 2 define extra keys (Highland: image keys; Afterwriting: notes, copyright, date) | None for open keys. A fixed key list as in Afterwriting would conflict ([OFF-17](official.md#off-17-title-page-edges)). | No | Fade In's treatment of unknown keys is unclear (see "Title-page keys"). |
| 3 | Sections deeper than three levels | 2 (Afterwriting has no cap; screenplay-tools stops at seven and documents six). Beat: not determined | None: 1.1 sets no limit ([OFF-20](official.md#off-20-section-edges)) | No | The README's "usual three" is not in the spec. |
| 4 | Revision data kept in the file as an ignorable boneyard block (Beat's layout) | 1 (Beat) | None: it is a boneyard ([OFF-16](official.md#off-16-nested-unclosed-and-inline-boneyard)) | No | No shared syntax exists. The need is to preserve it byte for byte. |
| 5 | Typed `(CONT'D)` handling: keep as written, never add or strip in the lossless tree | 4 (Slugline, Afterwriting, Fade In, screenplay-tools) | Related to [OFF-3](official.md#off-3-character-cue-with-extensions-such-as-contd-or-vo) | No, if only exposed as a view | Slugline adds it at render time; Afterwriting's newer parser adds it by option; screenplay-tools strips it on read and re-adds it on write. |
| 6 | Locked scene numbers `##n#` | 1 (Slugline) | Yes: a number may not start with `#` ([OFF-8](official.md#off-8-scene-number-character-set-and-placement)) | Yes: today such a heading ends in text, not a number | Needed by any tool that edits Slugline files. |
| 7 | `!!` Shot | 1 (Beat); an FDX paragraph type | Yes: forced Action in 1.1 ([OFF-22](official.md#off-22-forcing-sigils-leading-whitespace-stacking-and-lines-starting-with-)) | Yes | Could be a flag. |
| 8 | Wider scene-heading prefixes (`e/i`) and lowercase headings | 1 (Beat); Afterwriting also accepts `est` and other forms | Partly: 1.1 lists the prefixes and is case-insensitive ([OFF-6](official.md#off-6-scene-heading-versus-action-and-the-blank-line-and-prefix-edges)) | Yes for new prefixes | Low value. |
| 9 | Brace directives `{{ ... }}` (macros, header, footer, include, series, markers) | 2 (Beat, Highland), with **different meanings** | None: braces are ordinary text in 1.1 | Yes if the text is expanded; no if only preserved | Do not interpret. Preserve as opaque text, possibly tagged by app. |
| 10 | Line tags `#tag` | 1 (screenplay-tools), off by default | None stated. Shares `#` with sections and scene numbers ([OFF-8](official.md#off-8-scene-number-character-set-and-placement), [OFF-20](official.md#off-20-section-edges)); by reading, a well-formed `#n#` is not a tag | Yes: text after the first tag is cut off | Single library. |
| 11 | Highland text styles (`+highlight+`, strikethrough, term lists, right align, checklists) | 1 (Highland) | If the strikethrough token is Markdown's double tilde (not determined), a line starting with it meets Lyrics ([OFF-13](official.md#off-13-lyrics-next-to-cues-and-dialogue), [OFF-22](official.md#off-22-forcing-sigils-leading-whitespace-stacking-and-lines-starting-with-)) | Yes | Single app. |
| 12 | Endnotes `((...))` and `[^label]` references | 1 (Highland 2.6) | None stated; `((` is ordinary text in 1.1 | Yes if parsed | Single app. |
| 13 | Full-width sigils | 1 (Beat) | Yes: such a line is Action in 1.1 | Yes | Niche (CJK input), flag at most. |
| 14 | Extra transitions as in Afterwriting (`FADE OUT.`, any `... TO:`) | 1 (Afterwriting); the reference parser also accepts more ([reference-parser.md](reference-parser.md)) | Yes ([OFF-9](official.md#off-9-transitions-that-do-not-end-in-to-and-the-blank-line-edges)) | Yes | This is a 1.1 edge case, not an extension. |
| 15 | Include files, snippets, watermark | Highland, Afterwriting | None | Yes (changes text) | Tool behaviour; out of scope for a parser. |

Observations, not rulings:

- Items 1, 2, 3, 4 and 5 add no parse changes to valid 1.1 input. They are the cheapest to carry and least likely to
  need a feature flag.
- Items 6, 7, 8, 10, 11, 13 and 14 each change how some valid 1.1 text parses. A flag would stop surprises.
- Item 9 changes parsing only if the braces are expanded; preserved as opaque text it changes nothing.
- No candidate is recorded by more than two of the six apps except `CONT'D` handling (four). The title-page key
  extensions are spread across four apps but differ from each other.
- Revision marks, the one extension the brief names, have no common syntax to support. Every app keeps them out of
  band or in a private form.

## FDX mapping notes

For a later crate. **Notes, not a spec.** No official public FDX schema or documentation was found (searches on 2026-11-05
returned only third-party descriptions). The notes below come from the open converters read at their pins: Beat's
[exporter][beat-fdx], Afterwriting's [FDX-to-Fountain converter][aw-fdx], and screenplay-tools' FDX code
([Python parser][st-fdxp], [Python writer][st-fdxw], [C# parser][st-cs]), plus the Fade In and Highland pages. Names
below are taken from those readers and writers; whether Final Draft itself writes or accepts each is **not determined**.

### Element mapping

| Fountain | FDX (as the converters write or read it) | Notes |
|---|---|---|
| Scene Heading | Paragraph `Type` "Scene Heading"; scene number in a `Number` attribute | Beat writes `Number` ([exporter][beat-fdx]). The Python FDX parser does not read it ([parser][st-fdxp]). Fade In exports even unlocked numbers (2.0.550) ([changelog][fi-cl]). |
| Forced heading | No equivalent | The leading `.` is a Fountain device. Afterwriting adds it back when the text has no recognised prefix ([`finaldraft.js`][aw-fdx]). |
| Action | "Action"; screenplay-tools also reads "General" as Action | |
| Character | "Character"; the extension, for example `(V.O.)`, is part of the text | screenplay-tools splits the last parenthetical off as the extension ([parser][st-fdxp]). |
| Parenthetical | "Parenthetical" | screenplay-tools strips and re-adds the parentheses. |
| Dialogue | "Dialogue" | |
| Transition | "Transition" | Afterwriting prefixes `> ` so it survives as a transition ([`finaldraft.js`][aw-fdx]). |
| Centered text | Paragraph with an alignment attribute | Beat writes the value `Centered` as an attribute on an Action paragraph; Afterwriting reads the value `Center`. Which one Final Draft writes: **not determined**. |
| Lyrics | Beat writes a "Lyrics" type | screenplay-tools' Python FDX writer has no case for it and writes Action ([writer][st-fdxw]). |
| Shot (`!!` in Beat) | "Shot" | screenplay-tools reads "Shot" as a scene heading. |
| Page break `===` | A paragraph attribute `StartsNewPage` | Afterwriting reads it and writes a `====` line ([`finaldraft.js`][aw-fdx]). Beat and screenplay-tools write no paragraph for a page break. |
| Section | Beat writes an "Outline" type with the depth appended to the name | screenplay-tools drops it. Not determined whether Final Draft reads it. |
| Synopsis | A scene-properties element inside the heading paragraph, with a paragraph of text | Afterwriting reads it into `=` lines ([`finaldraft.js`][aw-fdx]). Beat writes the element for scene colour only, and writes no synopsis. How Fade In maps synopses on FDX export: **not determined from public sources**. |
| Note `[[ ]]` | A script-note element with paragraphs of text | Afterwriting reads notes inside the paragraph ([`finaldraft.js`][aw-fdx]). Beat writes them as a separate collection with a character-offset range and a colour ([exporter][beat-fdx]). Placement differs; which is correct: **not determined**. |
| Boneyard `/* */` | No equivalent | Dropped by every converter read. |
| Dual dialogue `^` | A paragraph that holds a dual-dialogue element, which holds two groups of paragraphs | Beat opens it at the first cue and closes it after the second block ([exporter][beat-fdx]). Afterwriting reads the inner paragraphs and appends `^` to the second cue ([`finaldraft.js`][aw-fdx]). screenplay-tools' Python parser does not read it: it looks only at top-level paragraphs and their own text ([parser][st-fdxp]). |
| Emphasis `*`, `**`, `_` | A `Style` attribute on each text run, values joined with `+` | Beat writes styles only on Action and Dialogue paragraphs, and its exporter says the reason is that Final Draft is fussy about styles on other types. screenplay-tools flattens text runs to plain text. |
| Title page | A title-page content block of paragraphs | Afterwriting maps the first three non-empty groups to Title, Credit and Author. Beat writes title, credit, author, source, draft date and contact. Other keys: no equivalent. |

### Things with no equivalent either way

- **Fountain → FDX, lost or dropped:** boneyard; forced-element marks (`.`, `!`, `@`, `~`, `>`); key/value title-page
  keys other than a few; the exact blank-line layout; `{{ }}` and `((...))` extensions.
- **FDX → Fountain, lost or dropped:** paragraph and page layout; headers and footers (Fade In works to import and export
  them, 4.0.12); revision sets, colours and A-pages; the breakdown tagging system (Beat writes categories, definitions
  and per-run tag numbers; [exporter][beat-fdx]); element styles other than bold, italic and underline; `MORE` and `CONT'D`
  paragraphs (Beat writes a "More" type).
- **Revisions:** Beat writes a list of revision entries (ID, colour, mark, name, page colour) and tags text runs with a
  revision ID and colour ([exporter][beat-fdx]). Fade In records FDX revision marks including deletions as imported and
  exported (2.0.519) ([changelog][fi-cl]). Fountain has no place for them, so a round trip through Fountain loses them
  unless the side block described under "Revision marks" is carried along.
- **Round-trip risks seen in the code:** text-run splitting, curly quotes (Afterwriting normalises them), upper-casing of
  character, heading and transition text (Afterwriting upper-cases on import), and forced markers added on import.
- **Version attribute:** screenplay-tools' Python writer writes a root with a `Version` of 5 and `Template` of "No"
  ([writer][st-fdxw]). Which versions a later crate should write and accept: **not determined**.

## Summary for #4

| Topic | What the survey found | Where |
|---|---|---|
| Revision marks | No shared syntax. Beat: JSON in a trailing boneyard. Slugline: file comparison. Others: native or not determined | Revision marks |
| Notes conventions | Beat (colour, marker) and Slugline (tag prefixes) put structure inside `[[ ]]`; Highland adds endnotes `((...))` and `[^label]`; others none | Notes conventions |
| Sections and synopses | All agree on `#` and `=`. Depth beyond three appears in screenplay-tools; Afterwriting has no cap | Sections and synopses |
| Stage plays | No syntax in any app. Beat has a stylesheet, Fade In has native templates | Stage-play variants |
| Dual dialogue | All use the caret. Differences are layout and where the caret sits when written | Dual dialogue |
| Scene numbers | Locked `##n#` is Slugline only; Afterwriting accepts any characters | Scene numbers |
| Title-page keys | Open set in most; fixed list in Afterwriting; image keys in Highland; Fade In may drop unknown keys | Title-page keys |
| Other | `{{ }}` has two incompatible uses; `!!` Shot; tags `#tag`; full-width sigils | Other extensions |

Points for `SPEC.md`:

- Carry the Beat revision block as an ordinary boneyard; decide whether the typed view exposes it.
- The writer must not add or strip `(CONT'D)` in the lossless tree ([OFF-3](official.md#off-3-character-cue-with-extensions-such-as-contd-or-vo)).
- `OFF-8` (scene-number characters and `##`), `OFF-15` (unclosed notes), `OFF-17` (title-page keys and indents) and
  `OFF-22` (stacked sigils, `!!`) are the cases this survey touches most.
- screenplay-tools is the planned oracle for F4 ([brief.md](../brief.md)). Its divergences listed above will appear as
  diffs: `FADE IN:` as a heading, notes closed at an empty line, seven-level sections, `CONT'D` stripped on read and added
  on write, tab indentation on write, and an FDX layer that drops notes, sections, synopses, lyrics and dual dialogue.

### Not determined from public sources

- Highland: where Revision Mode data is stored; whether scene numbers can be locked; the title-page key list; any
  stage-play support; Highland Pro.
- Slugline: where the "Include Outline Elements" setting is stored; the title-page key list; revision colours.
- Fade In: the exact Fountain export markup for notes, synopses, sections, dual dialogue, scene numbers and revisions;
  current behaviour for items last touched in 2013 to 2016; the current changelog (not reachable).
- Beat: how a document selects the stage-play style; whether the code enforces the spec's scene-number characters;
  whether `EST` is accepted.
- FDX: what Final Draft itself writes for alignment, dual-dialogue and scene-property markup; no official schema found.

[beat-repo]: https://github.com/lmparppei/Beat
[beat-license]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/LICENSE.md
[beat-manual]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Manual/beat_manual.html
[beat-format]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Developer%20Documentation/Beat%20File%20Format%20Specification.md
[beat-changelog]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/CHANGELOG.md
[beat-rules]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatParsing/BeatParsing/Parsing/Extensions/ContinuousFountainParser%2BParsingRules.m
[beat-parser]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatParsing/BeatParsing/Parsing/ContinuousFountainParser.m
[beat-title]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatParsing/BeatParsing/Parsing/Extensions/ContinuousFountainParser%2BTitlePage.m
[beat-notes]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatParsing/BeatParsing/Parsing/Extensions/ContinuousFountainParser%2BNotes.m
[beat-rev]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatCore/BeatCore/Screenplay%20data/Revisions/BeatRevisions.m
[beat-fdx]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatFileExport/BeatFileExport/Export/Modules/Final%20Draft/BeatFDXExport.m
[beat-stage]: https://github.com/lmparppei/Beat/blob/ab2382184ca41e50440ff695373b17b4e57c915f/Frameworks/BeatCore/BeatCore/Styles/Style%20definitions/Stageplay.beatCSS
[hl-syntax]: https://highland-kb.quoteunquoteapps.com/kb/fountain-syntax
[hl-nsa]: https://highland-kb.quoteunquoteapps.com/kb/fountain-syntax/notes-synopses-and-such
[hl-fmt]: https://highland-kb.quoteunquoteapps.com/kb/fountain-syntax/formatting
[hl-hf]: https://highland-kb.quoteunquoteapps.com/kb/fountain-syntax/headers-and-footers
[hl-markers]: https://highland-kb.quoteunquoteapps.com/kb/editor/markers
[hl-series]: https://highland-kb.quoteunquoteapps.com/kb/editor/series-564de914a9fda318
[hl-ts]: https://highland-kb.quoteunquoteapps.com/kb/editor/timestamp-d27be931d2e5db36
[hl-tpb]: https://highland-kb.quoteunquoteapps.com/kb/tools/title-page-builder
[hl-export]: https://highland-kb.quoteunquoteapps.com/kb/preview/export-options
[hl-26]: https://quoteunquoteapps.com/highland-2/whats-new/whats-new-2-6.php
[hl-rev]: https://nofilmschool.com/highland-2-revision-mode-update
[hl-dev]: https://groups.google.com/g/fountain-dev/c/DpUIW2AlLjQ
[sl-dual]: https://www.slugline.co/faqs/dualdialogue
[sl-fdx]: https://www.slugline.co/faqs/fdx
[sl-contd]: https://www.slugline.co/faqs/contd
[sl-2]: https://www.slugline.co/blog/slugline2
[sl-sn]: https://www.slugline.co/faqs/scenenumbers
[sl-sn12]: https://www.slugline.co/blog/tag/Scene+Numbers
[sl-outline]: https://www.slugline.co/blog/outlining
[sl-notes]: https://www.slugline.co/blog/tag/Notes
[sl-basics]: https://www.slugline.co/basics
[aw-labs]: https://github.com/ifrost/afterwriting-labs/tree/f1912c08d0905bac42e1805830a1e72f98b88c2f
[aw-clients]: https://github.com/ifrost/afterwriting-labs/blob/f1912c08d0905bac42e1805830a1e72f98b88c2f/docs/clients.md
[aw-fdx]: https://github.com/ifrost/afterwriting-labs/blob/f1912c08d0905bac42e1805830a1e72f98b88c2f/js/utils/converters/finaldraft.js
[aw-pre]: https://github.com/ifrost/afterwriting-labs/blob/f1912c08d0905bac42e1805830a1e72f98b88c2f/js/utils/fountain/preprocessor.js
[awp-repo]: https://github.com/afterwriting/aw-parser/tree/cf3c2c246e1d90992c69e3c11d237205e611543a
[awp-parser]: https://github.com/afterwriting/aw-parser/blob/cf3c2c246e1d90992c69e3c11d237205e611543a/parser.js
[awp-master]: https://github.com/afterwriting/aw-parser/blob/090e911c49599e0dbe41229e4f6a2c0f9639df8c/parser.js
[fi-cl]: https://www.fadeinpro.com/page.pl?content=changelog_old
[fi-fd]: https://www.fadeinpro.com/kb/content/1/44/en/can-fade-in-open-final-draft-screenplays.html
[fi-punct]: https://www.fadeinpro.com/kb/content/1/142/en/missing-punctuation-after-text-or-fountain-import.html
[fountain-apps]: https://fountain.io/apps/
[st-repo]: https://github.com/wildwinter/screenplay-tools/tree/e3659406de7ab293b17125e065ed8b4f9af74203
[st-readme]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/README.md
[st-parser]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/python/src/screenplay_tools/fountain/parser.py
[st-writer]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/python/src/screenplay_tools/fountain/writer.py
[st-fdxp]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/python/src/screenplay_tools/fdx/parser.py
[st-fdxw]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/python/src/screenplay_tools/fdx/writer.py
[st-cs]: https://github.com/wildwinter/screenplay-tools/blob/e3659406de7ab293b17125e065ed8b4f9af74203/csharp/ScreenplayTools/FDX/Parser.cs
