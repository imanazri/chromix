use chromix_core::{Gradient, GradientKind, Oklch, Ramp};
use owo_colors::{OwoColorize, Rgb};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const HEADER: Rgb = Rgb(235, 235, 235);
const LABEL: Rgb = Rgb(200, 200, 200);
const DIM: Rgb = Rgb(130, 130, 130);
const OK_GREEN: Rgb = Rgb(126, 196, 145);

/// Band width in cells — also the number of stops generated.
pub const GRADIENT_WIDTH: usize = 40;
const GRADIENT_HEIGHT: usize = 2;
const GRADIENT_FRAMES: usize = 18;

pub fn render_color(oklch: Oklch, copied: bool) {
    let srgb = oklch.to_srgb();
    let hex = srgb.to_hex();
    let oklch_str = oklch.to_css();

    println!();
    println!("  {} converted to OKLCH", "✓".color(OK_GREEN).bold());
    println!();

    print!("  ");
    print!("\x1b[48;2;{};{};{}m      \x1b[0m", srgb.r, srgb.g, srgb.b);
    print!("  {}", oklch_str.color(HEADER).bold());
    if copied {
        print!(" {}", "📋".color(OK_GREEN));
    }
    print!("  | {}", hex.color(DIM));
    println!();

    if copied {
        println!();
        println!("  {}", "copied to clipboard".color(DIM));
    }
    println!();
}

pub fn render_input_header(oklch: Oklch) {
    let srgb = oklch.to_srgb();
    let hex = srgb.to_hex();

    println!();
    print!("  ");
    print!("\x1b[48;2;{};{};{}m      \x1b[0m", srgb.r, srgb.g, srgb.b);
    print!("  {}  ", "input".color(DIM));
    println!("{}", hex.color(LABEL));
}

pub fn render_ramp(name: &str, ramp: &Ramp) {
    println!();
    println!(
        "{} · {} steps",
        name.color(HEADER).bold(),
        ramp.entries.len().to_string().color(DIM)
    );
    println!();

    for entry in &ramp.entries {
        print!("  ");
        print!(
            "\x1b[48;2;{};{};{}m      \x1b[0m",
            entry.srgb.r, entry.srgb.g, entry.srgb.b
        );

        let step_str = format!("{:>4}", entry.step);
        let oklch_str = format!("{:<24}", entry.oklch.to_css());

        println!(
            " {}  {}  {}",
            step_str.color(LABEL),
            oklch_str.color(LABEL),
            entry.hex.color(DIM)
        );
    }

    println!();
    println!(
        "{} {}",
        "Tips:".color(HEADER).bold(),
        "use chromix export to export as json".color(DIM)
    );
    println!();
}

fn variant_label(kind: GradientKind) -> &'static str {
    match kind {
        GradientKind::Analogous => "analogous",
        GradientKind::Complementary => "complementary",
        GradientKind::Monochromatic => "monochromatic",
    }
}

/// Draw the gradient as a band of background-colored blocks. Cells at or past
/// `reveal` are left blank (no background) for the wipe-in animation.
fn draw_band(gradient: &Gradient, reveal: usize) {
    for _ in 0..GRADIENT_HEIGHT {
        print!("  ");
        for (x, stop) in gradient.stops.iter().enumerate() {
            if x < reveal {
                let s = stop.srgb;
                print!("\x1b[48;2;{};{};{}m ", s.r, s.g, s.b);
            } else {
                print!("\x1b[0m ");
            }
        }
        println!("\x1b[0m");
    }
}

/// Render one variant as a row: its title, the band (animated when on a TTY),
/// and the start/end OKLCH values on their own line for easy copy-paste.
fn render_band(gradient: &Gradient) {
    println!("  {}", variant_label(gradient.kind).color(HEADER).bold());

    if atty::is(atty::Stream::Stdout) {
        for frame in 0..=GRADIENT_FRAMES {
            let t = frame as f64 / GRADIENT_FRAMES as f64;
            let eased = 1.0 - (1.0 - t).powi(3);
            let reveal = ((eased * GRADIENT_WIDTH as f64).round() as usize).min(GRADIENT_WIDTH);

            draw_band(gradient, reveal);
            io::stdout().flush().ok();

            thread::sleep(Duration::from_millis(18));

            if frame < GRADIENT_FRAMES {
                print!("\x1b[{}A", GRADIENT_HEIGHT);
            }
        }
    } else {
        draw_band(gradient, GRADIENT_WIDTH);
    }

    let first = &gradient.stops[0];
    let last = &gradient.stops[gradient.stops.len() - 1];
    println!(
        "  {} {} {}",
        first.oklch.to_css().color(LABEL),
        "→".color(DIM),
        last.oklch.to_css().color(LABEL)
    );
    println!();
}

fn section_header(icon_label: &str) {
    println!("  {}", icon_label.color(LABEL).bold());
    println!();
}

pub fn render_gradient(base: Oklch, pairs: &[(Gradient, Gradient)]) {
    render_input_header(base);
    println!();

    section_header("☀ light mode");
    for (light, _) in pairs {
        render_band(light);
    }

    // Separator between the light and dark sections.
    println!("  {}", "─".repeat(GRADIENT_WIDTH).color(DIM));
    println!();

    section_header("☾ dark mode");
    for (_, dark) in pairs {
        render_band(dark);
    }
}
