# DSP Playground

A 100-day audio DSP learning project in Rust. Goal: build a reusable `dsp-lib` crate by working through structured curriculum chunks, 20–30 minutes per day.

## Project structure

```
dsp-playground/
├── CLAUDE.md               ← you are here
├── README.md               ← curriculum status table, architecture, principles
├── curriculum/             ← one .md file per chunk (0-scaffold.md, 1-oscillators.md, …)
├── src/                    ← the DSP library (pure Rust, no audio drivers)
├── examples/               ← CPAL runners, one per chunk (chunk0_scaffold.rs, …)
└── .claude/skills/         ← four Claude Code skills (see below)
```

Current progress is tracked in the README.md status table. Chunk details and daily steps live in `curriculum/<N>-<topic>.md`.

## Commands

```bash
cargo build                                    # compile the library
cargo run --example chunk<N>_<topic>           # run a chunk example
cargo test                                     # run tests
```

## Hard rules

**No external dependencies in `[dependencies]`.** `src/` is pure DSP math. CPAL, hound, and anyhow belong in `[dev-dependencies]` only. If any file under `src/` contains `use cpal` or `use hound`, that is a bug.

**No `mod.rs` files.** This project uses the modern flat module style: `src/oscillator.rs` is the entry point for the `oscillator` module, with submodules in `src/oscillator/sine.rs` etc. Never create `mod.rs`.

**Denormal flush in every feedback loop.** Any IIR filter or delay with internal state that feeds back must include `if x.abs() < 1e-15 { x = 0.0 }` after each state update.

**No `% 1.0` in hot paths.** Use `if phase >= 1.0 { phase -= 1.0 }` for phase wrapping inside `process()`. The modulo operator compiles to a slow `fmod` call; the branch is two instructions.

## Skills — use these

Four skills are installed in `.claude/skills/`. They orient themselves by reading README.md and the `curriculum/` files.

| Say this | Skill | When |
|---|---|---|
| "start my session" | `daily-session` | Every time you sit down to work |
| "I don't understand X" | `dsp-instructor` | Stuck on a concept or debugging |
| "review my chunk" | `chunk-reviewer` | Before marking a chunk ✅ done |
| "add a chunk for X" | `add-chunk` | Extending the curriculum |

## DSP library conventions

- All DSP structs use `f32` throughout — no `f64` in hot paths
- Generators (oscillators, noise) implement `Process` — no input argument
- Processors (filters, effects) implement `Transform` — takes `f32` input
- Every struct also implements `Reset` and `SetSampleRate`
- `new(sample_rate: f32)` is the standard constructor signature
- Parameters are set via methods (`set_frequency`, `set_cutoff`) not via `process()`
- Modulated parameters must be smoothed via `OnePole` before reaching the hot path