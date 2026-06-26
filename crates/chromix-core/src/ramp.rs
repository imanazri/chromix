use super::{gamut, Oklch, Srgb};
use serde::Serialize;

pub const DEFAULT_STEPS: &[u16] = &[50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

const LIGHTEST: f64 = 0.971;
const DARKEST: f64 = 0.220;

const TUNED_LIGHTNESS: [(u16, f64); 11] = [
    (50, 0.971),
    (100, 0.936),
    (200, 0.885),
    (300, 0.808),
    (400, 0.704),
    (500, 0.610),
    (600, 0.530),
    (700, 0.452),
    (800, 0.378),
    (900, 0.310),
    (950, 0.220),
];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RampEntry {
    pub step: u16,
    pub oklch: Oklch,
    pub srgb: Srgb,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ramp {
    pub entries: Vec<RampEntry>,
}

pub fn generate_ramp(base: Oklch) -> Ramp {
    generate_ramp_with_steps(base, DEFAULT_STEPS)
}

pub fn generate_ramp_with_steps(base: Oklch, steps: &[u16]) -> Ramp {
    let n = steps.len();
    let mut entries = Vec::with_capacity(n);

    for (i, &step) in steps.iter().enumerate() {
        let l = target_lightness(step, i, n);
        let t = if n > 1 { i as f64 / (n - 1) as f64 } else { 0.5 };
        let taper = 1.0 - (2.0 * t - 1.0).powi(2) * 0.5;
        let chroma = base.c * taper;

        let target = Oklch::new(l, chroma, base.h);
        let oklch = gamut::clamp_to_gamut(target);
        let srgb = oklch.to_srgb();
        let hex = srgb.to_hex();

        entries.push(RampEntry {
            step,
            oklch,
            srgb,
            hex,
        });
    }

    Ramp { entries }
}

fn target_lightness(step: u16, index: usize, count: usize) -> f64 {
    // Check if step is in the tuned table
    for &(s, l) in &TUNED_LIGHTNESS {
        if s == step {
            return l;
        }
    }

    // Positional interpolation for non-default steps
    let t = if count > 1 { index as f64 / (count - 1) as f64 } else { 0.5 };
    LIGHTEST + (DARKEST - LIGHTEST) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ramp_length() {
        let base = Oklch::new(0.6, 0.1, 180.0);
        let ramp = generate_ramp(base);
        assert_eq!(ramp.entries.len(), DEFAULT_STEPS.len());
    }

    #[test]
    fn test_ramp_monotonic_lightness() {
        let base = Oklch::new(0.6, 0.15, 180.0);
        let ramp = generate_ramp(base);

        for window in ramp.entries.windows(2) {
            assert!(window[0].oklch.l >= window[1].oklch.l);
        }
    }

    #[test]
    fn test_all_in_gamut() {
        let base = Oklch::new(0.6, 0.2, 180.0);
        let ramp = generate_ramp(base);

        for entry in &ramp.entries {
            assert!(gamut::in_gamut(entry.oklch));
        }
    }

    #[test]
    fn test_custom_steps() {
        let base = Oklch::new(0.6, 0.1, 180.0);
        let custom_steps: Vec<u16> = vec![100, 300, 500, 700];
        let ramp = generate_ramp_with_steps(base, &custom_steps);
        assert_eq!(ramp.entries.len(), 4);
    }
}