@AGENTS.md
@~/.config/twopop/agents.local.md

## Claude Code specifics

- **Worktrees:** after `wt switch --create <branch> --no-cd --yes`, call `EnterWorktree` with the `path` that
  `git worktree list` prints for `fountain-syntax.<branch>`. After the merge, `ExitWorktree` with `action: "keep"`
  returns to `main`; then pull `main` and `wt remove <branch>`.
- **Auto memory:** `.claude/settings.json` points every worktree at one memory directory. Keep durable conventions in
  `AGENTS.md`; memory is only for machine-local notes.
