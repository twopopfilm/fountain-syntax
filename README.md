# fountain-syntax

A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays, in Rust.

- Parses every Fountain element, plus documented extensions behind feature flags.
- Keeps source spans and round-trips byte-for-byte, so editors and tools can change one detail without disturbing
  the rest of the script.
- Never fails on input; ambiguous cases come back as diagnostics.

## Status

Pre-alpha. There's no code yet; work starts with a standards study (milestone F1). The goals and milestones are in
[docs/brief.md](docs/brief.md). Work is tracked on the
[Twopop Studio project board](https://github.com/orgs/twopopfilm/projects/2).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
