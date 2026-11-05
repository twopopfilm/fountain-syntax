# The official Fountain syntax: a study

Input to `SPEC.md` (issue #4), from issue #2 (F1.1). This file records what the official syntax says, element by
element, and lists the cases it leaves open. It makes **no rulings**: `SPEC.md` rules on each `OFF-n` case below.

## Source

| Item | Value |
|---|---|
| Document | "Syntax" on fountain.io: <https://fountain.io/syntax> |
| Revision found | **Fountain 1.1, dated 14 March 2014**, the newest entry in the page's own changelog (1.0 is dated 7 February 2012) |
| Access date | **2026-11-04** (the page was read on that day; nothing on it is dated later than 2014) |
| Copyright | The page footer says "© 2023 Quote-Unquote Apps". That is the site footer, not a spec revision. The page has no licence statement. |
| Section links | `https://fountain.io/syntax#<id>`. The ids were read from the page's HTML. The page's own menu mistypes the Character link as `#charater`; the real id is `#character`. The Completed Example section has an id (`#completed-example`) but no menu entry. |

### Own words, original examples

The spec has no licence, so nothing here is copied from it.

- Every rule is **restated in our own words** and credited to its spec section by link.
- Every example is **original**, built with the same structure as the spec's. Only bare syntax tokens are quoted
  (`INT.`, `.`, `!`, `@`, `~`, `>`, `<`, `^`, `===`, `[[ ]]`, `/* */`, `#`, `=`, `*`, `_`, `\`, `TO:`).
- A spec example is **referenced by its section** and never reproduced. The conformance corpus (#15) must use the
  original examples here, or get written permission first.
- Where this file says "the spec's example shows X", it describes what was observed on the page, not quoted text.

### What was checked for a newer revision (2026-11-04)

| Checked | Result |
|---|---|
| The Syntax page and its changelog | Newest entry: 1.1, 14 March 2014. Lists what 1.1 added (`@`, `!`, `~`) and what it deprecated (see below). |
| The Developers, FAQ and How-to pages | No other revision, draft or errata is mentioned. They point back to the Syntax page as the rules. |
| The site's feed | Newest posts are FAQ items from May 2023. No syntax news. |
| The `fountain` organisation on GitHub | One repository (`.github`, last updated April 2023). No spec, draft or syntax repository. |
| The reference code linked from the Developers page (an Objective-C parser and writer) | Its README defers to fountain.io for the format. No spec file, tag or release. |
| A GitHub search of the `fountain` organisation | Same single repository. |

**Conclusion:** 1.1 (2014) is the current official revision. No newer revision or draft was found. The page's
sections are the 21 listed below, in the page's order.

**Method note.** The page was first fetched through a summarising web tool, which was too lossy for a faithful
study. The raw HTML was then downloaded and read in full, so every rule below comes from the complete page text.

### The 1.1 changelog, in brief

- **New in 1.1:** forcing a Character with `@`, forcing Action with `!`, and Lyrics with `~`.
- **Deprecated in 1.1:** the old requirement that Character extensions be uppercase, and the old rule that two
  trailing spaces force an Action line. The changelog files both under "Deprecated". The Character section says
  extensions may be either case, so the reading is: both rules are gone. Parsers written for 1.0 may still apply them.

## Conventions used below

- "Empty line" means the spec's blank line. The spec never defines it precisely: see OFF-1.
- "Block" means a run of consecutive non-empty lines.
- "Forced" means the writer overrides detection with a leading sigil. Whether the sigil is removed from the element's
  text is stated only for some sigils (see the forced-elements table).
- `OFF-n` refers to the ambiguous case with that number in [Ambiguous cases](#ambiguous-cases).
- Examples use `<2sp>` on a line to mean a line that holds exactly two spaces.

## Elements

### Overview

[Spec: Overview](https://fountain.io/syntax#overview)

- States the guiding principle: a Fountain file should look like a screenplay when read as plain text.
- Gives five basic rules: Scene Headings start with a heading prefix, Character names are uppercase, Dialogue
  follows Character, Parentheticals are in parentheses, Transitions end in `TO:`.
- Names the "power user" features: forcing a Scene Heading with a period, forcing a Transition with `>`, and the
  three emphasis styles with their combinations.
- Introduces nothing that the later sections do not repeat. It is a summary, not a separate rule set.
- The only rule worth noting here: the Overview describes Transitions as "end in TO:" without the uppercase and
  blank-line requirements that the Transition section adds. The Transition section is the fuller statement.

### Scene Headings

[Spec: Scene Headings](https://fountain.io/syntax#scene-headings)

**Recognition.** A line is a Scene Heading when all of these hold:

1. It begins with one of the prefixes `INT`, `EXT`, `EST`, `INT./EXT`, `INT/EXT` or `I/E`.
2. The prefix is followed by a dot or a space.
3. At least one empty line comes before it, and an empty line follows it.
4. It is not forced to Action with a leading `!`.

Matching is **case-insensitive**. Uppercase is recommended for readability but not required.

**Forced form.** A single leading `.` forces any line to be a Scene Heading. The `.` is removed from the formatted
output. The spec says only a *single* period followed by an *alphanumeric* character forces a heading. That keeps
lines that start with an ellipsis (`...`) out of the heading rule.

**Scene numbers.** A heading may end with a scene number wrapped in `#`. The characters allowed are alphanumerics
plus dashes and periods. The spec's valid forms (see its section) include digits only, digits and letters in either
order, mixed case, dash-separated groups, a trailing period, and a number after a long heading with a parenthesised
note.

**Follows / interactions.**

- A heading is followed by an empty line, then Action, Character or anything else.
- The `!` forcing mark beats the prefix rule.
- A forced heading needs no prefix at all.
- Uppercase, blank-line-framed lines that end in `TO:` are Transitions, not headings. A leading `.` turns such a
  line into a heading instead (see Transition).

**Original example.**

```fountain
INT. LIGHTHOUSE KEEPER'S ROOM - NIGHT #4B#

MARA winds the clock.

.THE LAMP ROOM

A beam sweeps the sea.

...and the storm answers.
```

The first line is a heading with scene number `4B`. `.THE LAMP ROOM` is a forced heading. The last line starts with
three periods, so it stays Action.

Open points: OFF-6, OFF-7, OFF-8, OFF-22.

### Action

[Spec: Action](https://fountain.io/syntax#action)

**Recognition.** Action is any paragraph that does not match another element. Every line break is taken as intended:
single and double spacing are both kept. Any number of empty lines is passed through as empty Action lines.

**Whitespace.** Leading tabs and spaces are **kept** in Action, so a writer can indent by hand. Tabs are converted to
four spaces. When an Action line is centered (see Centered Text), its leading spaces are not kept.

**Forced form.** A leading `!` forces Action. It is for uppercase Action that sits directly above another line and
would otherwise read as a Character cue plus Dialogue.

**Follows / interactions.**

- Action is the fallback: when in doubt, the spec returns text as Action (see Error Handling).
- A forced heading, Character or Transition is the way to leave the fallback. `!` is the way back into it.
- Adding spaces after a `TO:` line's colon turns a would-be Transition into Action (see Transition).

**Original example.**

```fountain
MARA reads the dial.

    11
    14
    Zero.

!KNOCK KNOCK
Nobody there.
```

The indented lines are Action with their indentation kept. `!KNOCK KNOCK` would be read as a Character cue and the
next line as its Dialogue; the `!` forces it to Action.

Open points: OFF-1, OFF-18, OFF-22.

### Character

[Spec: Character](https://fountain.io/syntax#character)

**Recognition.** A Character cue is a line that is entirely uppercase, has one empty line before it, and has no
empty line after it. It must contain at least one alphabetical character: a name like `R2D2` qualifies, a bare
number does not.

**Extensions.** Parenthetical notes after the name on the same line (the spec calls them "Character Extensions") may
be in uppercase or lowercase. The spec's own examples show an extension with a space inside it and one in plain
lowercase words.

**Forced form.** A leading `@` forces a Character cue. It is meant for names that need lowercase letters and for
non-Roman scripts. The `@` is removed and the name keeps its case.

**Follows / interactions.**

- Dialogue (or a Parenthetical) follows on the next line, with no empty line between.
- An uppercase line with an empty line after it is not a Character cue.
- A trailing `^` makes the cue the second half of a dual dialogue (see Dual Dialogue).
- `!` before an uppercase line keeps it as Action.

**Original example.**

```fountain
OTTO
Put the kettle on.

NELL (on the phone)
It's already on.

@deLaCruz
Then it's ready.
```

Open points: OFF-3, OFF-4, OFF-5, OFF-22, OFF-23.

### Dialogue

[Spec: Dialogue](https://fountain.io/syntax#dialogue)

**Recognition.** Dialogue is any text that follows a Character cue or a Parenthetical. Manual line breaks inside
Dialogue are kept. The spec also allows intentionally empty lines inside Dialogue; to make one, the line must hold two
spaces so that it is not a real empty line (see Line Breaks).

**Forced form.** None. Dialogue is whatever follows its cue.

**Follows / interactions.**

- Dialogue ends at an empty line. Whether a line of only spaces ends it is the question in OFF-1.
- A Parenthetical may sit between two stretches of Dialogue.
- Emphasis applies inside Dialogue.

**Original example.**

```fountain
MARA
The tide is in.
_Everything_ is in.
```

Open points: OFF-1, OFF-13, OFF-24.

### Parenthetical

[Spec: Parenthetical](https://fountain.io/syntax#parenthetical)

**Recognition.** A Parenthetical is wrapped in parentheses and follows a Character cue or a Dialogue line, directly
(no empty line between).

**Forced form.** None.

**Follows / interactions.** Dialogue follows it. It may appear again after Dialogue lines. A Character cue's own
extension (the parentheses after the name) is a different thing and stays on the cue line.

**Original example.**

```fountain
OTTO
(whispering)
We should not be here.
(louder)
Run.
```

Open points: OFF-12.

### Dual Dialogue

[Spec: Dual Dialogue](https://fountain.io/syntax#dual-dialogue)

**Recognition.** A caret `^` placed after the **second** speaker's Character cue marks the two dialogue blocks as
simultaneous. Any number of spaces may sit between the name and the caret and are ignored. The caret must be the last
character on the line.

**Forced form.** The caret is itself the marker. It combines with the other Character rules: the cue is a normal
(or `@`-forced) Character line.

**Follows / interactions.** The first speaker's block comes first with an ordinary cue. The second block's cue ends
with `^`. Nothing in the section covers a third speaker or a caret with no earlier block.

**Original example.**

```fountain
NELL
I'll go left.

OTTO   ^
I'll go right.
```

Open points: OFF-11, OFF-23.

### Lyrics

[Spec: Lyrics](https://fountain.io/syntax#lyrics)

**Recognition.** A Lyric is a line that starts with `~`. The changelog states the tilde goes on **each** line. The
`~` is removed, and the spec leaves styling to the application. Lyrics are **always forced**: there is no automatic
detection.

**Follows / interactions.** The section's example puts consecutive lyric lines together as a block. It does not say
how Lyrics relate to Dialogue, or whether blank lines are needed around them.

**Original example.**

```fountain
~Down by the harbour, the lamps are low,
~Down by the harbour, the lanterns glow.
```

Open points: OFF-13, OFF-22, OFF-24.

### Transition

[Spec: Transition](https://fountain.io/syntax#transition)

**Recognition.** A Transition is a line that is all uppercase, ends in `TO:`, has an empty line before it, and has
an empty line after it.

**Forced form.** A leading `>` forces any line to be a Transition. The spec does not say whether the `>` is dropped from
the text (it says so for `.`, `@` and `~`); see OFF-22.

**Turning it off.** If a line meets the Transition rules but should be something else, the spec gives two options:
a leading `.` makes it a Scene Heading, or one or more spaces after the colon make it Action, because the line no
longer ends with the colon.

**Follows / interactions.**

- A forced Transition and Centered Text both begin with `>`; centered text also ends with `<` (see Centered Text).
- A line that does not end in `TO:` is not an automatic Transition, whatever it says. A forced `>` is the only way.

**Original example.**

```fountain
MARA closes the book.

DISSOLVE TO:

EXT. HARBOUR - DAWN

Boats rock in place.

>Fade to grey.
```

The first transition is automatic. The last line is forced by `>` and may be mixed case.

Open points: OFF-9, OFF-10, OFF-22, OFF-23.

### Centered Text

[Spec: Centered Text](https://fountain.io/syntax#centered-text)

**Recognition.** Text wrapped in `>` and `<` is centered. It is an **Action** element. Leading spaces are normally
kept in Action but not here, so spaces may be added between the text and the `>`/`<` marks.

**Forced form.** The `>` ... `<` pair is the marker. Emphasis can sit inside it (the Page Breaks section's example
shows bold inside centered text).

**Follows / interactions.** Because the line starts with `>`, a parser must tell it from a forced Transition by the
closing `<` (OFF-10). The spec does not say whether the marks are dropped from the text (OFF-10).

**Original example.**

```fountain
>FIN<

> **ACT TWO** <
```

Open points: OFF-10, OFF-14.

### Emphasis

[Spec: Emphasis](https://fountain.io/syntax#emphasis)

**Recognition.** The spec says to follow Markdown's emphasis rules, with one change: underscores mean underline, not
italics, because the two are different in screenplays.

| Source | Meaning |
|---|---|
| `*text*` | italics |
| `**text**` | bold |
| `***text***` | bold italics |
| `_text_` | underline |

Styles combine and nest, for example an underlined phrase containing an italic word.

**Escapes.** A backslash before an emphasis character makes it literal, as in Markdown. The spec's example wraps an
escaped asterisk pair inside real bold.

**Spacing.** As in Markdown, the spaces around an emphasis mark matter. An asterisk with a space on its left cannot
open italics. The spec shows that two asterisks that each have a space on their left do not italicise the text between
them, but that a pair where the first is flush against the following word and the second is flush against the
preceding word does. It advises escaping one or both marks to avoid accidental italics.

**Line breaks.** As in Markdown, emphasis is not carried across line breaks: an asterisk on one line and another on the
next stay as asterisks. This is a different rule from Notes and Boneyard (see OFF-14, OFF-15, OFF-16).

**Original example.**

```fountain
She types *the code*, then **hits** ENTER, then ***waits***.
A _long, quiet_ moment. Press \*9\* to continue.
He counted *4 and then 7, and kept on.
Nothing was italic above. *This line* is.
```

Line 1 and the underline in line 2 are emphasis; `\*9\*` shows literal asterisks; the third line has an
unmatched asterisk that stays literal.

Open points: OFF-14.

### Title Page

[Spec: Title Page](https://fountain.io/syntax#title-page)

**Recognition.** The optional title page is the first thing in the file. It is a list of `key: value` pairs. A key may
contain spaces but must end with a colon.

**Values.** A value may follow the key on the same line, or sit on the lines below, indented by **three or more spaces
or a tab**. Indented lines allow several values for one key (several authors, several address lines).

**Keys.**

- The spec's example uses Title, Credit, Author (or Authors, both accepted), Source, Draft date and Contact.
- It recommends Title, Credit, Author/Authors and Source be centered on the page, with Contact and Draft date at the
  lower left.
- Other keys are allowed. Unsupported ones are ignored for formatting but may be kept as metadata.
- Every key is optional; a title page of one key is valid.

**Ending.** A page break is implicit after the title page. The spec's advice is to leave a gap of two lines and start
the screenplay. It does not define precisely where the title page stops.

**Forced form.** None.

**Follows / interactions.** Values can carry emphasis (the spec's example styles a title). No other element rule
applies inside the title page.

**Original example.**

```fountain
Title:
    _**THE LAST FERRY**_
Credit: A short film by
Author: A. Example
Draft date: 2026-01-05
Contact:
    Example Pictures
    1 Sample Street
    Anytown

INT. FERRY TERMINAL - NIGHT
```

Open points: OFF-17.

### Page Breaks

[Spec: Page Breaks](https://fountain.io/syntax#page-breaks)

**Recognition.** A line of **three or more consecutive equals signs** and nothing else is a page break. The spec says
it is useful for act breaks in television scripts and for a "vanity" opening page.

**Forced form.** None.

**Follows / interactions.** The marker is the same character as the Synopsis prefix `=`; a longer run is a break and a
single `=` followed by text is a synopsis (see Sections and Synopses). The spec's example puts centered bold text on
each side of the break.

**Original example.**

```fountain
>**END OF PART ONE**<

===

>**PART TWO**<
```

Open points: OFF-19 (the three-or-more rule itself is explicit).

### Punctuation

[Spec: Punctuation](https://fountain.io/syntax#punctuation)

- Fountain does **no typographic conversion**: no curly quotes, no em-dash from `---`, no ellipsis from `...`. Text
  appears as typed, following the typewriter convention.
- Practical effect for a parser: keep the characters of the source untouched. There is nothing to decode.
- The spec presents this in contrast to Markdown processors that do convert such shorthands.

No example needed. There are no open points here. (OFF-7 is about the `.` forcing rule, not conversion.)

### Line Breaks

[Spec: Line Breaks](https://fountain.io/syntax#line-breaks)

**Rule.** Every line break is intended. Single and double spacing in Action are preserved as written.

**Failure case.** An uppercase line directly above another line looks like a Character cue and Dialogue, even when it
is Action. The fix is a leading `!`. After the fix, the surrounding heading and Action lines read as intended.

**Dialogue breaks.** Consecutive lines in a Dialogue block form one continuous block with forced line breaks. To keep a
real gap inside Dialogue, the "empty" line must contain **two spaces**. A truly empty line would end the Dialogue, and
the next line would be read as Action.

**Original example.**

```fountain
OTTO
One.
Two.
<2sp>
Three, and out.

NELL
Done.
```

Without the two spaces on the third line, `Three, and out.` would be Action.

Open points: OFF-1, OFF-2, OFF-23.

### Indenting

[Spec: Indenting](https://fountain.io/syntax#indenting)

**Rule.** Leading tabs and spaces are **ignored** for every element **except Action**. A writer can indent to make a
file look like a formatted page. The spec's prose describes tabs, while its example block as served uses spaces
(see OFF-18). The indent levels are four for Transitions, three for Character cues, two for Parentheticals and one
for Dialogue.

**Exception.** In Action, leading tabs and spaces are kept (see Action).

**Key statement.** Tabs do not hint at formatting. A line is read as if its leading whitespace were absent.

**Original example.**

```fountain
                    DISSOLVE TO:

INT. GALLEY - NIGHT

            OTTO
        (quietly)
    Pass the salt.
```

Open points: OFF-18.

### Notes

[Spec: Notes](https://fountain.io/syntax#notes)

**Recognition.** A Note is text wrapped in double brackets `[[ ... ]]`. It can sit on its own line or inside a line.
The empty lines around a Note on its own line are removed when parsing.

**Multi-line.** A Note may contain line breaks. To put an empty line inside a Note, the line must hold two spaces, to
"connect" the Note into one element.

**Compatibility.** Notes are designed to match the inserted annotations common in screenwriting software. To comment
out a larger stretch of text, use Boneyard.

**Forced form.** None.

**Follows / interactions.** A Note can sit mid-line without ending the surrounding element (the spec's example puts
one inside an Action paragraph). See OFF-15 for nesting and unclosed Notes,
and Error Handling for the limit on how far a closing mark is searched.

**Original example.**

```fountain
MARA checks the chart.[[Is this the right harbour?]]

[[Cut this paragraph if the budget drops.]]

OTTO
Starboard.
```

A longer Note that includes an empty line:

```fountain
She hesitates.[[Rework this beat.
<2sp>
Check timing.]] Then goes.
```

Open points: OFF-1, OFF-15.

### Boneyard

[Spec: Boneyard](https://fountain.io/syntax#boneyard)

**Recognition.** Text wrapped in `/* ... */` is ignored: it does not appear in formatted output. The spec's
example comments out a whole scene.

**Spanning.** The boneyard is the **exception** to the rule that markup does not carry across line breaks. Its pair
can cover as much of the file as the writer likes, including empty lines.

**Forced form.** None.

**Original example.**

```fountain
OTTO
Away we go.

/*
INT. GALLEY - NIGHT

The galley is empty. Cut for time.
*/

EXT. DECK - DAY
```

Open points: OFF-16.

### Sections and Synopses

[Spec: Sections and Synopses](https://fountain.io/syntax#sections-synopses)

**Sections.** A Section is a line that starts with one or more `#` characters. More `#` characters mean deeper
nesting. Sections are navigation aids that some apps show as nested folders, similar to Markdown headers. They are
**ignored in formatted output**.

**Synopses.** A Synopsis is a single line that starts with `=`. It can appear anywhere in the screenplay and is also
ignored in formatted output. It describes a Section or a scene.

**Interoperability note.** Not every app supports Sections or Synopses. The spec suggests apps may import them as
Notes, and says Synopses attached to a scene heading may import more cleanly than those attached to a Section.

**Forced form.** None.

**Original example.**

```fountain
# Part One
## Arrival

= The crew reaches the island.

INT. BOAT - DAY
```

Open points: OFF-8, OFF-19, OFF-20, OFF-21.

### Error Handling

[Spec: Error Handling](https://fountain.io/syntax#error-handling)

- **Fallback:** when in doubt, return the text as Action. It is better to show the writer what they wrote in the wrong
  format than to drop it.
- **Containment:** a closing mark is not searched for **past a double line break**. This keeps a stray `[[` or `*`
  from turning half the script into a note or italics.
- **Exception:** the Boneyard may span many lines.

This section is the closest the spec comes to a general error model, and it creates two conflicts with the element
sections: see OFF-14 and OFF-16.

Open points: OFF-14, OFF-15, OFF-16.

### Completed Example

[Spec: Completed Example](https://fountain.io/syntax#completed-example)

This section holds a full short script that uses most elements together, plus downloads of three sample scripts
(each as `.fountain`, `.pdf` and `.fdx`). It is **not reproduced here**. What it demonstrates:

- a title page with several keys and two multi-line values;
- headings, Action, Character cues, Parentheticals and Dialogue in sequence;
- a dual-dialogue pair;
- automatic transitions and a forced one;
- a forced Scene Heading used for a title card;
- centered text and a forced Transition in the closing lines;
- emphasis inside Dialogue and Action.

It has no Note, Boneyard, Section, Synopsis, Lyric or page break. Those are shown only in their own sections.

The three downloads are also not reproduced. Their licences are unstated, so use them in the corpus only after getting
permission (see #15).

## Forced-elements table

[Spec: Overview](https://fountain.io/syntax#overview) lists the idea; each sigil is defined in its own section.

| Sigil | Element forced | Added in | Stated caveats |
|---|---|---|---|
| `.` | [Scene Heading](https://fountain.io/syntax#scene-headings) | before 1.1 | A **single** period, followed by an **alphanumeric** character. Removed from output. Not triggered by `...`. A leading `.` is also the way to stop a `TO:` line being a Transition. |
| `!` | [Action](https://fountain.io/syntax#action) | 1.1 | Overrides the heading prefix rule and the Character rule (uppercase Action above another line). The spec does not say the `!` is dropped (OFF-22). |
| `@` | [Character](https://fountain.io/syntax#character) | 1.1 | For names with lowercase letters and for non-Roman scripts. Removed; the name keeps its case. |
| `>` | [Transition](https://fountain.io/syntax#transition) | before 1.1 | Any line, any case. The spec does not say the `>` is dropped (OFF-22). Same lead character as centered text, which also needs a closing `<`. |
| `>` ... `<` | [Centered Text](https://fountain.io/syntax#centered-text) | before 1.1 | An Action element. Spaces between the text and the marks are allowed; leading spaces are not kept. The spec does not say whether the marks are dropped (OFF-10). |
| `~` | [Lyrics](https://fountain.io/syntax#lyrics) | 1.1 | Always forced; there is no automatic form. On each lyric line. The `~` is dropped. Styling is left to the app. |
| `^` | [Dual Dialogue](https://fountain.io/syntax#dual-dialogue) | before 1.1 | After the **second** Character cue; must be the last character on the line; spaces before it are ignored. |
| `#` ... `#` | [Scene number](https://fountain.io/syntax#scene-headings) | before 1.1 | Alphanumerics plus dashes and periods. At the end of a heading. |
| `#` | [Section](https://fountain.io/syntax#sections-synopses) | before 1.1 | One or more at the start of a line; more means deeper. Not shown in output. |
| `=` | [Synopsis](https://fountain.io/syntax#sections-synopses) | before 1.1 | Single line. Not shown in output. |
| `===` | [Page Break](https://fountain.io/syntax#page-breaks) | before 1.1 | Three or more `=` and nothing else on the line. |
| `[[ ]]` | [Note](https://fountain.io/syntax#notes) | before 1.1 | Inline or on its own line. Empty line inside needs two spaces. |
| `/* */` | [Boneyard](https://fountain.io/syntax#boneyard) | before 1.1 | May span lines and empty lines; not subject to the double-line-break limit. |
| `*`, `**`, `***`, `_` | [Emphasis](https://fountain.io/syntax#emphasis) | before 1.1 | Markdown rules, but `_` is underline. A backslash escapes. Not carried across line breaks. |

"Added in" comes from the changelog, which names 1.1 for `@`, `!` and `~`. For the rest the changelog gives no date, so
"before 1.1" means only that 1.1 did not add them.

## Rules the spec states plainly

These were assessed as candidates and are **explicit** in the spec, so they are not ambiguity cases. Residual open
points, if any, are noted in the case they feed.

| Topic | What the spec says | Residual |
|---|---|---|
| `.` forcing versus an ellipsis | Only a single period followed by an alphanumeric character forces a heading, so `...` stays Action | What counts as alphanumeric (OFF-7) |
| A transition not ending in `TO:` | Not automatic. Only `>` forces it | Other uppercase endings and blank-line edges (OFF-9) |
| Page break with more than three `=` | Three **or more** consecutive equals signs and nothing else | `==`, whitespace around the run (OFF-19) |
| Heading prefix case | Case-insensitive | Prose false positives (OFF-6) |
| Character extension case | May be uppercase or lowercase | Shape of the extension (OFF-3) |
| Text conversion | None. Typed punctuation passes through | Nothing |
| Closing-mark search limit | Stops at a double line break; Boneyard exempt | See OFF-14, OFF-16 |

## Ambiguous cases

Each case gives a short original example, the competing readings, and the spec's relevant text paraphrased. None is
ruled on here. `SPEC.md` (#4) assigns each a decision and a test.

### OFF-1: What is an "empty" line?

```fountain
OTTO
One.
<2sp>
Two.
```

Where `<2sp>` is a line holding two spaces, then variants with one space, a tab, or three spaces.

- **Reading A:** only a line with zero characters is empty; any other whitespace-only line is content.
- **Reading B:** any whitespace-only line is empty, except the specific two-space form the spec names for Dialogue and
  Notes.
- **Reading C:** the two-space exception applies to two *or more* spaces, or to any whitespace line, inside Dialogue and
  Notes only.

**Spec text.** The Line Breaks and Notes sections say a line of two spaces keeps a Dialogue or Note together. Every
"preceded by / followed by an empty line" rule in the other sections depends on the same undefined term. Nothing says
what a line with one space, a tab, or only trailing whitespace is. Outside Dialogue and Notes, the spec does not say
whether the two-space line has any special meaning (Action says empty lines pass through as empty Action lines).

### OFF-2: Line endings, trailing newline, encoding

```text
OTTO\r\nOne.\r\n      (CRLF)
OTTO\rOne.\r          (lone CR)
OTTO\nOne.            (no final newline)
```

- **Reading A:** `\n`, `\r\n` and `\r` are all one line break; a missing final newline changes nothing.
- **Reading B:** only `\n` is a line break; a `\r` stays in the text (and would break the "last character" rule for
  `^` and the trailing-space rule for `TO:`).
- **Reading C:** a final newline adds a trailing empty line that can affect "followed by an empty line" rules.

**Spec text.** The spec is silent on line-ending bytes, on whether a file ends with a newline, and on the character
encoding or a byte-order mark. It does say "carriage return" in Line Breaks and Notes when it means "line break". It
does not require the bytes to be CR.

### OFF-3: Character cue with extensions such as `(cont'd)` or `(V.O.)`

```fountain
NELL (V.O.)
MARA (cont'd)
OTTO (V.O.) (CONT'D)
MARA (on the radio, softly)
```

- **Reading A:** the "entirely uppercase" test applies to the text before the first `(`; anything in parentheses after
  it is an extension.
- **Reading B:** the test applies to the whole line, so a lowercase extension is not allowed (this was the 1.0 rule,
  which the 1.1 changelog lists as deprecated).
- **Reading C:** only a trailing parenthesised group counts as an extension; several groups, or a group followed by
  more text, make it something else.
- How extensions relate to the dual-dialogue caret (`MARA (cont'd) ^`) is also open (see OFF-11).

**Spec text.** The Character section says extensions may be in either case, and shows ones with a space inside and one
in plain lowercase words. It never defines what an extension is: where the name ends, how many may appear, whether
apostrophes or digits are allowed, and whether an extension may follow a forced `@` name.

### OFF-4: Non-ASCII uppercase and caseless names

```fountain
JOSÉ
ЕЛЕНА
ΣΩΚΡΑΤΗΣ
李明
MR. O'BRIEN
```

- **Reading A:** "uppercase" is Unicode-aware, so `JOSÉ` and `ЕЛЕНА` are cues; `李明` has no case, so it fails the
  test unless forced with `@`.
- **Reading B:** "uppercase" means ASCII `A-Z`; accented and non-Latin names need `@`.
- **Reading C:** a name is a cue when it has at least one alphabetic character and no lowercase one, which accepts
  caseless scripts.

**Spec text.** The rule is "entirely uppercase" with at least one alphabetical character (the spec's examples are
`R2D2` accepted and `23` rejected, so digits are tolerated). It recommends the `@` force for names that need lowercase
and "non-Roman languages", which suggests such names are not reliably auto-detected, but it does not say so. Punctuation
inside names (`MR. O'BRIEN`) is not discussed.

### OFF-5: A cue at the end of the file, or followed by an empty line

```fountain
Mara waits.

OTTO
```

(file ends here)

- **Reading A:** no line follows, so there is no "empty line after", and the line is a Character with no Dialogue.
- **Reading B:** an uppercase line needs real following text to be a cue; here it is Action.

**Spec text.** A Character cue has no empty line after it. The spec does not say what the end of the file counts as.
A cue followed by an empty line is explicitly not a cue (Action instead), per the same rule. What counts as an empty
line here is OFF-1.

### OFF-6: Scene heading versus Action, and the blank-line and prefix edges

```fountain
Mara waits.
INT. HOUSE - DAY
Lamps burn.

Ext. 12 is busy.

INT HOUSE - DAY
EST. SKYLINE
INT.
```

- **Reading A (strict):** a heading needs an empty line before and after it. The first line pair stays Action.
- **Reading B (lenient; excluded by the spec's literal text, listed because tools may differ):** the prefix alone
  suffices and the blank lines are not required.
- **Reading B2:** the start of the file counts as "preceded by an empty line".
- **Reading C:** `Ext. 12 is busy.` is a heading (this is what the literal case-insensitive rule gives; listed so #4
  can decide whether to flag it), and `INT` followed by a tab or the end of the line either is or is not.
- **Start of file and after the title page:** whether they count as "preceded by an empty line".

**Spec text.** The heading is a prefixed line with a blank line after it, and it always has at least one blank line
before it. The prefix must be followed by a dot or a space, and matching is case-insensitive.
The spec does not address file start, a heading directly after the title page, a prefix followed by a tab or a line
end, or prose that happens to start with `int ` or `ext `. What counts as an empty line here is OFF-1.

### OFF-7: Forced heading `.` and the meaning of "alphanumeric"

```fountain
.LAMP ROOM
.5 HOURS LATER
.ÉPILOGUE
. SPACE AFTER
..DOUBLE
...trailing off
```

- **Reading A:** the character after `.` must be ASCII `A-Z`, `a-z` or `0-9`.
- **Reading B:** it may be any Unicode letter or digit.
- **Reading C:** anything but another period forces a heading.

**Spec text.** A single period followed by an alphanumeric character forces a heading; this keeps ellipses from being
read as headings. The spec does not define alphanumeric, and is silent on a period followed by a space, on `..`, and on
a period followed by a non-letter symbol. (The ellipsis rule itself is explicit.)

### OFF-8: Scene-number character set and placement

```fountain
INT. KITCHEN - DAY #12#
INT. KITCHEN - DAY #1 A#
INT. KITCHEN - DAY #1_A#
INT. KITCHEN - DAY #É1#
INT. KITCHEN - DAY #7# trailing words
.TITLE CARD #3#
```

- **Reading A:** ASCII letters, digits, `-` and `.` only; anything else makes the `#...#` text part of the heading.
- **Reading B:** Unicode alphanumerics are allowed.
- **Reading C:** the number must be the last thing on the line; text after it voids it.
- **Reading D:** forced headings may carry numbers too (or not).

**Spec text.** Scene numbers are alphanumerics, plus dashes and periods, wrapped in `#`; the section lists valid
forms. The spec does not say whether spaces are allowed, whether it must end the line, what happens with text
after it, or whether a forced heading may have one. It also does not mention the clash with a Section line: a line like
`# 12 #` (see OFF-20).

### OFF-9: Transitions that do not end in `TO:`, and the blank-line edges

```fountain
FADE OUT.

FADE IN:

Mara nods.
BACK TO:
Otto shrugs.

CUT TO:
INT. DECK - DAY
```

- **Reading A:** none of the lines is a Transition: `FADE OUT.` and `FADE IN:` do not end in `TO:`, and `BACK TO:` and
  `CUT TO:` lack the empty line before or after that the spec requires. The first two are Action.
- **Reading B:** `FADE IN:` is a Transition because it is uppercase with a trailing colon (not in the spec; listed
  because the colon ending is common in screenplays).
- **Reading C (the literal result of the Character rule, listed for the Transition/Character interaction):** `CUT TO:`
  directly followed by a heading is not a Transition (no empty line after) and reads as a Character cue with
  Dialogue.

**Spec text.** Automatic Transitions must be uppercase, framed by empty lines, and end in `TO:`. A leading `>` forces
anything. The spec does not name `FADE IN:` or `FADE OUT.` and gives no other ending; it does not say what a
`TO:` line becomes when it fails only the "followed by empty line" test. It does say that spaces after the colon turn a
`TO:` line into Action, which means trailing whitespace is significant (see OFF-23). What counts as an empty line in the
framing rule is OFF-1.

### OFF-10: Centered text versus forced Transition

```fountain
>FIN<
> FIN <
>FIN< 
> FIN
>> FIN <<
>FIN< and more
```

- **Reading A:** a line that both starts with `>` and ends with `<` is centered; any other `>` line is a Transition.
- **Reading B:** trailing whitespace after `<` is ignored, so line 3 is centered; or it is not, so line 3 is a
  Transition.
- **Reading C:** nested marks (`>>` ... `<<`) are centered text containing an `>`; or something else.
- **Reading D:** a line like `> FIN` in the middle of Dialogue is part of the Dialogue.
- **Reading E:** the `>` and `<` marks are dropped from the text, or kept as part of it.

**Spec text.** Centered text is enclosed between a `>` and a `<`. A forced Transition begins with `>`. The spec does
not say how to tell them apart other than the closing `<`, whether `<` must be the last character, whether whitespace
may follow it, or what an unmatched `>` ... `<` pair inside text means. The spec's own Centered Text example appears
with a trailing space after the `<`, and its Completed Example shows a `>` line with a space after the `>` and no
closing `<`.

### OFF-11: Dual dialogue without a preceding dialogue, and caret placement

```fountain
OTTO ^
I'm first and last.

NELL
One.

OTTO^
Zero spaces.

MARA (V.O.) ^
After an extension.

PIP
Third.

TESS ^
Fourth.
```

- **Reading A:** a `^` cue needs an immediately preceding dialogue block to pair with; with none, it is an ordinary
  cue, or the caret is literal, or it is Action.
- **Reading B:** the caret stays a marker and the document has an unpaired dual dialogue (the parser may flag it).
- **Reading C:** zero spaces before the caret (`OTTO^`) counts as "any number of spaces".
- **Reading D:** a third `^` after a pair starts a new pair or extends the first.

**Spec text.** The caret goes after the second speaker's cue, any number of spaces before it are ignored, and it
must be the last character on the line. The spec says nothing about a missing first block, about a second `^` block in
a row, about the caret after an extension, or about the caret together with `@`.

### OFF-12: Parenthetical shape

```fountain
(whispering) and then speaks
(not closed
(multi-line
paragraph)
```

- **Reading A:** a Parenthetical is a whole line that starts with `(` and ends with `)`; anything else after a cue is
  Dialogue.
- **Reading B:** a line that only starts with `(` is a Parenthetical.
- **Reading C:** a Parenthetical may span lines when no empty line is inside it.

**Spec text.** A Parenthetical follows a cue or a Dialogue line and is enclosed in parentheses. There is no
mention of text outside the parentheses, an unclosed parenthesis, or multiple lines, nor of a `(` line that follows
neither a cue nor Dialogue.

### OFF-13: Lyrics next to cues and Dialogue

```fountain
OTTO
~Over the hill the lanterns glow
~Under the stars the rivers flow

~A lyric on its own

 ~ indented lyric
```

- **Reading A:** a `~` line after a Character cue is Dialogue; `~` is only a lyric where no dialogue could apply.
- **Reading B:** `~` always makes the line a Lyric, even inside a dialogue block (a character singing).
- **Reading C:** leading whitespace before `~` stops it (or does not).

**Spec text.** Lyrics start with `~`, on each line, and are always forced. The spec does not mention any adjacency
to Dialogue, whether empty lines are needed around Lyrics, or whether a leading space before `~` matters.

### OFF-14: Emphasis edges

```fountain
She said snake_case_name is fine.
*bold **inside** italic*
*start _nest* end_
***triple*
\_literal\_ and \\ and \# and a trailing \
*across
lines*
*across

lines*
```

- **Reading A:** the emphasis rules are those of one named Markdown implementation (the original one, a dialect such as
  CommonMark, or another); which determines `snake_case_name`, overlap and unmatched cases.
- **Reading B:** an emphasis span cannot cross a single line break. (This is what the Emphasis section says.)
- **Reading C:** an emphasis span can cross a single line break but not a blank line. (This is what the Error Handling
  wording suggests.)
- **Reading D:** a backslash escapes only emphasis characters; or any character; or `\\` yields one literal backslash.

**Spec text.** Emphasis follows the rules of Markdown except for underscore, without naming a version. The Emphasis
section says emphasis is not carried across line breaks, while Error Handling says a closing mark is not searched for
past a *double* line break, which is a looser limit. The escape rule names only the emphasis characters. Intraword
underscores, overlapping spans, `***` handling, and emphasis in titles or inside Notes and Boneyard are not covered.
What counts as a blank line or a double line break is OFF-1.

### OFF-15: Nested and unclosed Notes

```fountain
Hello [[outer [[inner]] outer]] world.
Hello [[never closed

Next paragraph.
[[ note spanning
two lines ]]
Hello ]] stray
```

- **Reading A:** a Note ends at the first `]]`; the nested text is plain.
- **Reading B:** Notes nest and the outer one ends at the matching `]]`.
- **Reading C:** an unclosed `[[` is plain text (Error Handling); or it is a Note to the end of the paragraph; or to the
  end of the file.
- **Reading D:** whether a standalone Note inside a Dialogue block splits the block (the empty lines around it are
  "removed" per the spec).

**Spec text.** Notes are double-bracket spans, inline or on their own line, and may include line breaks. The empty
lines around a standalone Note are removed in parsing. Error Handling says a closing mark is not searched for past a
double line break. The spec says nothing about nesting, a stray `]]`, a Note in the title page, or one inside a
Boneyard (and vice versa). What counts as an empty line, which these limits depend on, is OFF-1.

### OFF-16: Nested, unclosed and inline Boneyard

```fountain
Mara /* aside */ waits.
/* outer /* inner */ still outer? */
/* never closed

INT. HOUSE - DAY
```

- **Reading A:** a Boneyard ends at the first `*/`; the rest is plain text.
- **Reading B:** Boneyards nest.
- **Reading C:** an unclosed `/*` runs to the end of the file (the spec exempts Boneyard from the double-line-break
  limit); or it is ignored (plain text) because it never closes.
- **Reading D:** a Boneyard removes its text *before* element detection (so `/* x */INT. A` is a heading), or after.
- **Reading E:** a `*/` with no opening is plain text.

**Spec text.** Boneyards wrap text in `/* */` and may span as many lines as the writer likes; they are exempt from the
closing-mark limit. The spec does not say what an unclosed one does, whether they nest, how they combine with
Notes or Emphasis (`/**/`, `[[/*]]`), or how the lines around a removed span affect adjacent element detection.
Whether a line left empty by a removal counts as an empty line is OFF-1.

### OFF-17: Title page edges

```fountain
Note: this is a first line of action.

Title: A Quiet Evening
title: lower case key
Title: Duplicate
Brand New Key: kept?
Contact: Someone
  one-space indent
   three-space indent
	tab indent
Draft date: 5:30 pm
```

- **Reading A (end of the title page):** it runs to the first empty line.
- **Reading B:** it runs to the first line that is not a key or an indented value, with or without an empty line.
- **Reading C (start):** the first line being `Word: text` means a title page. The Action line above would then be
  misread; or a parser requires a known key.
- **Reading D (keys):** matching is case-sensitive, or insensitive; duplicates merge, overwrite or keep both; unknown
  keys are kept as metadata.
- **Reading E (values):** a one- or two-space indent is not a continuation (so it is a new element); a value may mix an
  inline start and indented lines; an empty-valued key is allowed.
- **Reading F (before):** blank lines before the first key are allowed, or they mean there is no title page.

**Spec text.** The title page always comes first in the document. Keys end with a colon and may contain
spaces. Values are inline or indented by three or more spaces or a tab, and can have several lines. Unsupported keys
are ignored but can be metadata. The spec says to drop down "two lines" to start the screenplay. It does not state
key-case rules, duplicate handling, blank-line handling inside or before the page, a one-/two-space indent, or how to
tell a title page from an Action line that has a colon. What counts as an empty line, where the title page ends, is
OFF-1.

### OFF-18: Tabs, indentation and recognising the element first

```fountain
	OTTO
Hello.

    THE BIG ROOM

	INT. HOUSE - DAY
```

- **Reading A:** leading whitespace is stripped before element detection (except for Action). Line 1 is a Character
  cue; line 3 may be a Character cue and not Action.
- **Reading B:** detection works on the raw line; an indented line is Action (whose whitespace is kept) unless it
  matches with the whitespace present.
- **Reading C:** the tab-to-four-spaces conversion applies only in Action, or to every element's inner tabs.

**Spec text.** Leading whitespace is ignored for every element **except Action**, but to know whether a line is Action
you have to recognise it first. The spec does not resolve this circle, nor say whether the conversion of a tab to four
spaces applies in Dialogue or elsewhere. The spec's prose and its served examples also differ: the Action section's
prose speaks of ten leading spaces on the card lines, while the example block on the page shows four; and the Indenting
section's prose describes tabs while its example is shown with spaces. This file reports what the served HTML contains
(checked in the raw bytes) without explaining the difference.

### OFF-19: `=` runs: page break, synopsis and whitespace

```fountain
===
====
==
=
= Summary text
  ===
=== 
= = =
```

- **Reading A:** `===` or longer, alone on the line, is a page break; `=` followed by text is a Synopsis; `==` and `=`
  alone are neither (Action or a Synopsis with empty text).
- **Reading B:** whitespace around the run is ignored (`  ===`, `===  `); or it voids the break ("nothing more").
- **Reading C:** `=== text` is a Synopsis of `== text`, or Action.

**Spec text.** A page break is three or more consecutive equals signs and nothing more; a Synopsis is a single line
prefixed by `=`. The "three or more" rule is explicit. The spec does not cover two equals signs, a lone `=`, leading or
trailing whitespace on a break, or mixed `= = =`.

### OFF-20: Section edges

```fountain
#Section without space
# 
##### Deep
#1 priority item
INT. HOUSE - DAY #5#
# 12 #
```

- **Reading A:** a Section needs `#` at the line start and nothing else (any text, with or without a space).
- **Reading B:** a space after the `#`s is required.
- **Reading C:** an empty Section (`# `) is a Section; or Action.
- **Reading D:** the last lines show the scene-number delimiter `#` clashing with Section syntax (OFF-8).

**Spec text.** A Section is a line that starts with one or more `#`, nested by more `#`. It cites Markdown ATX headers
as the model, and Markdown requires a space only in some dialects. The spec gives no maximum depth and does not say
whether empty text is allowed, whether an empty line must surround a Section, or how a Section inside a Dialogue block
interacts with it.

### OFF-21: Synopsis and Section placement inside blocks

```fountain
OTTO
Hello.
= a synopsis in dialogue
Goodbye.

NELL
# a section here?
Yes.
```

- **Reading A:** Sections and Synopses are removed from the output and a surrounding Dialogue block continues across
  them.
- **Reading B:** they end the block, and the following line starts a new element.
- **Reading C:** inside a Dialogue block they are literal text.

**Spec text.** A Synopsis can go anywhere in the screenplay. Sections and Synopses are ignored in
formatted output. The spec does not say whether being removed means the surrounding block continues.

### OFF-22: Forcing sigils: leading whitespace, stacking, and lines starting with `>`

```fountain
  !indented action
!.dot after bang
@!after at
 .INDENTED HEADING

>>> a line of arrows
!>NOT A TRANSITION
.>HEADING
```

- **Reading A:** a sigil counts only as the first character of the line.
- **Reading B:** leading whitespace before the sigil is skipped.
- **Reading C:** sigils may stack, with a defined precedence; or the second sigil is plain text.
- **Reading D:** a line that merely starts with `>` as ordinary text is a Transition anyway.
- **Reading E:** the `!` and the `>` are dropped from the text (the spec says so only for `.`, `@` and `~`), or kept.

**Spec text.** The sigils are "preceding", "starting" or "beginning" the line. The spec does not say whether `!` and `>`
are dropped like the others, how sigils combine, how they interact with leading whitespace (ignored for non-Action
elements) or whether `!` and `@` apply when followed by whitespace.

### OFF-23: Significant trailing whitespace

```fountain
CUT TO:
CUT TO: 
MARA ^ 
Two trailing spaces.  
```

- **Reading A:** trailing spaces after `TO:` make it Action (the spec says so); trailing whitespace is otherwise
  stripped.
- **Reading B:** trailing whitespace is never stripped, so a caret followed by a space is not "the last character".
- **Reading C:** trailing double spaces keep their pre-1.1 meaning of forcing an Action line.
- **Reading D:** carriage returns (OFF-2) count as trailing whitespace, which changes lines 1, 3 and 4.

**Spec text.** Spaces after the colon on a `TO:` line cause Action; the caret must be the last character; and the
1.1 changelog lists "two trailing spaces force an Action line" as deprecated. The spec gives no general rule on
trailing whitespace, so each rule that depends on "last character" is exposed to it.

### OFF-24: What a Dialogue block may contain

```fountain
OTTO
.This looks like a heading
>This looks like a transition
[[This is a note]]
=This is a synopsis
/* boneyard */
That is all.
```

- **Reading A:** inside a Dialogue block every line is Dialogue except Notes and Boneyard, which are removed.
- **Reading B:** forced elements (`.`, `>`, `!`) still take effect inside a block.
- **Reading C:** Sections, Synopses and page breaks end the block.

**Spec text.** Dialogue is any text that follows a cue or Parenthetical (so, Reading A), and elsewhere the spec
describes forced elements as overriding detection (so, Reading B). The spec states no priority between "text after a
cue" and a forcing sigil at the start of that text.

## Summary for #4

| Group | Cases |
|---|---|
| Blank lines, line endings, whitespace | OFF-1, OFF-2, OFF-23 |
| Character cues and dual dialogue | OFF-3, OFF-4, OFF-5, OFF-11, OFF-12 |
| Headings and numbers | OFF-6, OFF-7, OFF-8 |
| Transitions and centered text | OFF-9, OFF-10 |
| Lyrics, Dialogue contents | OFF-13, OFF-24 |
| Inline markup | OFF-14, OFF-15, OFF-16 |
| Title page | OFF-17 |
| Indentation and forcing sigils | OFF-18, OFF-22 |
| Page breaks, Sections, Synopses | OFF-19, OFF-20, OFF-21 |

Cases that are **root causes** for others: OFF-1 (empty line) feeds OFF-5, OFF-6, OFF-9, OFF-14, OFF-15, OFF-16 and
OFF-17; OFF-2 (line endings) feeds OFF-23.
