# Machine context for agents (template)

Copy this file to `~/.config/twopop/agents.local.md` and fill it in. That file lives outside every repository and
is never committed. `CLAUDE.md` imports it, and `AGENTS.md` tells other harnesses to read it.

## Machine

- OS and version:
- CPU/GPU and memory (e.g. Apple Silicon with unified memory; or the GPU model and its VRAM):
- Memory budget notes (what the ComfyUI models hold, and what's left for a local LLM):

## Local folders

- Where the repositories live (the bare repo and worktrees for twopop-studio, fountain-syntax and twopop):
- `$TWOPOP_HOME` (worlds and projects):

## ComfyUI

- Install type (Desktop, or a headless `comfy launch`) and version:
- URL and port:
- Shared input/output folders and the model-path configuration:
- Other users of the server, and any house rules beyond `AGENTS.md`:

## Other tools

- Blender (version and path; or `$TWOPOP_BLENDER`):
- FFmpeg:
- Local LLM servers (oMLX, LM Studio, Ollama: URLs and models):
- CLI harnesses installed (Claude Code, Codex, Gemini CLI, Kiro, opencode…):
