use super::SAMPLE_RATE;

/// An elapsed-seconds window, relative to the start of a rendered clip, to
/// trim [`super::write_wav`]'s output down to — e.g. the web app's "play selection"
/// narrowing a measure-range clip down to just the drag-selected notes'
/// real time span, rather than the full boundary measures they touch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrimWindow {
    pub start_s: f64,
    pub end_s: f64,
}

/// Linear fade applied at a trim's cut points, long enough to mask the
/// sample discontinuity a hard cut would leave (an audible click) but short
/// enough not to perceptibly shorten the clip.
const TRIM_FADE_SAMPLES: usize = SAMPLE_RATE as usize / 100; // 10 ms

/// Extra audio kept past a trim's nominal end, borrowed from the
/// already-rendered synth output that continues naturally beyond it (later
/// notes, reverb tail, or silence) — so a note still decaying or
/// reverberating right at `end_s` isn't chopped off mid-decay.
const TRIM_RELEASE_TAIL_SAMPLES: usize = SAMPLE_RATE as usize * 3 / 10; // 300 ms

/// Cuts `l`/`r` down to `trim`'s elapsed-seconds window — plus
/// [`TRIM_RELEASE_TAIL_SAMPLES`] of natural decay past its end — and fades
/// the new cut points in/out so the splice doesn't click. Sample-accurate:
/// operates on the actual rendered PCM, unlike seeking/pausing an `<audio>`
/// element client-side, which can only approximate a cut and can't avoid it
/// clicking. A no-op if `trim`'s window doesn't leave anything to keep.
pub(super) fn trim_and_fade(l: &mut Vec<f32>, r: &mut Vec<f32>, trim: TrimWindow) {
    let total = l.len();
    let start = ((trim.start_s * f64::from(SAMPLE_RATE)).round() as usize).min(total);
    let end = (((trim.end_s * f64::from(SAMPLE_RATE)).round() as usize)
        + TRIM_RELEASE_TAIL_SAMPLES)
        .min(total);
    if end <= start {
        return;
    }
    *l = l.iter().copied().skip(start).take(end - start).collect();
    *r = r.iter().copied().skip(start).take(end - start).collect();
    fade_edges(l, r);
}

/// Linearly ramps the first/last [`TRIM_FADE_SAMPLES`] of `l`/`r` to/from
/// silence (fewer, symmetrically, for a clip shorter than twice that).
fn fade_edges(l: &mut [f32], r: &mut [f32]) {
    let fade = TRIM_FADE_SAMPLES.min(l.len() / 2);
    for (i, (ls, rs)) in l.iter_mut().zip(r.iter_mut()).enumerate().take(fade) {
        let gain = i as f32 / fade as f32;
        *ls *= gain;
        *rs *= gain;
    }
    for (i, (ls, rs)) in l
        .iter_mut()
        .rev()
        .zip(r.iter_mut().rev())
        .enumerate()
        .take(fade)
    {
        let gain = i as f32 / fade as f32;
        *ls *= gain;
        *rs *= gain;
    }
}

#[cfg(test)]
#[path = "wav_trim_tests.rs"]
mod tests;
