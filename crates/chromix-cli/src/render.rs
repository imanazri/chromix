use chromix_core::{Oklch, Ramp};
use owo_colors::{OwoColorize, Rgb};

const HEADER: Rgb = Rgb(235, 235, 235);
const LABEL: Rgb = Rgb(200, 200, 200);
const DIM: Rgb = Rgb(130, 130, 130);
const OK_GREEN: Rgb = Rgb(126, 196, 145);

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
