/// Soft-knee safety limiter.
///
/// Transparent below `threshold` (bit-identical pass-through).
/// Above threshold, applies a smooth tanh-shaped curve that asymptotically
/// approaches `ceiling`. No state, no latency, no artifacts at normal levels.
///
/// # Why not hard clip?
/// A hard clip at ±1.0 (`clamp`) is a discontinuity in the signal's derivative.
/// That sharp corner generates harmonics at every frequency — the "harsh digital
/// distortion" sound. A soft limiter rounds the corner so the transition from
/// linear to limited is smooth, producing far fewer (and lower-order) harmonics.
///
/// # Where to use it
/// At the very end of your signal chain, after all processing:
/// ```
/// use dsp_lib::core::limiter::safety_limit;
///
/// let output = 1.5_f32; // some hot signal
/// let safe = safety_limit(output);
/// // send `safe` to audio output
/// ```

const THRESHOLD: f32 = 1.0;
const CEILING: f32 = 1.2;

/// Soft-limit a sample. Transparent below ±THRESHOLD, smoothly approaches
/// ±CEILING above it. Stateless, zero latency.
///
/// The curve is:
///   - |x| <= threshold  →  x  (linear, untouched)
///   - |x| > threshold   →  threshold + (ceiling - threshold) * tanh((|x| - threshold) / (ceiling - threshold))
///
/// This maps the "overflow" region [threshold..inf) onto [threshold..ceiling)
/// using tanh, which is infinitely differentiable — no sharp corners.
#[inline]
pub fn safety_limit(x: f32) -> f32 {
    safety_limit_with(x, THRESHOLD, CEILING)
}

/// Soft-limit with custom threshold and ceiling.
///
/// - `threshold`: level below which signal passes untouched (e.g. 0.95)
/// - `ceiling`: maximum output level (e.g. 1.0)
///
/// The knee width is `ceiling - threshold`. A wider knee (lower threshold)
/// engages earlier but more gently. A narrow knee is more transparent
/// but transitions more abruptly.
#[inline]
pub fn safety_limit_with(x: f32, threshold: f32, ceiling: f32) -> f32 {
    let knee = ceiling - threshold;
    let abs_x = x.abs();

    if abs_x <= threshold {
        return x;
    }

    let excess = (abs_x - threshold) / knee;
    let limited = threshold + knee * soft_saturate(excess);

    if x > 0.0 { limited } else { -limited }
}

/// Process a buffer in-place with the default safety limiter.
pub fn safety_limit_block(buffer: &mut [f32]) {
    for sample in buffer.iter_mut() {
        *sample = safety_limit(*sample);
    }
}

/// Soft saturation: x / (1 + |x|).
/// Always in (-1, 1) by construction — no clamp needed.
/// Cheaper than tanh and algebraically bounded for any input.
#[inline]
fn soft_saturate(x: f32) -> f32 {
    if !x.is_finite() { return if x > 0.0 { 1.0 } else { -1.0 }; }
    x / (1.0 + x.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparent_below_threshold() {
        for &x in &[0.0, 0.1, -0.5, 0.94, -1.0, 0.0001] {
            let out = safety_limit(x);
            assert_eq!(out, x, "should be transparent at x={x}");
        }
    }

    #[test]
    fn limits_above_threshold() {
        for &x in &[1.5, 2.0, 5.0, 10.0, 100.0] {
            let out = safety_limit(x);
            assert!(out < x, "should reduce x={x}, got {out}");
            assert!(out >= THRESHOLD, "should stay above threshold, got {out}");
            assert!(out < CEILING, "should stay below ceiling, got {out}");
        }
    }

    #[test]
    fn negative_mirrors_positive() {
        for &x in &[1.0, 2.0, 5.0] {
            let pos = safety_limit(x);
            let neg = safety_limit(-x);
            assert!(
                (pos + neg).abs() < 1e-10,
                "should be symmetric: f({x})={pos}, f(-{x})={neg}"
            );
        }
    }

    #[test]
    fn monotonic() {
        let mut prev = safety_limit(0.0);
        for i in 1..1000 {
            let x = i as f32 * 0.01;
            let out = safety_limit(x);
            assert!(
                out >= prev,
                "should be monotonic: f({})={} < f({})={}",
                x - 0.01, prev, x, out
            );
            prev = out;
        }
    }

    #[test]
    fn never_exceeds_ceiling() {
        for &x in &[10.0, 100.0, 1000.0, f32::MAX / 2.0] {
            let out = safety_limit(x);
            assert!(
                out <= CEILING,
                "should never exceed ceiling: f({x})={out}"
            );
        }
    }

    #[test]
    fn smooth_at_threshold() {
        let below = safety_limit(THRESHOLD - 0.001);
        let at = safety_limit(THRESHOLD);
        let above = safety_limit(THRESHOLD + 0.001);

        let diff_below = at - below;
        let diff_above = above - at;

        assert!(
            (diff_below - diff_above).abs() < 0.005,
            "transition should be smooth: delta_below={diff_below:.6}, delta_above={diff_above:.6}"
        );
    }

    #[test]
    fn block_processing_matches() {
        let input = vec![0.5, 0.95, 1.0, 1.5, -2.0, 0.0];
        let mut buffer = input.clone();
        safety_limit_block(&mut buffer);
        for (x, out) in input.iter().zip(buffer.iter()) {
            assert_eq!(*out, safety_limit(*x));
        }
    }
}
