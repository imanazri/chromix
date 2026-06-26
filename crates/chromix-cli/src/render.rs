use chromix_core::{Oklch, Ramp};
use owo_colors::{OwoColorize, Rgb};

// Grayscale palette
const HEADER: Rgb = Rgb(235, 235, 235);
const LABEL: Rgb = Rgb(200, 200, 200);
const DIM: Rgb = Rgb(130, 130, 130);
const OK_GREEN: Rgb = Rgb(126, 196, 145);

pub fn render_color(oklch: Oklch, copied: bool) {
    let srgb = oklch.to_srgb();
    let hex = srgb.to_hex();
    let oklch_str = oklch.to_css();

    let ok_green = OK_GREEN;

    println!();
    print!("  {} converted to OKLCH", "✓".color(ok_green).bold());
    println!(
        "{}",
        format!("  {}", oklch_str).color(HEADER)
    );
    println!();
    
    // Swatch with background
    print!("      ");
    print!("\x1b[48;2;{};{};{}m", srgb.r, srgb.g, srgb.b);
    print!(" ");
    print!("\x1b[0m");
    
    print!("  ");
    print!("{}", oklch_str.color(HEADER));
    if copied {
        print!(" {} ", "📋".color(ok_green));
    }
    println!("| {}", hex.color(DIM));
    if copied {
        println!();
        println!("{} copied to clipboard", " ".repeat(2).color(DIM));
    }
    println!();
}

pub fn render_input_header(oklch: Oklch) {
    let srgb = oklch.to_srgb();
    let hex = srgb.to_hex();

    println!();
    print!("      ");
    print!("\x1b[48;2;{};{};{}m", srgb.r, srgb.g, srgb.b);
    print!(" ");
    print!("\x1b[0m");
    print!("  input ");
    print!("{}", hex.color(LABEL));
    println!();
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
        // Swatch with background
        print!("      ");
        print!("\x1b[48;2;{};{};{}m", entry.srgb.r, entry.srgb.g, entry.srgb.b);
        print!(" ");
        print!("\x1b[0m");
        
        let step_str = format!("{:>4}", entry.step);

        println!(
            " {} {}   | {}",
            step_str.color(LABEL),
            entry.oklch.to_css().color(LABEL),
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