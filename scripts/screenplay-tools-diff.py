#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["screenplay-tools==0.0.10"]
# ///
"""Export screenplay-tools' reading of the test corpus as a snapshot of "line form" strings.

Run: `uv run --script scripts/screenplay-tools-diff.py`. It rewrites `tests/corpus/screenplay-tools.json`, then runs
`cargo test --test screenplay_tools` (which flattens this crate's typed view into the same line form and diffs it
against the snapshot) and exits with cargo's status.

Each file is decoded from its raw bytes (strict UTF-8, no newline translation), so screenplay-tools sees the CR and CRLF
bytes and does its own line splitting.

Line form: one string per element, in document order; U+23CE stands for an embedded line break.
  TITLE <key>: <value>      each line of a multi-line value stripped, empty ones dropped, joined with the break
  HEADING <text>[ #<num>]   CENTERED <text> (a centered Action)     ACTION <text>
  CHARACTER <name>[ (<ext>)][ ^]            PAREN <text>            DIALOGUE <text>       LYRIC <text>
  TRANSITION <text>         SECTION <depth> <text>                  SYNOPSIS <text>       PAGEBREAK
  HASH_V1 <first 12 hex of text_sha>        last line of each scene
Text is the parser's `text_raw` with note and boneyard markers removed and the ends stripped; a text element that is
then empty is dropped. Elements before the first heading are listed but belong to no scene.
"""

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

from screenplay_tools.fountain.parser import Parser
from screenplay_tools.screenplay import Character, ElementType, SceneHeading, Section

ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "tests" / "corpus"
SNAPSHOT = CORPUS / "screenplay-tools.json"
BREAK = "⏎"

# Restates the Twopop importer's text_sha rule: the heading, then the kept kinds, hashed as normalised lines.
_MARKERS = re.compile(r"\[\[\d+\]\]|/\*\d+\*/")
_PREFIX = {
    ElementType.ACTION: "ACTION",
    ElementType.DIALOGUE: "DIALOGUE",
    ElementType.PARENTHETICAL: "PAREN",
    ElementType.TRANSITION: "TRANSITION",
    ElementType.LYRIC: "LYRIC",
    ElementType.SYNOPSIS: "SYNOPSIS",
}


def story_text(raw: str) -> str:
    return _MARKERS.sub("", raw).strip()


def normalize(text: str) -> str:
    return " ".join(text.split())


def text_sha(heading: str, kept: list[str]) -> str:
    return hashlib.sha256((normalize(heading).upper() + "\n" + "\n".join(map(normalize, kept))).encode()).hexdigest()


def line_form(text: str) -> list[str]:
    parser = Parser()
    parser.add_text(text)
    parser.finalize()
    script = parser.script

    lines = []
    for entry in script.titleEntries:
        parts = [p.strip() for p in story_text(entry.text_raw).split("\n")]
        lines.append(f"TITLE {entry.key}: {BREAK.join(p for p in parts if p)}")

    kept: list[str] | None = None  # the open scene's hashed texts; None before the first heading
    heading = ""

    def close_scene() -> None:
        if kept is not None:
            lines.append("HASH_V1 " + text_sha(heading, kept)[:12])

    for e in script.elements:
        if isinstance(e, SceneHeading):
            close_scene()
            heading, kept = normalize(story_text(e.text_raw)), []
            lines.append(f"HEADING {story_text(e.text_raw)}" + (f" #{e.scene_number}" if e.scene_number else ""))
        elif isinstance(e, Character):
            lines.append(
                f"CHARACTER {e.name}" + (f" ({e.extension})" if e.extension else "") + (" ^" if e.is_dual_dialogue else "")
            )
            if kept is not None:
                kept.append(normalize(e.name).upper())
        elif e.type == ElementType.PAGEBREAK:
            lines.append("PAGEBREAK")
        else:
            body = story_text(e.text_raw)
            if not body:  # a standalone note parses as an empty element
                continue
            shown = body.replace("\n", BREAK)
            if isinstance(e, Section):
                lines.append(f"SECTION {e.level} {shown}")
            else:
                centered = e.type == ElementType.ACTION and e.centered
                lines.append(f"{'CENTERED' if centered else _PREFIX[e.type]} {shown}")
                if kept is not None:
                    kept.append(body)
    close_scene()
    return lines


def main() -> int:
    files = {}
    for path in sorted(CORPUS.rglob("*.fountain")):
        data = path.read_bytes()
        files[path.relative_to(CORPUS).as_posix()] = {
            "sha256": hashlib.sha256(data).hexdigest(),
            "lines": line_form(data.decode("utf-8")),  # real bytes: no CR/CRLF translation, as in the Rust side
        }
    SNAPSHOT.write_text(
        json.dumps({"tool": "screenplay-tools 0.0.10", "files": files}, indent=2, sort_keys=True, ensure_ascii=False)
        + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(f"wrote {SNAPSHOT.relative_to(ROOT)} ({len(files)} entries)")
    return subprocess.run(["cargo", "test", "--test", "screenplay_tools"], cwd=ROOT).returncode


if __name__ == "__main__":
    sys.exit(main())
