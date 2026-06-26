use super::{Oklch, Srgb};

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn linear_srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    #[allow(non_snake_case)]
    let L = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    (L, a, b)
}

#[allow(non_snake_case)]
fn oklab_to_linear_srgb(L: f64, a: f64, b: f64) -> (f64, f64, f64) {
    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    (r, g, b)
}

pub fn srgb_to_oklch(srgb: Srgb) -> Oklch {
    let r = srgb.r as f64 / 255.0;
    let g = srgb.g as f64 / 255.0;
    let b = srgb.b as f64 / 255.0;

    let r_lin = srgb_to_linear(r);
    let g_lin = srgb_to_linear(g);
    let b_lin = srgb_to_linear(b);

    let (l, a, b) = linear_srgb_to_oklab(r_lin, g_lin, b_lin);

    let c = (a * a + b * b).sqrt();
    let h = b.atan2(a).to_degrees();
    let h = if h < 0.0 { h + 360.0 } else { h };

    Oklch::new(l, c, h)
}

pub fn oklch_to_srgb(oklch: Oklch) -> Srgb {
    let h_rad = oklch.h.to_radians();
    let a = oklch.c * h_rad.cos();
    let b = oklch.c * h_rad.sin();
    let (r, g, b) = oklab_to_linear_srgb(oklch.l, a, b);

    let r = linear_to_srgb(r).clamp(0.0, 1.0) * 255.0;
    let g = linear_to_srgb(g).clamp(0.0, 1.0) * 255.0;
    let b = linear_to_srgb(b).clamp(0.0, 1.0) * 255.0;

    Srgb::new(r.round() as u8, g.round() as u8, b.round() as u8)
}

pub fn oklch_to_linear_srgb(oklch: Oklch) -> (f64, f64, f64) {
    let h_rad = oklch.h.to_radians();
    let a = oklch.c * h_rad.cos();
    let b = oklch.c * h_rad.sin();
    oklab_to_linear_srgb(oklch.l, a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_linear_roundtrip() {
        for v in [0.0, 0.01, 0.04045, 0.1, 0.5, 1.0] {
            let linear = srgb_to_linear(v);
            let back = linear_to_srgb(linear);
            assert!((v - back).abs() < 1e-6, "roundtrip failed for v={}", v);
        }
    }

    #[test]
    fn test_roundtrip_known_colors() {
        let colors = [
            Srgb::new(0x3b, 0x82, 0xf6),
            Srgb::new(0xef, 0x44, 0x44),
            Srgb::new(0x10, 0xb9, 0x81),
            Srgb::new(0x00, 0x00, 0x00),
            Srgb::new(0xff, 0xff, 0xff),
            Srgb::new(0x80, 0x80, 0x80),
        ];

        for original in colors {
            let oklch = original.to_oklch();
            let back = oklch.to_srgb();
            assert!(
                (original.r as i16 - back.r as i16).abs() <= 2,
                "roundtrip failed for {:?}: original={:?}, back={:?}",
                original,
                oklch,
                back
            );
            assert!((original.g as i16 - back.g as i16).abs() <= 2);
            assert!((original.b as i16 - back.b as i16).abs() <= 2);
        }
    }

    #[test]
    fn test_known_value_blue_500() {
        let srgb = Srgb::new(0x3b, 0x82, 0xf6);
        let oklch = srgb.to_oklch();
        assert!(
            (oklch.l - 0.623).abs() < 0.05,
            "L={}, expected ~0.623",
            oklch.l
        );
        assert!(
            (oklch.c - 0.188).abs() < 0.05,
            "C={}, expected ~0.188",
            oklch.c
        );
        assert!(
            (oklch.h - 259.8).abs() < 10.0,
            "H={}, expected ~259.8",
            oklch.h
        );
    }

    #[test]
    fn test_extremes() {
        let white_oklch = Srgb::new(0xff, 0xff, 0xff).to_oklch();
        assert!(
            (white_oklch.l - 1.0).abs() < 0.05,
            "white L={}",
            white_oklch.l
        );

        let black_oklch = Srgb::new(0x00, 0x00, 0x00).to_oklch();
        assert!(
            (black_oklch.l - 0.0).abs() < 0.05,
            "black L={}",
            black_oklch.l
        );
    }
}
