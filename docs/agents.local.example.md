# Machine context for agents (template)

Copy this file to `~/.config/twopop/agents.local.md` and fill it in. That file lives outside every repository and
is never committed. `CLAUDE.md` imports it, and `AGENTS.md` tells other harnesses to read it.

It lists only what work on this library uses. The same local file serves the other Twopop repositories, so yours may
also hold the sections their templates add.

## Machine

- OS and version:
- CPU and memory:

## Local folders

- Where this repository lives (the bare repo and its worktrees):
- Where the Twopop Studio repository's `main` worktree lives (it holds `scripts/board-items.sh`, which `AGENTS.md` uses to read the board; it sits beside this repository's folder):

## Tools

- Rust: the stable toolchain, and whether the 1.85 toolchain (the minimum supported version) is installed:
- `cargo-hack` (version; `AGENTS.md` pins the one CI uses):
- `uv` (runs `scripts/screenplay-tools-diff.py`):
- `jq` (filters the board reader's JSON, as `AGENTS.md` shows):
- `gh` (signed in; opens PRs and edits issues, and the board reader calls it):
- `wt` (worktrunk; creates and removes the branch worktrees `AGENTS.md` describes):
- CLI harnesses installed (Claude Code, Codex, Gemini CLI, Kiro, opencode…):
