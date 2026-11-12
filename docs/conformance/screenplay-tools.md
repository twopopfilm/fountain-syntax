# Differences from screenplay-tools

Every place where this crate's typed view of a corpus file differs from the reading of screenplay-tools, and what we
decided about it. The test `screenplay_tools_every_difference_is_classified` in `tests/screenplay_tools.rs` fails on
any difference that has no entry here, on any entry that no difference matches, and on any entry still marked `todo`
or `ours-wrong`.

## What is compared

- **The tool.** screenplay-tools 0.0.10 from PyPI (MIT), the Python parser. `docs/research/extensions.md` read a git
  commit of its repository instead, so a behaviour read there can differ from the release diffed here.
- **Our side.** `parse(src).screenplay()` with the default features. The test file is compiled out when the `shot` or
  `locked-scene-numbers` feature is on (`cargo test --all-features` skips it), because those features change the view.
- **Their side.** `tests/corpus/screenplay-tools.json`, written by `scripts/screenplay-tools-diff.py`. The test fails
  when the snapshot does not hold exactly the corpus files with the hash of each one's bytes.
- **The line form.** One string per element, in document order, as in the script's docstring: `TITLE <key>: <value>`,
  `HEADING <text>[ #<number>]`, `ACTION`, `CENTERED`, `CHARACTER <name>[ (<ext>)]...[ ^]`, `PAREN`, `DIALOGUE`, `LYRIC`,
  `TRANSITION`, `SECTION <depth> <text>`, `SYNOPSIS`, `PAGEBREAK`, and a last line `HASH_V1 <12 hex digits>` for each
  scene. A line break inside a text is written U+23CE. Text with its note and boneyard markers removed and its ends
  stripped; a text element that is then empty is dropped, but `PAGEBREAK`, `CHARACTER` and `HEADING` never are. For a
  run of merged Actions the rule applies to the whole run: an empty Action inside it stays as an empty paragraph, and
  the line is dropped only when the run is empty. Our `HASH_V1` is the first 12 hex digits of the SHA-256 of `Scene::normalized_text_v1()`.
  A `HASH_V1` hunk is classified by the element difference it follows from and cites `SPEC.md` 11.7's known-cases
  row for it; that table has no row for `OFF-7`, `OFF-8` or `OFF-16e`, so those hash hunks cite the §6 row.
- **The diff.** A longest-common-subsequence line diff of ours against theirs per file. Adjacent changes form one hunk.
  An entry shows `-` for our lines and `+` for theirs.

## What we change on our side, and what that hides

Each change makes our output match screenplay-tools' representation. A difference hidden by one of them is not seen.

- **Stripped ends, tabs.** Only the two ends of a text line are stripped, with Rust's `trim`, once per line: for a
  run of merged Actions, the ends of the whole merged line, so every later paragraph keeps its leading indentation
  (`OFF-18`) and a text keeps the indentation of its inner lines. The one indentation not shown is what lies at the
  start or end of the line itself. An Action's tabs become four spaces first (`Action::display`, `OFF-18c`, which
  screenplay-tools also does). Only a title value is stripped line by line, with its empty lines dropped, as the
  script does. Rust's `trim` and Python's `strip` agree on U+0085 and U+2028 and differ on U+001C to U+001F; no corpus
  file has one today.
- **Parentheticals.** A `PAREN` line holds the text inside the parentheses, trimmed, because screenplay-tools gives that
  (the library's `DialogueItem::Parenthetical` keeps the parentheses). This hides only the parentheses.
- **Emphasis.** The markers are written back from `Run::Emphasis`: `***` for bold and italic, `**` for bold, `*` for
  italic, `_` for underline (outermost). `Run::Escaped(c)` is written `\c`. This hides nothing.
- **Notes and boneyards.** `Run::Note`, `Run::Boneyard` and whole-line `Element::Note` and `Element::Boneyard` are
  left out, because screenplay-tools removes their markers the same way. This hides the notes' own text and position;
  their effect on the elements around them stays visible.
- **Merged Actions and Dialogue.** Consecutive non-centered Actions are joined with U+23CE twice, also across a
  left-out Note or Boneyard, as screenplay-tools merges them, and an empty Action (a forced `!` alone, `OFF-22f`)
  is one of the run. Consecutive `DialogueItem::Line`s are joined with
  U+23CE. This hides only where one Action paragraph ends and the next starts; it is the same merge as `SPEC.md`
  11.7 rule 4.
- **Cues.** Our `(CONT'D)` extensions are dropped, and forcing flags (`@`, `!`, `>`, `.`) are not compared. This hides
  the `OFF-3e` ruling that `(CONT'D)` is kept as written, which `EXT-contd-1` tests: screenplay-tools strips it. Any
  other extension is written ` (<ext>)`, one group each; the script writes one group from the library's single
  extension string.
- **Order.** Our scene's Sections and Synopses that start before its heading come before `HEADING`, the rest after
  it. An Action's interjections follow their Action line.

## The classes

- `theirs-wrong`: the official Fountain syntax settles the point the hunk turns on, `SPEC.md` follows it, and
  screenplay-tools departs from it.
- `documented-choice`: the official syntax leaves the point open, or this project's own ruling decides it, and
  `SPEC.md` rules for ours, whichever way the two parsers differ.
- `ours-wrong`: this crate does not do what `SPEC.md` says. It is a bug: it needs an issue and a fix, and the entry is
  removed with the fix.
- `todo`: no `SPEC.md` ruling covers the hunk yet.

## Open for SPEC.md

- **The caret that is not last.** The carve-out in §4.0 reads as if `NAME ^ (V.O.)` falls through to Action, while
  row `OFF-11c` and this crate make it a cue whose name holds the caret. Ruled in #16: the row wins and the §4.0 prose
  is to be corrected. This touches the unforced half of off-11c.
- **A bare prefix with a trailing space.** §3.3 says trailing whitespace is not significant apart from `OFF-23a`, which
  would read `INT ` as a bare `INT` with no separator, while row `OFF-6d` makes `INT ` a heading and this crate follows
  the row. Ruled in #16: the row wins and the §3.3 prose is to be corrected. This touches off-6d.
- **Indentation before a forcing `!`.** off-22a is recorded as `theirs-wrong`, because §4.3 and `OFF-18b` keep the
  indentation in Action text, but no test yet pins the two spaces before a forcing `!` in the view's text. A pinning
  test is a follow-up.

An entry is a heading with the file, a line with the class, the `SPEC.md` rulings and the section of
`docs/research/reference-parser.md` that covers the point (or "not covered"), a one-line reason, and the hunk in a
four-backtick `diff` block.

## The command

```sh
uv run --script scripts/screenplay-tools-diff.py
```

It rewrites the snapshot, then runs `cargo test --test screenplay_tools`.

## Entries

### off/off-11a.fountain

**Class:** documented-choice · **Ruling:** OFF-11a · **Reference parser:** OFF-11

A caret cue with no Dialogue block before it: ours is an ordinary, unpaired block (no right half, so no caret in the line form); theirs flags it dual with no partner.

````diff
-CHARACTER OTTO
+CHARACTER OTTO ^
````

### off/off-11c.fountain

**Class:** theirs-wrong · **Ruling:** OFF-11c, OFF-3d · **Reference parser:** OFF-11

A caret that is not last: ours keeps both lines as non-dual cues whose names hold the caret; theirs makes neither a cue, not even the forced `@Nell ^ (V.O.)`, and merges both blocks into the Action.

````diff
-ACTION Mara waits.
-CHARACTER NAME ^ (V.O.)
-DIALOGUE This caret is not last.
-ACTION Mara waits again.
-CHARACTER Nell ^ (V.O.)
-DIALOGUE Neither is this forced one.
+ACTION Mara waits.⏎⏎NAME ^ (V.O.)⏎This caret is not last.⏎⏎Mara waits again.⏎⏎@Nell ^ (V.O.)⏎Neither is this forced one.
````

### off/off-11d.fountain

**Class:** documented-choice · **Ruling:** OFF-11d · **Reference parser:** OFF-11

A third caret block in a row: ours leaves `ELI ^` unpaired because `OTTO ^` is already a right half; theirs flags it dual again.

````diff
-CHARACTER ELI
+CHARACTER ELI ^
````

### off/off-11e.fountain

**Class:** documented-choice · **Ruling:** OFF-11e · **Reference parser:** OFF-11

`ELI ^` after an Action: ours finds no partner, since the Action breaks the pairing; theirs flags it dual anyway.

````diff
-CHARACTER ELI
+CHARACTER ELI ^
````

### off/off-13a.fountain

**Class:** documented-choice · **Ruling:** OFF-13a, OFF-24c · **Reference parser:** OFF-13

The line after a `~` lyric inside a Dialogue block: ours keeps it Dialogue because the lyric interrupts the block without ending it; theirs makes it Action.

````diff
-DIALOGUE And then she stops.
+ACTION And then she stops.
````

### off/off-15b.fountain

**Class:** documented-choice · **Ruling:** OFF-15b · **Reference parser:** OFF-15

An unclosed `[[`: ours keeps it and its text as plain text (the tie-breaker); theirs closes the Note at the blank line and drops `[[check the key and more text`.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The letter says [[check the key and more text⏎⏎The next paragraph goes on.
+ACTION The kettle clicks off and nobody moves.⏎⏎The letter says ⏎The next paragraph goes on.
````

### off/off-15d.fountain

**Class:** documented-choice · **Ruling:** OFF-15d, OFF-15e · **Reference parser:** OFF-15

A whole-line Note over two lines between Action paragraphs: ours hides it and the paragraphs join once; theirs hides it too but leaves an empty paragraph in its place. The inline two-line Note is removed by both.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits  and goes.
+ACTION The kettle clicks off and nobody moves.⏎⏎⏎⏎Mara waits  and goes.
````

### off/off-15e.fountain

**Class:** documented-choice · **Ruling:** OFF-15e · **Reference parser:** OFF-15

Whole-line Notes inside a Dialogue block, inside an Action paragraph and between paragraphs: ours makes each line invisible, so the block and paragraph continue with no gap; theirs leaves an empty line or empty paragraph where each Note was.

````diff
-DIALOGUE I hear it.⏎And I will answer.
-ACTION The lights flicker.⏎The lights hold.⏎⏎She waits.⏎⏎She goes.
+DIALOGUE I hear it.⏎⏎And I will answer.
+ACTION The lights flicker.⏎⏎The lights hold.⏎⏎She waits.⏎⏎⏎⏎She goes.
````

### off/off-15f.fountain

**Class:** theirs-wrong · **Ruling:** OFF-15f, OFF-16a · **Reference parser:** OFF-15, OFF-16

Theirs ends the title page at the Note line, so `Author: A. Writer` becomes Action (OFF-15f hides the Note and the title page goes on, a choice), and theirs drops the whole last line, losing visible text around an inline Boneyard; ours keeps `Mara  waits, then  goes.` as OFF-16a requires.

````diff
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara  waits, then  goes.
+ACTION Author: A. Writer⏎⏎The kettle clicks off and nobody moves.
````

### off/off-15g.fountain

**Class:** documented-choice · **Ruling:** OFF-15b, OFF-15g · **Reference parser:** OFF-15

A `[[` whose `]]` lies past a blank line: ours finds no closer and keeps both halves as plain text; theirs closes the Note at the blank line and drops `[[This note`. The two-space line and the Boneyard over a blank line agree.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. [[This note⏎⏎ends at the blank line.]] The curtain stirs.⏎⏎The floor creaks.  The step is gone.
+ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. ⏎ends at the blank line.]] The curtain stirs.⏎⏎The floor creaks.  The step is gone.
````

### off/off-16c.fountain

**Class:** documented-choice · **Ruling:** OFF-16c · **Reference parser:** OFF-16

An unclosed `/*`: ours keeps it as plain text and the rest of the file stays visible; theirs hides everything after it.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits. /* an aside that never closes⏎⏎The rest of the file is still here.⏎⏎And so is this last line.
+ACTION The kettle clicks off and nobody moves.
````

### off/off-16e.fountain

**Class:** documented-choice · **Ruling:** OFF-16e, OFF-9c · **Reference parser:** OFF-16

Ours deletes hidden spans before reading the lines: `/* x */INT. A - DAY` is a framed heading, and the whole-line Boneyard leaves no blank line, so `CUT TO:` sits directly above `INT. B - NIGHT` and is a cue with that line as Dialogue; theirs makes the first heading Action and counts the Boneyard's place as a blank line, so `INT. B - NIGHT` is a heading. The official syntax leaves open whether a span is removed before detection.

````diff
-ACTION The kettle clicks off and nobody moves.
-HEADING INT. A - DAY
-CHARACTER CUT TO:
-DIALOGUE INT. B - NIGHT
+ACTION The kettle clicks off and nobody moves.⏎⏎INT. A - DAY⏎⏎CUT TO:
+HEADING INT. B - NIGHT
````

### off/off-16e.fountain

**Class:** documented-choice · **Ruling:** OFF-16e, OFF-9c; 11.7 row `CUT TO:` forms · **Reference parser:** OFF-16

The scene hash differs only because the elements do (above); it follows the OFF-16e difference, where `INT. B - NIGHT` is Dialogue in ours and a heading in theirs.

````diff
-HASH_V1 42697bf48a7b
+HASH_V1 b7fb49974f7b
````

### off/off-17d.fountain

**Class:** documented-choice · **Ruling:** OFF-17d · **Reference parser:** OFF-17

One- and two-space lines in the title page: ours keeps them as stray lines attached to no key (not in the line form, which writes only keys) and reads `Author` as a key; theirs ends the title page there and makes the rest Action. The official syntax asks for three spaces or a tab to continue a value but leaves open what a less-indented line is.

````diff
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎The lights hold.
+ACTION one-space stray⏎  two-space stray⏎Author: A. Writer⏎⏎The kettle clicks off and nobody moves.⏎⏎The lights hold.
````

### off/off-17e.fountain

**Class:** documented-choice · **Ruling:** OFF-17e, OFF-17b · **Reference parser:** OFF-17

An inline value followed by indented lines: ours keeps both as the value and the title page runs to the blank line; theirs ends the title page after `Contact: A. Writer` and makes the following lines, keys included, Action.

````diff
-TITLE Contact: A. Writer⏎Street line⏎Town line
-TITLE Draft date: 5:30 pm
-TITLE Source: 
-TITLE Credit: Written by
-ACTION The kettle clicks off and nobody moves.⏎⏎The lights hold.
+TITLE Contact: A. Writer
+ACTION Street line⏎   Town line⏎Draft date: 5:30 pm⏎Source:⏎Credit: Written by⏎⏎The kettle clicks off and nobody moves.⏎⏎The lights hold.
````

### off/off-17f.fountain

**Class:** documented-choice · **Ruling:** OFF-17f, OFF-1a · **Reference parser:** OFF-17

Blank lines, one of them two spaces, before the first key: ours reads them as leading trivia and finds the title page; theirs makes the keys Action.

````diff
-TITLE Title: The Kettle
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎The lights hold.
+ACTION Title: The Kettle⏎Author: A. Writer⏎⏎The kettle clicks off and nobody moves.⏎⏎The lights hold.
````

### off/off-1a.fountain

**Class:** documented-choice · **Ruling:** OFF-1a · **Reference parser:** OFF-1

Whitespace-only lines between paragraphs: ours reads them as blank; theirs keeps them as content lines inside the Action.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Rain taps the glass.⏎⏎The lamp flickers twice.⏎⏎The room goes still.⏎⏎The door shuts.
+ACTION The kettle clicks off and nobody moves.⏎⏎Rain taps the glass.⏎ ⏎The lamp flickers twice.⏎    ⏎The room goes still.⏎  ⏎The door shuts.
````

### off/off-1b.fountain

**Class:** documented-choice · **Ruling:** OFF-1a, OFF-1b · **Reference parser:** OFF-1

A one-space line in a Dialogue block: ours reads it as blank, so it ends the block and the next line is Action; theirs keeps it as a line and the Dialogue goes on. Only the two-space line is literal text; what a one-space line is, the official syntax leaves open.

````diff
-DIALOGUE I said no.
-ACTION Not again.
+DIALOGUE I said no.⏎⏎Not again.
````

### off/off-1b.fountain

**Class:** documented-choice · **Ruling:** OFF-1a, OFF-1b · **Reference parser:** OFF-1

A three-space line in a Dialogue block: ours reads it as blank, so it ends the block and the next line is Action; theirs keeps it as a line and the Dialogue goes on. What a whitespace-only line other than two spaces is, the official syntax leaves open.

````diff
-DIALOGUE One more.
-ACTION Three spaces.
+DIALOGUE One more.⏎⏎Three spaces.
````

### off/off-1b.fountain

**Class:** documented-choice · **Ruling:** OFF-1a, OFF-1b · **Reference parser:** OFF-1

A tab-only line in a Dialogue block: ours reads it as blank, so it ends the block and the next line is Action; theirs keeps it as a line and the Dialogue goes on. What a tab-only line is, the official syntax leaves open.

````diff
-DIALOGUE Last.
-ACTION A tab.
+DIALOGUE Last.⏎⏎A tab.
````

### off/off-1c.fountain

**Class:** documented-choice · **Ruling:** OFF-15b, OFF-15g · **Reference parser:** OFF-15

The same as off-15g: ours keeps `[[This note` with no closer before the blank line as plain text; theirs closes it at the blank line and drops it. The two-space line inside the first Note (OFF-1c) agrees.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. [[This note⏎⏎ends at the blank line.]] The curtain stirs.
+ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. ⏎ends at the blank line.]] The curtain stirs.
````

### off/off-20c.fountain

**Class:** documented-choice · **Ruling:** OFF-20c · **Reference parser:** OFF-20

A ten-`#` Section: ours has depth 10; theirs stops at depth 7 and puts the other three `#` in the text. The official syntax gives no maximum depth, so a cap is open; SPEC.md sets none.

````diff
-SECTION 10 Ten
+SECTION 7 ### Ten
````

### off/off-20e.fountain

**Class:** documented-choice · **Ruling:** OFF-21b, OFF-20e · **Reference parser:** OFF-20, OFF-21

A Section between two Action lines: ours keeps one Action across it (the Section is an interjection); theirs splits the Action around it. The flattener writes our Section after the whole Action, so the order of the lines is its own.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The light dims.⏎Mara fills the kettle.
+ACTION The kettle clicks off and nobody moves.⏎⏎The light dims.
````

### off/off-20e.fountain

**Class:** documented-choice · **Ruling:** OFF-21b, OFF-20e · **Reference parser:** OFF-20, OFF-21

The second half of the same split: theirs makes `Mara fills the kettle.` its own Action after the Section; ours has it inside the one Action above.

````diff
+ACTION Mara fills the kettle.
````

### off/off-21a.fountain

**Class:** documented-choice · **Ruling:** OFF-21a · **Reference parser:** OFF-21

A Section inside a Dialogue block: ours continues the block, so `I swear.` is Dialogue; theirs makes it Action.

````diff
-DIALOGUE I swear.
+ACTION I swear.
````

### off/off-21b.fountain

**Class:** documented-choice · **Ruling:** OFF-21b · **Reference parser:** OFF-21

A Synopsis between two Action lines: ours keeps one Action across it; theirs splits the Action around it. The order of the lines is the flattener's.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The kettle ticks as it cools.⏎Nobody speaks.
+ACTION The kettle clicks off and nobody moves.⏎⏎The kettle ticks as it cools.
````

### off/off-21b.fountain

**Class:** documented-choice · **Ruling:** OFF-21b · **Reference parser:** OFF-21

The second half of the same split: theirs makes `Nobody speaks.` its own Action after the Synopsis; ours has it inside the one Action above.

````diff
+ACTION Nobody speaks.
````

### off/off-21c.fountain

**Class:** documented-choice · **Ruling:** OFF-21b, OFF-21c · **Reference parser:** OFF-21

A Section between two Action lines: ours keeps one Action across it (the Section is transparent); theirs splits the Action around it. The order of the lines is the flattener's.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits by the door.⏎She opens it at last.
+ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits by the door.
````

### off/off-21c.fountain

**Class:** documented-choice · **Ruling:** OFF-21b, OFF-21c · **Reference parser:** OFF-21

The second half of the same split: theirs makes `She opens it at last.` its own Action after the Section; ours has it inside the one Action above.

````diff
+ACTION She opens it at last.
````

### off/off-22a.fountain

**Class:** theirs-wrong · **Ruling:** OFF-22a, OFF-18b, OFF-22d · **Reference parser:** OFF-22

`  !indented action`: both force Action and drop the `!`; ours keeps the two spaces before it, as Action text keeps its leading whitespace (4.3), and theirs drops them.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎  indented action
+ACTION The kettle clicks off and nobody moves.⏎⏎indented action
````

### off/off-23a.fountain

**Class:** theirs-wrong · **Ruling:** OFF-23a · **Reference parser:** OFF-23

`CUT TO: ` with a trailing space: ours makes it an Action of its own (the line form's Action merge joins it with the Actions around it); theirs makes it a Transition. The official syntax says spaces after the colon make Action.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎CUT TO:⏎⏎Mara sets the cup down.
+ACTION The kettle clicks off and nobody moves.
+TRANSITION CUT TO:
+ACTION Mara sets the cup down.
````

### off/off-24c.fountain

**Class:** documented-choice · **Ruling:** OFF-24c · **Reference parser:** OFF-24

A Lyrics line, Section, Synopsis and Page Break inside a Dialogue block: ours continues the block, so `Then I will go.` is Dialogue; theirs makes it Action.

````diff
-DIALOGUE Then I will go.
+ACTION Then I will go.
````

### off/off-24d.fountain

**Class:** documented-choice · **Ruling:** OFF-24d, OFF-15e · **Reference parser:** OFF-24

A whole-line Note and Boneyard inside a Dialogue block: ours hides both lines and the Dialogue goes on with no gap; theirs leaves an empty line for each.

````diff
-DIALOGUE I will stay.⏎Then I will go.
+DIALOGUE I will stay.⏎⏎⏎Then I will go.
````

### off/off-2c.fountain

**Class:** documented-choice · **Ruling:** OFF-2c · **Reference parser:** OFF-2 (not determined there)

A leading byte-order mark: ours keeps it as trivia, out of the text; theirs keeps it in the first Action's text (the script reads the file as plain UTF-8, so the mark reaches the parser).

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara lifts the cup and drinks.
+ACTION ﻿The kettle clicks off and nobody moves.⏎⏎Mara lifts the cup and drinks.
````

### off/off-2d.fountain

**Class:** documented-choice · **Ruling:** OFF-2d · **Reference parser:** OFF-2

U+0085, U+2028, U+2029, vertical tab and form feed: ours reads them as ordinary characters inside a line; theirs breaks the line at each.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara readsthe note aloud.⏎The page turns and she stops.⏎Her voice drops to a whisper.⏎The tape rollsback.⏎The screen flickersonce.
+ACTION The kettle clicks off and nobody moves.⏎⏎Mara reads⏎the note aloud.⏎The page turns⏎and she stops.⏎Her voice drops⏎to a whisper.⏎The tape rolls⏎back.⏎The screen flickers⏎once.
````

### off/off-3c.fountain

**Class:** documented-choice · **Ruling:** OFF-3c · **Reference parser:** OFF-3

`MARA ((HI))`, a nested group: ours does not make it a cue, so it and its line stay Action; theirs makes it a cue with Dialogue. The other three lines are Action in both.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎MARA (HI) THERE⏎Text after a group, so this is not a cue.⏎⏎MARA (HI⏎The group is never closed.⏎⏎MARA ((HI))⏎The group is nested.⏎⏎MARA (HI) (THERE⏎The second group is never closed.
+ACTION The kettle clicks off and nobody moves.⏎⏎MARA (HI) THERE⏎Text after a group, so this is not a cue.⏎⏎MARA (HI⏎The group is never closed.
+CHARACTER MARA ((HI))
+DIALOGUE The group is nested.
+ACTION MARA (HI) (THERE⏎The second group is never closed.
````

### off/off-4a.fountain

**Class:** documented-choice · **Ruling:** OFF-4a · **Reference parser:** OFF-4

Cyrillic and Greek uppercase names: ours makes `ЕЛЕНА` and `ΣΩΚΡΑΤΗΣ` cues; theirs makes them Action. Both keep `JOSÉ` a cue and `елена` Action.

````diff
-CHARACTER ЕЛЕНА
-DIALOGUE Доброй ночи.
-CHARACTER ΣΩΚΡΑΤΗΣ
-DIALOGUE Καλησπέρα.
-ACTION елена⏎All lowercase Cyrillic is not a cue.
+ACTION ЕЛЕНА⏎Доброй ночи.⏎⏎ΣΩΚΡΑΤΗΣ⏎Καλησπέρα.⏎⏎елена⏎All lowercase Cyrillic is not a cue.
````

### off/off-5b.fountain

**Class:** documented-choice · **Ruling:** OFF-1a, OFF-5b · **Reference parser:** OFF-1, OFF-5

`ELI` over a one-space line: both make `ELI` Action (OFF-5b agrees); ours reads the one-space line as blank, theirs keeps it as a content line in the Action.

````diff
-ACTION ELI⏎⏎He waits.⏎⏎OTTO⏎⏎He waits too.
+ACTION ELI⏎ ⏎He waits.⏎⏎OTTO⏎⏎He waits too.
````

### off/off-6a.fountain

**Class:** theirs-wrong · **Ruling:** OFF-6a; 11.7 row `OFF-6` · **Reference parser:** OFF-6

Two headings missing a blank line, one before and one after: ours keeps both in the Action; theirs makes both headings, which also adds two scenes and their hashes.

````diff
-ACTION Mara waits by the stove.⏎⏎The street outside is empty.⏎EXT. STREET - NIGHT⏎⏎Rain falls on the cars.⏎⏎INT. HALLWAY - DAY⏎Footsteps cross the floor.
+ACTION Mara waits by the stove.⏎⏎The street outside is empty.
+HASH_V1 563a5c1bce38
+HEADING EXT. STREET - NIGHT
+ACTION Rain falls on the cars.
+HASH_V1 d933327fdc68
+HEADING INT. HALLWAY - DAY
+ACTION Footsteps cross the floor.
````

### off/off-6a.fountain

**Class:** theirs-wrong · **Ruling:** OFF-6a, OFF-24a; 11.7 row `OFF-6` · **Reference parser:** OFF-6

`INT. STAIRS - DAY` inside a Dialogue block: ours keeps it and the next line as Dialogue; theirs makes it a heading. The hash lines follow from that.

````diff
-DIALOGUE Is anyone there?⏎INT. STAIRS - DAY⏎Nobody answers.
-ACTION The attic door creaks.
-HASH_V1 8cf6bd449e4f
+DIALOGUE Is anyone there?
+HASH_V1 518857e149ec
+HEADING INT. STAIRS - DAY
+ACTION Nobody answers.⏎⏎The attic door creaks.
+HASH_V1 182582e283cc
````

### off/off-6a.fountain

**Class:** documented-choice · **Ruling:** OFF-6a, OFF-15e; 11.7 row `OFF-6` · **Reference parser:** OFF-15

`INT. CELLAR - DAY` under a whole-line Note under text: ours skips the invisible Note line, finds no blank line before, and keeps it Action; theirs counts the Note's gap as a blank line and makes it a heading. The hash lines follow from that.

````diff
-ACTION Dust hangs in the light.⏎Footsteps stop.⏎INT. CELLAR - DAY⏎⏎Water drips.
-HASH_V1 9495b7195a51
+ACTION Dust hangs in the light.⏎Footsteps stop.
+HASH_V1 8b81294fb5ca
+HEADING INT. CELLAR - DAY
+ACTION Water drips.
+HASH_V1 41326aff40d9
````

### off/off-6b.fountain

**Class:** documented-choice · **Ruling:** OFF-6b; 11.7 row `OFF-6` · **Reference parser:** OFF-6

`INT<tab>HOUSE`: ours makes it Action, since only a period or a space may follow the prefix; theirs takes the tab as a separator and makes it a heading. The official syntax does not address a tab after the prefix. The hash lines follow from that.

````diff
-ACTION Mara stands in the doorway.⏎⏎INT    HOUSE⏎⏎She does not move.⏎⏎INT-HOUSE⏎⏎The light stays off.⏎⏎INTERIOR⏎⏎Nothing happens.
-HASH_V1 11958d3e2324
+ACTION Mara stands in the doorway.
+HASH_V1 bbc91cde19aa
+HEADING INT	HOUSE
+ACTION She does not move.⏎⏎INT-HOUSE⏎⏎The light stays off.⏎⏎INTERIOR⏎⏎Nothing happens.
+HASH_V1 4bfaa0248489
````

### off/off-6d.fountain

**Class:** documented-choice · **Ruling:** OFF-6d; 11.7 row `OFF-6` · **Reference parser:** OFF-6

`INT ` with a trailing space: ours makes it a heading with the text `INT`; theirs makes it Action. The official syntax leaves open a prefix followed by a line end, and what trailing whitespace counts for. The hash lines follow from that.

````diff
-ACTION A bare prefix with a period.
-HASH_V1 04bc32c86866
-HEADING INT
-ACTION A bare prefix with a space.
-HASH_V1 957c99895ef4
+ACTION A bare prefix with a period.⏎⏎INT⏎⏎A bare prefix with a space.
+HASH_V1 9d73c00f8a2d
````

### off/off-7a.fountain

**Class:** documented-choice · **Ruling:** OFF-7a (11.7 has no row for OFF-7) · **Reference parser:** OFF-7

`.ÉPILOGUE`: ours forces a heading (a Unicode letter is alphanumeric); theirs makes it Action with the period. The hash lines follow from that.

````diff
-HASH_V1 81ce394cd321
-HEADING ÉPILOGUE
-HASH_V1 1078bc372abf
+ACTION .ÉPILOGUE
+HASH_V1 cde919eedc99
````

### off/off-8a.fountain

**Class:** documented-choice · **Ruling:** OFF-8a (11.7 has no row for OFF-8) · **Reference parser:** OFF-8

`#É1#`: ours takes it as the scene number; theirs leaves it in the heading text. Scheme 1 leaves the number out, so the heading line and the hash differ.

````diff
-HEADING INT. HOUSE - DUSK #É1
-HASH_V1 9b7240482b64
+HEADING INT. HOUSE - DUSK #É1#
+HASH_V1 019035066005
````

### off/off-8c.fountain

**Class:** documented-choice · **Ruling:** OFF-8c (11.7 has no row for OFF-8) · **Reference parser:** OFF-8

`MORNING#7#` with no space before the marks: ours leaves `#7#` in the heading text, because OFF-8c asks for a space or tab before the number; theirs takes it as the number. The official syntax only says the number is appended, so the space is this project's choice. The hash follows from the heading text.

````diff
-HEADING INT. HOUSE - MORNING#7#
-HASH_V1 8dd7fe3c1daa
+HEADING INT. HOUSE - MORNING #7
+HASH_V1 acc0dc1be6a5
````

### off/off-8g.fountain

**Class:** documented-choice · **Ruling:** OFF-8g, OFF-8c (11.7 has no row for OFF-8) · **Reference parser:** not covered

`##12A#` without the `locked-scene-numbers` feature: ours keeps it as heading text, because no space or tab stands before the last `#` pair; theirs takes `12A` as the number and leaves a `#` in the text. The space-or-tab requirement is OFF-8c's own, not the official syntax's. The hash follows from the heading text.

````diff
-HEADING INT. HOUSE - DAY ##12A#
-HASH_V1 99eb4b7408b0
+HEADING INT. HOUSE - DAY # #12A
+HASH_V1 8ad90dc8393d
````

### off/off-9a.fountain

**Class:** theirs-wrong · **Ruling:** OFF-9a, OFF-6b; 11.7 row `CUT TO:` forms · **Reference parser:** OFF-9

`FADE OUT.`, `FADE IN:`, `FADE TO BLACK.` and `CUT TO BLACK.`: ours keeps all four Action; theirs keeps three Action but makes `FADE IN:` a scene heading, which no heading prefix allows, and so adds a scene and its hash.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎FADE OUT.⏎⏎FADE IN:⏎⏎FADE TO BLACK.⏎⏎CUT TO BLACK.
+ACTION The kettle clicks off and nobody moves.⏎⏎FADE OUT.
+HEADING FADE IN:
+ACTION FADE TO BLACK.⏎⏎CUT TO BLACK.
+HASH_V1 05e94db8f535
````

### off/off-9c.fountain

**Class:** theirs-wrong · **Ruling:** OFF-9c, OFF-6a; 11.7 row `CUT TO:` forms · **Reference parser:** OFF-9

`CUT TO:` directly above `INT. HOUSE - DAY`: ours makes a cue whose Dialogue is the heading line; theirs makes `CUT TO:` Action and the next line a heading, though it has no blank line before it.

````diff
-ACTION The kettle clicks off and nobody moves.
-CHARACTER CUT TO:
-DIALOGUE INT. HOUSE - DAY
+ACTION The kettle clicks off and nobody moves.⏎⏎CUT TO:
+HEADING INT. HOUSE - DAY
````

### off/off-9c.fountain

**Class:** theirs-wrong · **Ruling:** OFF-9c, OFF-6a; 11.7 row `CUT TO:` forms · **Reference parser:** OFF-9

The scene hash differs only because the elements do (above); theirs has a scene for the heading that ours reads as Dialogue.

````diff
+HASH_V1 d1d78fb2ce92
````

### twopop/blue_rev2.fountain

**Class:** documented-choice · **Ruling:** OFF-15e · **Reference parser:** OFF-15

A whole-line Note between two Action paragraphs: ours hides it and the paragraphs join once; theirs leaves an empty paragraph where it was. The scene hashes agree.

````diff
-ACTION Rows of steel shelving. A caged bulb flickers, then steadies.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
+ACTION Rows of steel shelving. A caged bulb flickers, then steadies.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
````

### twopop/two_scene.fountain

**Class:** documented-choice · **Ruling:** OFF-15e · **Reference parser:** OFF-15

A whole-line Note between two Action paragraphs: ours hides it and the paragraphs join once; theirs leaves an empty paragraph where it was. The scene hashes agree.

````diff
-ACTION Rows of steel shelving. A single caged bulb swings.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
+ACTION Rows of steel shelving. A single caged bulb swings.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
````
