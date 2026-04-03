# Chunk 4 — Signal Chain

**Day budget:** 5 days (Days 26–30)
**Status:** ⬜ Not started

### What does it do?
A signal chain connects DSP modules together so you don't have to manually call `process()` on each one and pass the result to the next. It's the equivalent of patch cables in a modular synth — you describe the routing once, and the chain handles the per-sample wiring.

### What are the knobs?
- **Module order** — which modules are in the chain and in what sequence
- **Modulation routing** — which parameter of which module is controlled by which source (e.g. LFO → filter cutoff)

### How does it connect?
- **Input:** a generator (`Process`) as the head of the chain
- **Output:** the final processed `f32` sample
- **Typical chain:** `Oscillator → Filter → Gain → Output`
- Modulation sources (LFOs, envelopes) connect as side-chains to parameters

### Reference material
- DaisySP: `DaisySP` doesn't have a chain abstraction — each example wires manually
- MI: Elements uses a `Patch` struct to hold all parameters and a `Voice` to process the chain
- Concept: modular synth signal flow, Max/MSP patching, SuperCollider SynthDefs

### Daily steps

#### Day 4.1 — Chain: generator → processors
**Goal:** Build a `Chain` struct that connects one `Process` source through a sequence of `Transform` processors.

The simplest useful abstraction: a struct that holds a generator and a `Vec` (or fixed array) of processors, and calls them in order each sample.

```rust
// Instead of:
let sample = gain.process(filter.process(osc.process()));

// You write:
let sample = chain.process();
```

Think about the trait bounds: the generator implements `Process`, each processor implements `Transform`. The challenge is storing different processor types — this is where Rust's `Box<dyn Transform>` comes in.

**Exercise:** Rebuild the Day 2.6 patch (saw → SVF → output) using your Chain.

---

#### Day 4.2 — Adding and removing processors
**Goal:** Make the chain mutable at runtime.

Add methods to insert, remove, or bypass processors in the chain. This lets you toggle effects on and off, or rearrange the signal path while audio is running.

**Exercise:** Add a "bypass" toggle to your chain that skips a processor without removing it.

---

#### Day 4.3 — Modulation routing
**Goal:** Connect control signals (LFOs, envelopes) to module parameters.

This is the hard part. A modulation route says "take the output of this source and feed it to this parameter of this module." The challenge: Rust's ownership model makes it tricky to have one module write to another module's parameter.

Approaches to consider:
- **Parameter bus:** a shared `Vec<f32>` of modulation values, indexed by ID. Sources write, destinations read.
- **Closure-based:** each modulation route is a closure that reads a source and calls a setter.
- **Message passing:** modulation values are sent via channels.

Start with the simplest approach that works. You can refine later.

**Exercise:** Wire an LFO to the SVF cutoff through the modulation system instead of manually.

---

#### Day 4.4 — Patch: a complete voice
**Goal:** Combine Chain + modulation into a `Patch` struct that represents one complete synth voice.

A Patch holds the signal chain, the modulation routes, and processes everything in one call. This is the abstraction you'll use for every example from now on.

**Exercise:** Rebuild the Day 2.6 patch as a Patch: saw → smoothed SVF → output, with LFO modulating cutoff.

---

#### Day 4.5 — Review and clean up
- Ensure Chain and Patch implement `Process` so they can be nested
- Write `examples/chunk4_signal_chain.rs` — demonstrate building a patch from parts
- Verify all previous examples still compile

### Module checklist
- [ ] `Chain` struct with `Process` trait (generator + transforms)
- [ ] Dynamic add/remove/bypass of processors
- [ ] Modulation routing system (LFO/envelope → parameter)
- [ ] `Patch` struct combining chain + modulation
- [ ] Chain and Patch implement `Process` for nesting
- [ ] All implement `Reset` + `SetSampleRate`
- [ ] Example runs: full patch with modulated filter, built via Chain/Patch API
- [ ] Previous examples still compile and run
