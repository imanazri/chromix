use super::{Oklch, convert};

pub const EPS: f64 = 1e-4;

/// Check if a color is in the sRGB gamut
pub fn in_gamut(c: Oklch) -> bool {
    let (r, g, b) = convert::oklch_to_linear_srgb(c);
    (-EPS..=1.0 + EPS).contains(&r) && (-EPS..=1.0 + EPS).contains(&g) && (-EPS..=1.0 + EPS).contains(&b)
}

/// Clamp an out-of-gamut color by reducing chroma
pub fn clamp_to_gamut(c: Oklch) -> Oklch {
    if in_gamut(c) {
        return c;
    }

    let mut lo = 0.0;
    let mut hi = c.c;

    // Exactly 24 binary search iterations
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if in_gamut(Oklch::new(c.l, mid, c.h)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Oklch::new(c.l, lo, c.h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Srgb;

    #[test]
    fn test_in_gamut_gray() {
        // Grays should always be in gamut
        let gray = Srgb::new(128, 128, 128);
        let oklch = gray.to_oklch();
        assert!(in_gamut(oklch));
    }

    #[test]
    fn test_clamp_idempotent() {
        // Create an out-of-gamut color by using high chroma
        let out_of_gamut = Oklch::new(0.6, 0.3, 0.0);
        let clamped_once = clamp_to_gamut(out_of_gamut);
        let clamped_twice = clamp_to_gamut(clamped_once);

        // Clamping twice should change chroma by < 1e-9
        assert!((clamped_once.c - clamped_twice.c).abs() < 1e-9);
        assert!(in_gamut(clamped_once));
    }

    #[test]
    fn test_in_gamut_unchanged() {
        // Use a gray color which is always in gamut
        let oklch = Oklch::new(0.5, 0.0, 0.0);
        let result = clamp_to_gamut(oklch);
        // Gray should be unchanged (chroma = 0)
        assert!((result.l - oklch.l).abs() < 1e-9);
        assert!((result.c - oklch.c).abs() < 1e-9);
        assert!((result.h - oklch.h).abs() < 1e-9);
    }
}