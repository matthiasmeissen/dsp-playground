# DSP Playground

A 100-day structured journey through audio DSP programming in Rust.
Philosophy: build fluency, not academic mastery. Know what things do, what the knobs are, and how to connect them.


## Curriculum Status

| Chunk | Topic          | Days  | Status      |
|-------|----------------|-------|-------------|
| 0     | Scaffold       | 1–6   | ✅ Done |
| 1     | Oscillators    | 7–12  | ✅ Done |
| 2     | Filters        | 13–20 | ⬜ Not started |
| 3     | Envelopes      | 21–25 | ⬜ Not started |
| 4     | Noise & Random | 26–28 | ⬜ Not started |
| 5     | Delay          | 29–36 | ⬜ Not started |
| 6     | Clock          | 37–41 | ⬜ Not started |
| 7     | Effects        | 42–47 | ⬜ Not started |
| 8     | Granular       | 48–60 | ⬜ Not started |
| Flex  | Revisit / Extend | —  | —           |

Status values: `⬜ Not started` → `🔄 In progress` → `✅ Done`

---

## How to Use This Repository

Each **Chunk** is a topic area with a flexible day budget. Work through the daily steps at 20–30 min/day. At the end of each chunk you have a working, tested Rust module in your library.

**The library is the real output. The days are the process.**
The growing `dsp-lib` crate is what you're actually building — not a series of daily exercises. Treat it that way. Each module should be something you'd be happy to reach for in a future project. The day structure just keeps you moving.

**Extensibility is built in.** If you want to add a new topic — physical modeling, karplus-strong, a vocoder, anything — it just becomes a new chunk. Fill in the template, add a module folder, add an example file. The scaffold from Chunk 0 means anything new plugs in immediately without touching existing code.

**Adjust freely.** Spend more days on a chunk that catches your interest, fewer on one that clicks quickly. Reorder chunks if it makes sense. The curriculum is a map, not a schedule.

**The three questions to answer for every module:**
1. **What does it do?** — What does this thing do to audio or control signals?
2. **What are the knobs?** — What parameters does it expose?
3. **How does it connect?** — What does it take as input, what does it output, what comes before or after it?

---

## Working with Claude Code

This project includes four Claude Code skills in `.claude/skills/`.

### The four skills

**`daily-session`** — your main daily driver. Start here every time you sit down to work. Say "start my session" or "let's work on the curriculum" and it will find where you left off, orient you on the current step, walk you through it in 20–30 minutes, and leave a checkpoint comment in the code when you're done. One step per session, no more.

**`dsp-instructor`** — for when you're stuck or curious. Ask it anything: "how does a filter work", "why does this sound like noise", "what is resonance", "I don't understand polyBLEP". It reads your current code first, leads with an analogy rather than math, and ends with a concrete next action. Use it mid-session when something doesn't click, or between sessions when you want to go deeper on a concept.

**`chunk-reviewer`** — run this when you think a chunk is done. It compiles your code, checks trait compliance, verifies the library has no CPAL dependency, hunts for missing denormal flushes and parameter smoothing, and walks through the module checklist. Only mark a chunk ✅ after it passes.

**`add-chunk`** — for extending the curriculum. Say "add a chunk for reverb" and it creates `curriculum/<N>-reverb.md` with the three questions filled in, scaffolds `src/reverb.rs`, stubs the example file, updates the README status table, and points you at the relevant DaisySP and MI source files. The entire curriculum is designed to grow this way.

### Typical daily flow

```
"start my session"          → daily-session picks up where you left off
"I don't understand X"      → dsp-instructor explains with an analogy
"review my chunk"           → chunk-reviewer checks everything before ✅
"add a chunk for reverb"    → add-chunk scaffolds the next topic
```

### Tips

Keep Claude Code open in your project root so the skills can read `README.md`, the `curriculum/` files, and your `src/` tree. The skills orient themselves by reading those files — the more complete your code is, the better the context they have.

If a skill doesn't trigger automatically, invoke it explicitly: *"use the dsp-instructor skill to explain filters"* or *"run the chunk-reviewer skill on my oscillator code"*.

---

## Library Architecture

```
dsp-playground/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Declares all modules, re-exports shared traits
│   ├── core.rs             # Module entry point for core traits
│   ├── core/
│   │   └── traits.rs       # Process, Transform, Reset, SetSampleRate
│   ├── oscillator.rs       # Module entry point: declares pub mod sine; etc.
│   ├── oscillator/
│   │   ├── sine.rs
│   │   ├── wavetable.rs
│   │   └── blep.rs
│   ├── filter.rs           # Module entry point: declares pub mod one_pole; etc.
│   ├── filter/
│   │   ├── one_pole.rs
│   │   ├── biquad.rs
│   │   └── svf.rs
│   ├── envelope.rs
│   ├── noise.rs
│   ├── delay.rs
│   ├── clock.rs
│   ├── effects.rs
│   └── granular.rs
└── examples/
    ├── chunk0_scaffold.rs
    ├── chunk1_oscillators.rs
    └── chunk2_filters.rs
```
> Module convention: This project uses the modern flat style — src/oscillator.rs is the entry point for the oscillator module, not src/oscillator/mod.rs. Both work identically in Rust, but the flat style keeps your editor cleaner (no tabs full of files all named mod.rs). Submodules like sine live in src/oscillator/sine.rs and are declared inside src/oscillator.rs with pub mod sine;.


### Key Library Principles

**Strict separation of concerns.** `dsp-lib` contains only DSP math. It has no dependency on CPAL or any audio driver. Examples bring in CPAL via `dev-dependencies`. This is what makes the library portable to embedded.

**Everything works at `f32`.** Consistent precision across desktop and embedded (Cortex-M4/M33 have hardware `f32` FPUs). No `f64` in hot paths.

**Sample-accurate, one sample at a time.** `process()` is called once per sample inside a loop. This is slightly less CPU-efficient than block processing (which allows SIMD), but it is far easier to reason about when learning and works identically on embedded. The tradeoff is deliberate.

**Each module struct owns its state.** No shared mutable state, no global variables. Every oscillator, filter, and envelope is a self-contained value you can freely compose.

**Parameters are set by methods, not passed into `process()`.** This keeps the hot path clean. *However*, see the notes on parameter smoothing and denormals below — both will bite you without warning if you skip them.

**Warning: Parameter smoothing is mandatory for anything audio-rate.** Setting a parameter (e.g. filter cutoff) to a new value between samples creates a discontinuity — an audible click or zipper noise. Any parameter driven by an LFO or envelope must be smoothed through a one-pole lowpass filter applied to the parameter value itself. You will build this in Chunk 2, Day 2.1, and use it everywhere from then on. Until then: sudden large parameter jumps will click.

**Warning: Denormal numbers will silently destroy CPU performance.** When an IIR filter or delay feedback loop decays toward silence, the signal can reach subnormal float values (`1e-38` and below). CPUs handle these in software rather than hardware, causing CPU usage to spike from ~1% to near 100% with no obvious cause. The fix is one line per filter state variable: `if value.abs() < 1e-15 { value = 0.0 }`. This is covered in Chunk 2 and must be applied to every feedback structure you build.
