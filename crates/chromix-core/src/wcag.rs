use super::Srgb;
use serde::Serialize;

pub const AA_NORMAL_TEXT: f64 = 4.5;
pub const AAA_NORMAL_TEXT: f64 = 7.0;
pub const AA_LARGE_OR_UI: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Usage {
    Text,
    Border,
    Background,
}

impl Usage {
    pub fn label(self) -> &'static str {
        match self {
            Usage::Text => "text",
            Usage::Border => "border",
            Usage::Background => "bg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Aaa,
    Aa,
    Fail,
}

impl Rating {
    pub fn label(self) -> &'static str {
        match self {
            Rating::Aaa => "AAA",
            Rating::Aa => "AA",
            Rating::Fail => "—",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Wcag {
    pub usage: Usage,
    pub rating: Rating,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<Srgb>,
    pub vs_white: f64,
    pub vs_black: f64,
}

const WHITE: Srgb = Srgb::new(255, 255, 255);
const BLACK: Srgb = Srgb::new(0, 0, 0);

fn relative_luminance(srgb: Srgb) -> f64 {
    let r = srgb.r as f64 / 255.0;
    let g = srgb.g as f64 / 255.0;
    let b = srgb.b as f64 / 255.0;

    let r_lin = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g_lin = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b_lin = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

pub fn contrast_ratio(a: Srgb, b: Srgb) -> f64 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (hi, lo) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn analyze(swatch: Srgb) -> Wcag {
    let vs_white = contrast_ratio(swatch, WHITE);
    let vs_black = contrast_ratio(swatch, BLACK);

    if vs_white >= AAA_NORMAL_TEXT {
        Wcag {
            usage: Usage::Text,
            rating: Rating::Aaa,
            on: None,
            vs_white,
            vs_black,
        }
    } else if vs_white >= AA_NORMAL_TEXT {
        Wcag {
            usage: Usage::Text,
            rating: Rating::Aa,
            on: None,
            vs_white,
            vs_black,
        }
    } else if vs_white >= AA_LARGE_OR_UI {
        Wcag {
            usage: Usage::Border,
            rating: Rating::Aa,
            on: None,
            vs_white,
            vs_black,
        }
    } else {
        let on = if vs_black >= vs_white { BLACK } else { WHITE };
        let on_contrast = if vs_black >= vs_white {
            vs_black
        } else {
            vs_white
        };

        let rating = if on_contrast >= AAA_NORMAL_TEXT {
            Rating::Aaa
        } else if on_contrast >= AA_NORMAL_TEXT {
            Rating::Aa
        } else {
            Rating::Fail
        };

        Wcag {
            usage: Usage::Background,
            rating,
            on: Some(on),
            vs_white,
            vs_black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_black_contrast() {
        let ratio = contrast_ratio(WHITE, BLACK);
        assert!((ratio - 21.0).abs() < 0.01);

        let ratio = contrast_ratio(WHITE, WHITE);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_white_analysis() {
        let result = analyze(WHITE);
        assert_eq!(result.usage, Usage::Background);
        assert_eq!(result.rating, Rating::Aaa);
        assert_eq!(result.on, Some(BLACK));
    }

    #[test]
    fn test_black_analysis() {
        let result = analyze(BLACK);
        assert_eq!(result.usage, Usage::Text);
        assert_eq!(result.rating, Rating::Aaa);
        assert_eq!(result.on, None);
    }

    #[test]
    fn test_blue_2563eb() {
        let blue = Srgb::new(0x25, 0x63, 0xeb);
        let result = analyze(blue);
        assert_eq!(result.usage, Usage::Text);
    }

    #[test]
    fn test_light_blue_eff6ff() {
        let light = Srgb::new(0xef, 0xf6, 0xff);
        let result = analyze(light);
        assert_eq!(result.usage, Usage::Background);
        assert_eq!(result.on, Some(BLACK));
    }

    #[test]
    fn test_gray_949494() {
        let gray = Srgb::new(0x94, 0x94, 0x94);
        let result = analyze(gray);
        assert_eq!(result.usage, Usage::Border);
        assert_eq!(result.rating, Rating::Aa);
    }
}
