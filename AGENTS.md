# fountain-syntax: agent instructions

A Rust library for parsing and writing Fountain screenplays: lossless, spec-complete, and never failing on input.
It is used by [Twopop Studio](https://github.com/twopopfilm/twopop-studio) and is meant to be useful on its own.

- **Goals, milestones (F1–F5) and the "done when" checks:** `docs/brief.md`.
- **Status:** pre-F1, so there's no code yet. F1 is a standards study that ends in `SPEC.md`. Write no parser code
  until `SPEC.md` is merged.
- **Machine-specific context** is kept outside the repository, in `~/.config/twopop/agents.local.md`. Read it if it
  exists. The template is `docs/agents.local.example.md`.

## Keep the repository publishable

This repository may go public at any time. Its history, issues and the project board must never contain personal
data.

- **Never commit:**
  - personal names (write "the maintainer");
  - email addresses other than noreply ones;
  - absolute or home-directory paths;
  - machine specs or local install locations;
  - credentials.
- **Run `scripts/check-public.sh` before every commit.** CI runs it, plus gitleaks, on every PR. The private name list
  lives at `~/.config/twopop/publiccheck.local` and is never committed.
- **Commit identity** is the maintainer's GitHub noreply address, set in this repository's local git config; don't
  change it.
- **Test scripts and fixtures** must be original, public-domain, or used under terms that allow redistribution.
  Record each one's source and licence in the corpus README. Don't copy test files from other projects without
  checking their licence.

## Repository layout and workflow

- **Layout** (worktrunk's default path template). Paths below are relative to this `main` worktree:
  - bare repository: `../fountain-syntax`
  - this `main` worktree: `.`, kept pristine; never commit on `main` directly
  - one worktree per branch: `../fountain-syntax.<branch>`
- **The loop, one task at a time.**
  1. Sessions open in the `main` worktree. The board is shared with Twopop Studio, so the next task here is the first
     Todo item **from this repository**, in board order:
     ```sh
     gh project item-list 2 --owner twopopfilm --format json --limit 500 \
       --jq '[.items[] | select(.status == "Todo" and .content.repository == "twopopfilm/fountain-syntax")][0] | {title, url: .content.url}'
     ```
     Epics (label `epic`) are tracking issues. Work on their sub-issues.
  2. Set the issue In Progress, create the worktree with `wt switch --create <branch> --no-cd --yes`, and move the
     session into it.
  3. Work test-first, commit, push, and open the PR.
  4. After the merge: on `main`, run `git pull --ff-only` first, then `wt remove <branch>`.
- **The project board** is org project #2, "Twopop Studio". Its field and option IDs and the `gh` commands are listed
  in Twopop Studio's `AGENTS.md`, "The project board". Board order is the work order.
- **Land changes through PRs:** `gh pr create` with a bare `Closes #N` as the first line of the body, squash-merged.
  Merging is the maintainer's call unless they ask you to merge. Publishing to crates.io also needs the maintainer's
  OK.

## Develop

Fill this section in when the crate exists (F2). Rules that already apply:

- **Rust stable;** no `unsafe`; minimal dependencies (the core parser should need none beyond `serde`, behind a
  feature flag).
- **Every `SPEC.md` decision has a test** that names it.
- **The writer must round-trip unchanged input byte-for-byte.**
- **Before every PR:** `cargo test`, `cargo clippy`, `cargo fmt --check` and `scripts/check-public.sh`.
- **Keep `README.md` current.** It is the project's public face. A PR that changes what a user sees (install
  steps, commands, flags, examples, supported features, milestone state) updates the README in the same PR. Label
  planned features as planned; never describe something as working before it does.

## /next: project rules

- **Before every commit** run `scripts/check-public.sh`. Files, issues, PRs and commit messages must hold no personal names, non-noreply email addresses, absolute or home-directory paths, machine specs or credentials.
- **No parser code until `SPEC.md` is merged.** F1 is a standards study that ends in `SPEC.md`.
- **Every new issue needs a milestone** of this repository (F1–F5) and goes on org project #2. The queue pass reports an open issue that lacks a milestone.
- **Never edit another repository** (Twopop Studio, the Python `twopop`, the kit). Stop, say what and why, and let the maintainer decide. Never touch the shared ComfyUI server.
- **Test fixtures** must be original, public-domain or redistributable. Record each one's source and licence in the corpus README.
- **Merging, and publishing to crates.io,** are the maintainer's call.
