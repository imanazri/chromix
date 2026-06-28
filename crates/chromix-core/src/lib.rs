pub mod color;
pub mod convert;
pub mod gamut;
pub mod gradient;
pub mod ramp;
pub mod wcag;

pub use color::{Oklch, ParseHexError, Srgb};
pub use gradient::{generate_gradient, Appearance, Gradient, GradientKind, GradientStop};
pub use ramp::{generate_ramp, generate_ramp_with_steps, Ramp, RampEntry, DEFAULT_STEPS};
pub use wcag::{analyze, contrast_ratio, Rating, Usage, Wcag};
