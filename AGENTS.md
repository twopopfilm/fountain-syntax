# fountain-syntax: agent instructions

A Rust library for parsing and writing Fountain screenplays: lossless, spec-complete, and never failing on input.
It is used by [Twopop Studio](https://github.com/twopopfilm/twopop-studio) and is meant to be useful on its own.

- **Goals, milestones (F1–F5) and the "done when" checks:** `docs/brief.md`.
- **Status:** F1 (the standards study, which ends in `SPEC.md`) is in progress, and there's still no code. Write no
  parser code until `SPEC.md` is merged.
- **Machine-specific context** is kept outside the repository, in `~/.config/twopop/agents.local.md`. Read it if it
  exists. The template is `docs/agents.local.example.md`.

## Keep the repository publishable

This repository may go public at any time. Its history, issues and the project board must never contain personal data
beyond the maintainer's name.

- **Never commit:**
  - email addresses other than noreply ones;
  - other people's personal data (names of private individuals, photos or likenesses);
  - absolute or home-directory paths;
  - machine specs or local install locations;
  - credentials.

  The maintainer is identifiable: their name may appear (commit authorship, credits, the licence line), and their
  personal email address never does.
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

The crate is at the repository root (`Cargo.toml`, `src/`). `parse` builds the syntax tree,
`Document::screenplay()` the typed view, and `Document::diagnostics()` the diagnostics of `SPEC.md` §7, with no
required dependencies. The features `locked-scene-numbers`, `shot`, `serde` and `hash` exist; hash scheme 1
(issue #35) is still planned. CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings` and `cargo test` on Linux, macOS and Windows, then
`cargo clippy --all-targets --all-features -- -D warnings` and `cargo test --all-features`. Run the same commands
locally. Rules:

- **Rust stable;** no `unsafe`; no required dependencies. `SPEC.md` section 8.1 lists the optional Cargo
  features: `serde` (adds `serde`), `hash` (adds `sha2` and `unicode-normalization`), `locked-scene-numbers` and
  `shot` (no dependency). All four exist.
- **Every `SPEC.md` decision has a test** that names it.
- **The writer must round-trip unchanged input byte-for-byte.**
- **Before every PR:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` and
  `scripts/check-public.sh`.
- **Keep `README.md` current.** It is the project's public face. A PR that changes what a user sees (install
  steps, commands, flags, examples, supported features, milestone state) updates the README in the same PR. Label
  planned features as planned; never describe something as working before it does.

## /next: project rules

- **Before every commit** run `scripts/check-public.sh`. Files, issues, PRs and commit messages must hold no non-noreply email addresses, absolute or home-directory paths, machine specs or credentials.
- **No parser code until `SPEC.md` is merged.** F1 is a standards study that ends in `SPEC.md`.
- **Every new issue needs a milestone** of this repository (F1–F5) and goes on org project #2. The queue pass reports an open issue that lacks a milestone.
- **Never edit another repository** (Twopop Studio, the Python `twopop`, the kit). Stop, say what and why, and let the maintainer decide. Never touch the shared ComfyUI server.
- **Test fixtures** must be original, public-domain or redistributable. Record each one's source and licence in the corpus README.
- **Merging, and publishing to crates.io,** are the maintainer's call.
