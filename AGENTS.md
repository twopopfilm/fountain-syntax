# fountain-syntax: agent instructions

A Rust library for parsing and writing Fountain screenplays: lossless, spec-complete, and never failing on input.
It is used by [Twopop Studio](https://github.com/twopopfilm/twopop-studio) and is meant to be useful on its own.

- **Goals, milestones (F1–F6) and the "done when" checks:** `docs/brief.md`.
- **Status:** the status note and the milestone table in `README.md` (kept current as a `status_doc` in `.claude/next.yml`).
- **Machine-specific context** is kept outside the repository, in `~/.config/twopop/agents.local.md`. Read it if it
  exists. The template is `docs/agents.local.example.md`.

## Keep the repository publishable

This repository may go public at any time. Its history, issues and the project board must never contain personal data
beyond the maintainer's name.

- **Never commit:**
  - email addresses other than noreply ones;
  - other people's personal data (names of private individuals, photos or likenesses);
  - paths that name a user's home folder or the local checkout: the forms `PATH_PATTERN` in
    `scripts/check-public.sh` lists. A tilde path to a tool's own folder, which is the same on every machine
    (`~/.config/twopop/…`, `~/.claude/…`), is allowed;
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
  1. Sessions open in the `main` worktree. The board is shared with the other Twopop repositories, so the next task
     here is the first Todo item **from this repository**, in board order:
     ```sh
     ../../twopop-studio/twopop-studio.main/scripts/board-items.sh twopopfilm 2 \
       | jq '[.items[] | select(.content.repository == "twopopfilm/fountain-syntax")]
             | {todo: [.[] | select(.status == "Todo") | {title, url: .content.url, labels}],
                untriaged: [.[] | select(.status == "Backlog" and .priority == null
                                                and ((.labels // []) | index("optional") | not))] | length}'
     ```
     The script lives in the Twopop Studio repository (its `main` worktree, beside this repository's folder), and
     `/workflow:next` uses it too. Don't use `gh project item-list`: it costs about 100 GraphQL points a page against
     about 1, and sessions share one budget.
     Epics (label `epic`) are tracking issues. Work on their sub-issues.
  2. Set the issue In Progress, create the worktree with `wt switch --create <branch> --no-cd --yes`, and move the
     session into it.
  3. Work test-first, commit, push, and open the PR.
  4. After the merge: on `main`, run `git pull --ff-only` first, then `wt remove <branch>`.
- **The project board** is org project #2, "Twopop", which holds the issues of every Twopop
  repository. Its field and option IDs and the `gh` commands are listed in Twopop Studio's `AGENTS.md`, "The project
  board". Board order is the work order.
- **Land changes through PRs:** `gh pr create` with a bare `Closes #N` as the first line of the body, squash-merged.
  Merging is the maintainer's call unless they ask you to merge. Publishing to crates.io also needs the maintainer's
  OK: that is the approval of the `crates-io` environment (see the Release rule).

## Issue conventions

The standard for issues, milestones and board fields is Twopop Studio's `AGENTS.md`, "Issue conventions" (milestones,
scheduling, the Size definitions, native dependencies, tracking issues, the label table). This section copies none of
it and states only what is local to this repository:

- The milestone prefix is `F`; `docs/brief.md` lists the milestones.
- Track is `Fountain`.
- A Backlog issue has no milestone and no parent.
- Every F tracking issue is closed today, so promoting an issue to Todo needs a tracking issue that the maintainer
  opens or chooses.

## Develop

The crate is at the repository root (`Cargo.toml`, `src/`). `parse` builds the syntax tree,
`Document::screenplay()` the typed view, and `Document::diagnostics()` the diagnostics of `SPEC.md` §7, with no
required dependencies. `Document::write` gives the source back byte for byte, and `canonical` (`src/write.rs`) writes
a view built in memory (`SPEC.md` §10). `Document::with_replaced` and the named edits (`src/edit.rs`, issue #13)
change one span and re-parse. The features `locked-scene-numbers`, `shot`, `serde` and `hash` exist, and
hash scheme 1 (issue #35) is implemented. CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings` and `cargo test` on Linux, macOS and Windows, then
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features` and
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`. The Linux `features` job runs
`cargo hack clippy --feature-powerset --exclude-features default --all-targets -- -D warnings` and
`cargo hack test --feature-powerset --exclude-features default`, which lint and test every combination of the four
features. Install cargo-hack with `cargo install cargo-hack --locked --version 0.6.46`. The exclusion is there
because `default = []` is declared explicitly, so cargo-hack would otherwise count it as a fifth feature and run every
build twice. The Linux `msrv` job runs
`cargo check --locked --all-targets`, with and without `--all-features`, on the minimum supported Rust (1.85, the
`rust-version` in `Cargo.toml`). The `msrv` job needs a 1.85 toolchain (for example
`rustup run 1.85 cargo check --locked --all-targets`). CI has not been able to run it yet, so run it locally when you
have a 1.85 toolchain, before relying on the declared minimum. Run the same commands locally. Rules:

- **Rust stable, minimum supported version 1.85;** no `unsafe`; no required dependencies. `SPEC.md` section 8.1 lists
  the optional Cargo features: `serde` (adds `serde`), `hash` (adds `sha2` and `unicode-normalization`),
  `locked-scene-numbers` and `shot` (no dependency). All four exist.
- **Every `SPEC.md` decision has a test** that names it. `tests/spec_ids.rs` enforces this, so a new ID must be in a
  test's function name or a `///` line in `tests/*.rs`.
- **The `screenplay-tools` diff** (issue #16): `tests/screenplay_tools.rs` compares the typed view of each corpus
  script with the committed snapshot `tests/corpus/screenplay-tools.json` and checks every difference against
  `docs/conformance/screenplay-tools.md`, so CI's `cargo test` enforces the record without Python. It is compiled out
  whenever `shot` or `locked-scene-numbers` is on, because those features change the
  view. After editing a corpus script or changing the parser, run
  `uv run --script scripts/screenplay-tools-diff.py` (local only, not part of the gate; needs `uv` and a network),
  commit the regenerated snapshot, and classify any new hunk in the record (the test prints each unrecorded hunk ready
  to paste).
- **The writer must round-trip unchanged input byte-for-byte.**
- **Before every PR:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`,
  `cargo hack clippy --feature-powerset --exclude-features default --all-targets -- -D warnings`,
  `cargo hack test --feature-powerset --exclude-features default`,
  `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` and `scripts/check-public.sh`.
- **Keep `README.md` current.** It is the project's public face. A PR that changes what a user sees (install
  steps, commands, flags, examples, supported features, milestone state) updates the README in the same PR. Label
  planned features as planned; never describe something as working before it does. Every Rust code block in it
  is a doctest (`src/lib.rs` includes it), so `cargo test` runs it and a broken example fails the build. The `sh`
  install line is not run.
- **Release.** A release PR bumps `version` in `Cargo.toml` (and `Cargo.lock`), moves `[Unreleased]` in
  `CHANGELOG.md` under a `## [x.y.z] - YYYY-MM-DD` heading (the form `scripts/check-release.sh` requires), leaves an
  empty `[Unreleased]` and updates the link references at the foot of the file, and updates the README if a statement
  changes. After the maintainer merges it, the maintainer pushes the tag `vx.y.z` on the squash commit; an agent never
  pushes a release tag. `.github/workflows/publish.yml` then runs `scripts/check-release.sh`, calls `ci.yml`, waits
  for the maintainer's approval in the `crates-io` environment and runs `cargo publish --locked`. crates.io trusts
  that workflow file and environment (trusted publishing), so no token is stored. Never move or delete a tag (a
  ruleset blocks it): fix forward with a new patch version. 0.1.0 was published by hand, since a crate's first
  version can't come from a workflow. "Run workflow" on `main` rehearses everything except the publish, and
  `scripts/check-release.sh` with no arguments rehearses the guard locally. Until the maintainer has created the
  `crates-io` environment (with themself as required reviewer) and registered the trusted publisher on crates.io, no
  tag may be pushed.

## /next: project rules

- **Before every commit** run `scripts/check-public.sh`. Files, issues, PRs and commit messages must hold no non-noreply email addresses, paths that name a user's home folder or the local checkout (see "Keep the repository publishable"), machine specs or credentials.
- **Required issue fields.** Every issue has Priority, Size and Track (`Fountain`) and goes on org project #2. A
  numbered F milestone and a parent epic apply only from Status Todo onward, never to a Backlog issue. A `chore` has no
  milestone, and a tracking issue needs no parent. See *Issue conventions*.
- **Never edit another repository** (Twopop Studio, the Python `twopop`, the kit). Stop, say what and why, and let the maintainer decide. Never touch the shared ComfyUI server.
- **Test fixtures** must be original, public-domain or redistributable. Record each one's source and licence in the corpus README.
- **Merging, and publishing to crates.io,** are the maintainer's call.
