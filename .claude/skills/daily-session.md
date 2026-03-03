---
name: daily-session
description: Run a focused 20-30 minute DSP learning session for the dsp-lib curriculum. Use when the user says "start my session", "let's work on the curriculum", "what's next", "continue where I left off", "daily session", or just sits down to work on dsp-lib without a specific goal. Reads current progress and guides through exactly one daily step.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
---

# Daily Session

Runs a focused, timeboxed learning session. Reads where the student is, picks up the current daily step, and guides them through it in 20–30 minutes. One step at a time.

## Session startup sequence

### 1. Find the current position

```bash
# Find the in-progress chunk from README.md status table
grep -n "🔄 In progress" README.md

# If nothing is in progress, find the next not-started chunk
grep -n "⬜ Not started" README.md | head -3

# List all chunk files to find the current one
ls curriculum/

# Read the current chunk file
cat curriculum/<N>-<topic>.md
```

### 2. Find the current day step

Look at which daily steps have code that exists vs which are still stubs:

```bash
# Check what source files exist in the current chunk's module
find src/ -name "*.rs" | grep -v target | sort

# Check the example file for completed sections
cat examples/chunk<N>_*.rs 2>/dev/null | head -50

# Check git log for recent work (if git is available)
git log --oneline -5 2>/dev/null || echo "no git"
```

Cross-reference with the chunk file's daily steps to determine which step to run today.

### 3. Orient the student

Tell them clearly:
- **Where they are:** "You're on Chunk 1, Day 1.3 — Wavetable Oscillator"
- **What came before:** one sentence recap of the previous step
- **What today builds:** one sentence on the goal
- **Time estimate:** "This step should take about 20–25 minutes"

Do not dump the full step content yet. Ask: "Ready to start?"

## Running the session

### Present the step in phases

**Phase 1 — The mental model (5 min)**
Before any code: answer the three questions for today's concept.
- What does it do?
- What are the knobs?
- How does it connect to what you built yesterday?

Keep this to 3–5 sentences. The student should be able to explain it back to you in one sentence before touching the keyboard.

**Phase 2 — The implementation (15–20 min)**
Walk through the code from the chunk file (`curriculum/<N>-<topic>.md`). Don't dump it all at once:
1. Show the struct definition — ask the student to think about what state is needed
2. Show the key method — explain each line once, briefly
3. Show the trait implementation
4. Have them type it in (don't just copy-paste everything)

If the step has an exercise, make sure they do it — don't skip it.

**Phase 3 — Verify it works (5 min)**
```bash
cargo build
cargo run --example chunk<N>_*
```

If it doesn't compile, help debug. Read the error message carefully — most early DSP Rust errors are:
- Missing `use` import for a trait (Process, Transform, Reset)
- Borrow checker issue from moving into closure
- Type mismatch (usize vs f32 in array indexing)

### If the student is faster than 20 minutes

If they finish early and still have time:
- Do the exercise if they skipped it
- Suggest rendering to WAV and opening in Audacity
- Preview what Day N+1 will build
- Do NOT start the next step — that's tomorrow's session

### If the student is stuck

Apply the instructor approach (see `dsp-instructor` skill):
1. Read their code before guessing what's wrong
2. Go back to the mental model — do they understand what the code should do?
3. Check the "why does this sound wrong" checklist
4. Show the minimal fix, not a full rewrite

## Session close

At the end of every session:

**1. Update the chunk file and README.md**

Mark the chunk as in-progress in `curriculum/<N>-<topic>.md` if not already:
```
**Status:** 🔄 In progress
```
Also update the matching row in the README.md status table.

**2. Leave a breadcrumb comment in the example file**

```rust
// SESSION CHECKPOINT: Completed Day 1.3 — Wavetable with linear interpolation
// Next: Day 1.4 — FM basics (connect osc output to freq input of another osc)
```

**3. Tell the student**
- What they built today in one sentence
- What connects to it next session
- Optionally: a "thing to notice" — something to listen for or think about between sessions

Example close:
> "Today you built a wavetable oscillator with linear interpolation between table entries. Next session you'll use its output to modulate another oscillator's frequency — that's the foundation of FM synthesis. Between now and then, try changing the table contents to something other than a sine and notice how the timbre changes."

## Session discipline rules

- **One step per session.** Never start a second daily step in the same session, even if there's time left. The 20–30 min rhythm matters more than speed.
- **Verify before moving on.** The step is not done until `cargo build` passes and you can hear output.
- **Mental model before code.** If the student can't explain what the code does, the session isn't done even if it compiles.
- **No rabbit holes.** If a concept triggers curiosity about something off-curriculum (e.g. FFT, convolution, ML synthesis), note it in the `## Notes & Discoveries` section of README.md and stay on track.

## Logging discoveries

Anything interesting the student notices — a sound they didn't expect, a Rust pattern that surprised them, a connection between two concepts — add to the Notes section:

```bash
# Append to the Notes & Discoveries section at the bottom of README.md
```

Example entries:
- "Day 1.3: linear interpolation at 2048 entries is indistinguishable from computed sin() by ear"
- "Day 2.1: OnePole smoother with coeff=0.001 sounds like a slow filter sweep — basically an LFO"
- "Day 0.6: Relaxed ordering feels weird but makes sense — each param is independent, no ordering needed"