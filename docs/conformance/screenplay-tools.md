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
  scene. A line break inside a text is written U+23CE. Text with its note and boneyard markers removed and each line
  trimmed; a text element that is then empty is dropped, but `PAGEBREAK`, `CHARACTER` and `HEADING` never are. Our
  `HASH_V1` is the first 12 hex digits of the SHA-256 of `Scene::normalized_text_v1()`.
- **The diff.** A longest-common-subsequence line diff of ours against theirs per file. Adjacent changes form one hunk.
  An entry shows `-` for our lines and `+` for theirs.

## What we change on our side, and what that hides

Each change makes our output match screenplay-tools' representation. A difference hidden by one of them is not seen.

- **Trimmed lines, tabs.** Each line of a text is trimmed with Rust's `trim`, and an Action's tabs become four spaces
  (`Action::display`, `OFF-18c`). This hides only indentation, which our own tests cover. Rust's `trim` and Python's
  `strip` differ on U+0085, U+2028 and the separators U+001C to U+001F; no corpus file has one today.
- **Emphasis.** The markers are written back from `Run::Emphasis`: `***` for bold and italic, `**` for bold, `*` for
  italic, `_` for underline (outermost). `Run::Escaped(c)` is written `\c`. This hides nothing.
- **Notes and boneyards.** `Run::Note`, `Run::Boneyard` and whole-line `Element::Note` and `Element::Boneyard` are
  left out, because screenplay-tools removes their markers the same way. This hides the notes' own text and position;
  their effect on the elements around them stays visible.
- **Merged Actions and Dialogue.** Consecutive non-centered Actions are joined with U+23CE twice, also across a
  left-out Note or Boneyard, as screenplay-tools merges them. Consecutive `DialogueItem::Line`s are joined with
  U+23CE. This hides only where one Action paragraph ends and the next starts; it is the same merge as `SPEC.md`
  11.7 rule 4.
- **Cues.** Our `(CONT'D)` extensions are dropped, and forcing flags (`@`, `!`, `>`, `.`) are not compared. This hides
  the `OFF-3e` ruling that `(CONT'D)` is kept as written, which `EXT-contd-1` tests: screenplay-tools strips it. Any
  other extension is written ` (<ext>)`, one group each; the script writes one group from the library's single
  extension string.
- **Order.** Our scene's Sections and Synopses that start before its heading come before `HEADING`, the rest after
  it. An Action's interjections follow their Action line.

## The classes

- `theirs-wrong`: screenplay-tools disagrees with the Fountain format or with a ruling of `SPEC.md`.
- `ours-wrong`: this crate is wrong. It is a bug: it needs an issue and a fix, and the entry is removed with the fix.
- `documented-choice`: both are defensible and `SPEC.md` rules for ours.
- `todo`: not classified yet.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-CHARACTER OTTO
+CHARACTER OTTO ^
````

### off/off-11c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-CHARACTER ELI
+CHARACTER ELI ^
````

### off/off-11e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-CHARACTER ELI
+CHARACTER ELI ^
````

### off/off-12a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (softly)
+PAREN softly
````

### off/off-12a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (  a beat  )
+PAREN a beat
````

### off/off-13a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE And then she stops.
+ACTION And then she stops.
````

### off/off-15b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The letter says [[check the key and more text⏎⏎The next paragraph goes on.
+ACTION The kettle clicks off and nobody moves.⏎⏎The letter says ⏎The next paragraph goes on.
````

### off/off-15d.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits  and goes.
+ACTION The kettle clicks off and nobody moves.⏎⏎⏎⏎Mara waits  and goes.
````

### off/off-15e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE I hear it.⏎And I will answer.
-ACTION The lights flicker.⏎The lights hold.⏎⏎She waits.⏎⏎She goes.
+DIALOGUE I hear it.⏎⏎And I will answer.
+ACTION The lights flicker.⏎⏎The lights hold.⏎⏎She waits.⏎⏎⏎⏎She goes.
````

### off/off-15f.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara  waits, then  goes.
+ACTION Author: A. Writer⏎⏎The kettle clicks off and nobody moves.
````

### off/off-15g.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. [[This note⏎⏎ends at the blank line.]] The curtain stirs.⏎⏎The floor creaks.  The step is gone.
+ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. ⏎ends at the blank line.]] The curtain stirs.⏎⏎The floor creaks.  The step is gone.
````

### off/off-16b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎still outer? */
+ACTION The kettle clicks off and nobody moves.⏎⏎ still outer? */
````

### off/off-16c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits. /* an aside that never closes⏎⏎The rest of the file is still here.⏎⏎And so is this last line.
+ACTION The kettle clicks off and nobody moves.
````

### off/off-16e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.
-HEADING INT. A - DAY
-CHARACTER CUT TO:
-DIALOGUE INT. B - NIGHT
+ACTION The kettle clicks off and nobody moves.⏎⏎INT. A - DAY⏎⏎CUT TO:
+HEADING INT. B - NIGHT
````

### off/off-16e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-HASH_V1 42697bf48a7b
+HASH_V1 b7fb49974f7b
````

### off/off-17d.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎The lights hold.
+ACTION one-space stray⏎  two-space stray⏎Author: A. Writer⏎⏎The kettle clicks off and nobody moves.⏎⏎The lights hold.
````

### off/off-17e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-TITLE Title: The Kettle
-TITLE Author: A. Writer
-ACTION The kettle clicks off and nobody moves.⏎⏎The lights hold.
+ACTION Title: The Kettle⏎Author: A. Writer⏎⏎The kettle clicks off and nobody moves.⏎⏎The lights hold.
````

### off/off-18b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara stands at the sink.
+ACTION The kettle clicks off and nobody moves.⏎⏎   Mara stands at the sink.
````

### off/off-18b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (quietly)
+PAREN quietly
````

### off/off-18c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara crosses    the room.⏎⏎She stops at the window.
+ACTION The kettle clicks off and nobody moves.⏎⏎    Mara crosses    the room.⏎⏎        She stops at the window.
````

### off/off-18d.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (low)
+PAREN low
````

### off/off-1a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Rain taps the glass.⏎⏎The lamp flickers twice.⏎⏎The room goes still.⏎⏎The door shuts.
+ACTION The kettle clicks off and nobody moves.⏎⏎Rain taps the glass.⏎ ⏎The lamp flickers twice.⏎    ⏎The room goes still.⏎  ⏎The door shuts.
````

### off/off-1b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE Wait.⏎Do not go yet.
+DIALOGUE Wait.⏎⏎Do not go yet.
````

### off/off-1b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE I said no.
-ACTION Not again.
+DIALOGUE I said no.⏎⏎Not again.
````

### off/off-1b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE One more.
-ACTION Three spaces.
+DIALOGUE One more.⏎⏎Three spaces.
````

### off/off-1b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE Last.
-ACTION A tab.
+DIALOGUE Last.⏎⏎A tab.
````

### off/off-1c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. [[This note⏎⏎ends at the blank line.]] The curtain stirs.
+ACTION The kettle clicks off and nobody moves.⏎⏎The door is locked.  The handle turns.⏎⏎The window is shut. ⏎ends at the blank line.]] The curtain stirs.
````

### off/off-20c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-SECTION 10 Ten
+SECTION 7 ### Ten
````

### off/off-20e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The light dims.⏎Mara fills the kettle.
+ACTION The kettle clicks off and nobody moves.⏎⏎The light dims.
````

### off/off-20e.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
+ACTION Mara fills the kettle.
````

### off/off-21a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE I swear.
+ACTION I swear.
````

### off/off-21b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎The kettle ticks as it cools.⏎Nobody speaks.
+ACTION The kettle clicks off and nobody moves.⏎⏎The kettle ticks as it cools.
````

### off/off-21b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
+ACTION Nobody speaks.
````

### off/off-21c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits by the door.⏎She opens it at last.
+ACTION The kettle clicks off and nobody moves.⏎⏎Mara waits by the door.
````

### off/off-21c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
+ACTION She opens it at last.
````

### off/off-22f.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎@
+ACTION The kettle clicks off and nobody moves.⏎⏎⏎⏎@
````

### off/off-23a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎CUT TO:⏎⏎Mara sets the cup down.
+ACTION The kettle clicks off and nobody moves.
+TRANSITION CUT TO:
+ACTION Mara sets the cup down.
````

### off/off-24a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (beat)
+PAREN beat
````

### off/off-24c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE Then I will go.
+ACTION Then I will go.
````

### off/off-24d.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-DIALOGUE I will stay.⏎Then I will go.
+DIALOGUE I will stay.⏎⏎⏎Then I will go.
````

### off/off-2c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara lifts the cup and drinks.
+ACTION ﻿The kettle clicks off and nobody moves.⏎⏎Mara lifts the cup and drinks.
````

### off/off-2d.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎Mara readsthe note aloud.⏎The page turns and she stops.⏎Her voice drops to a whisper.⏎The tape rollsback.⏎The screen flickersonce.
+ACTION The kettle clicks off and nobody moves.⏎⏎Mara reads⏎the note aloud.⏎The page turns⏎and she stops.⏎Her voice drops⏎to a whisper.⏎The tape rolls⏎back.⏎The screen flickers⏎once.
````

### off/off-3c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎MARA (HI) THERE⏎Text after a group, so this is not a cue.⏎⏎MARA (HI⏎The group is never closed.⏎⏎MARA ((HI))⏎The group is nested.⏎⏎MARA (HI) (THERE⏎The second group is never closed.
+ACTION The kettle clicks off and nobody moves.⏎⏎MARA (HI) THERE⏎Text after a group, so this is not a cue.⏎⏎MARA (HI⏎The group is never closed.
+CHARACTER MARA ((HI))
+DIALOGUE The group is nested.
+ACTION MARA (HI) (THERE⏎The second group is never closed.
````

### off/off-4a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-CHARACTER ЕЛЕНА
-DIALOGUE Доброй ночи.
-CHARACTER ΣΩΚΡΑΤΗΣ
-DIALOGUE Καλησπέρα.
-ACTION елена⏎All lowercase Cyrillic is not a cue.
+ACTION ЕЛЕНА⏎Доброй ночи.⏎⏎ΣΩΚΡΑΤΗΣ⏎Καλησπέρα.⏎⏎елена⏎All lowercase Cyrillic is not a cue.
````

### off/off-5b.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION ELI⏎⏎He waits.⏎⏎OTTO⏎⏎He waits too.
+ACTION ELI⏎ ⏎He waits.⏎⏎OTTO⏎⏎He waits too.
````

### off/off-6a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

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

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-HASH_V1 81ce394cd321
-HEADING ÉPILOGUE
-HASH_V1 1078bc372abf
+ACTION .ÉPILOGUE
+HASH_V1 cde919eedc99
````

### off/off-8a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-HEADING INT. HOUSE - DUSK #É1
-HASH_V1 9b7240482b64
+HEADING INT. HOUSE - DUSK #É1#
+HASH_V1 019035066005
````

### off/off-8c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-HEADING INT. HOUSE - MORNING#7#
-HASH_V1 8dd7fe3c1daa
+HEADING INT. HOUSE - MORNING #7
+HASH_V1 acc0dc1be6a5
````

### off/off-8g.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-HEADING INT. HOUSE - DAY ##12A#
-HASH_V1 99eb4b7408b0
+HEADING INT. HOUSE - DAY # #12A
+HASH_V1 8ad90dc8393d
````

### off/off-9a.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.⏎⏎FADE OUT.⏎⏎FADE IN:⏎⏎FADE TO BLACK.⏎⏎CUT TO BLACK.
+ACTION The kettle clicks off and nobody moves.⏎⏎FADE OUT.
+HEADING FADE IN:
+ACTION FADE TO BLACK.⏎⏎CUT TO BLACK.
+HASH_V1 05e94db8f535
````

### off/off-9c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION The kettle clicks off and nobody moves.
-CHARACTER CUT TO:
-DIALOGUE INT. HOUSE - DAY
+ACTION The kettle clicks off and nobody moves.⏎⏎CUT TO:
+HEADING INT. HOUSE - DAY
````

### off/off-9c.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
+HASH_V1 d1d78fb2ce92
````

### twopop/blue_rev2.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION Rows of steel shelving. A caged bulb flickers, then steadies.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
+ACTION Rows of steel shelving. A caged bulb flickers, then steadies.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
````

### twopop/blue_rev2.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (holding up the ledger)
+PAREN holding up the ledger
````

### twopop/edge_cases.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (quietly)
+PAREN quietly
````

### twopop/two_scene.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-ACTION Rows of steel shelving. A single caged bulb swings.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
+ACTION Rows of steel shelving. A single caged bulb swings.⏎⏎MARA VOSS (40s, cropped grey hair, long oilskin coat) slips through the door, clutching a red leather LEDGER.⏎⏎⏎⏎DESMOND (60s, night guard, cardigan) looks up from his desk.
````

### twopop/two_scene.fountain

**Class:** todo · **Ruling:** ? · **Reference parser:** ?

Unclassified.

````diff
-PAREN (holding up the ledger)
+PAREN holding up the ledger
````
