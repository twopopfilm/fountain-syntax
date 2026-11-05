# The reference parser and the OFF cases: a study

Input to `SPEC.md` (issue #4), from issue #26 (F1). [official.md](official.md) lists 24 cases the official syntax
leaves open (`OFF-1` to `OFF-24`) and marks a few as settled by the literal text of the spec. This file records how the
Objective-C reference implementation treats each case's example. It makes **no rulings**: one implementation's
behaviour is **evidence, not a ruling**, and `SPEC.md` (#4) rules on each case.

## Source

| Item | Value |
|---|---|
| Repository | `nyousefi/Fountain` on GitHub, branch `master`: <https://github.com/nyousefi/Fountain> |
| Pinned commit | `c09b65e9d4bf6cb1c672501f793b9b3f2becd452` (committed 2015-03-31) |
| Licence | MIT (the repository's `LICENSE` file) |
| Access date | **2026-11-04** |
| Files read | `Fountain/FastFountainParser.m` and `.h`, `Fountain/FountainParser.m` and `.h`, `Fountain/FountainRegexes.m` and `.h`, `Fountain/FNScript.m` and `.h`, `Fountain/FNElement.m`, `Fountain/FountainWriter.m` and `Fountain/FNHTMLScript.m` (read for what happens to the parsed text afterwards), the repository's `README.markdown`, and the small fixtures and tests in `FountainTests/` (cited by file name only) |
| Files named, not used | `Big Fish.fountain` and `Brick And Steel.txt` in `FountainTests/` are third-party scripts. They were not read for this study. |

### Method

- **A static reading.** The code was read and each case's example was traced by hand through it. Nothing was built,
  run or executed, and no test was run. Running the reference parser as a test oracle is out of scope for this study and is not done here.
- Where a result could not be settled by reading, the entry says **not determined by reading** rather than guessing.
- The legacy parser (below) is a chain of text substitutions, which is much harder to trace by hand than the default
  line-by-line parser. Its notes are marked as lower confidence.

### Which parser is the default

The repository holds two parsers for the body and the title page:

- **The default parser** is `FastFountainParser`, a line-by-line parser with one pass over the lines. `FNScript`
  uses it for every plain load method. The repository's `README.markdown` describes it as the default parser and says it replaced
  the older parser because it is faster and relies less on regular expressions. The test classes in `FountainTests/`
  load their fixtures through it, directly or through `FNScript`. This file reports **this parser** unless an entry
  says otherwise.
- **The legacy parser** is `FountainParser`, which turns the whole text into tagged markup with a chain of regular
  expression substitutions (defined in `FountainRegexes.m`) and then splits the tags into elements. `FNScript` uses it
  only when the caller asks for the regex option on the older, longer load methods. The `README.markdown` describes it as
  kept for legacy use and says its regular expressions do not fully match the tests. It has no
  rule for the 1.1 sigils `!`, `@` and `~`.

Where the two differ on a case, both are recorded, with the default parser first. The text styling (bold, italic,
underline) is not done by either parser; the `README.markdown` says markup is passed along to whatever displays the
script. The sample HTML renderer (`FNHTMLScript.m`) does it, and it also strips the sigils `@`, `~`, `!` and `.` from
the stored text, so it matters for OFF-14, OFF-15 and OFF-22.

### Own words, original examples

The code is MIT-licensed, but this file copies none of it.

- Behaviour is **described in our own words**. No code, regular expression, pattern text or test-file content from the
  reference repository appears here. Patterns are described by what they accept, in plain English. Constants and
  functions are named only so a reader can find them.
- Tests and fixtures are cited **by file name only**. Where a test shows the author's intent, this file says in its
  own words what the test expects.
- Examples are the **original ones** in [official.md](official.md), referred to by case number. Only bare syntax
  tokens and single lines of those examples are quoted.

### How to read the entries

- Where a case lists several lines, each is traced as if it began its own block: a blank line before it, and a line
  of text after it wherever a rule looks at the next line. The reason is that most rules look at the neighbours, and a
  stack of lines would hide the rule under test. When adjacency matters, the entry traces the lines as given and says
  so.
- Both parsers work on lines. The default parser keeps a count of blank lines seen since the last element and a flag
  for being inside a Dialogue block. "Blank" means a line with no characters (see OFF-1).
- **Verdict**, always against the spec's **literal text** as paraphrased in official.md:
  - **agrees**: the parser does what the literal text says for the example.
  - **contradicts**: the parser does something the literal text rules out, or fails to do something it requires.
  - **does not cover**: the literal text does not decide the example (so there is nothing to agree or contradict), or
    the parser has no rule for it. The entry says which.
  - The reading letter (A, B, C...) is the one in official.md that the parser's behaviour matches. Where the parser
    matches no reading or a mix, the entry says so. Where an example has several lines with different results, the
    verdict is for the line the case is about, and the entry names the exceptions.

## Entries

### OFF-1: What is an "empty" line?

- **Default parser.** After line endings are normalised, a line is blank only if it holds no characters at all. A
  line that holds only whitespace is content:
  - Exactly two whitespace characters (two spaces, or two tabs) **inside a Dialogue block** keep the Dialogue going.
    The line is appended to the Dialogue text, so `One.` and `Two.` stay in one Dialogue element.
  - Two or more whitespace characters **outside** a Dialogue block make an Action element that holds the whitespace.
    Three or more inside a Dialogue block do the same, and the Dialogue then resumes as a new Dialogue element.
  - One space or one tab is not blank either. Inside a Dialogue block it joins the Dialogue as a line. Elsewhere it is
    treated like any other line of text (it can even be read as a Character cue with blank text, if it has a blank
    line before it and a line after it).
  - Blank lines inside a Boneyard do not count. The title page ends at the first zero-length line only; a line of
    spaces does not end it.
  - The Notes variant of the two-space rule is not implemented (see OFF-15).
- **Legacy parser (lower confidence).** Blocks end at two consecutive line feeds, so any whitespace-only line stays
  inside a block. There is no special two-space rule.
- **Verdict: does not cover** (the spec does not define "empty"). The default parser matches **Reading A**, plus a
  dialogue-only exception for exactly two whitespace characters. That is narrower than Readings B and C (not three
  spaces, and not any whitespace line).
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the blank-line, two-space and whitespace-only-line steps,
  and the title-page split). `Dialogue.fountain` with `DialogueTests.m` (a Dialogue with a two-space line in the middle
  is expected to stay one Dialogue). `FountainParser.m`, `parseBodyOfString:`.

### OFF-2: Line endings, trailing newline, encoding

- **Default parser.** The first step turns carriage-return and line-feed pairs, lone carriage returns and lone line
  feeds all into one
  line feed (**Reading A**). It then adds two
  line feeds at the end, so a missing final newline changes nothing and an existing one adds no effect (**Reading C**
  has no visible effect here). The text is split into lines using the platform's newline set. By the platform's
  documentation that set also covers a few Unicode line and paragraph separators, so those would also split lines
  (not determined by reading).
- A file is read as UTF-8. If that read fails, the parser logs a message and returns an object with no elements, so an
  unreadable file gives an empty result and no error reaches the caller. A byte-order mark is left to the platform's
  UTF-8 reader (not determined by reading).
- **Legacy parser (lower confidence).** Its first substitution also turns all three line endings into line feeds.
  But the step that finds and removes the title page runs first and looks for two line feeds in a row, so for a
  CRLF-only file the title page appears not to be found and its lines would become body text.
- **Verdict: does not cover** (the spec is silent). The default parser matches **Reading A**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` and `initWithFile:`. `FountainParser.m`, `bodyOfString:` and
  `parseBodyOfString:`. `FountainRegexes.m`, the line-break pattern constant.

### OFF-3: Character cue with extensions such as `(cont'd)` or `(V.O.)`

- **Default parser.** A Character cue is a line with a blank line before it and a non-blank line after it, and with no
  lowercase ASCII letter anywhere in it, with one exception: the exact lowercase text `(cont'd)` may end the line.
  So `NELL (V.O.)`, `MARA (cont'd)` and `OTTO (V.O.) (CONT'D)` are cues. `MARA (on the radio, softly)` has lowercase
  letters, so it is not a cue; it becomes Action. A mixed-case form such as `(Cont'd)` fails too. The test is on the
  whole line (**Reading B**, the 1.0 rule), with the one hard-coded lowercase exception.
- **Legacy parser.** The pattern forbids lowercase letters throughout the line, with no exception, so even
  `(cont'd)` does not qualify (the repository's tests run against the default parser, so they do not exercise
  this). It also rejects a cue whose last character is a period or certain punctuation.
- **Verdict: contradicts.** The spec's literal text (official.md, Character section) says extensions may be in either
  case; the default parser accepts one lowercase form only.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Character step). `FountainRegexes.m`, the character cue
  pattern constant. `Dialogue.fountain` with `DialogueTests.m` (cues with a lowercase `(cont'd)` and an `(O.S.)`
  extension are expected to be Characters).

### OFF-4: Non-ASCII uppercase and caseless names

- **Default parser.** The test is only "no lowercase ASCII letter on the line". `JOSÉ`, `ЕЛЕНА`, `ΣΩΚΡΑΤΗΣ`, `李明` and
  `MR. O'BRIEN` are all Character cues (given a blank line before and a line after). Caseless scripts are accepted
  (**Reading C**), and so is any line with no ASCII lowercase at all, with or without a letter. A line of digits, or of
  punctuation only, is a cue too. The parser has no "at least one alphabetical character" test.
- **Legacy parser.** The same idea: its first character may be anything that is not lowercase ASCII or whitespace, and
  later characters may not be lowercase ASCII.
- **Verdict: does not cover** for the example (the spec says nothing about non-ASCII names, and every line in the
  example has a letter). Reading **C**. The missing letter test is a separate challenge to the spec's literal text and
  is listed in the closing section.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Character step). `FountainRegexes.m`, the character cue
  pattern constant. `Dialogue.fountain` with `DialogueTests.m` (a cue with a number and an extension).

### OFF-5: A cue at the end of the file, or followed by an empty line

- **Default parser.** Two line feeds are added after the text, so the line after the last cue is always blank. A cue
  needs a non-blank line after it, so `OTTO` at the end of the file is not a Character: it becomes Action. A cue
  followed by a blank line is Action too. A cue followed by a whitespace-only line is a cue, because only a
  zero-length line counts as blank (see OFF-1).
- **Legacy parser.** The same result. The text is padded with line feeds, and the cue pattern needs a non-line-feed
  character after the line.
- **Verdict: does not cover** (the spec does not say what the end of the file counts as). **Reading B**: an
  uppercase line needs real following text. This is the direction the Error Handling tie-breaker leans.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the padding and the Character look-ahead).
  `FountainParser.m`, `bodyOfString:`.

### OFF-6: Scene heading versus Action, and the blank-line and prefix edges

Traced as given, since adjacency is the point.

- **Default parser.**
  - A heading needs a blank line before it. The start of the file counts as one, and so does the gap left by a
    removed title page (**Reading B2**).
  - The blank line after is enforced after the fact: if the next line is plain text with no blank line between, the
    heading is turned into Action and the next line is joined to it.
  - In the example, `INT. HOUSE - DAY` directly under `Mara waits.` and above `Lamps burn.` is joined into one Action
    paragraph with them. Nothing is a heading until there is a blank line before it (**Reading A**).
  - `Ext. 12 is busy.`, alone, is a heading: the prefix match ignores case and nothing else is checked (**Reading C**).
  - `INT HOUSE - DAY` is a heading. The line under it, `EST. SKYLINE`, has no blank line before it, so the heading
    is turned into Action together with `EST. SKYLINE` and `INT.`.
  - `INT.` alone is not a heading: the prefix must be followed by a separator and at least one more character. A
    prefix followed by a tab is accepted as a separator.
  - Beyond the spec: a hyphen is also accepted after the prefix, and an indented line is not a heading (see OFF-18).
- **Legacy parser (lower confidence).** Lenient (**Reading B**). The heading pattern needs only a single line break
  before and after, and the prefix `INT` or `EXT` needs no separator after it, so a line that merely starts with those
  letters qualifies. The `EST` prefix needs a non-word character in front of it, so it seems not to match at a line
  start.
- **Verdict: agrees** for the default parser (blank lines required, prefix case-insensitive, start of file counts).
  Readings **A**, **B2** and **C**. The extra hyphen separator goes beyond the literal text without conflicting with
  the example.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the heading step and the step that joins unrecognised lines
  to the previous element). `SceneHeaders.fountain` with `SceneHeaderTests.m` (the author expects: no heading when the
  blank lines are missing, no heading when the prefix has no separator, a lowercase heading is a heading).
  `FountainRegexes.m`, the scene header pattern constant.

### OFF-7: Forced heading `.` and the meaning of "alphanumeric"

- **Default parser.** A line forces a heading when its first character is a period and its second is **anything other
  than a period**. The second character is never tested for being a letter or digit. So:
  - `.LAMP ROOM`, `.5 HOURS LATER` and `.ÉPILOGUE` are headings (the period is dropped from the text).
  - `. SPACE AFTER` is a heading with the text `SPACE AFTER`.
  - `.>HEADING` is a heading with the text `>HEADING`.
  - `..DOUBLE` and `...trailing off` are not forced. `..DOUBLE` goes on to the other rules (Action when alone;
    a Character cue if it has a line of text under it, as nothing in it is lowercase). `...trailing off` is Action.
  - The forced-heading step does not need a blank line before it.
- **Legacy parser (lower confidence).** A leading period followed by anything forms a heading, except that a run of
  three periods is shielded first. So `..DOUBLE` seems to become a heading there.
- **Verdict: contradicts.** The literal text (official.md, OFF-7) excludes a period followed by a space or a symbol;
  the default parser accepts both. It matches **Reading C**, which official.md excludes. On Readings A and B it is
  silent, because it never tests for "alphanumeric".
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the forced-heading step). `SceneHeaders.fountain` with
  `SceneHeaderTests.m` (a forced heading is a heading; a line of three periods and text is not). `FountainRegexes.m`,
  the scene header pattern constant.

### OFF-8: Scene-number character set and placement

- **Default parser.** A scene number is a `#` pair at the **end** of the line (trailing whitespace allowed), with any
  characters except `#` between them. Nothing restricts the characters. It is tried for prefix headings and forced
  headings alike. It is removed from the heading text, and for a forced heading the text is also trimmed.
  - `#12#` is number `12`.
  - `#1 A#` and `#1_A#` are numbers (`1 A` and `1_A`): **Reading E**, which official.md excludes by the literal text.
  - `#É1#` is a number: Unicode is allowed (**Reading B**, and in fact anything is allowed).
  - `#7# trailing words` is **not** a number, because it is not at the end of the line; the text stays in the heading
    (**Reading C**).
  - `.TITLE CARD #3#` is a heading with number `3` (**Reading D**: forced headings can carry numbers).
  - A Section line (`# 12 #`) is read as a Section and never reaches this step (see OFF-20).
- **Legacy parser (lower confidence).** The number may hold ASCII letters, digits, periods, dashes and a closing
  parenthesis (the last is beyond the spec). `#1 A#`, `#1_A#` and `#É1#` are therefore not numbers (Reading A for the
  character
  set; Reading E is not accepted). The number is not required to be last, so `#7# trailing words` has its number taken
  from the middle
  (against Reading C), and a forced heading may carry one (Reading D). Whether the unanchored Section pattern disturbs
  a heading that ends in a `#` pair is **not determined by reading**.
- **Verdict: contradicts** for the default parser (Reading E accepted). It agrees with the literal "appended" reading
  (**C**) and with **D**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the scene-number handling in the forced-heading and heading
  steps). `FountainParser.m`, `parseBodyOfString:`. `FountainRegexes.m`, the scene number pattern constant.
  `SceneNumbers.fountain` with `SceneNumberTests.m` (digits, digits with letters either way round, mixed case, dashes,
  a trailing period, and a heading with a parenthesised note; the heading text keeps a trailing space where the number
  was).

### OFF-9: Transitions that do not end in `TO:`, and the blank-line edges

Traced as given, since adjacency is the point.

- **Default parser.**
  - A Transition is any line that **ends with `TO:` in capitals** (the test finds `TO:` at the end of the line; what
    comes before it is not checked, so a mixed-case line ending in `TO:` qualifies). No blank line before or after is
    required.
  - Separately, a short built-in list of whole lines is also a Transition: `FADE OUT.`, `CUT TO BLACK.` and `FADE TO
    BLACK.` (leading whitespace ignored).
  - In the example, `FADE OUT.` **is** a Transition. `FADE IN:` is **not**: it ends in a colon but not in `TO:`, so it
    is Action when alone. (If it had text directly under it, it would be a Character cue.)
  - `BACK TO:` is a Transition although it sits between two Action lines. The line under it, `Otto shrugs.`, has no
    blank line before it, so it is joined to the Transition's text.
  - `CUT TO:` directly above `INT. DECK - DAY` is a Transition too; the heading cannot be a heading without a blank
    line before it, so it is joined to the Transition's text. It is not read as a Character cue (**not Reading C**).
- **Legacy parser (lower confidence).** The same leniency: a single line break on each side is enough, no lowercase
  letters may come before `TO:`, and the same three built-in lines are accepted.
- **Verdict: contradicts.** The literal text requires an empty line before and after. The behaviour matches none of
  Readings A, B and C as written: it agrees with A that `FADE IN:` is Action, but it disagrees with A on `BACK TO:`
  and `CUT TO:` (Transitions) and on `FADE OUT.` (a Transition).
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Transition steps and the join step).
  `Transitions.fountain` with `TransitionTests.m` (author expects: `TO:` lines, `FADE TO BLACK.` and a bare `TO:`
  are Transitions; one with trailing spaces after the colon is not). `FountainRegexes.m`, the transition pattern
  constant.

### OFF-10: Centered text versus forced Transition

- **Default parser.** Any line that begins with `>` is looked at:
  - If its **last character is `<`**, it is centered Action; both marks are dropped and the text is trimmed.
  - Otherwise it is a Transition with the first `>` dropped and the text trimmed.
  - Trailing whitespace is not ignored. Results for the example:
    - `>FIN<` and `> FIN <` are centered, text `FIN`.
    - `>FIN<` followed by a space is a **Transition** with the text `FIN<` (**Reading B**: trailing whitespace is
      not ignored, so line 3 is not centered).
    - `> FIN` is a Transition with the text `FIN`.
    - `>> FIN <<` is centered; only the outer marks are dropped, so the text is `> FIN <` (**Reading C**).
    - `>FIN< and more` is a Transition with the text `FIN< and more`.
  - The check does not look at context, so a `>` line inside a Dialogue block is still a Transition (against Reading
    D). A `>` after leading whitespace is not recognised, because only the first character is tested.
  - The marks are always dropped (**Reading E**: dropped).
- **Legacy parser (lower confidence).** Whitespace after the `<` is tolerated for the centered form, so a trailing
  space does not turn line 3 into a Transition. A forced Transition keeps its `>` in the text.
- **Verdict: does not cover** (the spec does not say how to tell them apart beyond the closing `<`). Readings **A**,
  **B** (trailing whitespace stops it), **C** and **E**. The behaviour for a trailing space differs from what
  official.md describes the spec's own Centered Text example as showing (a trailing space after the `<`). That makes it
  a point for #4 to weigh.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the forced-transition step). `CenteredText.fountain` with
  `CenteredTests.m` (author expects: marks with or without spaces, and with tabs, are centered Action; a `>` with no
  closing `<` is a Transition). `Simple.fountain` (a forced Transition and a centered line). `FountainParser.m`,
  `parseBodyOfString:` and `FountainRegexes.m`, the false transition and centered text pattern constants.

### OFF-11: Dual dialogue without a preceding dialogue, and caret placement

- **Default parser.** A Character cue whose line ends with a caret, optionally followed by whitespace, is flagged as
  dual dialogue and the caret and spaces before it are removed from the text. The parser then walks back to the nearest
  earlier Character and flags it too. It does not require that a block exists or that the earlier one is unpaired.
  Results for the example:
  - `OTTO ^` as the first block: flagged dual, with no partner (**Reading B**: the caret stays a marker, an unpaired
    dual dialogue).
  - `OTTO^` with no spaces: a dual cue (**Reading C**).
  - `MARA (V.O.) ^`: a dual cue; the extension stays in the text.
  - `TESS ^` after `PIP`: pairs with `PIP`.
  - `MARA (V.O.) ^` right after `OTTO^` (which was already the second of a pair with `NELL`) is flagged too, and the
    flag is set again on `OTTO^`, because the parser only looks for "the nearest earlier cue" and keeps no count of
    pairs (**Reading D**: a caret in a row chains onto the block before it; it neither starts a separate pair nor
    extends the first). The writer and renderer count two flagged cues at a time.
- **Legacy parser.** A caret cue is treated as dual only from the third element onward. Earlier it is an ordinary cue
  with the caret left in its text (**Reading A**).
- **Verdict: does not cover** (the spec is silent on a missing first block and on chained carets). Readings **B**,
  **C** and **D** as above. The zero-space result agrees with the literal "any number of spaces" (official.md).
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Character step and its dual-dialogue handling).
  `FountainParser.m`, `parseBodyOfString:` (the third pass). `DualDialogue.fountain` and `Dialogue.fountain` with
  `DialogueTests.m` (a cue with a caret has its caret removed). `FountainWriter.m`, `bodyFromScript:` (pairs flagged
  cues by counting two at a time).

### OFF-12: Parenthetical shape

- **Default parser.** A Parenthetical is only recognised **inside a Dialogue block**: a line with no blank line before
  it, whose first non-whitespace character is `(`. It does not have to end with `)` or be a whole parenthesis, and it
  does not have to follow the cue directly. The result for the example:
  - As given, with no cue above, none of the four lines is a Parenthetical. They form one Action paragraph
    (consistent with the tie-breaker's lean towards Action).
  - If a cue were above them, `(whispering) and then speaks` and `(not closed` and `(multi-line` would each be a
    Parenthetical (**Reading B**: a line that only starts with `(`), and `paragraph)` would be Dialogue. No
    Parenthetical
    spans lines (not Reading C).
- **Legacy parser.** The Parenthetical pattern is not tied to a cue and can run over line breaks up to a `)` at a line
  end, so its result for the example is **not determined by reading**.
- **Verdict: does not cover** (the spec says nothing about text after the parenthesis, an unclosed one, or several
  lines). **Reading B** inside a block; Action when there is no cue.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Dialogue and Parenthetical step). `Dialogue.fountain` with
  `DialogueTests.m` (parentheticals after a cue, at the end of a block and indented). `FountainRegexes.m`, the
  parenthetical pattern constant.

### OFF-13: Lyrics next to cues and Dialogue

- **Default parser.** A line whose **first character is `~`** is a Lyrics element, checked before every other rule and
  regardless of the surroundings (**Reading B**). The `~` stays in the stored text and is removed by the renderer.
  - After the cue `OTTO`, both `~` lines are Lyrics, not Dialogue.
  - Lyrics groups that are separated by a blank line get a one-space Lyrics element between them, as a spacer.
  - `~A lyric on its own` is Lyrics with no neighbours needed.
  - ` ~ indented lyric` (leading space) is not Lyrics, because only the first character is tested; it is Action
    (**Reading C**: leading whitespace stops it).
- **Legacy parser.** It has no rule for lyrics at all, so a `~` line after a cue would be Dialogue and elsewhere
  would be Action (**Reading A**, by absence).
- **Verdict: does not cover** (the spec says nothing about adjacency). **Reading B**, with **C** for the indented line.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the first step in the body loop). `ForcedElements.fountain`
  with `ForcedElementTests.m` (the author expects the `~` lines under a cue to be Lyrics). `FNHTMLScript.m` (strips
  the `~`).

### OFF-14: Emphasis edges

- **Both parsers** leave emphasis marks in the text. The sample renderer applies emphasis, using the author's own
  fixed set of seven patterns, which the code labels as extras for FDX styling. They are not a Markdown
  implementation.
- Behaviour of those patterns, in plain English:
  - They run on one element's text at a time, in a fixed order from the most specific (bold, italic and underline
    together) to the least. Each pairs a run of marks with a run of the same marks around text that holds no angle
    brackets.
  - Text between the marks **may contain a line break**. Elements end at a blank line, so a span cannot cross a blank
    line, but it can cross a single line break inside an element (**Reading C**). In isolation, `*across` then
    `lines*` on the next line is one span, and `*across`, a blank line, then `lines*` is not.
  - The underscore pattern pairs two underscores around any text with no underscore in it, with no check on what
    surrounds them, so `snake_case_name` underlines `case`.
  - Only the single-asterisk pattern checks for a preceding backslash. No code in what was read removes a backslash,
    so the escaped underscores, the doubled backslash and the escaped hash in the example are not unescaped there, and
    the other patterns do not check for one.
  - The result of the overlapping lines in the example (`*bold **inside** italic*`, `*start _nest* end_`, `***triple*`)
    depends on the order and greediness of the patterns and is **not determined by reading**.
- **Verdict: does not cover** (the spec names Markdown without a version; the reference code does not implement a
  Markdown dialect). Behaviour matches **Reading C** for line breaks; Reading A (a named Markdown) does not apply;
  **Reading D** applies only to a leading single asterisk.
- **Evidence.** `FountainRegexes.m`, the seven styling pattern constants (bold, italic, underline and their
  combinations). `FNHTMLScript.m` (where they are applied, in order). `README.markdown` (the parser leaves styling to
  downstream code).

### OFF-15: Nested and unclosed Notes

- **Default parser.** It recognises a Note only as a **standalone line**: a blank line before it, the whole line
  wrapped in double brackets, and no closing bracket inside. That makes a Comment element. Everything else stays in
  the text of the surrounding element:
  - Inline Notes, nested Notes, an unclosed `[[`, a stray `]]`: all plain text for the parser.
  - A Note over several lines is **not** recognised as one. The fixture `Notes.fountain` has a two-line Note, but
    `NoteTests.m` tests only the single-line one.
  - A standalone Note with no blank line before it (directly under a cue) is part of the Dialogue text.
  - A standalone Note with blank lines around it does split a Dialogue block (**Reading D**: it splits). The parser
    does not remove the blank lines.
- **The renderer** removes `[[ ... ]]` inline with a pattern that ends at the first closing pair and, by the usual
  default for the dot in that regex library, does not span a line break. So `Hello [[outer [[inner]] outer]] world.`
  loses `[[outer [[inner]]` and keeps ` outer]] world.`
  (**Reading A**: the first `]]` ends it). An unclosed `[[` stays as text (**Reading C**, first alternative).
- **Legacy parser.** A Note on its own lines may span several lines (newlines inside are protected first); an unclosed
  one matches nothing and stays text.
- **Verdict: does not cover** (the spec says nothing on nesting or unclosed Notes). **Readings A and C** (renderer),
  and **D**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Comment step). `FNHTMLScript.m` (the inline removal).
  `Notes.fountain` with `NoteTests.m`. `FountainParser.m`, `parseBodyOfString:` (first pass).

### OFF-16: Nested, unclosed and inline Boneyard

- **Default parser.** It works on lines and only at the start of a line:
  - A line that **starts with `/*` and ends with `*/`** is a one-line Boneyard element. All opening and closing
    marks on the line are removed, so `/* outer /* inner */ still outer? */` is one Boneyard with no text left over.
    The first `*/` does not end it (not Reading A), and nothing stays as text (closest to **Reading B**, though the
    mechanism is by line and not by matching).
  - A line that starts with `/*` and does **not** end with `*/` opens a block, whatever else is on the line. The rest
    of that line is dropped. Following lines, blank ones included, are collected until a line that **ends with `*/`**.
    So an unclosed `/* never closed` runs to the **end of the file** (**Reading C**, first alternative), and its
    collected text is never made into an element: it is silently lost.
  - A mid-line Boneyard (`Mara /* aside */ waits.`) is not recognised and stays in the Action text (kept for later).
  - A line that **ends with `*/`** outside any block, such as a lone `*/` or `Mara /* aside */`, makes an empty
    Boneyard element and the line's other text is lost (**Reading E** does not hold: it is not plain text).
  - Lines in an open block that start with `~`, `!` or `@` are still classified as Lyrics, Action or Character first,
    before the block handling.
  - Text directly after a closing mark that is not at the line end (`/* x */INT. A`) is not handled. A line like `/* x
    */INT. A` starts
    with the opening mark and does not end with the closing one, so it opens a block and the whole line is lost. So
    **Reading D** (remove first, then detect) does not hold.
  - For a layout like the spec's own example described in official.md (a Transition above the opening mark and a
    heading below the closing mark, with no blank lines): the Transition rule needs no blank line, and the heading
    below has no blank line before it, so by reading it is joined to the Boneyard element's text and so hidden.
- **Legacy parser (lower confidence).** A Boneyard is recognised only when it starts at a line start and ends at a
  line end; it may span lines and ends at the first closing mark followed by a line break (Reading A in that form).
  An unclosed `/*` matches nothing and stays text (**Reading C**, second alternative).
- **Verdict: does not cover** (the spec leaves nesting and unclosed marks open). The default parser matches **Reading
  C** (to the end of the file, text lost) for an unclosed mark, and nothing for inline use.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the three Boneyard steps). `Boneyard.fountain` with
  `BoneyardTests.m` (author expects a one-line and a multi-line Boneyard; the inline one in the fixture is not
  tested). `FountainParser.m`, `parseBodyOfString:` (first pass). `FountainRegexes.m`, the block comment pattern
  constant.

### OFF-17: Title page edges

- **Default parser.** It looks only at the **first block**, up to the first line with no characters.
  - **Start (Reading C).** If any line in that block has the shape "a key, a colon, then a value" or "a key and a
    colon alone", the whole block is taken as the title page. Any key will do. As written, the example's first block
    is only `Note: this is a first line of action.`, so that line **is** the whole title page (the misreading
    official.md warns of), and everything from `Title: A Quiet Evening` down is **body text** (an Action paragraph).
    The test is per line, so a first block of ordinary text containing one such line also becomes a title page.
    A line that ends in a colon, such as `CUT TO:` or `FADE IN:`, also opens a title page when it is the first line of
    the file and has a line directly under it. A case example saved as a whole file can therefore vanish into the
    title page (relevant to the corpus, #15).
  - **End (Reading A).** It ends at the first zero-length line.
  - **Keys.** Lowercased, so matching is case-insensitive (`Title` and `title` are the same key). Duplicates are
    kept as separate entries, in order. Unknown keys are kept. `Author` is renamed `authors`. A value may hold
    colons (`5:30 pm`).
  - **Values.** Lines after a key with an empty value are its values, and **no indentation is checked**: a one-space
    indent, a three-space indent, a tab and no indent at all are treated alike, each trimmed. A line with an inline
    value cannot take indented lines: they are lost (neither title page nor body). That answers the "mixed" question
    with "not supported". So the example, even if its `Title:` block came first, would not exercise the indent rule:
    its indented lines follow `Contact: Someone`, which has an inline value. The "any indent, or none" result applies
    under a bare key such as `Contact:` on its own.
  - **Before (Reading F).** Leading blank lines are stripped first, so they do not stop the title page.
- **Legacy parser (lower confidence).** Same start rule, with a stricter key (letters, digits, spaces and ampersands
  only), and a value set that excludes some punctuation. Continuation lines must be indented by **two to eight spaces
  or a tab**, so a one-space indent is dropped and a two-space indent counts. Indented lines after an inline value
  seem to be lost as well.
- **Verdict: contradicts.** The literal text (official.md, Reading E) says three or more spaces or a tab; the default
  parser accepts any indent or none (under a bare key), and the legacy parser accepts two. It matches Readings A, C and
  F, and Reading D (keys case-insensitive, duplicates kept, unknown keys kept).
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the title-page section). `FountainParser.m`,
  `parseTitlePageOfString:` and `titlePageOfString:`. `FountainRegexes.m`, the four title page pattern constants.
  `Simple.fountain` and `FNScriptTests.m` (a title with indented lines round-trips; a value with a colon).

### OFF-18: Tabs, indentation and recognising the element first

- **Default parser.** Leading whitespace is **not stripped** before detection, and no tab is converted to spaces. What
  happens depends on the element:
  - A Character cue, Transition and Parenthetical tolerate leading whitespace (their checks do not need the first
    character), and the stored text keeps the whitespace. A tab-indented `OTTO` is a Character, and the line under it
    is Dialogue.
  - A Scene Heading must start at the first character, so a tab-indented `INT. HOUSE - DAY` is **not** a heading.
    Alone, it is
    Action with its tab kept.
  - `    THE BIG ROOM`, alone, is Action (no line of text under it); with text under it, it would be a Character.
  - A forced sigil must be the first character too (see OFF-22).
  - So the circle is broken by looking at the raw line (**Reading B** for headings), except for the elements that
    tolerate indentation (**Reading A**). Reading C (tab conversion) is not implemented at all.
- **Legacy parser (lower confidence).** The cue pattern allows leading spaces and tabs; the heading pattern starts at
  the line start.
- **Verdict: does not cover** (the spec leaves the circle open). A mix of **A** and **B**, as above.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Character, Transition, Parenthetical and heading steps).
  `Indenting.fountain` with `IndentingTests.m` (author expects an indented Transition, cue, Parenthetical and Dialogue
  to be recognised). `Dialogue.fountain` (an indented cue and parenthetical).

### OFF-19: `=` runs: page break, synopsis and whitespace

- **Default parser.**
  - A line of **three or more `=` and optional trailing whitespace** is a Page Break. `===`, `====` and `=== ` (with a
    trailing space) are Page Breaks. There is no upper limit.
  - Any other line whose first non-whitespace character is `=` is a Synopsis. The first `=` is dropped:
    - `==` is a Synopsis with the text `=`.
    - `=` alone is a Synopsis with empty text.
    - `= Summary text` is a Synopsis with the text ` Summary text`.
    - `  ===` (leading whitespace) is a Synopsis with the text `==`.
    - `= = =` is a Synopsis with the text ` = =`.
    - `=== text` would be a Synopsis with the text `== text` (**Reading C**, first alternative).
  - **Reading A** holds for the run lengths. For whitespace, a trailing run is ignored (**Reading B**, in part) and a
    leading one voids the break.
- **Legacy parser.** The page-break pattern accepts a run of **three to eight** characters that may be `=`, `-` or `_`
  in any mixture, with whitespace around it. So `---` and `___` break pages there and a run of nine `=` does not. The
  Synopsis pattern needs at least two characters after the `=`, so `=` and `==` are not Synopses.
- **Verdict: does not cover** (the spec leaves `==`, `=` and the whitespace cases open). Matches **Reading A**. The
  trailing-whitespace tolerance sits a little outside the "nothing more" wording.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the page-break and Synopsis steps). `PageBreaks.fountain` with
  `PageBreakTests.m` (author expects three equals signs and a nine-equals run to be page breaks; the second is a case
  the legacy parser fails, which fits its README caveat). `Synopses.fountain` with `SynopsisTests.m`.
  `FountainRegexes.m`, the page break and synopsis pattern constants.

### OFF-20: Section edges

- **Default parser.** A line whose first non-whitespace character is `#` is a Section. It needs no blank line before
  it. The depth is the length of the `#` run (and, apparently by oversight, any leading whitespace before it is
  counted too); there is no maximum.
  - `#Section without space` is a Section with the text `Section without space` (**Reading A**: no space needed).
  - `# ` (a hash and a space) is a Section whose text is the single space (**Reading C**, first alternative). A bare
    `#` with nothing after it is dropped, with a log message.
  - `##### Deep` is a Section of depth 5.
  - `#1 priority item` is a Section with the text `1 priority item`.
  - `INT. HOUSE - DAY #5#` is a heading with a number (see OFF-8).
  - `# 12 #` is a Section with the text ` 12 #`: the Section reading wins at the line start (**Reading D**).
- **Legacy parser (lower confidence).** The Section pattern is not tied to a line start, so a `#` in the middle of a
  line may be picked up. How that interacts with other lines is **not determined by reading**.
- **Verdict: does not cover** (the spec gives no depth limit and does not address a missing space or empty text).
  Readings **A**, **C** and **D**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Section step). `SectionHeaders.fountain` with
  `SectionHeaderTests.m` (no space after the hash, all capitals, a number only, depths two and three).
  `FountainRegexes.m`, the section header pattern constant.

### OFF-21: Synopsis and Section placement inside blocks

- **Default parser.** Neither the Synopsis rule nor the Section rule looks at a blank line or at the Dialogue flag.
  `= a synopsis in dialogue` inside OTTO's block becomes a Synopsis element, and `Goodbye.` under it is Dialogue again,
  in a new Dialogue element. `# a section here?` under `NELL` becomes a Section element and `Yes.` is Dialogue. So the
  block continues across the removed lines (**Reading A**), but as separate elements.
- **Legacy parser (lower confidence).** The Dialogue pattern cannot contain a tag, so a Synopsis tag inside a block
  appears to break the Dialogue recognition (like **Reading B**). Not determined by reading.
- **Verdict: does not cover** (the spec says only that they are ignored in formatted output). **Reading A**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Synopsis, Section and Dialogue steps).
  `Sections-Complex.fountain`
  with `SectionAndSynopsisTests.m` (the tests expect a Section and a Synopsis to be recognised even with no blank line
  before them).

### OFF-22: Forcing sigils: leading whitespace, stacking, and lines starting with `>`

- **Default parser.** Each sigil is looked at only as the **first character** of the line (**Reading A**).
  The checks run in a fixed order: `~`, then `!`, then `@`, and `.` and `>` a good deal later. The first that fits
  wins, and the rest of the line is plain text (**Reading C**, second alternative). Results:
  - `  !indented action`: not forced, because of the leading spaces; it is plain Action text.
  - `!.dot after bang`: Action. The `!` wins and the `.` is plain text.
  - `@!after at`: Character. The `@` wins.
  - ` .INDENTED HEADING`: not forced (leading space).
  - `>>> a line of arrows`: a Transition with the text `>> a line of arrows`, so a line that merely starts with `>`
    is a Transition (**Reading D**).
  - `!>NOT A TRANSITION`: Action. The `!` wins.
  - `.>HEADING`: a Scene Heading with the text `>HEADING` (see OFF-7).
- **Which sigils are dropped (Reading E).** In the parser's output, `.` and `>` are dropped from the text, and `!`, `@`
  and `~` are **kept**; the renderer removes them later. `FountainWriter.m` relies on that: it writes `!`, `@` and `~`
  text back unchanged and puts the `.` back on a heading only when the text no longer looks like one.
- **Legacy parser.** No rule for `!`, `@` or `~`. A forced Transition keeps its `>` in the text (lower confidence).
- **Verdict: does not cover** (the spec is silent on whitespace, stacking and which sigils are dropped). Readings
  **A**, **C**, **D** and a mix on **E**.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the sigil steps, in order). `FNHTMLScript.m` (strips `@`, `~`
  and `!`, and `.` from headings). `FountainWriter.m`, `bodyFromScript:`. `ForcedElements.fountain` with
  `ForcedElementTests.m`.

### OFF-23: Significant trailing whitespace

- **Default parser.**
  - `CUT TO:` is a Transition. `CUT TO: ` (trailing space) is not, because the `TO:` has to be at the very end of the
    line; it is Action (or a Character cue, with a line of text under it) (**Reading A**, for this line).
  - `MARA ^ ` (caret, then a space) is a dual cue: whitespace after the caret is accepted, so trailing whitespace is
    stripped for the caret test (**Reading A**; against Reading B). The spec's "last character" wording leaves this
    open (official.md), so this is evidence for #4 and not a contradiction.
  - `Two trailing spaces.  ` is plain Action, with the spaces kept in the text. Two trailing spaces do not force Action
    or anything else (**Reading C** is not implemented, consistent with the 1.1 changelog).
  - Carriage returns are turned into line feeds first, so they never count as trailing whitespace (Reading D has no
    effect here).
- **Legacy parser.** The same for `TO:` (the pattern needs a line end right after) and for the caret (trailing
  whitespace allowed). Lower confidence.
- **Verdict: agrees.** Spaces after `TO:` make Action, and two trailing spaces force nothing, which confirms the
  exclusion of Reading C. On the remaining lines the parser follows **Reading A**: trailing whitespace is otherwise
  ignored, including after the caret.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the Transition and Character steps). `Transitions.fountain`
  with `TransitionTests.m` (a line with trailing spaces after the colon is expected not to be a Transition).
  `FountainRegexes.m`, the dual dialogue and transition pattern constants.

### OFF-24: What a Dialogue block may contain

- **Default parser.** Several sigil and marker steps run **before** the Dialogue step and ignore the Dialogue flag, so
  forced elements do take effect inside a block (**Reading B**):
  - `.This looks like a heading`: a Scene Heading.
  - `>This looks like a transition`: a Transition.
  - `[[This is a note]]`: not recognised as a Note (it has no blank line before it), so it is Dialogue text; the
    renderer removes it later.
  - `=This is a synopsis`: a Synopsis element.
  - `/* boneyard */`: a Boneyard element.
  - `That is all.`: Dialogue again, in a new element after each interrupting element.
  - `!` and `~` lines are Action and Lyrics too. `@` starts a new cue.
  - Sections, Synopses and Page Breaks do not end the block (not Reading C); only a blank line does.
- **Legacy parser.** **Not determined by reading** (tag-based matching makes the result depend on how the tags nest).
- **Verdict: does not cover** (the spec gives no priority between "text after a cue" and a sigil). **Reading B**, with
  Notes left in the text and Boneyard removed as its own element.
- **Evidence.** `FastFountainParser.m`, `parseContents:` (the order of the body-loop steps). `ForcedElements.fountain`
  with `ForcedElementTests.m` (a `~` line under a cue is Lyrics). `Dialogue.fountain` with `DialogueTests.m`.

## Literal-text calls confirmed or challenged

official.md marks some points as settled by the spec's literal text (OFF-7, OFF-8, OFF-17 and OFF-23), and its table of
plainly stated rules adds more. "Challenged" means the default parser behaves
differently from that text; it is evidence for #4 to weigh, not a verdict on the call.

| Call (official.md) | Literal text says | Default parser | Result |
|---|---|---|---|
| OFF-7: only a period followed by an alphanumeric forces a heading, so Reading C is out | `. SPACE AFTER` and `.>HEADING` do not force a heading | Both do (anything but a second period) | **Challenged** |
| OFF-8: only alphanumerics, dashes and periods in a scene number, so Reading E is out; "appended" means the end of the line | `#1 A#` and `#1_A#` are not numbers; the number is at the end | Both are numbers; `#7# trailing words` is not a number | **Challenged** (character set); **confirmed** (end of the line). The legacy parser confirms the character set and not the end of the line |
| OFF-17: indent of three or more spaces, or a tab | One- and two-space lines do not continue a value | Any indent, or none, continues a value (legacy: two to eight spaces) | **Challenged** |
| OFF-23: two trailing spaces no longer force Action; spaces after `TO:` make Action | Reading C is out; `CUT TO: ` is Action | No pre-1.1 rule; `CUT TO: ` is not a Transition | **Confirmed** |
| OFF-9: a Transition needs empty lines around it and an uppercase `TO:` ending | `BACK TO:` and `CUT TO:` between text lines are not Transitions | They are Transitions; also `FADE OUT.` | **Challenged** |
| OFF-6: a heading needs an empty line before and after; matching ignores case | Reading B (no blank lines) is out; `Ext. 12 is busy.` qualifies | Blank lines are required (the one after is enforced by demoting the heading); case ignored | **Confirmed** (legacy parser lenient) |
| OFF-11: "any number of spaces" before the caret includes zero | `OTTO^` is a dual cue | A dual cue | **Confirmed** |
| OFF-3 (Character section): extensions may be in either case | `MARA (on the radio, softly)` is a cue | Not a cue (only the exact `(cont'd)` is allowed in lowercase) | **Challenged** |
| OFF-4 (Character section): a cue needs at least one alphabetical character | A line of digits is not a cue | No letter test; a line of digits is a cue | **Challenged** |
| OFF-19: three or more `=` and nothing else is a page break | `====` is one; `==` is not | The same, but trailing whitespace is accepted; legacy: three to eight of `=`, `-` or `_` | **Confirmed** (default); the legacy parser **challenges** it |
| OFF-11 and OFF-23 (Dual Dialogue section): the caret must be the last character | Not settled by the literal text (Reading A or B) | A dual cue; trailing whitespace after the caret is ignored | **Not settled**; the parser follows Reading A |
| Error Handling: a closing mark is not searched for past a double line break; Boneyard is exempt | An unclosed `[[` or `*` is bounded | Notes and Boneyard are matched by line shape, not searched for; an unclosed `/*` runs to the end of the file | **Confirmed** for Boneyard; not applicable to Notes and emphasis (see OFF-14, OFF-15) |

## Summary for #4

"Reading" is the one in official.md that the default parser's behaviour matches. The last column notes where the
legacy parser differs (lower confidence).

| Case | Verdict | Reading | Legacy parser differs |
|---|---|---|---|
| OFF-1 | does not cover | A, plus an exact-two exception in Dialogue | No special rule; any whitespace line stays in the block |
| OFF-2 | does not cover | A | A CRLF title page appears not to be found |
| OFF-3 | contradicts | B, plus one lowercase form | Rejects even `(cont'd)` |
| OFF-4 | does not cover | C (and looser) | No |
| OFF-5 | does not cover | B | No |
| OFF-6 | agrees | A, B2, C | Lenient (B) |
| OFF-7 | contradicts | C | `..DOUBLE` seems to be a heading too |
| OFF-8 | contradicts | E accepted; C and D hold | Confirms the character set; the number need not be last |
| OFF-9 | contradicts | None of A, B, C | Same leniency |
| OFF-10 | does not cover | A, B, C, E | Trailing space after `<` tolerated |
| OFF-11 | does not cover | B, C, D | A caret cue early in the file is an ordinary cue |
| OFF-12 | does not cover | B (inside a block); Action with no cue | Not determined by reading |
| OFF-13 | does not cover | B, and C for an indented line | No lyric rule |
| OFF-14 | does not cover | C (line breaks); no named Markdown | Not applicable |
| OFF-15 | does not cover | A and C (renderer), D | Multi-line standalone Notes work |
| OFF-16 | does not cover | C (to the end of the file, text lost) | Unclosed stays text |
| OFF-17 | contradicts | A, C, D; E not held | Two to eight spaces |
| OFF-18 | does not cover | A for cues; B for headings | Cue pattern allows indent |
| OFF-19 | does not cover | A (trailing space tolerated) | Three to eight of `=`, `-`, `_` |
| OFF-20 | does not cover | A, C, D | Unanchored; not determined by reading |
| OFF-21 | does not cover | A | Appears to be B; not determined by reading |
| OFF-22 | does not cover | A, C, D; mixed on E | No `!`, `@`, `~` |
| OFF-23 | agrees | A; C not implemented | Same |
| OFF-24 | does not cover | B (Notes stay in the text) | Not determined by reading |
