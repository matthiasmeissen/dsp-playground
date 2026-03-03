---
name: chunk-reviewer
description: Review a completed DSP curriculum chunk implementation for correctness, Rust best practices, and curriculum conventions. Use when the user says "review my chunk", "check my implementation", "is this correct", "done with chunk N", "review my filter/oscillator/envelope code", or when they want feedback before marking a chunk as done.
allowed-tools: Read, Grep, Glob, Bash
---

# Chunk Reviewer

Reviews a completed chunk implementation against the curriculum's architectural rules, DSP correctness requirements, and the chunk's own checklist. Gives actionable feedback, not just a pass/fail.

## Before reviewing

Ask the user which chunk they want reviewed, then read everything relevant:

```bash
# Read the chunk file to get the checklist
cat curriculum/<N>-*.md

# Find all source files for this chunk
find src/<module_name>/ -name "*.rs" | sort

# Find the example file
ls examples/chunk<N>_*.rs

# Check that it compiles
cargo build 2>&1
cargo run --example chunk<N>_* 2>&1 | head -20
```

Read all the source files before forming any opinion.

## Review checklist — apply to every chunk

### 1. Trait compliance

```rust
// Every module must implement the right traits:
// Generator (produces signal, no input):    Process + Reset + SetSampleRate
// Processor (transforms signal, has input): Transform + Reset + SetSampleRate
```

Check:
- [ ] `Process` (generators) or `Transform` (processors) — correct one chosen?
- [ ] `Reset` implemented — does it actually reset all stateful fields to initial values?
- [ ] `SetSampleRate` implemented — does it recalculate derived values (coefficients, phase increment)?
- [ ] No audio logic in trait-irrelevant methods

### 2. Library separation

```bash
# The library must have no runtime dependencies
grep -A5 "\[dependencies\]" Cargo.toml | grep -v "^#" | grep -v "^\[" | grep -v "^$"
```

Check:
- [ ] `[dependencies]` section in Cargo.toml is empty (no cpal, no hound)
- [ ] No `use cpal` or `use hound` in any `src/` file
- [ ] No `std::thread`, `std::sync`, or IO in `src/` (only in `examples/`)

### 3. Denormal handling (IIR and feedback structures only)

Applies to: filters, delay lines, reverbs, anything with a feedback loop.

```bash
grep -n "1e-15\|flush\|denormal\|subnormal" src/<module>/*.rs
```

Check:
- [ ] Every feedback state variable has a denormal flush: `if x.abs() < 1e-15 { x = 0.0; }`
- [ ] Flush is applied *after* the state update, not before
- [ ] Biquad: both `y1` and `y2` flushed; SVF: both `lp` and `bp` flushed

### 4. Parameter smoothing (modulated parameters only)

Applies to: any parameter expected to be driven by an LFO, envelope, or real-time control.

Check:
- [ ] Is cutoff frequency smoothed before being passed to filter coefficients?
- [ ] Is there a `OnePole` smoother in the example for any swept parameter?
- [ ] No direct `set_param(lfo.process())` without a smoother in between

### 5. Audio range and gain staging

```bash
# Check for magic amplitude constants
grep -n "0\.[0-9]\|amplitude\|gain\|tanh\|clip" examples/chunk<N>_*.rs
```

Check:
- [ ] Output values stay within -1.0..1.0 under normal use
- [ ] Multiple voices mixed together are divided by voice count (or use tanh as a soft limit)
- [ ] No hardcoded amplitudes > 0.5 in examples (0.2–0.3 is a good default)

### 6. Phase accumulator correctness (oscillators only)

```bash
grep -n "phase\|% 1.0\|fract\|wrap" src/oscillator/*.rs
```

Check:
- [ ] Phase wrapped with `% 1.0` (or `.fract()`) — not allowed to grow unbounded
- [ ] `phase_increment` computed as `freq / sample_rate`, not `freq * TAU / sample_rate`
  (TAU multiplication happens at the sin() call, not in the increment)
- [ ] Frequency change preserves phase continuity (no phase reset on frequency change)

### 7. Code quality

- [ ] No `unwrap()` in library code (examples may use `?` or `expect()` with a message)
- [ ] No `println!` in `src/` (debug output belongs in examples)
- [ ] Public API is clean: only the struct and its methods are `pub`, internal helpers are private
- [ ] `new()` takes `sample_rate: f32` as its first argument (curriculum convention)

### 8. Example quality

- [ ] Example actually produces audible output (not just silence or a compile-check)
- [ ] The key concept of the chunk is demonstrated, not just "it runs"
- [ ] Comments explain what the listener should hear
- [ ] `render_to_wav` used for at least one non-trivial output (recommended)

## Chunk-specific checks

### Chunk 1 — Oscillators
- [ ] polyBLEP applied at *every* discontinuity (saw: wrap; square: wrap and midpoint)
- [ ] Wavetable uses linear interpolation between entries (no nearest-neighbor rounding)
- [ ] LFO reuses oscillator code at < 20Hz (not a separate implementation)
- [ ] Unison voices use `cents_to_ratio()` correctly: `2.0f32.powf(cents / 1200.0)`

### Chunk 2 — Filters
- [ ] OnePole has both lowpass and highpass modes
- [ ] Biquad coefficient formulas match a known reference (EarLevel or Audio EQ Cookbook)
- [ ] SVF returns all three outputs (LP, BP, HP) — not just one
- [ ] DC blocker exists as a utility (OnePole highpass at ~10Hz)

## How to deliver feedback

Structure your response as:

**1. What's working well** — be specific, cite the actual code
**2. Issues to fix** — prioritised: blocking (must fix) vs suggested (nice to have)
**3. For each blocking issue:** show the exact fix, not just the problem
**4. Checklist status** — which items pass, which need work

Be direct. The student's goal is a real, reusable library — not just passing a checklist. If something will cause a subtle bug later (like missing denormal flush), explain *why* it matters, not just that it's missing.

## After review

If all blocking issues are resolved:

```bash
# Update the chunk status in two places:
# 1. In curriculum/<N>-<topic>.md — change the Status line:
#    **Status:** 🔄 In progress  →  **Status:** ✅ Done
# 2. In README.md — update the status table row:
#    🔄 In progress  →  ✅ Done
```

Tell the student they're ready to move to the next chunk and what the first day of that chunk will build on from what they just completed.