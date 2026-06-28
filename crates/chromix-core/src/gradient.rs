use super::{gamut, Oklch, Srgb};
use serde::Serialize;

/// Total hue span (degrees) for the analogous variant, centered on the base hue.
const ANALOGOUS_SPAN: f64 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum GradientKind {
    Analogous,
    Complementary,
    Monochromatic,
}

/// Which background the gradient is tuned for. The light variant uses darker,
/// more saturated colors that read on a light UI; the dark variant uses lighter
/// colors that read on a dark UI.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Appearance {
    Light,
    Dark,
}

impl Appearance {
    /// Lightness held constant for the hue-varying variants.
    fn mid_lightness(self) -> f64 {
        match self {
            Appearance::Light => 0.62,
            Appearance::Dark => 0.80,
        }
    }

    /// (lightest, darkest) lightness for the monochromatic sweep.
    fn mono_band(self) -> (f64, f64) {
        match self {
            Appearance::Light => (0.74, 0.42),
            Appearance::Dark => (0.88, 0.60),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GradientStop {
    pub position: f64,
    pub oklch: Oklch,
    pub srgb: Srgb,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Gradient {
    pub kind: GradientKind,
    pub appearance: Appearance,
    pub stops: Vec<GradientStop>,
}

pub fn generate_gradient(
    base: Oklch,
    kind: GradientKind,
    appearance: Appearance,
    n: usize,
) -> Gradient {
    let mut stops = Vec::with_capacity(n);

    for i in 0..n {
        let t = if n > 1 {
            i as f64 / (n - 1) as f64
        } else {
            0.0
        };

        let (l, c, h) = sample(base, kind, appearance, t);
        let target = Oklch::new(l, c, normalize_hue(h));
        let oklch = gamut::clamp_to_gamut(target);
        let srgb = oklch.to_srgb();
        let hex = srgb.to_hex();

        stops.push(GradientStop {
            position: t,
            oklch,
            srgb,
            hex,
        });
    }

    Gradient {
        kind,
        appearance,
        stops,
    }
}

fn sample(base: Oklch, kind: GradientKind, appearance: Appearance, t: f64) -> (f64, f64, f64) {
    match kind {
        GradientKind::Analogous => {
            let h = base.h - ANALOGOUS_SPAN / 2.0 + ANALOGOUS_SPAN * t;
            (appearance.mid_lightness(), base.c, h)
        }
        GradientKind::Complementary => {
            let h = base.h + 180.0 * t;
            (appearance.mid_lightness(), base.c, h)
        }
        GradientKind::Monochromatic => {
            let (hi, lo) = appearance.mono_band();
            let l = hi + (lo - hi) * t;
            // Taper chroma toward the ends so the mids stay saturated.
            let taper = 1.0 - (2.0 * t - 1.0).powi(2) * 0.5;
            (l, base.c * taper, base.h)
        }
    }
}

fn normalize_hue(h: f64) -> f64 {
    h.rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_length() {
        let base = Oklch::new(0.6, 0.15, 250.0);
        let g = generate_gradient(base, GradientKind::Analogous, Appearance::Dark, 36);
        assert_eq!(g.stops.len(), 36);
    }

    #[test]
    fn test_all_in_gamut() {
        let base = Oklch::new(0.6, 0.25, 250.0);
        for kind in [
            GradientKind::Analogous,
            GradientKind::Complementary,
            GradientKind::Monochromatic,
        ] {
            for appearance in [Appearance::Light, Appearance::Dark] {
                let g = generate_gradient(base, kind, appearance, 36);
                for stop in &g.stops {
                    assert!(gamut::in_gamut(stop.oklch), "{:?} out of gamut", kind);
                }
            }
        }
    }

    #[test]
    fn test_monochromatic_lightness_monotonic() {
        let base = Oklch::new(0.6, 0.12, 250.0);
        let g = generate_gradient(base, GradientKind::Monochromatic, Appearance::Dark, 24);
        for window in g.stops.windows(2) {
            assert!(window[0].oklch.l >= window[1].oklch.l);
        }
    }

    #[test]
    fn test_dark_is_lighter_than_light() {
        let base = Oklch::new(0.6, 0.12, 250.0);
        let light = generate_gradient(base, GradientKind::Analogous, Appearance::Light, 3);
        let dark = generate_gradient(base, GradientKind::Analogous, Appearance::Dark, 3);
        assert!(dark.stops[1].oklch.l > light.stops[1].oklch.l);
    }

    #[test]
    fn test_analogous_hue_endpoints() {
        let base = Oklch::new(0.7, 0.1, 100.0);
        let g = generate_gradient(base, GradientKind::Analogous, Appearance::Dark, 3);
        assert!((g.stops[0].oklch.h - (100.0 - 30.0)).abs() < 1e-6);
        assert!((g.stops[2].oklch.h - (100.0 + 30.0)).abs() < 1e-6);
    }

    #[test]
    fn test_complementary_hue_endpoints() {
        let base = Oklch::new(0.7, 0.1, 30.0);
        let g = generate_gradient(base, GradientKind::Complementary, Appearance::Dark, 3);
        assert!((g.stops[0].oklch.h - 30.0).abs() < 1e-6);
        assert!((g.stops[2].oklch.h - 210.0).abs() < 1e-6);
    }

    #[test]
    fn test_hue_wraps() {
        let base = Oklch::new(0.7, 0.1, 350.0);
        let g = generate_gradient(base, GradientKind::Complementary, Appearance::Dark, 3);
        // 350 + 180 = 530 -> 170 after normalization.
        assert!((g.stops[2].oklch.h - 170.0).abs() < 1e-6);
    }
}
