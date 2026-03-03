---
name: add-chunk
description: Scaffold a new DSP learning chunk for the dsp-lib curriculum. Use when adding a new topic to the curriculum, creating a new DSP module category, or when the user says "add a chunk", "new chunk", "add topic", or mentions wanting to learn a new DSP concept like reverb, chorus, karplus-strong, physical modeling, pitch shifting, etc.
allowed-tools: Read, Write, Edit, Glob, Bash
---

# Add Chunk — DSP Curriculum Scaffolder

Scaffolds everything needed for a new curriculum chunk: a new file in curriculum/, an update to README.md, the Rust module files, and the example stub. Follows the exact conventions established in Chunks 0–2.

## What you need from the user

Before doing anything, confirm:
1. **Topic name** — what DSP concept is this? (e.g. "Reverb", "Pitch Shifting", "Karplus-Strong")
2. **Chunk number** — check CURRICULUM.md status table for the next available number
3. **Day budget** — rough estimate (simple = 3–5 days, medium = 6–8 days, complex = 10+ days)

If the user hasn't provided these, ask for them before proceeding.

## Step-by-step process

### Step 1: Read existing curriculum for context

Read the project structure to find:
- The next chunk number from the README.md status table
- Existing chunk files in `curriculum/` to stay consistent in style

```bash
grep -n "^| " README.md | tail -20        # show status table
ls curriculum/                             # list existing chunk files
```

### Step 2: Fill in the three questions

Before writing any files, answer these in the CURRICULUM.md entry:

1. **What does it do?** — What does this module do to audio or control signals in plain language? One paragraph, no math.
2. **What are the knobs?** — List every user-facing parameter with units and typical ranges.
3. **How does it connect?** — Input type, output type, what comes before/after in a signal chain.

Use `curriculum/1-oscillators.md` and `curriculum/2-filters.md` as style references.

### Step 3: Scaffold the Rust module files

Use the modern Rust module convention — **no `mod.rs`**. Instead:

- The module entry point is `src/<module_name>.rs` (declared in `src/lib.rs`)
- Submodules live in `src/<module_name>/sine.rs`, `src/<module_name>/wavetable.rs`, etc.
- `src/<module_name>.rs` declares those submodules with `pub mod sine;`

```
src/
├── lib.rs                    ← add: pub mod <module_name>;
├── <module_name>.rs          ← module entry point
└── <module_name>/
    └── (submodule .rs files added as you build them)
```

**`src/<module_name>.rs`** template:
```rust
//! # <Topic Name>
//!
//! What it does: <one sentence>
//! What the knobs are: <comma-separated list>
//! How it connects: <input> → <o>
//!
//! Reference:
//! - DaisySP: <class name if applicable>
//! - MI: <module/file if applicable>

// Submodules declared here as you build them:
// pub mod <submodule>;
// pub use <submodule>::<Type>;
```

**`src/lib.rs`** — add the new module declaration:
```rust
pub mod core;
pub mod oscillator;
pub mod filter;
pub mod <module_name>;   // ← add this line
```

> **Why no `mod.rs`?** The compiler accepts both `src/foo/mod.rs` and `src/foo.rs` as the entry point for module `foo`. The flat `src/foo.rs` style is now idiomatic — it avoids having many files all named `mod.rs` open simultaneously in your editor, and makes the `src/` directory much easier to scan.

### Step 4: Create the example stub

Create `examples/chunk<N>_<snake_case_topic>.rs`:

```rust
//! Chunk <N> — <Topic Name>
//!
//! Run with: cargo run --example chunk<N>_<snake_case>
//!
//! This example is built up day by day through the chunk.
//! Uncomment sections as you complete each daily step.

// Day <N>.1 stub — replace with real code as you progress
fn main() {
    println!("Chunk <N> — <Topic Name>");
    println!("Start with Day <N>.1 in curriculum/<N>-<snake_case_topic>.md");
}
```

### Step 5: Create the chunk file and update README.md

**1. Create `curriculum/<N>-<snake_case_topic>.md`** — the full chunk entry as a standalone file:

```bash
touch curriculum/<N>-<snake_case_topic>.md
```

**2. Update the status table in README.md** — add a new row:
```
| <N>   | <Topic>        | <day range> | ⬜ Not started |
```

The chunk file should follow this structure (matching existing chunk files):

```markdown
## Chunk <N> — <Topic Name>
**Day budget:** <X> days
**Status:** ⬜ Not started

### What does it do?
<one paragraph>

### What are the knobs?
- **<Param>** (<unit>) — <description, typical range>

### How does it connect?
- **Input:** <description>
- **Output:** <description>
- **Typical chain:** <Module A> → <This Module> → <Module B>

### Reference material
- DaisySP: `<ClassName>` (`daisysp/Source/<path>`)
- MI: `<module>/<file>.cc`
- Other: <link>

### Daily steps

#### Day <N>.1 — <First step title>
**Goal:** <one sentence goal>

*(Fill in — follow the Chunk 1/2 pattern)*

### Module checklist
- [ ] Struct defined with all state fields
- [ ] `Process` or `Transform` trait implemented
- [ ] `Reset` trait implemented
- [ ] `SetSampleRate` trait implemented
- [ ] Denormal flush on all feedback state variables (if IIR/feedback)
- [ ] Parameters smoothed via `OnePole` before hot path (if modulated)
- [ ] Example file runs and produces audio
- [ ] Behavior matches reference (by ear or via render_to_wav)
```

### Step 6: Suggest reference material

Based on the topic, point the user to the most relevant source files:

| Topic | DaisySP | Mutable Instruments |
|---|---|---|
| Reverb | `ReverbSc`, `Reverbsc.h` | elements/dsp/resonator.h |
| Delay | `DelayLine.h` | clouds/dsp/granular_processor.h |
| Chorus/Flanger | `Chorus.h` | — |
| Karplus-Strong | `KarplusStrong.h` | — |
| Pitch Shift | `PitchShifter.h` | — |
| Granular | `GranularSampler` | clouds/dsp/ |
| Physical Modeling | `ModalVoice`, `StringVoice` | elements/dsp/ |
| Compressor | `Compressor.h` | — |
| Wavefolder | — | warps/dsp/waveshaper.h |

If the topic isn't in the table, search the repos by concept.

## Validation

After scaffolding, confirm:
- `cargo build` still passes (new module is not yet `pub mod`'d in lib.rs — that's intentional until first code lands)
- `curriculum/<N>-<snake_case_topic>.md` exists with the three questions answered (not left as placeholders)
- README.md status table has the new row
- The example file exists and has the correct run command in the comment

## Output to user

Tell the user:
1. What files were created/modified
2. Where to start (`curriculum/<N>-<snake_case_topic>.md`, Day N.1)
3. Which reference files to look at first
4. The `cargo run --example` command for their new example