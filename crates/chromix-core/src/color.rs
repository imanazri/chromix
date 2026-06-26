use crate::convert;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl Oklch {
    pub const fn new(l: f64, c: f64, h: f64) -> Self {
        Self { l, c, h }
    }

    pub fn to_srgb(self) -> Srgb {
        convert::oklch_to_srgb(self)
    }

    pub fn to_css(self) -> String {
        format!("oklch({:.1}% {:.3} {:.1})", self.l * 100.0, self.c, self.h)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Srgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Srgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_oklch(self) -> Oklch {
        convert::srgb_to_oklch(self)
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseHexError {
    BadLength(usize),
    BadDigit(char),
}

impl fmt::Display for ParseHexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseHexError::BadLength(n) => write!(f, "expected 3, 6, or 8 hex digits, got {}", n),
            ParseHexError::BadDigit(ch) => write!(f, "invalid hex digit '{}'", ch),
        }
    }
}

impl std::error::Error for ParseHexError {}

impl Srgb {
    pub fn from_hex(s: &str) -> Result<Self, ParseHexError> {
        let s = s.strip_prefix('#').unwrap_or(s);

        // Check for bad digit first (before length check)
        for ch in s.chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(ParseHexError::BadDigit(ch));
            }
        }

        let (r, g, b) = match s.len() {
            3 | 4 => {
                // Shorthand: double each digit, ignore alpha (4th digit)
                let r = (s.as_bytes()[0] as char).to_digit(16).unwrap() as u8 * 17;
                let g = (s.as_bytes()[1] as char).to_digit(16).unwrap() as u8 * 17;
                let b = (s.as_bytes()[2] as char).to_digit(16).unwrap() as u8 * 17;
                (r, g, b)
            }
            6 | 8 => {
                // Full form, ignore alpha (8th digit)
                let r = u8::from_str_radix(&s[0..2], 16).unwrap();
                let g = u8::from_str_radix(&s[2..4], 16).unwrap();
                let b = u8::from_str_radix(&s[4..6], 16).unwrap();
                (r, g, b)
            }
            n => return Err(ParseHexError::BadLength(n)),
        };

        Ok(Self::new(r, g, b))
    }
}

impl Oklch {
    pub fn from_hex(s: &str) -> Result<Self, ParseHexError> {
        Ok(Srgb::from_hex(s)?.to_oklch())
    }
}